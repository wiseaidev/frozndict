// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # Python Bindings
//!
//! Exposes the `frozndict` library to Python via [`pyo3`].
//!
//! ## Storage Layout (`FrozenDictInner`)
//!
//! ```text
//! entries: Box<[(hash, key, value)]>    ← insertion-ordered
//! lookup:  Box<[(hash, u32)]>           ← hash-sorted for O(log n) lookup
//! hash:    isize                         ← O(1) pre-computed aggregate
//! cached_keys/values/items: OnceLock    ← built lazily on first access
//! ```
//!
//! **O(1) FrozenDict→FrozenDict clone**: `frozendict(fd)` with no kwargs
//! short-circuits to `Arc::clone(&fd.inner)` - no rehashing or allocation.
//!
//! **O(n log n) deduplication**: pairs are sorted by hash, then a linear scan
//! resolves duplicates within same-hash runs.  For the common case (dict/int/str
//! keys with distinct hashes) the inner equality loop never runs, reducing the
//! cost from O(n²) equality checks to zero.
//!
//! **No eager list construction**: `cached_keys`, `cached_values`, and
//! `cached_items` are built lazily via [`OnceLock`] on first call to `keys()`,
//! `values()`, or `items()`.  Construction no longer allocates three `PyList`
//! objects and N `PyTuple` objects.
//!
//! **Inline hash mixing**: `hash((k, v))` tuple allocation is replaced with
//! `k_hash * M1 ^ v_hash * M2` using two multiplicative constants, eliminating
//! N temporary Python tuple objects per construction.
//!
//! **O(1) self-equality**: `__eq__` checks `Arc::ptr_eq` first; `o == o`
//! returns `true` in a single pointer comparison.
//!
//! ## Complexity Summary
//!
//! | Operation                        | Time        | Notes                          |
//! |----------------------------------|-------------|--------------------------------|
//! | `frozendict(fd)` (FrozenDict src)| O(1)        | Arc clone                      |
//! | `frozendict(dict)` (n entries)   | O(n log n)  | Sort + O(n) dedup scan         |
//! | `frozendict(**kwargs)`           | O(n log n)  |                                |
//! | `__getitem__` / `get`            | O(log n)    | Binary search via lookup       |
//! | `__contains__`                   | O(log n)    |                                |
//! | `__hash__`                       | O(1)        | Pre-computed                   |
//! | `__eq__` (same object)           | O(1)        | Arc pointer equality           |
//! | `__eq__` (hash mismatch)         | O(1)        | Cached hash short-circuit      |
//! | `__eq__` (full compare)          | O(n)        |                                |
//! | `keys()` / `values()` / `items()`| O(1)        | View wrapping Arc              |
//! | First iteration (lazy init)      | O(n)        | Builds PyList once             |
//! | Subsequent iterations            | O(n)        | Traverse pre-built PyList      |
//! | `copy()`                         | O(1)        | Arc clone                      |

use once_cell::sync::OnceCell;
use pyo3::exceptions::PyAttributeError;
use pyo3::exceptions::PyKeyError;
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::types::PyFrozenSet;
use pyo3::types::PyIterator;
use pyo3::types::PyList;
use pyo3::types::PySet;
use pyo3::types::PyTuple;
use pyo3::types::PyType;
use std::sync::Arc;

/// The error message raised by every mutating method.
const MUTATION_ERROR: &str = "'frozendict' object does not support mutation";

/// The error message raised by `__dir__`.
const ACCESS_DENIED: &str = "Access is Denied!";

/// Multiplicative constant for the key-hash contribution to the aggregate hash.
/// (first 64 bits of the golden-ratio constant -x 2^64)
const MIX_KEY: u64 = 0x9e3779b97f4a7c15;

/// Multiplicative constant for the value-hash contribution.
const MIX_VAL: u64 = 0x517cc1b727220a95;

/// Convenience alias for an owned reference to any Python object.
type Obj = Py<PyAny>;

/// The shared, heap-allocated core of a [`FrozenDict`] and all its views.
///
/// All fields are read-only after construction.  The three `cached_*` fields
/// are populated lazily via [`OnceLock`] on first access, so construction
/// never allocates PyList or PyTuple objects.
///
/// Since only one OS thread can hold the Python GIL at a time, the `OnceLock`
/// guarantees exactly-once initialisation without risk of deadlock.
struct FrozenDictInner {
    /// Insertion-ordered `(python_hash, key, value)` triples.
    entries: Box<[(isize, Obj, Obj)]>,

    /// `(python_hash, index_into_entries)` pairs, sorted by hash for binary search.
    lookup: Box<[(isize, u32)]>,

    /// Pre-computed aggregate hash: `XOR of (k_hash * MIX_KEY ^ v_hash * MIX_VAL)`.
    ///
    /// Order-independent because XOR is commutative and associative.
    hash: isize,

    /// Lazily-built `PyList` of keys in insertion order.
    cached_keys: OnceCell<Obj>,

    /// Lazily-built `PyList` of values in insertion order.
    cached_values: OnceCell<Obj>,

    /// Lazily-built `PyList` of `(key, value)` tuples in insertion order.
    cached_items: OnceCell<Obj>,
}

impl FrozenDictInner {
    /// Return (and build if necessary) the lazily-initialised keys list.
    ///
    /// # Complexity
    ///
    /// - First call: O(n) - allocates one `PyList`.
    /// - Subsequent calls: O(1) - `OnceLock::get` is lock-free after init.
    fn get_keys(&self, py: Python<'_>) -> PyResult<&Obj> {
        self.cached_keys.get_or_try_init(|| {
            let v: Vec<Obj> = self
                .entries
                .iter()
                .map(|(_, k, _)| k.clone_ref(py))
                .collect();
            Ok(PyList::new(py, v)?.into_any().unbind())
        })
    }

    /// Return (and build if necessary) the lazily-initialised values list.
    ///
    /// # Complexity
    ///
    /// - First call: O(n); subsequent: O(1).
    fn get_values(&self, py: Python<'_>) -> PyResult<&Obj> {
        self.cached_values.get_or_try_init(|| {
            let v: Vec<Obj> = self
                .entries
                .iter()
                .map(|(_, _, v)| v.clone_ref(py))
                .collect();
            Ok(PyList::new(py, v)?.into_any().unbind())
        })
    }

    /// Return (and build if necessary) the lazily-initialised items list.
    ///
    /// # Complexity
    ///
    /// - First call: O(n); subsequent: O(1).
    fn get_items(&self, py: Python<'_>) -> PyResult<&Obj> {
        self.cached_items.get_or_try_init(|| {
            let v: Vec<Obj> = self
                .entries
                .iter()
                .map(|(_, k, v)| {
                    Ok(PyTuple::new(py, [k.clone_ref(py), v.clone_ref(py)])?
                        .into_any()
                        .unbind())
                })
                .collect::<PyResult<_>>()?;
            Ok(PyList::new(py, v)?.into_any().unbind())
        })
    }
}

/// A fully immutable, hashable Python dictionary with insertion-order semantics.
///
/// See the crate-level documentation for the full API description, complexity
/// table, and usage examples.
#[pyclass(name = "FrozenDict", frozen, subclass)]
pub struct FrozenDict {
    inner: Arc<FrozenDictInner>,
}

impl FrozenDict {
    /// Core constructor: builds a [`FrozenDictInner`] from pre-hashed pairs,
    /// with an optional deduplication step.
    ///
    /// When `may_have_dups` is `false` (e.g. source is a plain `dict`), the
    /// O(n log n) sort is still performed to build the lookup index, but the
    /// O(n) equality-check scan is skipped entirely.  This is the common case.
    ///
    /// When `may_have_dups` is `true` (e.g. source is kwargs + positional),
    /// same-hash runs are scanned for exact key equality to merge duplicates.
    ///
    /// # Complexity
    ///
    /// - Time: O(n log n) - sort dominates.  Equality checks add O(k) work
    ///   where k is the number of pairs sharing the same hash (usually 0).
    /// - Space: O(n)
    fn build_inner(
        py: Python<'_>,
        pairs: Vec<(isize, Obj, Obj)>,
        may_have_dups: bool,
    ) -> PyResult<Arc<FrozenDictInner>> {
        let n = pairs.len();

        let mut tagged: Vec<(u32, isize, Obj, Obj)> = pairs
            .into_iter()
            .enumerate()
            .map(|(i, (h, k, v))| (i as u32, h, k, v))
            .collect();

        tagged.sort_unstable_by_key(|t| t.1);

        let mut unique: Vec<(u32, isize, Obj, Obj)> = Vec::with_capacity(n);

        let mut i = 0;
        while i < tagged.len() {
            let cur_hash = tagged[i].1;
            let mut j = i + 1;
            while j < tagged.len() && tagged[j].1 == cur_hash {
                j += 1;
            }
            let run = &tagged[i..j];

            if run.len() == 1 || !may_have_dups {
                for t in run {
                    unique.push((t.0, t.1, t.2.clone_ref(py), t.3.clone_ref(py)));
                }
            } else {
                let mut sub: Vec<usize> = (i..j).collect();
                sub.sort_unstable_by_key(|&x| tagged[x].0);
                let mut reps: Vec<(u32, usize, usize)> = Vec::new();
                for &slot in &sub {
                    let key = &tagged[slot].2;
                    let mut found = false;
                    for rep in &mut reps {
                        if tagged[rep.1].2.bind(py).eq(key.bind(py))? {
                            rep.2 = slot;
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        reps.push((tagged[slot].0, slot, slot));
                    }
                }
                for (orig, key_slot, val_slot) in reps {
                    unique.push((
                        orig,
                        cur_hash,
                        tagged[key_slot].2.clone_ref(py),
                        tagged[val_slot].3.clone_ref(py),
                    ));
                }
            }
            i = j;
        }

        unique.sort_unstable_by_key(|t| t.0);

        let m = unique.len();

        let mut lookup: Vec<(isize, u32)> = Vec::with_capacity(m);
        let mut combined: u64 = 0;
        let mut entries: Vec<(isize, Obj, Obj)> = Vec::with_capacity(m);

        for (entry_idx, (_, h, k, v)) in unique.into_iter().enumerate() {
            let v_hash = v.bind(py).hash()? as u64;
            combined ^= (h as u64).wrapping_mul(MIX_KEY) ^ v_hash.wrapping_mul(MIX_VAL);
            lookup.push((h, entry_idx as u32));
            entries.push((h, k, v));
        }
        lookup.sort_unstable_by_key(|&(h, _)| h);

        Ok(Arc::new(FrozenDictInner {
            entries: entries.into_boxed_slice(),
            lookup: lookup.into_boxed_slice(),
            hash: combined as isize,
            cached_keys: OnceCell::new(),
            cached_values: OnceCell::new(),
            cached_items: OnceCell::new(),
        }))
    }

    /// Build from `(key, value)` pairs, hashing keys inline.
    ///
    /// `may_have_dups` controls whether the dedup pass runs.
    ///
    /// # Complexity
    ///
    /// - Time: O(n log n)
    /// - Space: O(n)
    fn from_pairs(py: Python<'_>, pairs: Vec<(Obj, Obj)>, may_have_dups: bool) -> PyResult<Self> {
        let hashed: Vec<(isize, Obj, Obj)> = pairs
            .into_iter()
            .map(|(k, v)| {
                let h = k.bind(py).hash()?;
                Ok((h, k, v))
            })
            .collect::<PyResult<_>>()?;
        Ok(Self {
            inner: Self::build_inner(py, hashed, may_have_dups)?,
        })
    }

    /// Wraps [`from_pairs`] in a new Python `FrozenDict` object.
    ///
    /// # Complexity
    ///
    /// - Time: O(n log n)
    /// - Space: O(n)
    #[inline]
    fn into_py_object(
        py: Python<'_>,
        pairs: Vec<(Obj, Obj)>,
        may_have_dups: bool,
    ) -> PyResult<Obj> {
        Ok(Py::new(py, Self::from_pairs(py, pairs, may_have_dups)?)?.into_any())
    }

    #[inline]
    fn num_entries(&self) -> usize {
        self.inner.entries.len()
    }

    #[inline]
    fn pairs(&self) -> &[(isize, Obj, Obj)] {
        &self.inner.entries
    }

    /// Binary-search the sorted lookup index for `hash`, then walk the
    /// collision run checking key equality.
    ///
    /// Returns the index into `entries` on a hit, `None` on a miss.
    ///
    /// # Complexity
    ///
    /// - Time: O(log n) amortised; O(n) worst-case (all hashes equal).
    /// - Space: O(1)
    #[inline]
    fn find_entry(
        &self,
        py: Python<'_>,
        key: &Bound<'_, PyAny>,
        hash: isize,
    ) -> PyResult<Option<usize>> {
        let lk = &self.inner.lookup;
        match lk.binary_search_by(|&(h, _)| h.cmp(&hash)) {
            Err(_) => Ok(None),
            Ok(idx) => {
                let mut i = idx;
                while i > 0 && lk[i - 1].0 == hash {
                    i -= 1;
                }
                while i < lk.len() && lk[i].0 == hash {
                    let ei = lk[i].1 as usize;
                    if self.inner.entries[ei].1.bind(py).eq(key)? {
                        return Ok(Some(ei));
                    }
                    i += 1;
                }
                Ok(None)
            }
        }
    }
}

/// Recursively converts unhashable containers into hashable equivalents.
///
/// - `dict`  → `FrozenDict`
/// - `list`  → `tuple`
/// - `set`   → `frozenset`
/// - Everything else is returned as-is.
fn freeze_value(val: Bound<'_, PyAny>) -> PyResult<Obj> {
    let py = val.py();
    if let Ok(d) = val.cast::<PyDict>() {
        let pairs: Vec<(Obj, Obj)> = d
            .iter()
            .map(|(k, v)| Ok((k.unbind(), freeze_value(v)?)))
            .collect::<PyResult<_>>()?;
        return FrozenDict::into_py_object(py, pairs, false);
    }
    if val.is_instance_of::<FrozenDict>() {
        return Ok(val.unbind());
    }
    if let Ok(lst) = val.cast::<PyList>() {
        let els: Vec<Obj> = lst.iter().map(freeze_value).collect::<PyResult<_>>()?;
        return Ok(PyTuple::new(py, els)?.into_any().unbind());
    }
    if let Ok(s) = val.cast::<PySet>() {
        let fs = PyFrozenSet::new(py, s.iter().collect::<Vec<_>>().iter())?;
        return Ok(fs.into_any().unbind());
    }
    Ok(val.unbind())
}

/// Collects `(key, value)` pairs from any Python source:
/// plain `dict`, `FrozenDict`, generic mapping, or iterable of pairs.
///
/// Returns `(pairs, may_have_dups)` - the flag signals whether the caller
/// should enable the deduplication pass in [`FrozenDict::build_inner`].
///
/// `dict` and `FrozenDict` sources are always duplicate-free; kwargs may
/// introduce duplicates when combined with a positional argument.
fn extract_pairs(source: &Bound<'_, PyAny>) -> PyResult<(Vec<(Obj, Obj)>, bool)> {
    let py = source.py();

    if let Ok(d) = source.cast::<PyDict>() {
        let pairs = d
            .iter()
            .map(|(k, v)| Ok((k.unbind(), freeze_value(v)?)))
            .collect::<PyResult<_>>()?;
        return Ok((pairs, false));
    }

    if let Ok(fd) = source.extract::<PyRef<'_, FrozenDict>>() {
        let pairs = fd
            .pairs()
            .iter()
            .map(|(_, k, v)| (k.clone_ref(py), v.clone_ref(py)))
            .collect();
        return Ok((pairs, false));
    }

    if source.hasattr("keys")? {
        let keys = source.call_method0("keys")?;
        let mut pairs = Vec::new();
        for k in keys.try_iter()? {
            let k = k?;
            let v = source.get_item(&k)?;
            pairs.push((k.unbind(), freeze_value(v)?));
        }
        return Ok((pairs, true));
    }

    let mut pairs: Vec<(Obj, Obj)> = Vec::new();
    for item in PyIterator::from_object(source)? {
        let item = item?;
        let pair = item.cast::<PyTuple>()?;
        if pair.len() != 2 {
            return Err(PyTypeError::new_err(
                "each item must be a (key, value) pair",
            ));
        }
        pairs.push((pair.get_item(0)?.unbind(), freeze_value(pair.get_item(1)?)?));
    }
    Ok((pairs, true))
}

/// Helper: build a `frozenset` from a cached list bound to `py`.
fn list_to_frozenset<'py>(py: Python<'py>, cached: &Obj) -> PyResult<Bound<'py, PyFrozenSet>> {
    let items: Vec<Bound<'_, PyAny>> = cached.bind(py).try_iter()?.collect::<PyResult<_>>()?;
    PyFrozenSet::new(py, items.iter())
}

#[pymethods]
impl FrozenDict {
    /// Construct a new :class:`FrozenDict`.
    ///
    /// **Fast paths (no allocation):**
    ///
    /// - ``frozendict(fd)`` where ``fd`` is already a :class:`FrozenDict` and
    ///   no kwargs are given: copies the internal ``Arc`` in O(1).
    ///
    /// **Normal path:**
    ///
    /// - O(n log n) sort-based deduplication.
    ///
    /// # Complexity
    ///
    /// - Time: O(1) for FrozenDict arg; O(n log n) otherwise.
    /// - Space: O(1) for FrozenDict arg; O(n) otherwise.
    #[new]
    #[pyo3(signature = (*args, **kwargs))]
    pub fn __new__(
        py: Python<'_>,
        args: &Bound<'_, PyTuple>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Self> {
        if args.len() > 1 {
            return Err(PyTypeError::new_err(format!(
                "expected at most 1 positional argument, got {}",
                args.len()
            )));
        }

        let no_kwargs = kwargs.map(|k| k.is_empty()).unwrap_or(true);

        if args.len() == 1 && no_kwargs {
            let first = args.get_item(0)?;
            if let Ok(fd) = first.extract::<PyRef<'_, FrozenDict>>() {
                return Ok(Self {
                    inner: Arc::clone(&fd.inner),
                });
            }
        }

        if args.is_empty() && no_kwargs {
            return Ok(Self {
                inner: Self::build_inner(py, vec![], false)?,
            });
        }

        let mut pairs: Vec<(Obj, Obj)> = Vec::new();
        let mut may_have_dups = false;

        if let Ok(first) = args.get_item(0) {
            let (p, dups) = extract_pairs(&first)?;
            pairs.extend(p);
            may_have_dups |= dups;
        }
        if let Some(kw) = kwargs
            && !kw.is_empty()
        {
            for (k, v) in kw.iter() {
                pairs.push((k.unbind(), freeze_value(v)?));
            }
            if !pairs.is_empty() {
                may_have_dups = true;
            }
        }

        Self::from_pairs(py, pairs, may_have_dups)
    }

    /// Return the pre-computed hash.
    ///
    /// # Complexity - O(1)
    pub fn __hash__(&self) -> isize {
        self.inner.hash
    }

    /// Return the number of entries.
    ///
    /// # Complexity - O(1)
    pub fn __len__(&self) -> usize {
        self.inner.entries.len()
    }

    /// Return ``True`` if ``key`` is present (binary search).
    ///
    /// # Complexity - O(log n)
    pub fn __contains__(&self, py: Python<'_>, key: &Bound<'_, PyAny>) -> PyResult<bool> {
        Ok(self.find_entry(py, key, key.hash()?)?.is_some())
    }

    /// Return the value for ``key`` or raise :exc:`KeyError`.
    ///
    /// # Complexity - O(log n)
    pub fn __getitem__(&self, py: Python<'_>, key: &Bound<'_, PyAny>) -> PyResult<Obj> {
        match self.find_entry(py, key, key.hash()?)? {
            Some(i) => Ok(self.inner.entries[i].2.clone_ref(py)),
            None => Err(PyKeyError::new_err(key.clone().unbind())),
        }
    }

    /// Return an iterator over keys in insertion order.
    ///
    /// Triggers lazy initialisation of the keys cache on first call.
    ///
    /// # Complexity - O(1) after first call; O(n) on first call.
    pub fn __iter__(&self, py: Python<'_>) -> PyResult<Obj> {
        Ok(self
            .inner
            .get_keys(py)?
            .bind(py)
            .try_iter()?
            .into_any()
            .unbind())
    }

    /// Return an iterator over keys in **reverse** insertion order.
    ///
    /// # Complexity - O(n)
    pub fn __reversed__(&self, py: Python<'_>) -> PyResult<Obj> {
        let keys: Vec<Obj> = self
            .inner
            .entries
            .iter()
            .rev()
            .map(|(_, k, _)| k.clone_ref(py))
            .collect();
        Ok(PyList::new(py, keys)?.try_iter()?.into_any().unbind())
    }

    /// Return ``frozendict({key: value, ...})`` in insertion order.
    ///
    /// # Complexity - O(n)
    pub fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        let mut parts = Vec::with_capacity(self.inner.entries.len());
        for (_, k, v) in self.inner.entries.iter() {
            parts.push(format!("{}: {}", k.bind(py).repr()?, v.bind(py).repr()?));
        }
        Ok(format!("frozendict({{{}}})", parts.join(", ")))
    }

    /// Test equality.
    ///
    /// **Short-circuits:**
    ///
    /// 1. `Arc` pointer equality (O(1)) - handles `o == o`.
    /// 2. Size or hash mismatch (O(1)).
    /// 3. Full key-value scan (O(n)).
    ///
    /// # Complexity - O(1) best-case; O(n) worst-case.
    pub fn __eq__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<bool> {
        if let Ok(other_fd) = other.extract::<PyRef<'_, FrozenDict>>() {
            if Arc::ptr_eq(&self.inner, &other_fd.inner) {
                return Ok(true);
            }
            if self.num_entries() != other_fd.num_entries()
                || self.inner.hash != other_fd.inner.hash
            {
                return Ok(false);
            }
            for (_, k1, v1) in self.inner.entries.iter() {
                let h = k1.bind(py).hash()?;
                let lk = &other_fd.inner.lookup;
                let found = match lk.binary_search_by(|&(lh, _)| lh.cmp(&h)) {
                    Err(_) => false,
                    Ok(idx) => {
                        let mut i = idx;
                        while i > 0 && lk[i - 1].0 == h {
                            i -= 1;
                        }
                        let mut hit = false;
                        while i < lk.len() && lk[i].0 == h {
                            let ei = lk[i].1 as usize;
                            if other_fd.inner.entries[ei].1.bind(py).eq(k1.bind(py))? {
                                if !v1.bind(py).eq(other_fd.inner.entries[ei].2.bind(py))? {
                                    return Ok(false);
                                }
                                hit = true;
                                break;
                            }
                            i += 1;
                        }
                        hit
                    }
                };
                if !found {
                    return Ok(false);
                }
            }
            return Ok(true);
        }
        if let Ok(dict) = other.cast::<PyDict>() {
            if self.num_entries() != dict.len() {
                return Ok(false);
            }
            for (_, k, v) in self.inner.entries.iter() {
                match dict.get_item(k.bind(py))? {
                    Some(dv) => {
                        if !v.bind(py).eq(&dv)? {
                            return Ok(false);
                        }
                    }
                    None => return Ok(false),
                }
            }
            return Ok(true);
        }
        Ok(false)
    }

    /// Return a new :class:`FrozenDict` merged with ``other`` (``|``).
    ///
    /// # Complexity - O((m + n) log(m + n))
    pub fn __or__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<Obj> {
        let mut pairs: Vec<(Obj, Obj)> = self
            .inner
            .entries
            .iter()
            .map(|(_, k, v)| (k.clone_ref(py), v.clone_ref(py)))
            .collect();
        let (other_pairs, _) = extract_pairs(other)?;
        pairs.extend(other_pairs);
        Self::into_py_object(py, pairs, true)
    }

    /// Support ``other | self``.
    ///
    /// # Complexity - O((m + n) log(m + n))
    pub fn __ror__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<Obj> {
        let (mut pairs, _) = extract_pairs(other)?;
        pairs.extend(
            self.inner
                .entries
                .iter()
                .map(|(_, k, v)| (k.clone_ref(py), v.clone_ref(py))),
        );
        Self::into_py_object(py, pairs, true)
    }

    /// Exposed as ``ior()`` - Python ``|=`` falls back to ``__or__``.
    ///
    /// (``__ior__`` cannot be a PyO3 magic method because the in-place slot
    /// requires ``()`` return type.)
    pub fn ior(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<Obj> {
        self.__or__(py, other)
    }

    /// Return a :class:`FrozenKeysView` in O(1).
    pub fn keys(&self, py: Python<'_>) -> PyResult<Obj> {
        Ok(Py::new(
            py,
            FrozenKeysView {
                inner: Arc::clone(&self.inner),
            },
        )?
        .into_any())
    }

    /// Return a :class:`FrozenValuesView` in O(1).
    pub fn values(&self, py: Python<'_>) -> PyResult<Obj> {
        Ok(Py::new(
            py,
            FrozenValuesView {
                inner: Arc::clone(&self.inner),
            },
        )?
        .into_any())
    }

    /// Return a :class:`FrozenItemsView` in O(1).
    pub fn items(&self, py: Python<'_>) -> PyResult<Obj> {
        Ok(Py::new(
            py,
            FrozenItemsView {
                inner: Arc::clone(&self.inner),
            },
        )?
        .into_any())
    }

    /// Look up ``key``, returning ``default`` if absent.
    ///
    /// # Complexity - O(log n)
    #[pyo3(signature = (key, default=None))]
    pub fn get(
        &self,
        py: Python<'_>,
        key: &Bound<'_, PyAny>,
        default: Option<Obj>,
    ) -> PyResult<Obj> {
        match self.find_entry(py, key, key.hash()?)? {
            Some(i) => Ok(self.inner.entries[i].2.clone_ref(py)),
            None => Ok(default.unwrap_or_else(|| py.None())),
        }
    }

    /// Return a new :class:`FrozenDict` sharing the same ``Arc``.
    ///
    /// # Complexity - O(1)
    pub fn copy(&self, py: Python<'_>) -> PyResult<Obj> {
        Ok(Py::new(
            py,
            FrozenDict {
                inner: Arc::clone(&self.inner),
            },
        )?
        .into_any())
    }

    /// Support ``copy.copy(d)`` - delegates to :meth:`copy`.
    pub fn __copy__(&self, py: Python<'_>) -> PyResult<Obj> {
        self.copy(py)
    }

    /// Support ``copy.deepcopy(d)`` - delegates to :meth:`copy`.
    ///
    /// Values must be hashable and are therefore immutable, so a deep copy
    /// is semantically identical to a shallow copy.
    #[pyo3(signature = (_memo))]
    pub fn __deepcopy__(&self, py: Python<'_>, _memo: &Bound<'_, PyAny>) -> PyResult<Obj> {
        self.copy(py)
    }

    /// Return ``(constructor, args)`` for pickle.
    ///
    /// # Complexity - O(n)
    pub fn __reduce__(&self, py: Python<'_>) -> PyResult<Obj> {
        let typ = py.get_type::<FrozenDict>();
        let d = PyDict::new(py);
        for (_, k, v) in self.inner.entries.iter() {
            d.set_item(k.bind(py), v.bind(py))?;
        }
        let args = PyTuple::new(py, [d.into_any().unbind()])?;
        Ok(
            PyTuple::new(py, [typ.into_any().unbind(), args.into_any().unbind()])?
                .into_any()
                .unbind(),
        )
    }

    /// Delegates to :meth:`__reduce__` for all protocols.
    #[pyo3(signature = (_protocol))]
    pub fn __reduce_ex__(&self, py: Python<'_>, _protocol: i32) -> PyResult<Obj> {
        self.__reduce__(py)
    }

    /// ``frozendict.fromkeys(keys, value=None)``.
    ///
    /// # Complexity - O(n log n)
    #[classmethod]
    #[pyo3(signature = (keys, value=None))]
    pub fn fromkeys(
        _cls: &Bound<'_, PyType>,
        py: Python<'_>,
        keys: &Bound<'_, PyAny>,
        value: Option<Obj>,
    ) -> PyResult<Obj> {
        let fill = value.unwrap_or_else(|| py.None());
        let pairs: Vec<(Obj, Obj)> = PyIterator::from_object(keys)?
            .map(|k| k.map(|k| (k.unbind(), fill.clone_ref(py))))
            .collect::<PyResult<_>>()?;
        Self::into_py_object(py, pairs, true)
    }

    /// ``FrozenDict[str, int]`` → ``types.GenericAlias``.
    #[classmethod]
    pub fn __class_getitem__(cls: &Bound<'_, PyType>, item: Obj) -> PyResult<Obj> {
        let py = cls.py();
        let args = PyTuple::new(py, [cls.clone().into_any().unbind(), item])?;
        Ok(py
            .import("types")?
            .call_method1("GenericAlias", args)?
            .unbind())
    }

    /// Return an indented pretty-print representation.
    ///
    /// # Complexity - O(n)
    #[pyo3(signature = (num_spaces=4))]
    pub fn pretty_repr(&self, py: Python<'_>, num_spaces: usize) -> PyResult<String> {
        let ind = " ".repeat(num_spaces);
        let mut out = String::from("frozendict({\n");
        for (_, k, v) in self.inner.entries.iter() {
            out.push_str(&format!(
                "{ind}{}: {},\n",
                k.bind(py).repr()?,
                v.bind(py).repr()?
            ));
        }
        out.push_str("})");
        Ok(out)
    }

    pub fn __delattr__(&self, _name: &str) -> PyResult<()> {
        Err(PyTypeError::new_err(MUTATION_ERROR))
    }

    pub fn __delitem__(&self, _key: &Bound<'_, PyAny>) -> PyResult<()> {
        Err(PyTypeError::new_err(MUTATION_ERROR))
    }

    pub fn __setattr__(&self, _name: &str, _value: &Bound<'_, PyAny>) -> PyResult<()> {
        Err(PyTypeError::new_err(MUTATION_ERROR))
    }

    pub fn __setitem__(&self, _key: &Bound<'_, PyAny>, _value: &Bound<'_, PyAny>) -> PyResult<()> {
        Err(PyTypeError::new_err(MUTATION_ERROR))
    }

    pub fn __dir__(&self) -> PyResult<Vec<String>> {
        Err(PyAttributeError::new_err(ACCESS_DENIED))
    }

    #[pyo3(signature = (*_args, **_kwargs))]
    pub fn pop(
        &self,
        _args: &Bound<'_, PyTuple>,
        _kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Obj> {
        Err(PyTypeError::new_err(MUTATION_ERROR))
    }

    #[pyo3(signature = (*_args, **_kwargs))]
    pub fn update(
        &self,
        _args: &Bound<'_, PyTuple>,
        _kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<()> {
        Err(PyTypeError::new_err(MUTATION_ERROR))
    }

    #[pyo3(signature = (_key, _default=None))]
    pub fn setdefault(&self, _key: &Bound<'_, PyAny>, _default: Option<Obj>) -> PyResult<Obj> {
        Err(PyTypeError::new_err(MUTATION_ERROR))
    }

    pub fn clear(&self) -> PyResult<()> {
        Err(PyTypeError::new_err(MUTATION_ERROR))
    }

    pub fn popitem(&self) -> PyResult<Obj> {
        Err(PyTypeError::new_err(MUTATION_ERROR))
    }

    /// ``__init_subclass__`` hook - no-op by default, may be overridden.
    #[classmethod]
    #[pyo3(signature = (**_kwargs))]
    pub fn __init_subclass__(
        _cls: &Bound<'_, PyType>,
        _kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<()> {
        Ok(())
    }
}

/// Set-like view over the keys of a :class:`FrozenDict` (insertion order).
///
/// Returned by :meth:`FrozenDict.keys`.
/// Supports ``&``, ``|``, ``-``, ``^``, ``isdisjoint``.
#[pyclass(name = "FrozenKeysView", frozen)]
pub struct FrozenKeysView {
    inner: Arc<FrozenDictInner>,
}

#[pymethods]
impl FrozenKeysView {
    pub fn __len__(&self) -> usize {
        self.inner.entries.len()
    }

    pub fn __iter__(&self, py: Python<'_>) -> PyResult<Obj> {
        Ok(self
            .inner
            .get_keys(py)?
            .bind(py)
            .try_iter()?
            .into_any()
            .unbind())
    }

    pub fn __contains__(&self, py: Python<'_>, key: &Bound<'_, PyAny>) -> PyResult<bool> {
        let hash = key.hash()?;
        let lk = &self.inner.lookup;
        match lk.binary_search_by(|&(h, _)| h.cmp(&hash)) {
            Err(_) => Ok(false),
            Ok(idx) => {
                let mut i = idx;
                while i > 0 && lk[i - 1].0 == hash {
                    i -= 1;
                }
                while i < lk.len() && lk[i].0 == hash {
                    let ei = lk[i].1 as usize;
                    if self.inner.entries[ei].1.bind(py).eq(key)? {
                        return Ok(true);
                    }
                    i += 1;
                }
                Ok(false)
            }
        }
    }

    pub fn __and__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<Obj> {
        Ok(list_to_frozenset(py, self.inner.get_keys(py)?)?
            .call_method1("__and__", (other,))?
            .unbind())
    }
    pub fn __or__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<Obj> {
        Ok(list_to_frozenset(py, self.inner.get_keys(py)?)?
            .call_method1("__or__", (other,))?
            .unbind())
    }
    pub fn __sub__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<Obj> {
        Ok(list_to_frozenset(py, self.inner.get_keys(py)?)?
            .call_method1("__sub__", (other,))?
            .unbind())
    }
    pub fn __xor__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<Obj> {
        Ok(list_to_frozenset(py, self.inner.get_keys(py)?)?
            .call_method1("__xor__", (other,))?
            .unbind())
    }

    pub fn isdisjoint(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<bool> {
        for item in other.try_iter()? {
            if self.__contains__(py, &item?)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        Ok(format!(
            "frozendict_keys({})",
            self.inner.get_keys(py)?.bind(py).repr()?
        ))
    }
}

/// Sequence-like view over the values of a :class:`FrozenDict` (insertion order).
///
/// Returned by :meth:`FrozenDict.values`.
#[pyclass(name = "FrozenValuesView", frozen)]
pub struct FrozenValuesView {
    inner: Arc<FrozenDictInner>,
}

#[pymethods]
impl FrozenValuesView {
    pub fn __len__(&self) -> usize {
        self.inner.entries.len()
    }

    pub fn __iter__(&self, py: Python<'_>) -> PyResult<Obj> {
        Ok(self
            .inner
            .get_values(py)?
            .bind(py)
            .try_iter()?
            .into_any()
            .unbind())
    }

    pub fn __contains__(&self, py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<bool> {
        for (_, _, v) in self.inner.entries.iter() {
            if v.bind(py).eq(value)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        Ok(format!(
            "frozendict_values({})",
            self.inner.get_values(py)?.bind(py).repr()?
        ))
    }
}

/// Set-like view over the ``(key, value)`` items of a :class:`FrozenDict`.
///
/// Returned by :meth:`FrozenDict.items`.
/// Supports ``&``, ``|``, ``-``, ``^``, ``isdisjoint``.
#[pyclass(name = "FrozenItemsView", frozen)]
pub struct FrozenItemsView {
    inner: Arc<FrozenDictInner>,
}

#[pymethods]
impl FrozenItemsView {
    pub fn __len__(&self) -> usize {
        self.inner.entries.len()
    }

    pub fn __iter__(&self, py: Python<'_>) -> PyResult<Obj> {
        Ok(self
            .inner
            .get_items(py)?
            .bind(py)
            .try_iter()?
            .into_any()
            .unbind())
    }

    pub fn __contains__(&self, py: Python<'_>, item: &Bound<'_, PyAny>) -> PyResult<bool> {
        let pair = match item.cast::<PyTuple>() {
            Ok(t) => t,
            Err(_) => return Ok(false),
        };
        if pair.len() != 2 {
            return Ok(false);
        }
        let key = pair.get_item(0)?;
        let val = pair.get_item(1)?;
        let hash = key.hash()?;
        let lk = &self.inner.lookup;
        match lk.binary_search_by(|&(h, _)| h.cmp(&hash)) {
            Err(_) => Ok(false),
            Ok(idx) => {
                let mut i = idx;
                while i > 0 && lk[i - 1].0 == hash {
                    i -= 1;
                }
                while i < lk.len() && lk[i].0 == hash {
                    let ei = lk[i].1 as usize;
                    if self.inner.entries[ei].1.bind(py).eq(&key)?
                        && self.inner.entries[ei].2.bind(py).eq(&val)?
                    {
                        return Ok(true);
                    }
                    i += 1;
                }
                Ok(false)
            }
        }
    }

    pub fn __and__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<Obj> {
        Ok(list_to_frozenset(py, self.inner.get_items(py)?)?
            .call_method1("__and__", (other,))?
            .unbind())
    }
    pub fn __or__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<Obj> {
        Ok(list_to_frozenset(py, self.inner.get_items(py)?)?
            .call_method1("__or__", (other,))?
            .unbind())
    }
    pub fn __sub__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<Obj> {
        Ok(list_to_frozenset(py, self.inner.get_items(py)?)?
            .call_method1("__sub__", (other,))?
            .unbind())
    }
    pub fn __xor__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<Obj> {
        Ok(list_to_frozenset(py, self.inner.get_items(py)?)?
            .call_method1("__xor__", (other,))?
            .unbind())
    }

    pub fn isdisjoint(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<bool> {
        for item in other.try_iter()? {
            if self.__contains__(py, &item?)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        Ok(format!(
            "frozendict_items({})",
            self.inner.get_items(py)?.bind(py).repr()?
        ))
    }
}

/// Register all Python-exposed types into the `_frozndict` extension module.
pub fn register_python_module(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<FrozenDict>()?;
    m.add_class::<FrozenKeysView>()?;
    m.add_class::<FrozenValuesView>()?;
    m.add_class::<FrozenItemsView>()?;
    Ok(())
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

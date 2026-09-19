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
//! entries: Box<[(python_hash, key, value)]>  ← insertion-ordered
//! lookup:  Box<[(python_hash, u32)]>          ← hash-sorted; O(log n) fallback
//! hash:    isize                              ← O(1) pre-computed aggregate
//! cached_keys/values/items: OnceLock         ← built lazily on first access
//! ```
//!
//! ## Key optimisations
//!
//! | Technique | Effect |
//! |---|---|
//! | `Arc<FrozenDictInner>` shared between copies | O(1) `copy()` / `clone()` |
//! | `OnceLock` lazy caches for keys/values/items | O(1) subsequent iteration |
//! | `LazyLock<Arc<...>>` global empty singleton    | O(1) `frozendict()` construction |
//! | `d.iter()` dict fast-iteration + `k.hash()`  | Avoids Python iterator protocol |
//! | Scalar fast-path in `freeze_value`            | ~3 ns vs ~20 ns for int/str/None |
//! | `partition_point` (lower_bound) for lookup   | No backward-walk correction step |
//! | `Arc::ptr_eq` for self-equality               | O(1) `o == o` |
//! | `#[cold]` on mutation paths                   | LLVM registers go to read paths |
//!
//! ## Complexity Summary
//!
//! | Operation                         | Time        | Notes                          |
//! |-----------------------------------|-------------|--------------------------------|
//! | `frozendict(fd)` (FrozenDict src) | O(1)        | Arc clone                      |
//! | `frozendict(dict)` (n entries)    | O(n log n)  | sort + O(n) dedup scan         |
//! | `frozendict(**kwargs)`            | O(n log n)  |                                |
//! | `__getitem__` / `get`             | O(log n)    | partition_point on lookup      |
//! | `__contains__`                    | O(log n)    |                                |
//! | `__hash__`                        | O(1)        | pre-computed                   |
//! | `__eq__` (same object)            | O(1)        | Arc pointer equality           |
//! | `__eq__` (hash mismatch)          | O(1)        | cached hash short-circuit      |
//! | `__eq__` (full compare)           | O(n)        |                                |
//! | `keys()` / `values()` / `items()` | O(1)        | view wrapping Arc              |
//! | First iteration (lazy init)       | O(n)        | builds PyList once             |
//! | Subsequent iterations             | O(n)        | traverse pre-built PyList      |
//! | `copy()`                          | O(1)        | Arc clone                      |

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
use std::sync::{Arc, LazyLock, OnceLock};

/// The error message raised by every mutating method.
const MUTATION_ERROR: &str = "'frozendict' object does not support mutation";

/// The error message raised by `__dir__`.
const ACCESS_DENIED: &str = "Access is Denied!";

/// Multiplicative constant for the key-hash contribution to the aggregate hash.
/// (first 64 bits of the golden-ratio constant × 2^64)
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
/// Since only one OS thread can hold the Python GIL at a time, `OnceLock`
/// guarantees exactly-once initialisation without risk of deadlock.
struct FrozenDictInner {
    /// Insertion-ordered `(python_hash, key, value)` triples.
    entries: Box<[(isize, Obj, Obj)]>,

    /// Hash-sorted `(python_hash, entry_index)` pairs for O(log n) lookup.
    lookup: Box<[(isize, u32)]>,

    /// Pre-computed aggregate hash of all `(key, value)` pairs.
    hash: isize,

    /// Lazily-built `PyList` of all keys (in insertion order).
    cached_keys: OnceLock<Obj>,

    /// Lazily-built `PyList` of all values (in insertion order).
    cached_values: OnceLock<Obj>,

    /// Lazily-built `PyList` of `(key, value)` `PyTuple`s (in insertion order).
    cached_items: OnceLock<Obj>,
}

impl FrozenDictInner {
    /// Return (and build if necessary) the lazily-initialised keys list.
    ///
    /// Uses a manual try-init pattern with `OnceLock` because
    /// `get_or_try_init` is not yet stable (the race is harmless: GIL
    /// ensures single-threaded Python access anyway).
    fn get_keys(&self, py: Python<'_>) -> PyResult<&Obj> {
        if let Some(v) = self.cached_keys.get() {
            return Ok(v);
        }
        let list = PyList::new(py, self.entries.iter().map(|(_, k, _)| k.clone_ref(py)))?
            .into_any()
            .unbind();
        let _ = self.cached_keys.set(list);
        Ok(self.cached_keys.get().unwrap())
    }

    /// Return (and build if necessary) the lazily-initialised values list.
    fn get_values(&self, py: Python<'_>) -> PyResult<&Obj> {
        if let Some(v) = self.cached_values.get() {
            return Ok(v);
        }
        let list = PyList::new(py, self.entries.iter().map(|(_, _, v)| v.clone_ref(py)))?
            .into_any()
            .unbind();
        let _ = self.cached_values.set(list);
        Ok(self.cached_values.get().unwrap())
    }

    /// Return (and build if necessary) the lazily-initialised items list.
    fn get_items(&self, py: Python<'_>) -> PyResult<&Obj> {
        if let Some(v) = self.cached_items.get() {
            return Ok(v);
        }
        let tuples: Vec<Obj> = self
            .entries
            .iter()
            .map(|(_, k, v)| {
                PyTuple::new(py, [k.clone_ref(py), v.clone_ref(py)]).map(|t| t.into_any().unbind())
            })
            .collect::<PyResult<_>>()?;
        let list = PyList::new(py, tuples)?.into_any().unbind();
        let _ = self.cached_items.set(list);
        Ok(self.cached_items.get().unwrap())
    }
}

/// Global singleton for the empty FrozenDict: zero-allocation `frozendict()`.
///
/// On first call (lazy, thread-safe) allocates one `FrozenDictInner` and
/// keeps it alive for the process lifetime.  All empty `FrozenDict` instances
/// share this `Arc`.
static EMPTY_INNER: LazyLock<Arc<FrozenDictInner>> = LazyLock::new(|| {
    Arc::new(FrozenDictInner {
        entries: Box::new([]),
        lookup: Box::new([]),
        hash: 0,
        cached_keys: OnceLock::new(),
        cached_values: OnceLock::new(),
        cached_items: OnceLock::new(),
    })
});

/// A fully immutable, hashable Python dictionary with insertion-order semantics.
///
/// See the crate-level documentation for the full API description, complexity
/// table, and usage examples.
#[pyclass(name = "FrozenDict", frozen, subclass)]
pub struct FrozenDict {
    inner: Arc<FrozenDictInner>,
}

impl FrozenDict {
    /// Core constructor: builds a [`FrozenDictInner`] from pre-hashed pairs.
    ///
    /// **3 allocations** (entries, lookup, OnceLock wrappers: all zero-cost).
    ///
    /// - No-dup path (plain `dict` source): O(n log n) sort + O(n) forward pass.
    /// - Dup path (kwargs/mapping): same sort + O(k) equality scan per hash-collision run.
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

        let mut entries: Vec<(isize, Obj, Obj)> = Vec::with_capacity(n);
        let mut lookup: Vec<(isize, u32)> = Vec::with_capacity(n);
        let mut combined: u64 = 0;

        if !may_have_dups {
            let mut ordered = tagged;
            ordered.sort_unstable_by_key(|t| t.0);
            for (entry_idx, (_, h, k, v)) in ordered.iter().enumerate() {
                let v_hash = v.bind(py).hash()? as u64;
                combined ^= (*h as u64).wrapping_mul(MIX_KEY) ^ v_hash.wrapping_mul(MIX_VAL);
                entries.push((*h, k.clone_ref(py), v.clone_ref(py)));
                lookup.push((*h, entry_idx as u32));
            }
        } else {
            let mut unique: Vec<(u32, isize, Obj, Obj)> = Vec::with_capacity(n);
            let mut i = 0;
            while i < tagged.len() {
                let cur_hash = tagged[i].1;
                let mut j = i + 1;
                while j < tagged.len() && tagged[j].1 == cur_hash {
                    j += 1;
                }
                let run = &tagged[i..j];
                if run.len() == 1 {
                    let t = &run[0];
                    unique.push((t.0, t.1, t.2.clone_ref(py), t.3.clone_ref(py)));
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
            for (entry_idx, (_, h, k, v)) in unique.into_iter().enumerate() {
                let v_hash = v.bind(py).hash()? as u64;
                combined ^= (h as u64).wrapping_mul(MIX_KEY) ^ v_hash.wrapping_mul(MIX_VAL);
                lookup.push((h, entry_idx as u32));
                entries.push((h, k, v));
            }
        }
        lookup.sort_unstable_by_key(|&(h, _)| h);

        Ok(Arc::new(FrozenDictInner {
            entries: entries.into_boxed_slice(),
            lookup: lookup.into_boxed_slice(),
            hash: combined as isize,
            cached_keys: OnceLock::new(),
            cached_values: OnceLock::new(),
            cached_items: OnceLock::new(),
        }))
    }

    /// Hashes each key then delegates to [`build_inner`].
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

    /// Wraps [`from_pairs`] returning a new Python `FrozenDict` object.
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
    /// Uses `partition_point` to land directly at the **start** of a collision
    /// run: no backward-walk correction needed.  The hit path is marked with
    /// `#[inline]`; the cold miss path is guided by LLVM's branch probability.
    ///
    /// # Returns
    ///
    /// `Some(entry_index)` if found, `None` otherwise.
    #[inline]
    fn find_entry(
        &self,
        py: Python<'_>,
        key: &Bound<'_, PyAny>,
        hash: isize,
    ) -> PyResult<Option<usize>> {
        let lk = &self.inner.lookup;
        let start = lk.partition_point(|&(h, _)| h < hash);
        let mut i = start;
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

/// Recursively converts unhashable containers into hashable equivalents.
///
/// - `dict`  → `FrozenDict`
/// - `list`  → `tuple`
/// - `set`   → `frozenset`
/// - Everything else is returned as-is.
///
/// **Scalar fast-path**: `is_instance_of` checks for `int`, `str`, `float`,
/// `bytes`, `bool`, and `None` using pyo3's safe type system, ~3 ns for the
/// common case, skipping all cast attempts below.
#[inline]
fn freeze_value(val: Bound<'_, PyAny>) -> PyResult<Obj> {
    use pyo3::types::{PyBool, PyBytes, PyFloat, PyInt, PyString};

    if val.is_instance_of::<PyInt>()
        || val.is_instance_of::<PyString>()
        || val.is_instance_of::<PyBool>()
        || val.is_instance_of::<PyFloat>()
        || val.is_instance_of::<PyBytes>()
        || val.is_none()
    {
        return Ok(val.unbind());
    }

    let py = val.py();
    if let Ok(d) = val.extract::<Py<PyDict>>() {
        let d = d.bind(py);
        let pairs: Vec<(Obj, Obj)> = d
            .iter()
            .map(|(k, v)| Ok((k.unbind(), freeze_value(v)?)))
            .collect::<PyResult<Vec<(Obj, Obj)>>>()?;
        return FrozenDict::into_py_object(py, pairs, false);
    }
    if val.is_instance_of::<FrozenDict>() {
        return Ok(val.unbind());
    }
    if let Ok(lst) = val.extract::<Py<PyList>>() {
        let lst = lst.bind(py);
        let els: Vec<Obj> = lst
            .iter()
            .map(freeze_value)
            .collect::<PyResult<Vec<Obj>>>()?;
        return Ok(PyTuple::new(py, els)?.into_any().unbind());
    }
    if let Ok(s) = val.extract::<Py<PySet>>() {
        let s = s.bind(py);
        let items: Vec<Obj> = s.iter().map(|x| x.unbind()).collect();
        let fs = PyFrozenSet::new(py, items.iter().map(|o| o.bind(py)))?;
        return Ok(fs.into_any().unbind());
    }
    Ok(val.unbind())
}

/// Collects `(key, value)` pairs from any Python source.
///
/// Returns `(pairs, may_have_dups)`.
fn extract_pairs(source: &Bound<'_, PyAny>) -> PyResult<(Vec<(Obj, Obj)>, bool)> {
    let py = source.py();

    if let Ok(d) = source.extract::<Py<PyDict>>() {
        let d = d.bind(py);
        let pairs: Vec<(Obj, Obj)> = d
            .iter()
            .map(|(k, v)| Ok((k.unbind(), freeze_value(v)?)))
            .collect::<PyResult<Vec<(Obj, Obj)>>>()?;
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
    for item in source.try_iter()? {
        let item = item?;
        let tup = item.extract::<Py<PyTuple>>()?.into_bound(py);
        if tup.len() != 2 {
            return Err(PyTypeError::new_err(
                "frozendict: each iterable item must be a 2-tuple",
            ));
        }
        let k = tup.get_item(0)?;
        let v = tup.get_item(1)?;
        pairs.push((k.unbind(), freeze_value(v)?));
    }
    Ok((pairs, true))
}

/// An immutable view over a [`FrozenDict`]'s keys.
#[pyclass(name = "FrozenKeysView", frozen)]
struct FrozenKeysView {
    inner: Arc<FrozenDictInner>,
}

#[pymethods]
impl FrozenKeysView {
    fn __len__(&self) -> usize {
        self.inner.entries.len()
    }

    fn __contains__(&self, py: Python<'_>, key: &Bound<'_, PyAny>) -> PyResult<bool> {
        let hash = key.hash()?;
        let lk = &self.inner.lookup;
        let start = lk.partition_point(|&(h, _)| h < hash);
        let mut i = start;
        while i < lk.len() && lk[i].0 == hash {
            let ei = lk[i].1 as usize;
            if self.inner.entries[ei].1.bind(py).eq(key)? {
                return Ok(true);
            }
            i += 1;
        }
        Ok(false)
    }

    fn __iter__(&self, py: Python<'_>) -> PyResult<Py<PyIterator>> {
        let list = self.inner.get_keys(py)?.bind(py).clone();
        Ok(list.try_iter()?.unbind())
    }

    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        let keys: Vec<String> = self
            .inner
            .entries
            .iter()
            .map(|(_, k, _)| k.bind(py).repr().map(|s| s.to_string()))
            .collect::<PyResult<_>>()?;
        Ok(format!("frozendict_keys([{}])", keys.join(", ")))
    }

    /// Returns `True` if the keys view and `other` have no elements in common.
    ///
    /// Accepts any iterable as `other`.
    fn isdisjoint(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<bool> {
        for item in other.try_iter()? {
            let item = item?;
            if self.__contains__(py, &item)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Returns the intersection of the keys view with `other` as a `frozenset`.
    fn __and__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<Py<PyFrozenSet>> {
        let result: Vec<Obj> = self
            .inner
            .entries
            .iter()
            .filter_map(|(_, k, _)| {
                let key = k.bind(py);
                other
                    .contains(key)
                    .ok()
                    .and_then(|c| c.then(|| k.clone_ref(py)))
            })
            .collect();
        PyFrozenSet::new(py, result.iter().map(|o| o.bind(py))).map(|b| b.unbind())
    }

    /// Returns the union of the keys view with `other` as a `frozenset`.
    fn __or__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<Py<PyFrozenSet>> {
        let mut items: Vec<Obj> = self
            .inner
            .entries
            .iter()
            .map(|(_, k, _)| k.clone_ref(py))
            .collect();
        for item in other.try_iter()? {
            items.push(item?.unbind());
        }
        PyFrozenSet::new(py, items.iter().map(|o| o.bind(py))).map(|b| b.unbind())
    }

    /// Returns the difference (self − other) as a `frozenset`.
    fn __sub__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<Py<PyFrozenSet>> {
        let result: Vec<Obj> = self
            .inner
            .entries
            .iter()
            .filter_map(|(_, k, _)| {
                let key = k.bind(py);
                other
                    .contains(key)
                    .ok()
                    .and_then(|c| (!c).then(|| k.clone_ref(py)))
            })
            .collect();
        PyFrozenSet::new(py, result.iter().map(|o| o.bind(py))).map(|b| b.unbind())
    }

    /// Returns the symmetric difference (self △ other) as a `frozenset`.
    fn __xor__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<Py<PyFrozenSet>> {
        let self_set: Py<PyFrozenSet> = self.__sub__(py, other)?;
        let mut result: Vec<Obj> = self_set.bind(py).iter().map(|o| o.unbind()).collect();
        for item in other.try_iter()? {
            let item = item?;
            if !self.__contains__(py, &item)? {
                result.push(item.unbind());
            }
        }
        PyFrozenSet::new(py, result.iter().map(|o| o.bind(py))).map(|b| b.unbind())
    }
}

/// An immutable view over a [`FrozenDict`]'s values.
#[pyclass(name = "FrozenValuesView", frozen)]
struct FrozenValuesView {
    inner: Arc<FrozenDictInner>,
}

#[pymethods]
impl FrozenValuesView {
    fn __len__(&self) -> usize {
        self.inner.entries.len()
    }

    fn __iter__(&self, py: Python<'_>) -> PyResult<Py<PyIterator>> {
        let list = self.inner.get_values(py)?.bind(py).clone();
        Ok(list.try_iter()?.unbind())
    }

    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        let vals: Vec<String> = self
            .inner
            .entries
            .iter()
            .map(|(_, _, v)| v.bind(py).repr().map(|s| s.to_string()))
            .collect::<PyResult<_>>()?;
        Ok(format!("frozendict_values([{}])", vals.join(", ")))
    }
}

/// An immutable view over a [`FrozenDict`]'s `(key, value)` pairs.
#[pyclass(name = "FrozenItemsView", frozen)]
struct FrozenItemsView {
    inner: Arc<FrozenDictInner>,
}

#[pymethods]
impl FrozenItemsView {
    fn __len__(&self) -> usize {
        self.inner.entries.len()
    }

    fn __contains__(&self, py: Python<'_>, item: &Bound<'_, PyAny>) -> PyResult<bool> {
        let Ok(tup) = item.extract::<Py<PyTuple>>() else {
            return Ok(false);
        };
        let tup = tup.bind(py);
        if tup.len() != 2 {
            return Ok(false);
        }
        let key = tup.get_item(0)?;
        let val = tup.get_item(1)?;
        let hash = key.hash()?;
        let lk = &self.inner.lookup;
        let start = lk.partition_point(|&(h, _)| h < hash);
        let mut i = start;
        while i < lk.len() && lk[i].0 == hash {
            let ei = lk[i].1 as usize;
            let entry = &self.inner.entries[ei];
            if entry.1.bind(py).eq(&key)? && entry.2.bind(py).eq(&val)? {
                return Ok(true);
            }
            i += 1;
        }
        Ok(false)
    }

    fn __iter__(&self, py: Python<'_>) -> PyResult<Py<PyIterator>> {
        let list = self.inner.get_items(py)?.bind(py).clone();
        Ok(list.try_iter()?.unbind())
    }

    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        let items: Vec<String> = self
            .inner
            .entries
            .iter()
            .map(|(_, k, v)| {
                let ks = k.bind(py).repr()?.to_string();
                let vs = v.bind(py).repr()?.to_string();
                Ok(format!("({ks}, {vs})"))
            })
            .collect::<PyResult<_>>()?;
        Ok(format!("frozendict_items([{}])", items.join(", ")))
    }

    /// Returns `True` if the items view and `other` have no elements in common.
    fn isdisjoint(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<bool> {
        for item in other.try_iter()? {
            let item = item?;
            if self.__contains__(py, &item)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Returns the intersection of the items view with `other` as a `frozenset`.
    fn __and__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<Py<PyFrozenSet>> {
        let result: Vec<Obj> = self
            .inner
            .entries
            .iter()
            .filter_map(|(_, k, v)| {
                let tup = PyTuple::new(py, [k.bind(py), v.bind(py)]).ok()?;
                let tup_any = tup.as_any();
                other
                    .contains(tup_any)
                    .ok()
                    .and_then(|c| c.then(|| tup.into_any().unbind()))
            })
            .collect();
        PyFrozenSet::new(py, result.iter().map(|o| o.bind(py))).map(|b| b.unbind())
    }

    /// Returns the union of the items view with `other` as a `frozenset`.
    fn __or__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<Py<PyFrozenSet>> {
        let mut items: Vec<Obj> = self
            .inner
            .entries
            .iter()
            .map(|(_, k, v)| {
                PyTuple::new(py, [k.bind(py), v.bind(py)]).map(|t| t.into_any().unbind())
            })
            .collect::<PyResult<_>>()?;
        for item in other.try_iter()? {
            items.push(item?.unbind());
        }
        PyFrozenSet::new(py, items.iter().map(|o| o.bind(py))).map(|b| b.unbind())
    }

    /// Returns the difference (self − other) as a `frozenset`.
    fn __sub__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<Py<PyFrozenSet>> {
        let result: Vec<Obj> = self
            .inner
            .entries
            .iter()
            .filter_map(|(_, k, v)| {
                let tup = PyTuple::new(py, [k.bind(py), v.bind(py)]).ok()?;
                let tup_any = tup.as_any();
                other
                    .contains(tup_any)
                    .ok()
                    .and_then(|c| (!c).then(|| tup.into_any().unbind()))
            })
            .collect();
        PyFrozenSet::new(py, result.iter().map(|o| o.bind(py))).map(|b| b.unbind())
    }

    /// Returns the symmetric difference (self △ other) as a `frozenset`.
    fn __xor__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<Py<PyFrozenSet>> {
        let self_set = self.__sub__(py, other)?;
        let mut result: Vec<Obj> = self_set.bind(py).iter().map(|o| o.unbind()).collect();
        for item in other.try_iter()? {
            let item = item?;
            if !self.__contains__(py, &item)? {
                result.push(item.unbind());
            }
        }
        PyFrozenSet::new(py, result.iter().map(|o| o.bind(py))).map(|b| b.unbind())
    }
}

#[pymethods]
impl FrozenDict {
    /// `frozendict(mapping_or_iterable=None, **kwargs)`
    ///
    /// Equivalent to `dict()` but the result is deeply immutable and hashable.
    #[new]
    #[pyo3(signature = (source=None, **kwargs))]
    fn __new__(
        py: Python<'_>,
        source: Option<&Bound<'_, PyAny>>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Self> {
        if source.is_none() && kwargs.is_none_or(|k| k.is_empty()) {
            return Ok(Self {
                inner: Arc::clone(&EMPTY_INNER),
            });
        }

        if kwargs.is_none_or(|k| k.is_empty())
            && let Some(fd) = source.and_then(|src| src.extract::<PyRef<'_, FrozenDict>>().ok())
        {
            return Ok(Self {
                inner: Arc::clone(&fd.inner),
            });
        }

        let mut all_pairs: Vec<(Obj, Obj)> = Vec::new();
        let mut may_have_dups = false;
        if let Some(src) = source {
            let (pairs, dups) = extract_pairs(src)?;
            all_pairs.extend(pairs);
            may_have_dups |= dups;
        }
        if kwargs.is_some_and(|kw| !kw.is_empty()) {
            let kw = kwargs.unwrap();
            all_pairs.extend(kw.iter().map(|(k, v)| (k.unbind(), v.unbind())));
            may_have_dups = true;
        }
        Self::from_pairs(py, all_pairs, may_have_dups)
    }

    fn __len__(&self) -> usize {
        self.num_entries()
    }

    fn __bool__(&self) -> bool {
        !self.inner.entries.is_empty()
    }

    fn __contains__(&self, py: Python<'_>, key: &Bound<'_, PyAny>) -> PyResult<bool> {
        let hash = key.hash()?;
        Ok(self.find_entry(py, key, hash)?.is_some())
    }

    fn __getitem__(&self, py: Python<'_>, key: &Bound<'_, PyAny>) -> PyResult<Obj> {
        let hash = key.hash()?;
        match self.find_entry(py, key, hash)? {
            Some(i) => Ok(self.inner.entries[i].2.clone_ref(py)),
            None => Err(PyKeyError::new_err(key.clone().unbind())),
        }
    }

    fn __iter__(&self, py: Python<'_>) -> PyResult<Py<PyIterator>> {
        let list = self.inner.get_keys(py)?.bind(py).clone();
        Ok(list.try_iter()?.unbind())
    }

    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        let items: Vec<String> = self
            .inner
            .entries
            .iter()
            .map(|(_, k, v)| {
                let ks = k.bind(py).repr()?.to_string();
                let vs = v.bind(py).repr()?.to_string();
                Ok(format!("{ks}: {vs}"))
            })
            .collect::<PyResult<_>>()?;
        Ok(format!("frozendict({{{}}})", items.join(", ")))
    }

    fn __str__(&self, py: Python<'_>) -> PyResult<String> {
        self.__repr__(py)
    }

    fn __hash__(&self) -> isize {
        self.inner.hash
    }

    fn __eq__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<bool> {
        if let Ok(other_fd) = other.extract::<PyRef<'_, FrozenDict>>() {
            if Arc::ptr_eq(&self.inner, &other_fd.inner) {
                return Ok(true);
            }
            if self.inner.hash != other_fd.inner.hash {
                return Ok(false);
            }
            if self.inner.entries.len() != other_fd.inner.entries.len() {
                return Ok(false);
            }
            for (h, k, v) in self.inner.entries.iter() {
                match other_fd.find_entry(py, k.bind(py), *h)? {
                    None => return Ok(false),
                    Some(i) => {
                        if !other_fd.inner.entries[i].2.bind(py).eq(v.bind(py))? {
                            return Ok(false);
                        }
                    }
                }
            }
            return Ok(true);
        }
        if let Ok(d) = other.extract::<Py<PyDict>>() {
            let d = d.bind(py);
            if d.len() != self.num_entries() {
                return Ok(false);
            }
            for (k, v) in d.iter() {
                let hash = k.hash()?;
                match self.find_entry(py, &k, hash)? {
                    None => return Ok(false),
                    Some(i) => {
                        if !self.inner.entries[i].2.bind(py).eq(&v)? {
                            return Ok(false);
                        }
                    }
                }
            }
            return Ok(true);
        }
        Ok(false)
    }

    fn __ne__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<bool> {
        self.__eq__(py, other).map(|r| !r)
    }

    fn __or__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<Obj> {
        let mut pairs: Vec<(Obj, Obj)> = self
            .pairs()
            .iter()
            .map(|(_, k, v)| (k.clone_ref(py), v.clone_ref(py)))
            .collect();
        let (other_pairs, _) = extract_pairs(other)?;
        pairs.extend(other_pairs);
        Self::into_py_object(py, pairs, true)
    }

    fn __ror__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<Obj> {
        let (mut pairs, _) = extract_pairs(other)?;
        pairs.extend(
            self.pairs()
                .iter()
                .map(|(_, k, v)| (k.clone_ref(py), v.clone_ref(py))),
        );
        Self::into_py_object(py, pairs, true)
    }

    #[cold]
    fn __setitem__(&self, _key: &Bound<'_, PyAny>, _val: &Bound<'_, PyAny>) -> PyResult<()> {
        Err(PyTypeError::new_err(MUTATION_ERROR))
    }

    #[cold]
    fn __delitem__(&self, _key: &Bound<'_, PyAny>) -> PyResult<()> {
        Err(PyTypeError::new_err(MUTATION_ERROR))
    }

    /// Raises [`TypeError`]: `frozendict` attributes are immutable.
    #[cold]
    fn __setattr__(&self, _name: &str, _value: &Bound<'_, PyAny>) -> PyResult<()> {
        Err(PyTypeError::new_err(MUTATION_ERROR))
    }

    /// Raises [`TypeError`]: `frozendict` attributes cannot be deleted.
    #[cold]
    fn __delattr__(&self, _name: &str) -> PyResult<()> {
        Err(PyTypeError::new_err(MUTATION_ERROR))
    }

    /// Raises [`TypeError`]: `frozendict` does not support `update()`.
    #[cold]
    #[pyo3(signature = (*_args, **_kwargs))]
    fn update(&self, _args: &Bound<'_, PyAny>, _kwargs: Option<&Bound<'_, PyAny>>) -> PyResult<()> {
        Err(PyTypeError::new_err(MUTATION_ERROR))
    }

    /// Raises [`TypeError`]: `frozendict` does not support `clear()`.
    #[cold]
    fn clear(&self) -> PyResult<()> {
        Err(PyTypeError::new_err(MUTATION_ERROR))
    }

    /// Raises [`TypeError`]: `frozendict` does not support `pop()`.
    #[cold]
    #[pyo3(signature = (*_args))]
    fn pop(&self, _args: &Bound<'_, PyAny>) -> PyResult<()> {
        Err(PyTypeError::new_err(MUTATION_ERROR))
    }

    /// Raises [`TypeError`]: `frozendict` does not support `popitem()`.
    #[cold]
    fn popitem(&self) -> PyResult<()> {
        Err(PyTypeError::new_err(MUTATION_ERROR))
    }

    /// Raises [`TypeError`]: `frozendict` does not support `setdefault()`.
    #[cold]
    #[pyo3(signature = (*_args))]
    fn setdefault(&self, _args: &Bound<'_, PyAny>) -> PyResult<()> {
        Err(PyTypeError::new_err(MUTATION_ERROR))
    }

    /// Returns a reverse iterator over the keys in reverse insertion order.
    fn __reversed__(&self, py: Python<'_>) -> PyResult<Py<PyIterator>> {
        let keys_list = self.inner.get_keys(py)?.bind(py);
        let mut reversed_keys: Vec<Obj> = Vec::new();
        for item in keys_list.try_iter()? {
            reversed_keys.push(item?.unbind());
        }
        reversed_keys.reverse();
        let rev_list = PyList::new(py, reversed_keys.iter().map(|o| o.bind(py)))?;
        Ok(rev_list.try_iter()?.unbind())
    }

    #[pyo3(signature = (key, default=None))]
    fn get(
        &self,
        py: Python<'_>,
        key: &Bound<'_, PyAny>,
        default: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Obj> {
        let hash = key.hash()?;
        match self.find_entry(py, key, hash)? {
            Some(i) => Ok(self.inner.entries[i].2.clone_ref(py)),
            None => Ok(default
                .map(|d| d.clone().unbind())
                .unwrap_or_else(|| py.None())),
        }
    }

    fn keys(&self, py: Python<'_>) -> PyResult<Py<FrozenKeysView>> {
        let view = FrozenKeysView {
            inner: Arc::clone(&self.inner),
        };
        Py::new(py, view)
    }

    fn values(&self, py: Python<'_>) -> PyResult<Py<FrozenValuesView>> {
        let view = FrozenValuesView {
            inner: Arc::clone(&self.inner),
        };
        Py::new(py, view)
    }

    fn items(&self, py: Python<'_>) -> PyResult<Py<FrozenItemsView>> {
        let view = FrozenItemsView {
            inner: Arc::clone(&self.inner),
        };
        Py::new(py, view)
    }

    fn copy(&self, py: Python<'_>) -> PyResult<Obj> {
        Ok(Py::new(
            py,
            Self {
                inner: Arc::clone(&self.inner),
            },
        )?
        .into_any())
    }

    fn __copy__(&self, py: Python<'_>) -> PyResult<Obj> {
        self.copy(py)
    }

    fn __deepcopy__(&self, py: Python<'_>, _memo: &Bound<'_, PyAny>) -> PyResult<Obj> {
        self.copy(py)
    }

    fn __reduce__(&self, py: Python<'_>) -> PyResult<Obj> {
        let cls = py.get_type::<FrozenDict>().into_any().unbind();
        let d = PyDict::new(py);
        for (_, k, v) in self.inner.entries.iter() {
            d.set_item(k.bind(py), v.bind(py))?;
        }
        let args = PyTuple::new(py, [d.into_any().unbind()])?;
        let args_obj = args.into_any().unbind();
        Ok(PyTuple::new(py, [cls, args_obj])?.into_any().unbind())
    }

    /// Returns a `types.GenericAlias` for `frozendict[K, V]` syntax support.
    #[classmethod]
    fn __class_getitem__(cls: &Bound<'_, PyType>, item: Obj) -> PyResult<Obj> {
        let types = cls.py().import("types")?;
        let generic_alias = types.getattr("GenericAlias")?;
        Ok(generic_alias.call1((cls, item))?.unbind())
    }

    /// Returns a `frozendict` mapped from `keys` to `value` (default `None`).
    ///
    /// Mirrors `dict.fromkeys`. When called on a subclass, returns an instance
    /// of that subclass.
    #[classmethod]
    #[pyo3(signature = (keys, value=None))]
    fn fromkeys(
        cls: &Bound<'_, PyType>,
        py: Python<'_>,
        keys: &Bound<'_, PyAny>,
        value: Option<Obj>,
    ) -> PyResult<Obj> {
        let value = value.unwrap_or_else(|| py.None());
        let mut pairs: Vec<(Obj, Obj)> = Vec::new();
        for key in keys.try_iter()? {
            pairs.push((key?.unbind(), value.clone_ref(py)));
        }
        let fd_rust = FrozenDict::from_pairs(py, pairs, true)?;
        let fd = Py::new(py, fd_rust)?.into_any();
        if cls.is(py.get_type::<FrozenDict>()) {
            return Ok(fd);
        }
        cls.call1((fd,)).map(|o| o.unbind())
    }

    fn __dir__(&self) -> PyResult<Vec<&str>> {
        Err(PyAttributeError::new_err(ACCESS_DENIED))
    }

    /// Return a new `FrozenDict` with the given pair added or replaced.
    fn set(&self, py: Python<'_>, key: Obj, value: Obj) -> PyResult<Obj> {
        let mut pairs: Vec<(Obj, Obj)> = self
            .pairs()
            .iter()
            .map(|(_, k, v)| (k.clone_ref(py), v.clone_ref(py)))
            .collect();
        pairs.push((key, value));
        Self::into_py_object(py, pairs, true)
    }

    /// Return a new `FrozenDict` with `key` removed.
    fn delete(&self, py: Python<'_>, key: &Bound<'_, PyAny>) -> PyResult<Obj> {
        let hash = key.hash()?;
        let pairs: Vec<(Obj, Obj)> = self
            .pairs()
            .iter()
            .filter(|(h, k, _)| *h != hash || !k.bind(py).eq(key).unwrap_or(false))
            .map(|(_, k, v)| (k.clone_ref(py), v.clone_ref(py)))
            .collect();
        Self::into_py_object(py, pairs, false)
    }

    /// Merge `other` into `self` (other wins on collision).
    fn merge(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<Obj> {
        let mut pairs: Vec<(Obj, Obj)> = self
            .pairs()
            .iter()
            .map(|(_, k, v)| (k.clone_ref(py), v.clone_ref(py)))
            .collect();
        let (other_pairs, dups) = extract_pairs(other)?;
        pairs.extend(other_pairs);
        Self::into_py_object(py, pairs, dups)
    }

    /// Return a new `FrozenDict` containing only keys in both `self` and `other`.
    fn intersection(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<Obj> {
        let pairs: Vec<(Obj, Obj)> = self
            .pairs()
            .iter()
            .filter(|(_, k, _)| other.get_item(k.bind(py)).is_ok())
            .map(|(_, k, v)| (k.clone_ref(py), v.clone_ref(py)))
            .collect();
        Self::into_py_object(py, pairs, false)
    }

    /// Return a new `FrozenDict` with keys from `other` removed.
    fn difference(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<Obj> {
        let pairs: Vec<(Obj, Obj)> = self
            .pairs()
            .iter()
            .filter(|(_, k, _)| other.get_item(k.bind(py)).is_err())
            .map(|(_, k, v)| (k.clone_ref(py), v.clone_ref(py)))
            .collect();
        Self::into_py_object(py, pairs, false)
    }

    #[pyo3(signature = (num_spaces=4))]
    fn pretty_repr(&self, py: Python<'_>, num_spaces: usize) -> PyResult<String> {
        let indent = " ".repeat(num_spaces);
        let mut out = String::with_capacity(14 + self.num_entries() * (num_spaces + 4));
        out.push_str("frozendict({\n");
        for (_, k, v) in self.inner.entries.iter() {
            out.push_str(&indent);
            out.push_str(&k.bind(py).repr()?.to_string());
            out.push_str(": ");
            out.push_str(&v.bind(py).repr()?.to_string());
            out.push_str(",\n");
        }
        out.push_str("})");
        Ok(out)
    }
}

/// Registers all Python types exported by this module.
pub fn register_python_module(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<FrozenDict>()?;
    m.add_class::<FrozenKeysView>()?;
    m.add_class::<FrozenValuesView>()?;
    m.add_class::<FrozenItemsView>()?;
    let _ = &*EMPTY_INNER;
    let _ = py;
    Ok(())
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

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
//! Every type is gated behind the `python` cargo feature.  The bindings
//! surface a single class, [`FrozenDict`], which is a fully immutable,
//! hashable mapping.
//!
//! ## Immutability guarantees
//!
//! Every mutating dunder method (`__setitem__`, `__delitem__`, `__setattr__`,
//! `__delattr__`, `pop`, `update`, `clear`, `popitem`, `setdefault`) raises
//! [`TypeError`] with the message
//! `"'frozendict' object does not support mutation"`.
//!
//! `__dir__` raises [`AttributeError`] with `"Access is Denied!"` so
//! introspection is blocked just like the pure-Python predecessor.
//!
//! ## Storage
//!
//! Entries are stored as a heap-allocated `Box<[(Py<PyAny>, Py<PyAny>)]>`.
//! Because Python objects are not `Ord`, we cannot sort by key; lookups use a
//! linear equality scan (O(n)).  The Python-level hash of the whole dict is
//! XOR-combined at construction and cached, making all subsequent `hash()`
//! calls O(1).
//!
//! ## See Also
//!
//! - [`crate::frozen_map`] - the generic Rust data structure.

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
use std::collections::HashMap;

/// The error message raised by every mutating method.
const MUTATION_ERROR: &str = "'frozendict' object does not support mutation";

/// The error message raised by `__dir__`.
const ACCESS_DENIED: &str = "Access is Denied!";

/// Convenience alias for an owned reference to any Python object.
type Obj = Py<PyAny>;

/// A fully immutable, hashable Python dictionary.
///
/// `FrozenDict` behaves like a standard Python `dict` for all **read**
/// operations and refuses every **write** operation with a clear error.
///
/// ## Construction
///
/// ```python
/// from frozndict import frozendict
///
/// # from keyword arguments
/// d = frozendict(a=1, b=2)
///
/// # from a positional mapping
/// d = frozendict({"a": 1, "b": 2})
///
/// # from an iterable of pairs
/// d = frozendict([("a", 1), ("b", 2)])
///
/// # fromkeys class method
/// d = frozendict.fromkeys(["x", "y"], 0)
/// ```
///
/// ## Immutability
///
/// Any attempt to mutate the dictionary raises :exc:`TypeError`:
///
/// ```python
/// d["x"] = 1          # TypeError
/// del d["x"]          # TypeError
/// d.update({"x": 1})  # TypeError
/// d.clear()           # TypeError
/// d.pop("x")          # TypeError
/// ```
///
/// `dir(d)` raises :exc:`AttributeError` with ``"Access is Denied!"``.
///
/// ## Hashing
///
/// `FrozenDict` is hashable.  Two `FrozenDict` instances with the same
/// key-value pairs always produce the same hash, regardless of insertion order:
///
/// ```python
/// assert hash(frozendict(a=1, b=2)) == hash(frozendict(b=2, a=1))
/// ```
#[pyclass(name = "FrozenDict", frozen)]
pub struct FrozenDict {
    /// Flat, deduplicated storage: (python_hash, key, value), sorted by hash for O(log n) binary search.
    entries: Box<[(isize, Obj, Obj)]>,

    /// Pre-computed Python hash value, XOR-combined over all `(key, value)` pairs.
    hash: isize,

    /// Pre-built Python list of keys - avoids per-call allocation in keys() / __iter__.
    cached_keys: Obj,

    /// Pre-built Python list of values - avoids per-call allocation in values().
    cached_values: Obj,

    /// Pre-built Python list of (k, v) tuples - avoids per-call allocation in items().
    cached_items: Obj,
}

impl FrozenDict {
    /// Builds a `FrozenDict` from a `Vec` of owned object pairs.
    /// Deduplicates by Python hash + equality (last-write-wins).
    fn from_pairs(py: Python<'_>, pairs: Vec<(Obj, Obj)>) -> PyResult<Self> {
        let mut map: HashMap<isize, Vec<(Obj, Obj)>> = HashMap::with_capacity(pairs.len());

        for (k, v) in pairs {
            let hash = k.bind(py).hash()?;
            let bucket = map.entry(hash).or_default();
            let mut found = false;
            for (ek, ev) in bucket.iter_mut() {
                if ek.bind(py).eq(k.bind(py))? {
                    *ev = v.clone_ref(py);
                    found = true;
                    break;
                }
            }
            if !found {
                bucket.push((k, v));
            }
        }

        let mut deduped: Vec<(isize, Obj, Obj)> = Vec::with_capacity(map.len());
        for (hash, bucket) in map {
            for (k, v) in bucket {
                deduped.push((hash, k, v));
            }
        }
        deduped.sort_unstable_by_key(|&(h, _, _)| h);

        let hash = compute_python_hash(py, &deduped)?;

        let cached_keys = PyList::new(
            py,
            deduped
                .iter()
                .map(|(_, k, _)| k.clone_ref(py))
                .collect::<Vec<_>>(),
        )?
        .into_any()
        .unbind();

        let cached_values = PyList::new(
            py,
            deduped
                .iter()
                .map(|(_, _, v)| v.clone_ref(py))
                .collect::<Vec<_>>(),
        )?
        .into_any()
        .unbind();

        let cached_items = PyList::new(
            py,
            deduped
                .iter()
                .map(|(_, k, v)| {
                    PyTuple::new(py, [k.clone_ref(py), v.clone_ref(py)])
                        .map(|t| t.into_any().unbind())
                })
                .collect::<PyResult<Vec<_>>>()?,
        )?
        .into_any()
        .unbind();

        Ok(Self {
            entries: deduped.into_boxed_slice(),
            hash,
            cached_keys,
            cached_values,
            cached_items,
        })
    }

    /// Wraps `from_pairs` in a new Python object and erases the concrete type.
    #[inline]
    fn into_py_object(py: Python<'_>, pairs: Vec<(Obj, Obj)>) -> PyResult<Obj> {
        Ok(Py::new(py, Self::from_pairs(py, pairs)?)?.into_any())
    }

    #[inline]
    fn num_entries(&self) -> usize {
        self.entries.len()
    }

    #[inline]
    fn pairs(&self) -> &[(isize, Obj, Obj)] {
        &self.entries
    }

    /// Perform a hash-then-equality lookup. Returns the index into `entries` on hit.
    /// Most key types (str, int) have unique hashes - the equality check is only
    /// reached during the rare hash-collision case.
    #[inline]
    fn find_entry(
        &self,
        py: Python<'_>,
        key: &Bound<'_, PyAny>,
        hash: isize,
    ) -> PyResult<Option<usize>> {
        match self.entries.binary_search_by(|&(h, _, _)| h.cmp(&hash)) {
            Err(_) => Ok(None),
            Ok(idx) => {
                let mut i = idx;
                while i > 0 && self.entries[i - 1].0 == hash {
                    i -= 1;
                }
                while i < self.entries.len() && self.entries[i].0 == hash {
                    if self.entries[i].1.bind(py).eq(key)? {
                        return Ok(Some(i));
                    }
                    i += 1;
                }
                Ok(None)
            }
        }
    }
}

/// XOR-combines `hash((k, v))` for all pairs.
///
/// Each pair is hashed as a Python `tuple((key, value))` to preserve
/// Python's own tuple-hashing semantics.  Results are XOR-combined, making
/// the aggregate hash order-independent: two [`FrozenDict`] instances with
/// the same key-value pairs always produce the same hash regardless of
/// insertion order.
///
/// # Complexity
///
/// - Time: O(n) - one Python hash per pair.
/// - Space: O(1) - no extra allocation.
fn compute_python_hash(py: Python<'_>, pairs: &[(isize, Obj, Obj)]) -> PyResult<isize> {
    let mut combined: u64 = 0;
    for (_, k, v) in pairs {
        let pair_hash = PyTuple::new(py, [k, v])?.hash()?;
        combined ^= pair_hash as u64;
    }
    Ok(combined as isize)
}

/// Recursively converts an unhashable Python value into a hashable equivalent.
///
/// Conversion rules (applied depth-first):
///
/// - `dict`  → `FrozenDict` (recurse into keys and values)
/// - `list`  → `tuple`      (recurse into elements)
/// - `set`   → `frozenset`  (elements must already be hashable)
/// - Everything else is returned as-is; if it turns out to be unhashable the
///   error will surface naturally when `hash()` is later called.
///
/// # Complexity
///
/// - Time: O(n) where n is the total number of nested objects.
/// - Space: O(depth) stack + O(n) for newly created containers.
fn freeze_value(val: Bound<'_, PyAny>) -> PyResult<Obj> {
    let py = val.py();

    if let Ok(dict) = val.cast::<PyDict>() {
        let pairs: Vec<(Obj, Obj)> = dict
            .iter()
            .map(|(k, v)| {
                let frozen_v = freeze_value(v)?;
                Ok((k.unbind(), frozen_v))
            })
            .collect::<PyResult<_>>()?;
        return FrozenDict::into_py_object(py, pairs);
    }

    if val.is_instance_of::<FrozenDict>() {
        return Ok(val.unbind());
    }

    if let Ok(list) = val.cast::<PyList>() {
        let elements: Vec<Obj> = list
            .iter()
            .map(|item| freeze_value(item))
            .collect::<PyResult<_>>()?;
        return Ok(PyTuple::new(py, elements)?.into_any().unbind());
    }

    if let Ok(set) = val.cast::<PySet>() {
        let frozen = PyFrozenSet::new(py, set.iter().collect::<Vec<_>>().iter())?;
        return Ok(frozen.into_any().unbind());
    }

    Ok(val.unbind())
}

/// Collects `(key, value)` pairs from any Python mapping or iterable.
///
/// The following sources are accepted, tried in order:
///
/// 1. A plain `dict` - iterated directly via [`PyDict::iter`].
/// 2. A [`FrozenDict`] - clones the pre-existing slice without re-hashing.
/// 3. Any object with a `keys()` method (generic mapping protocol).
/// 4. Any iterable of 2-element tuples - falls back to the sequence protocol.
///
/// This mirrors the behaviour of Python's built-in `dict()` constructor.
///
/// # Errors
///
/// Returns a [`PyTypeError`] if the source is a sequence but an item is not a
/// 2-element tuple.
///
/// # Complexity
///
/// - Time: O(n)
/// - Space: O(n)
fn extract_pairs(source: &Bound<'_, PyAny>) -> PyResult<Vec<(Obj, Obj)>> {
    let py = source.py();
    if let Ok(dict) = source.cast::<PyDict>() {
        return dict
            .iter()
            .map(|(k, v)| Ok((k.unbind(), freeze_value(v)?)))
            .collect::<PyResult<_>>();
    }
    if let Ok(fd) = source.extract::<PyRef<'_, FrozenDict>>() {
        return Ok(fd
            .pairs()
            .iter()
            .map(|(_, k, v)| (k.clone_ref(py), v.clone_ref(py)))
            .collect());
    }
    if source.hasattr("keys")? {
        let keys = source.call_method0("keys")?;
        let mut pairs = Vec::new();
        for k in keys.try_iter()? {
            let k = k?;
            let v = source.get_item(&k)?;
            pairs.push((k.unbind(), freeze_value(v)?));
        }
        return Ok(pairs);
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
    Ok(pairs)
}

#[pymethods]
impl FrozenDict {
    /// Construct a new :class:`FrozenDict`.
    ///
    /// - ``frozendict()`` - empty.
    /// - ``frozendict(mapping)`` - copy from any mapping.
    /// - ``frozendict(iterable)`` - from an iterable of ``(key, value)`` pairs.
    /// - ``frozendict(**kwargs)`` - from keyword arguments.
    /// - ``frozendict(mapping, **kwargs)`` - combination of both.
    #[new]
    #[pyo3(signature = (*args, **kwargs))]
    pub fn __new__(
        py: Python<'_>,
        args: &Bound<'_, PyTuple>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Self> {
        let mut pairs: Vec<(Obj, Obj)> = Vec::new();

        if args.len() > 1 {
            return Err(PyTypeError::new_err(
                "frozendict expected at most 1 positional argument",
            ));
        }
        if let Ok(first) = args.get_item(0) {
            pairs.extend(extract_pairs(&first)?);
        }
        if let Some(kw) = kwargs {
            for (k, v) in kw.iter() {
                pairs.push((k.unbind(), freeze_value(v)?));
            }
        }

        Self::from_pairs(py, pairs)
    }

    /// Return the cached hash of this :class:`FrozenDict`.
    pub fn __hash__(&self) -> isize {
        self.hash
    }

    /// Return the number of entries.
    pub fn __len__(&self) -> usize {
        self.entries.len()
    }

    /// Return ``True`` if ``key`` is present. O(log n) - binary search by hash then equality.
    pub fn __contains__(&self, py: Python<'_>, key: &Bound<'_, PyAny>) -> PyResult<bool> {
        Ok(self.find_entry(py, key, key.hash()?)?.is_some())
    }

    /// Return the value for ``key``, or raise :exc:`KeyError`. O(log n).
    pub fn __getitem__(&self, py: Python<'_>, key: &Bound<'_, PyAny>) -> PyResult<Obj> {
        let hash = key.hash()?;
        match self.find_entry(py, key, hash)? {
            Some(idx) => Ok(self.entries[idx].2.clone_ref(py)),
            None => Err(PyKeyError::new_err(key.clone().unbind())),
        }
    }

    /// Return an iterator over the keys. O(1) - iterator over pre-cached list.
    pub fn __iter__(&self, py: Python<'_>) -> PyResult<Obj> {
        let iterator = self.cached_keys.bind(py).try_iter()?;
        Ok(iterator.into_any().unbind())
    }

    /// Return a string representation.
    ///
    /// Format: ``frozendict({'key': value, ...})``.
    ///
    /// # Complexity
    ///
    /// - Time: O(n) - calls ``repr()`` on every key and value.
    /// - Space: O(n)
    pub fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        let mut parts: Vec<String> = Vec::with_capacity(self.entries.len());
        for (_, k, v) in self.entries.iter() {
            parts.push(format!("{}: {}", k.bind(py).repr()?, v.bind(py).repr()?));
        }
        Ok(format!("frozendict({{{}}})", parts.join(", ")))
    }

    /// Equality against another `FrozenDict` or a plain `dict`.
    pub fn __eq__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<bool> {
        if let Ok(other_fd) = other.extract::<PyRef<'_, FrozenDict>>() {
            if self.num_entries() != other_fd.num_entries() {
                return Ok(false);
            }
            for (_, k1, v1) in self.entries.iter() {
                let hash = k1.bind(py).hash()?;
                let mut found = false;
                if let Ok(idx) = other_fd.entries.binary_search_by(|&(h, _, _)| h.cmp(&hash)) {
                    let mut i = idx;
                    while i > 0 && other_fd.entries[i - 1].0 == hash {
                        i -= 1;
                    }
                    while i < other_fd.entries.len() && other_fd.entries[i].0 == hash {
                        if other_fd.entries[i].1.bind(py).eq(k1.bind(py))? {
                            if !v1.bind(py).eq(other_fd.entries[i].2.bind(py))? {
                                return Ok(false);
                            }
                            found = true;
                            break;
                        }
                        i += 1;
                    }
                }
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
            for (_, k, v) in self.entries.iter() {
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

    /// Return a new :class:`FrozenDict` that is the union of self and ``other``.
    ///
    /// Equivalent to the Python ``|`` operator.  Entries in ``other`` override
    /// entries with the same key from ``self``.
    ///
    /// # Complexity
    ///
    /// - Time: O(m + n) - collecting pairs from both operands.
    /// - Space: O(m + n)
    pub fn __or__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<Obj> {
        let mut pairs: Vec<(Obj, Obj)> = self
            .entries
            .iter()
            .map(|(_, k, v)| (k.clone_ref(py), v.clone_ref(py)))
            .collect();
        pairs.extend(extract_pairs(other)?);
        Self::into_py_object(py, pairs)
    }

    /// Return a list of all keys. O(1) - clones reference to pre-cached list.
    pub fn keys(&self, py: Python<'_>) -> PyResult<Obj> {
        Ok(self.cached_keys.clone_ref(py))
    }

    /// Return a list of all values. O(1) - clones reference to pre-cached list.
    pub fn values(&self, py: Python<'_>) -> PyResult<Obj> {
        Ok(self.cached_values.clone_ref(py))
    }

    /// Return a list of ``(key, value)`` tuples. O(1) - clones reference to pre-cached list.
    pub fn items(&self, py: Python<'_>) -> PyResult<Obj> {
        Ok(self.cached_items.clone_ref(py))
    }

    /// Look up ``key``, returning ``default`` if absent. O(log n).
    #[pyo3(signature = (key, default=None))]
    pub fn get(
        &self,
        py: Python<'_>,
        key: &Bound<'_, PyAny>,
        default: Option<Obj>,
    ) -> PyResult<Obj> {
        let hash = key.hash()?;
        match self.find_entry(py, key, hash)? {
            Some(idx) => Ok(self.entries[idx].2.clone_ref(py)),
            None => Ok(default.unwrap_or_else(|| py.None())),
        }
    }

    /// Return a shallow copy of this :class:`FrozenDict`.
    ///
    /// Because :class:`FrozenDict` is immutable the copy shares the same hash
    /// and is semantically identical to the original.
    ///
    /// # Complexity
    ///
    /// - Time: O(n) - increments refcounts for all stored objects.
    /// - Space: O(n)
    pub fn copy(&self, py: Python<'_>) -> PyResult<Obj> {
        let pairs: Vec<(Obj, Obj)> = self
            .entries
            .iter()
            .map(|(_, k, v)| (k.clone_ref(py), v.clone_ref(py)))
            .collect();
        Self::into_py_object(py, pairs)
    }

    /// Create a :class:`FrozenDict` from an iterable of keys, each mapped to
    /// the same ``value``.
    ///
    /// Mirrors ``dict.fromkeys()``.  If the key iterable contains duplicates,
    /// only one entry is kept (last-write-wins).
    ///
    /// # Complexity
    ///
    /// - Time: O(n)
    /// - Space: O(n)
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
        Self::into_py_object(py, pairs)
    }

    /// Return a pretty-printed, indented string representation.
    ///
    /// ``num_spaces`` (default ``4``) sets the number of leading spaces per
    /// entry.  Output format:
    ///
    /// ```text
    /// frozendict({
    ///     'key': value,
    ///     ...
    /// })
    /// ```
    ///
    /// # Complexity
    ///
    /// - Time: O(n)
    /// - Space: O(n)
    #[pyo3(signature = (num_spaces=4))]
    pub fn pretty_repr(&self, py: Python<'_>, num_spaces: usize) -> PyResult<String> {
        let indent = " ".repeat(num_spaces);
        let mut out = String::from("frozendict({\n");
        for (_, k, v) in self.entries.iter() {
            out.push_str(&format!(
                "{indent}{}: {},\n",
                k.bind(py).repr()?,
                v.bind(py).repr()?
            ));
        }
        out.push_str("})");
        Ok(out)
    }

    /// Raises :exc:`TypeError` - attribute deletion is not permitted.
    pub fn __delattr__(&self, _name: &str) -> PyResult<()> {
        Err(PyTypeError::new_err(MUTATION_ERROR))
    }

    /// Raises :exc:`TypeError` - item deletion is not permitted.
    pub fn __delitem__(&self, _key: &Bound<'_, PyAny>) -> PyResult<()> {
        Err(PyTypeError::new_err(MUTATION_ERROR))
    }

    /// Raises :exc:`TypeError` - attribute assignment is not permitted.
    pub fn __setattr__(&self, _name: &str, _value: &Bound<'_, PyAny>) -> PyResult<()> {
        Err(PyTypeError::new_err(MUTATION_ERROR))
    }

    /// Raises :exc:`TypeError` - item assignment is not permitted.
    pub fn __setitem__(&self, _key: &Bound<'_, PyAny>, _value: &Bound<'_, PyAny>) -> PyResult<()> {
        Err(PyTypeError::new_err(MUTATION_ERROR))
    }

    /// Blocks ``dir()`` introspection.
    pub fn __dir__(&self) -> PyResult<Vec<String>> {
        Err(PyAttributeError::new_err(ACCESS_DENIED))
    }

    /// Raises :exc:`TypeError` - ``pop()`` is not permitted.
    #[pyo3(signature = (*_args, **_kwargs))]
    pub fn pop(
        &self,
        _args: &Bound<'_, PyTuple>,
        _kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Obj> {
        Err(PyTypeError::new_err(MUTATION_ERROR))
    }

    /// Raises :exc:`TypeError` - ``update()`` is not permitted.
    #[pyo3(signature = (*_args, **_kwargs))]
    pub fn update(
        &self,
        _args: &Bound<'_, PyTuple>,
        _kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<()> {
        Err(PyTypeError::new_err(MUTATION_ERROR))
    }

    /// Raises :exc:`TypeError` - ``setdefault()`` is not permitted.
    #[pyo3(signature = (_key, _default=None))]
    pub fn setdefault(&self, _key: &Bound<'_, PyAny>, _default: Option<Obj>) -> PyResult<Obj> {
        Err(PyTypeError::new_err(MUTATION_ERROR))
    }

    /// Raises :exc:`TypeError` - ``clear()`` is not permitted.
    pub fn clear(&self) -> PyResult<()> {
        Err(PyTypeError::new_err(MUTATION_ERROR))
    }

    /// Raises :exc:`TypeError` - ``popitem()`` is not permitted.
    pub fn popitem(&self) -> PyResult<Obj> {
        Err(PyTypeError::new_err(MUTATION_ERROR))
    }
}

/// Register all Python-exposed types into the `_frozndict` extension module.
pub fn register_python_module(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<FrozenDict>()?;
    Ok(())
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

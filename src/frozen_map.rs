// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # FrozenMap
//!
//! This module provides [`FrozenMap<K, V>`], the core immutable key-value data
//! structure that backs the [`crate::python::FrozenDict`] Python class.
//!
//! ## Design Rationale
//!
//! The primary goal is **absolute immutability combined with minimum memory
//! consumption**.  To achieve this, entries are stored as a **sorted, contiguous
//! heap slice** (`Box<[(K, V)]>`).
//!
//! | Property | Detail |
//! |----------|--------|
//! | Storage layout | `Box<[(K, V)]>` - single contiguous allocation |
//! | Lookup complexity | O(log n) - binary search over sorted keys |
//! | Memory overhead | Zero: exactly `n × (size_of::<K>() + size_of::<V>())` bytes |
//! | Cache behaviour | Excellent: sequential memory access during search |
//! | Allocation count | One at construction time; none afterwards |
//! | Mutation | Impossible: no `&mut self` method is exposed |
//!
//! Binary search outperforms hash-map lookups for maps with fewer than ~64
//! entries because it avoids hashing and pointer-chasing overhead.  For larger
//! maps the trade-off is still acceptable given the memory savings: a hash map
//! typically occupies 1.5–2× more memory than it needs.
//!
//! The hash of the entire map is computed once during construction and stored
//! as a `u64` field, making repeated `__hash__()` calls O(1).
//!
//! ## See Also
//!
//! - [`crate::python`] - PyO3 Python bindings built on top of this module.

use ahash::AHasher;
use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;
use std::ops::Index;
use std::slice;
use std::vec;

/// A fully immutable, sorted, flat key-value store.
///
/// `FrozenMap<K, V>` stores entries in a single heap allocation
/// (`Box<[(K, V)]>`) sorted by key.  Lookups use binary search in O(log n).
/// No further allocation or reallocation ever occurs after construction.
///
/// The pre-computed [`FrozenMap::hash`] field makes repeated hashing O(1).
///
/// # Type Parameters
///
/// - `K` - key type; must implement [`Ord`] + [`Hash`].
/// - `V` - value type; must implement [`Hash`].
///
/// # Examples
///
/// ```rust
/// use frozndict::frozen_map::FrozenMap;
///
/// let map: FrozenMap<&str, i32> = FrozenMap::new([("b", 2), ("a", 1)]);
/// assert_eq!(map["a"], 1);
/// assert_eq!(map.len(), 2);
/// ```
#[derive(Clone)]
pub struct FrozenMap<K, V> {
    /// Sorted, flat slice of key-value pairs.
    ///
    /// Keys are sorted in ascending order at construction time.  Binary search
    /// is used for all subsequent lookups.  The slice is heap-allocated once
    /// and never resized.
    entries: Box<[(K, V)]>,

    /// Pre-computed hash of all entries.
    ///
    /// Computed once in [`FrozenMap::new`] by XOR-combining the individual
    /// hashes of each `(key, value)` pair, making the order of insertion
    /// irrelevant.  Stored so that repeated `hash()` calls are O(1).
    hash: u64,
}

impl<K, V> FrozenMap<K, V>
where
    K: Ord + Hash,
    V: Hash,
{
    /// Constructs a new [`FrozenMap`] from an iterator of `(key, value)` pairs.
    ///
    /// Duplicate keys are resolved by **last-write-wins**: if two entries share
    /// the same key only the final one (in iteration order) is kept.  Entries
    /// are sorted by key after deduplication.
    ///
    /// Internally, all pairs are inserted into a [`std::collections::BTreeMap`]
    /// which enforces key-uniqueness (last-write-wins) and returns entries in
    /// ascending key order.  The resulting sorted `Vec` is then frozen into a
    /// `Box<[(K, V)]>`.  This gives O(n log n) construction time with no
    /// `unsafe` code.
    ///
    /// # Complexity
    ///
    /// - Time: O(n log n) - sort dominates.
    /// - Space: O(n) - single contiguous allocation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frozndict::frozen_map::FrozenMap;
    ///
    /// let map: FrozenMap<&str, i32> = FrozenMap::new([("z", 26), ("a", 1)]);
    /// assert_eq!(map["a"], 1);
    /// assert_eq!(map["z"], 26);
    /// ```
    pub fn new(entries: impl IntoIterator<Item = (K, V)>) -> Self {
        let mut map: BTreeMap<K, V> = BTreeMap::new();
        for (k, v) in entries {
            map.insert(k, v);
        }
        let pairs: Vec<(K, V)> = map.into_iter().collect();
        let hash = Self::compute_hash(&pairs);
        Self {
            entries: pairs.into_boxed_slice(),
            hash,
        }
    }

    /// Constructs a [`FrozenMap`] by associating each key in `keys` with
    /// the same `value`.
    ///
    /// This mirrors Python's `dict.fromkeys()` behaviour.  If `keys` yields
    /// duplicate values only one entry is kept (last-write-wins).
    ///
    /// # Complexity
    ///
    /// - Time: O(n log n)
    /// - Space: O(n)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frozndict::frozen_map::FrozenMap;
    ///
    /// let map = FrozenMap::fromkeys(["x", "y", "z"], 0_i32);
    /// assert_eq!(map["x"], 0);
    /// assert_eq!(map.len(), 3);
    /// ```
    pub fn fromkeys(keys: impl IntoIterator<Item = K>, value: V) -> Self
    where
        V: Clone,
    {
        Self::new(keys.into_iter().map(|k| (k, value.clone())))
    }

    /// Returns the number of entries in the map.
    ///
    /// # Complexity
    ///
    /// - Time: O(1)
    /// - Space: O(1)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frozndict::frozen_map::FrozenMap;
    ///
    /// let map: FrozenMap<&str, i32> = FrozenMap::new([("a", 1)]);
    /// assert_eq!(map.len(), 1);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` if the map contains no entries.
    ///
    /// # Complexity
    ///
    /// - Time: O(1)
    /// - Space: O(1)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frozndict::frozen_map::FrozenMap;
    ///
    /// let empty: FrozenMap<&str, i32> = FrozenMap::new([]);
    /// assert!(empty.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns the pre-computed hash of this map.
    ///
    /// The hash is computed once at construction time and cached.
    ///
    /// # Complexity
    ///
    /// - Time: O(1)
    /// - Space: O(1)
    #[inline]
    pub fn precomputed_hash(&self) -> u64 {
        self.hash
    }

    /// Returns a reference to the value associated with the given key, or
    /// `None` if not found.
    ///
    /// Uses binary search.
    ///
    /// # Complexity
    ///
    /// - Time: O(log n)
    /// - Space: O(1)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frozndict::frozen_map::FrozenMap;
    ///
    /// let map: FrozenMap<&str, i32> = FrozenMap::new([("a", 1)]);
    /// assert_eq!(map.get("a"), Some(&1));
    /// assert_eq!(map.get("b"), None);
    /// ```
    #[inline]
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.entries
            .binary_search_by(|(k, _)| k.borrow().cmp(key))
            .ok()
            .map(|i| &self.entries[i].1)
    }

    /// Returns `true` if the map contains an entry for the given key.
    ///
    /// Uses binary search.
    ///
    /// # Complexity
    ///
    /// - Time: O(log n)
    /// - Space: O(1)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frozndict::frozen_map::FrozenMap;
    ///
    /// let map: FrozenMap<&str, i32> = FrozenMap::new([("a", 1)]);
    /// assert!(map.contains_key("a"));
    /// assert!(!map.contains_key("z"));
    /// ```
    #[inline]
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.get(key).is_some()
    }

    /// Returns an iterator over references to each key in sorted order.
    ///
    /// # Complexity
    ///
    /// - Time: O(n) total iteration
    /// - Space: O(1) for the iterator itself
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frozndict::frozen_map::FrozenMap;
    ///
    /// let map: FrozenMap<&str, i32> = FrozenMap::new([("b", 2), ("a", 1)]);
    /// let keys: Vec<_> = map.keys().collect();
    /// assert_eq!(keys, vec![&"a", &"b"]);
    /// ```
    #[inline]
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.entries.iter().map(|(k, _)| k)
    }

    /// Returns an iterator over references to each value, in key-sorted order.
    ///
    /// # Complexity
    ///
    /// - Time: O(n) total iteration
    /// - Space: O(1) for the iterator itself
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frozndict::frozen_map::FrozenMap;
    ///
    /// let map: FrozenMap<&str, i32> = FrozenMap::new([("b", 2), ("a", 1)]);
    /// let vals: Vec<_> = map.values().collect();
    /// assert_eq!(vals, vec![&1, &2]);
    /// ```
    #[inline]
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.entries.iter().map(|(_, v)| v)
    }

    /// Returns an iterator over `(&K, &V)` pairs in key-sorted order.
    ///
    /// # Complexity
    ///
    /// - Time: O(n) total iteration
    /// - Space: O(1) for the iterator itself
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frozndict::frozen_map::FrozenMap;
    ///
    /// let map: FrozenMap<&str, i32> = FrozenMap::new([("a", 1)]);
    /// let items: Vec<_> = map.items().collect();
    /// assert_eq!(items, vec![(&"a", &1)]);
    /// ```
    #[inline]
    pub fn items(&self) -> impl Iterator<Item = (&K, &V)> {
        self.entries.iter().map(|(k, v)| (k, v))
    }

    /// Returns an indented, human-readable representation of the map.
    ///
    /// Nested dictionary-like structures are printed with `num_spaces` spaces
    /// of indentation per level.
    ///
    /// # Arguments
    ///
    /// - `num_spaces` - Number of spaces to indent each level.
    ///
    /// # Complexity
    ///
    /// - Time: O(n) where n is the total number of characters in the output
    /// - Space: O(n) for the output string
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frozndict::frozen_map::FrozenMap;
    ///
    /// let map: FrozenMap<&str, i32> = FrozenMap::new([("a", 1)]);
    /// let s = map.pretty_repr(4);
    /// assert!(s.contains("a"));
    /// ```
    pub fn pretty_repr(&self, num_spaces: usize) -> String
    where
        K: fmt::Debug,
        V: fmt::Debug,
    {
        let indent = " ".repeat(num_spaces);
        let mut out = String::from("frozendict({\n");
        for (k, v) in self.entries.iter() {
            out.push_str(&format!("{indent}{k:?}: {v:?},\n"));
        }
        out.push_str("})");
        out
    }

    /// Computes the combined hash of all `(key, value)` pairs using XOR.
    ///
    /// XOR-combining individual pair hashes makes the result order-independent,
    /// which is the correct semantic for unordered maps: two maps with the same
    /// entries but different insertion orders must produce the same hash.
    ///
    /// # Complexity
    ///
    /// - Time: O(n)
    /// - Space: O(1)
    fn compute_hash(pairs: &[(K, V)]) -> u64 {
        let mut combined: u64 = 0;
        for (k, v) in pairs {
            let mut hasher = AHasher::default();
            k.hash(&mut hasher);
            v.hash(&mut hasher);
            combined ^= hasher.finish();
        }
        combined
    }
}

impl<K, V> Default for FrozenMap<K, V>
where
    K: Ord + Hash,
    V: Hash,
{
    /// Returns an empty [`FrozenMap`].
    ///
    /// # Complexity
    ///
    /// - Time: O(1)
    /// - Space: O(1)
    fn default() -> Self {
        Self {
            entries: Box::new([]),
            hash: 0,
        }
    }
}

impl<K, V> Hash for FrozenMap<K, V> {
    /// Feeds the pre-computed map hash into the given [`Hasher`].
    ///
    /// # Complexity
    ///
    /// - Time: O(1)
    /// - Space: O(1)
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash);
    }
}

impl<K, V> PartialEq for FrozenMap<K, V>
where
    K: PartialEq,
    V: PartialEq,
{
    /// Returns `true` if both maps contain the same set of key-value pairs.
    ///
    /// Comparison is performed over the sorted entry slices, so two maps built
    /// from the same entries in different orders compare as equal.
    ///
    /// # Complexity
    ///
    /// - Time: O(n)
    /// - Space: O(1)
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.entries == other.entries
    }
}

impl<K, V> Eq for FrozenMap<K, V>
where
    K: Eq,
    V: Eq,
{
}

impl<K, V> fmt::Debug for FrozenMap<K, V>
where
    K: fmt::Debug,
    V: fmt::Debug,
{
    /// Formats the map as `frozendict({key: value, ...})`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "frozendict({{")?;
        for (i, (k, v)) in self.entries.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{k:?}: {v:?}")?;
        }
        write!(f, "}})")
    }
}

impl<K, V> fmt::Display for FrozenMap<K, V>
where
    K: fmt::Display,
    V: fmt::Display,
{
    /// Formats the map as `frozendict({key: value, ...})`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "frozendict({{")?;
        for (i, (k, v)) in self.entries.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{k}: {v}")?;
        }
        write!(f, "}})")
    }
}

impl<K, V, Q> Index<&Q> for FrozenMap<K, V>
where
    K: Borrow<Q> + Ord + Hash,
    Q: Ord + ?Sized,
    V: Hash,
{
    type Output = V;

    /// Returns a reference to the value at `key`.
    ///
    /// # Panics
    ///
    /// Panics if `key` is not present in the map.
    ///
    /// # Complexity
    ///
    /// - Time: O(log n)
    /// - Space: O(1)
    #[inline]
    fn index(&self, key: &Q) -> &V {
        self.get(key).expect("key not found in FrozenMap")
    }
}

impl<K, V> IntoIterator for FrozenMap<K, V> {
    type Item = (K, V);
    type IntoIter = vec::IntoIter<(K, V)>;

    /// Consumes the map and returns an iterator over `(key, value)` pairs.
    ///
    /// # Complexity
    ///
    /// - Time: O(n) total iteration
    /// - Space: O(n) - the boxed slice is moved into a Vec
    fn into_iter(self) -> Self::IntoIter {
        Vec::from(self.entries).into_iter()
    }
}

impl<'a, K, V> IntoIterator for &'a FrozenMap<K, V> {
    type Item = &'a (K, V);
    type IntoIter = slice::Iter<'a, (K, V)>;

    /// Returns an iterator over references to `(key, value)` pairs.
    ///
    /// # Complexity
    ///
    /// - Time: O(n) total iteration
    /// - Space: O(1) for the iterator itself
    fn into_iter(self) -> Self::IntoIter {
        self.entries.iter()
    }
}

impl<K, V> FromIterator<(K, V)> for FrozenMap<K, V>
where
    K: Ord + Hash,
    V: Hash,
{
    /// Builds a [`FrozenMap`] from an iterator of `(key, value)` pairs.
    ///
    /// # Complexity
    ///
    /// - Time: O(n log n)
    /// - Space: O(n)
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        Self::new(iter)
    }
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

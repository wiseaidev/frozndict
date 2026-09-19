// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # Core Map Type
//!
//! Defines [`FrozenMap<K, V>`]: a fully immutable, sorted flat key-value
//! store with O(1) amortised lookup and O(n) cache-friendly iteration.
//!
//! ## Construction
//!
//! All entry points (`new`, `fromkeys`) sort the input pairs, deduplicate
//! by last-write-wins, accumulate the aggregate hash in a single forward
//! scan, then build the open-addressing probe table.
//!
//! ## Lookup
//!
//! [`find_idx`](FrozenMap::find_idx) hashes the query key with `AHasher`,
//! starts at `hash % capacity`, and probes linearly until a match or an
//! [`EMPTY`](super::slot::EMPTY) sentinel is found.
//!
//! ## Functional Updates
//!
//! Methods `merge`, `with`, `without`, `intersection`, `union`, and
//! `difference` all return new [`FrozenMap`] instances: no mutation of
//! any existing map ever occurs.

use super::slot::{EMPTY, MIX_KEY, MIX_VAL, Slot, probe_capacity, table_insert};
use ahash::AHasher;
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use core::borrow::Borrow;
use core::hash::{Hash, Hasher};
use core::slice;

/// A fully immutable, hashable flat key-value store.
///
/// # Layout
///
/// Keys and values are stored in **separate** parallel heap slices (SoA).
/// An open-addressing probe table affords O(1) amortised hash-based lookup.
/// An aggregate hash is pre-computed at construction time for O(1)
/// equality short-circuits and O(1) `Hash` impls.
///
/// # Deduplication
///
/// Duplicate keys are resolved by **last-write-wins** semantics.
///
/// # Examples
///
/// ```rust
/// use frozendict::frozen_map::FrozenMap;
///
/// let map: FrozenMap<&str, i32> = FrozenMap::new([("b", 2), ("a", 1)]);
/// assert_eq!(map["a"], 1);
/// assert_eq!(map.len(), 2);
/// ```
#[derive(Clone)]
pub struct FrozenMap<K, V> {
    /// Insertion-ordered key slice: iterated sequentially.
    pub(super) keys: Box<[K]>,

    /// Value slice parallel to `keys`.
    pub(super) vals: Box<[V]>,

    /// Open-addressing probe table.  `capacity` is always a power-of-two
    /// ≥ `2 × keys.len()` so load factor ≤ 0.5.
    pub(super) table: Box<[Slot]>,

    /// Pre-computed aggregate hash (insertion-order-independent XOR of mixed pairs).
    pub(super) hash: u64,
}

impl<K, V> FrozenMap<K, V>
where
    K: Ord + Hash,
    V: Hash,
{
    /// Constructs a new [`FrozenMap`] from an iterator of `(key, value)` pairs.
    ///
    /// Duplicate keys are resolved by **last-write-wins**.
    /// Construction is O(n log n) (sort) + O(n) (dedup + hash + table build).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frozendict::frozen_map::FrozenMap;
    ///
    /// let map: FrozenMap<&str, i32> = FrozenMap::new([("z", 26), ("a", 1)]);
    /// assert_eq!(map["a"], 1);
    /// assert_eq!(map["z"], 26);
    /// ```
    ///
    /// # Complexity
    ///
    /// - **Time**: O(n log n).
    /// - **Space**: O(n).
    pub fn new(entries: impl IntoIterator<Item = (K, V)>) -> Self {
        let mut pairs: Vec<(K, V)> = entries.into_iter().collect();

        if pairs.is_empty() {
            let cap = probe_capacity(0);
            return Self {
                keys: Box::new([]),
                vals: Box::new([]),
                table: vec![
                    Slot {
                        hash: EMPTY,
                        idx: 0
                    };
                    cap
                ]
                .into_boxed_slice(),
                hash: 0,
            };
        }

        pairs.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));

        let n = pairs.len();
        let mut keys: Vec<K> = Vec::with_capacity(n);
        let mut vals: Vec<V> = Vec::with_capacity(n);
        let mut agg_hash: u64 = 0;

        let mut iter = pairs.into_iter().peekable();
        while let Some((mut k, mut v)) = iter.next() {
            while iter.peek().is_some_and(|(nk, _)| nk == &k) {
                let (nk, nv) = iter.next().unwrap();
                k = nk;
                v = nv;
            }
            let mut kh = AHasher::default();
            k.hash(&mut kh);
            let k_hash = kh.finish();

            let mut vh = AHasher::default();
            v.hash(&mut vh);
            let v_hash = vh.finish();

            agg_hash ^= k_hash.wrapping_mul(MIX_KEY) ^ v_hash.wrapping_mul(MIX_VAL);
            keys.push(k);
            vals.push(v);
        }

        let m = keys.len();
        let cap = probe_capacity(m);
        let mut table = vec![
            Slot {
                hash: EMPTY,
                idx: 0
            };
            cap
        ]
        .into_boxed_slice();

        for (i, k) in keys.iter().enumerate() {
            let mut kh = AHasher::default();
            k.hash(&mut kh);
            table_insert(&mut table, kh.finish(), i as u32);
        }

        Self {
            keys: keys.into_boxed_slice(),
            vals: vals.into_boxed_slice(),
            table,
            hash: agg_hash,
        }
    }

    /// Constructs a [`FrozenMap`] by associating each key with the same value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frozendict::frozen_map::FrozenMap;
    ///
    /// let map = FrozenMap::fromkeys(["x", "y", "z"], 0_i32);
    /// assert_eq!(map["x"], 0);
    /// assert_eq!(map.len(), 3);
    /// ```
    ///
    /// # Complexity
    ///
    /// - **Time**: O(n log n).
    /// - **Space**: O(n).
    pub fn fromkeys(keys: impl IntoIterator<Item = K>, value: V) -> Self
    where
        V: Clone,
    {
        Self::new(keys.into_iter().map(|k| (k, value.clone())))
    }

    /// Returns the number of entries in the map. O(1).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frozendict::frozen_map::FrozenMap;
    ///
    /// let map: FrozenMap<&str, i32> = FrozenMap::new([("a", 1)]);
    /// assert_eq!(map.len(), 1);
    /// ```
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// Returns `true` if the map contains no entries. O(1).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frozendict::frozen_map::FrozenMap;
    ///
    /// let empty: FrozenMap<&str, i32> = FrozenMap::new([]);
    /// assert!(empty.is_empty());
    /// ```
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// Returns the pre-computed aggregate hash of all entries. O(1).
    ///
    /// Useful when the caller needs the raw hash value without going through
    /// the [`Hash`] trait.
    #[inline]
    #[must_use]
    pub fn precomputed_hash(&self) -> u64 {
        self.hash
    }

    /// O(1) amortised lookup via the open-addressing probe table.
    ///
    /// Linear probing with power-of-two capacity and load ≤ 0.5 guarantees
    /// ≤ 2 expected probes.  Returns the index into the parallel `keys` /
    /// `vals` slices, or `None` on a miss.
    ///
    /// # Complexity
    ///
    /// - **Time**: O(1) amortised.
    /// - **Space**: O(1).
    #[inline]
    fn find_idx<Q>(&self, key: &Q) -> Option<usize>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let mut kh = AHasher::default();
        key.hash(&mut kh);
        let key_hash = kh.finish();

        let cap = self.table.len();
        let mask = cap - 1;
        let mut slot = (key_hash as usize) & mask;

        loop {
            let s = self.table[slot];
            if s.hash == EMPTY {
                return miss();
            }
            if s.hash == key_hash {
                let idx = s.idx as usize;
                if self.keys[idx].borrow() == key {
                    return Some(idx);
                }
            }
            slot = (slot + 1) & mask;
        }
    }

    /// Returns a reference to the value for `key`, or `None`. O(1) amortised.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frozendict::frozen_map::FrozenMap;
    ///
    /// let map: FrozenMap<&str, i32> = FrozenMap::new([("a", 1)]);
    /// assert_eq!(map.get("a"), Some(&1));
    /// assert_eq!(map.get("b"), None);
    /// ```
    ///
    /// # Complexity
    ///
    /// - **Time**: O(1) amortised.
    /// - **Space**: O(1).
    #[inline]
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.find_idx(key).map(|i| &self.vals[i])
    }

    /// Returns `(&K, &V)` for `key`, or `None`. O(1) amortised.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frozendict::frozen_map::FrozenMap;
    ///
    /// let map: FrozenMap<String, i32> = FrozenMap::new([("a".to_string(), 1)]);
    /// let (k, v) = map.get_key_value("a").unwrap();
    /// assert_eq!(k, "a");
    /// assert_eq!(*v, 1);
    /// ```
    ///
    /// # Complexity
    ///
    /// - **Time**: O(1) amortised.
    /// - **Space**: O(1).
    #[inline]
    pub fn get_key_value<Q>(&self, key: &Q) -> Option<(&K, &V)>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.find_idx(key).map(|i| (&self.keys[i], &self.vals[i]))
    }

    /// Returns `true` if the map contains an entry for `key`. O(1) amortised.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frozendict::frozen_map::FrozenMap;
    ///
    /// let map: FrozenMap<&str, i32> = FrozenMap::new([("a", 1)]);
    /// assert!(map.contains_key("a"));
    /// assert!(!map.contains_key("z"));
    /// ```
    ///
    /// # Complexity
    ///
    /// - **Time**: O(1) amortised.
    /// - **Space**: O(1).
    #[inline]
    #[must_use]
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.find_idx(key).is_some()
    }

    /// Returns an iterator over references to each key in sorted order. O(n).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frozendict::frozen_map::FrozenMap;
    ///
    /// let map: FrozenMap<&str, i32> = FrozenMap::new([("b", 2), ("a", 1)]);
    /// let keys: Vec<_> = map.keys().collect();
    /// assert!(keys.contains(&&"a") && keys.contains(&&"b"));
    /// ```
    ///
    /// # Complexity
    ///
    /// - **Time**: O(n).
    /// - **Space**: O(1).
    #[inline]
    pub fn keys(&self) -> slice::Iter<'_, K> {
        self.keys.iter()
    }

    /// Returns an iterator over references to each value. O(n).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frozendict::frozen_map::FrozenMap;
    ///
    /// let map: FrozenMap<&str, i32> = FrozenMap::new([("b", 2), ("a", 1)]);
    /// let vals: Vec<_> = map.values().collect();
    /// assert!(vals.contains(&&1) && vals.contains(&&2));
    /// ```
    ///
    /// # Complexity
    ///
    /// - **Time**: O(n).
    /// - **Space**: O(1).
    #[inline]
    pub fn values(&self) -> slice::Iter<'_, V> {
        self.vals.iter()
    }

    /// Returns an iterator over `(&K, &V)` pairs in sorted-key order. O(n).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frozendict::frozen_map::FrozenMap;
    ///
    /// let map: FrozenMap<&str, i32> = FrozenMap::new([("a", 1)]);
    /// let items: Vec<_> = map.items().collect();
    /// assert_eq!(items, vec![(&"a", &1)]);
    /// ```
    ///
    /// # Complexity
    ///
    /// - **Time**: O(n).
    /// - **Space**: O(1).
    #[inline]
    pub fn items(&self) -> core::iter::Zip<slice::Iter<'_, K>, slice::Iter<'_, V>> {
        self.keys.iter().zip(self.vals.iter())
    }

    /// Returns the merge of `self` and `other` (other wins on key collision).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frozendict::frozen_map::FrozenMap;
    ///
    /// let a: FrozenMap<&str, i32> = FrozenMap::new([("a", 1), ("b", 2)]);
    /// let b: FrozenMap<&str, i32> = FrozenMap::new([("b", 99), ("c", 3)]);
    /// let merged = a.merge(&b);
    /// assert_eq!(merged["a"], 1);
    /// assert_eq!(merged["b"], 99);
    /// assert_eq!(merged["c"], 3);
    /// ```
    ///
    /// # Complexity
    ///
    /// - **Time**: O((n + m) log(n + m)).
    /// - **Space**: O(n + m).
    #[must_use]
    pub fn merge(&self, other: &Self) -> Self
    where
        K: Clone,
        V: Clone,
    {
        let mut pairs: Vec<(K, V)> = Vec::with_capacity(self.len() + other.len());
        pairs.extend(self.items().map(|(k, v)| (k.clone(), v.clone())));
        pairs.extend(other.items().map(|(k, v)| (k.clone(), v.clone())));
        Self::new(pairs)
    }

    /// Returns a new map with `key` mapped to `value`. O(n log n).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frozendict::frozen_map::FrozenMap;
    ///
    /// let a: FrozenMap<&str, i32> = FrozenMap::new([("a", 1)]);
    /// let b = a.with("b", 2);
    /// assert_eq!(b["a"], 1);
    /// assert_eq!(b["b"], 2);
    /// assert_eq!(a.len(), 1);
    /// ```
    ///
    /// # Complexity
    ///
    /// - **Time**: O(n log n).
    /// - **Space**: O(n).
    #[must_use]
    pub fn with(&self, key: K, value: V) -> Self
    where
        K: Clone,
        V: Clone,
    {
        let mut pairs: Vec<(K, V)> = Vec::with_capacity(self.len() + 1);
        pairs.extend(self.items().map(|(k, v)| (k.clone(), v.clone())));
        pairs.push((key, value));
        Self::new(pairs)
    }

    /// Returns a new map with `key` removed. O(n).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frozendict::frozen_map::FrozenMap;
    ///
    /// let a: FrozenMap<&str, i32> = FrozenMap::new([("a", 1), ("b", 2)]);
    /// let b = a.without("a");
    /// assert!(!b.contains_key("a"));
    /// assert_eq!(b["b"], 2);
    /// ```
    ///
    /// # Complexity
    ///
    /// - **Time**: O(n).
    /// - **Space**: O(n).
    #[must_use]
    pub fn without<Q>(&self, key: &Q) -> Self
    where
        K: Borrow<Q> + Clone + Hash + Ord,
        Q: Hash + Eq + Ord + ?Sized,
        V: Clone,
    {
        let pairs: Vec<(K, V)> = self
            .items()
            .filter(|(k, _)| <K as Borrow<Q>>::borrow(k) != key)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        Self::new(pairs)
    }

    /// Returns a new map containing only keys present in **both** maps
    /// (self's values win). O(n log m).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frozendict::frozen_map::FrozenMap;
    ///
    /// let a: FrozenMap<&str, i32> = FrozenMap::new([("a", 1), ("b", 2)]);
    /// let b: FrozenMap<&str, i32> = FrozenMap::new([("b", 99), ("c", 3)]);
    /// let inter = a.intersection(&b);
    /// assert_eq!(inter.len(), 1);
    /// assert_eq!(inter["b"], 2);
    /// ```
    ///
    /// # Complexity
    ///
    /// - **Time**: O(n log m).
    /// - **Space**: O(min(n, m)).
    #[must_use]
    pub fn intersection(&self, other: &Self) -> Self
    where
        K: Clone,
        V: Clone,
    {
        let pairs: Vec<(K, V)> = self
            .items()
            .filter(|(k, _)| other.contains_key(*k))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        Self::new(pairs)
    }

    /// Returns a new map with keys from either map (self priority on collision).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frozendict::frozen_map::FrozenMap;
    ///
    /// let a: FrozenMap<&str, i32> = FrozenMap::new([("a", 1), ("b", 2)]);
    /// let b: FrozenMap<&str, i32> = FrozenMap::new([("b", 99), ("c", 3)]);
    /// let u = a.union(&b);
    /// assert_eq!(u["b"], 2);
    /// assert_eq!(u["c"], 3);
    /// ```
    ///
    /// # Complexity
    ///
    /// - **Time**: O((n + m) log(n + m)).
    /// - **Space**: O(n + m).
    #[must_use]
    pub fn union(&self, other: &Self) -> Self
    where
        K: Clone,
        V: Clone,
    {
        let mut pairs: Vec<(K, V)> = Vec::with_capacity(self.len() + other.len());
        pairs.extend(other.items().map(|(k, v)| (k.clone(), v.clone())));
        pairs.extend(self.items().map(|(k, v)| (k.clone(), v.clone())));
        Self::new(pairs)
    }

    /// Returns a new map with all keys from `other` removed. O(n log m).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frozendict::frozen_map::FrozenMap;
    ///
    /// let a: FrozenMap<&str, i32> = FrozenMap::new([("a", 1), ("b", 2), ("c", 3)]);
    /// let b: FrozenMap<&str, i32> = FrozenMap::new([("b", 0), ("c", 0)]);
    /// let diff = a.difference(&b);
    /// assert_eq!(diff.len(), 1);
    /// assert_eq!(diff["a"], 1);
    /// ```
    ///
    /// # Complexity
    ///
    /// - **Time**: O(n log m).
    /// - **Space**: O(n).
    #[must_use]
    pub fn difference(&self, other: &Self) -> Self
    where
        K: Clone,
        V: Clone,
    {
        let pairs: Vec<(K, V)> = self
            .items()
            .filter(|(k, _)| !other.contains_key(*k))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        Self::new(pairs)
    }
}

/// Returns `None` while hinting to LLVM that this is a cold (unlikely) path.
///
/// Marking the miss branch cold lets the compiler prioritise register
/// allocation and instruction scheduling for the hit path.
#[cold]
#[inline(always)]
fn miss<T>() -> Option<T> {
    None
}

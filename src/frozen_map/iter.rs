// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # Iteration Traits
//!
//! Provides [`IntoIterator`] (consuming and borrowing) and [`FromIterator`]
//! for [`FrozenMap`](super::FrozenMap).
//!
//! All iterators traverse the ordered `keys` / `vals` slices, so iteration
//! is cache-friendly and O(n).

use super::FrozenMap;
use alloc::vec::Vec;
use core::hash::Hash;
use core::slice;

impl<K, V> IntoIterator for FrozenMap<K, V> {
    type Item = (K, V);
    type IntoIter = alloc::vec::IntoIter<(K, V)>;

    /// Consumes the map, returning an iterator of `(key, value)` pairs in
    /// insertion (sorted-key) order.
    ///
    /// # Complexity
    ///
    /// - **Time**: O(n): one allocation to zip keys and values into a `Vec`.
    /// - **Space**: O(n).
    fn into_iter(self) -> Self::IntoIter {
        let keys = Vec::from(self.keys);
        let vals = Vec::from(self.vals);
        keys.into_iter().zip(vals).collect::<Vec<_>>().into_iter()
    }
}

impl<'a, K, V> IntoIterator for &'a FrozenMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = core::iter::Zip<slice::Iter<'a, K>, slice::Iter<'a, V>>;

    /// Returns a borrowing iterator of `(&key, &value)` pairs in insertion
    /// (sorted-key) order.
    ///
    /// # Complexity
    ///
    /// - **Time**: O(n).
    /// - **Space**: O(1): no allocation.
    fn into_iter(self) -> Self::IntoIter {
        self.keys.iter().zip(self.vals.iter())
    }
}

impl<K, V> FromIterator<(K, V)> for FrozenMap<K, V>
where
    K: Ord + Hash,
    V: Hash,
{
    /// Builds a [`FrozenMap`] from an iterator of `(key, value)` pairs.
    ///
    /// Delegates to [`FrozenMap::new`], so duplicate keys are resolved with
    /// last-write-wins semantics and the aggregate hash is pre-computed.
    ///
    /// # Complexity
    ///
    /// - **Time**: O(n log n).
    /// - **Space**: O(n).
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        Self::new(iter)
    }
}

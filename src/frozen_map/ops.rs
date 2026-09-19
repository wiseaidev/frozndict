// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # Standard Trait Implementations
//!
//! Provides [`Default`], [`Hash`], [`PartialEq`], [`Eq`], and [`Index`] for
//! [`FrozenMap`](super::FrozenMap).
//!
//! All equality checks short-circuit in O(1) via the pre-computed aggregate
//! hash before falling through to O(n) element-wise comparison.

use super::FrozenMap;
use core::borrow::Borrow;
use core::hash::{Hash, Hasher};
use core::ops::Index;

impl<K, V> Default for FrozenMap<K, V>
where
    K: Ord + Hash,
    V: Hash,
{
    /// Returns an empty [`FrozenMap`].
    ///
    /// Equivalent to `FrozenMap::new(core::iter::empty())`.
    ///
    /// # Complexity
    ///
    /// - **Time**: O(1).
    /// - **Space**: O(1): allocates one power-of-two probe table of minimum size.
    fn default() -> Self {
        Self::new(core::iter::empty())
    }
}

impl<K, V> Hash for FrozenMap<K, V> {
    /// Feeds the pre-computed aggregate map hash into `state`. O(1).
    ///
    /// The aggregate hash was computed at construction time from all
    /// `(key_hash × MIX_KEY) ^ (val_hash × MIX_VAL)` contributions, so this
    /// is a single `write_u64` call regardless of map size.
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
    /// Returns `true` if the two maps contain identical entries in the same
    /// sorted order.
    ///
    /// Short-circuits in O(1) when the pre-computed aggregate hashes differ.
    /// When hashes match, falls through to element-wise `Box<[K]>` /
    /// `Box<[V]>` comparison over contiguous memory (compiles to `memcmp`).
    ///
    /// # Complexity
    ///
    /// - **Time**: O(1) on mismatch; O(n) on potential equality.
    /// - **Space**: O(1).
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash && self.keys == other.keys && self.vals == other.vals
    }
}

impl<K, V> Eq for FrozenMap<K, V>
where
    K: Eq,
    V: Eq,
{
}

impl<K, V, Q> Index<&Q> for FrozenMap<K, V>
where
    K: Borrow<Q> + Ord + Hash,
    Q: Hash + Eq + Ord + ?Sized,
    V: Hash,
{
    type Output = V;

    /// Returns a reference to the value for `key`.
    ///
    /// # Panics
    ///
    /// Panics with `"key not found in FrozenMap"` if `key` is absent.
    ///
    /// # Complexity
    ///
    /// - **Time**: O(1) amortised.
    /// - **Space**: O(1).
    #[inline]
    fn index(&self, key: &Q) -> &V {
        self.get(key).expect("key not found in FrozenMap")
    }
}

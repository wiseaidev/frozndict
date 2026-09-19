// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # Formatting Implementations
//!
//! Provides [`fmt::Debug`], [`fmt::Display`], and the [`FrozenMap::pretty_repr`]
//! helper for human-readable output of [`FrozenMap`](super::FrozenMap) values.
//!
//! All implementations iterate over the ordered `keys` / `vals` slices and
//! require only `O(n)` work.

use super::FrozenMap;
use alloc::string::String;
use core::fmt;
use core::fmt::Write;

impl<K, V> fmt::Debug for FrozenMap<K, V>
where
    K: fmt::Debug,
    V: fmt::Debug,
{
    /// Formats the map as `frozendict({k: v, ...})` using the `Debug` format
    /// for both keys and values.
    ///
    /// Entries are emitted in insertion (sorted-key) order.  O(n).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "frozendict({{")?;
        for (i, (k, v)) in self.keys.iter().zip(self.vals.iter()).enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{k:?}: {v:?}")?;
        }
        write!(f, "}}")
    }
}

impl<K, V> fmt::Display for FrozenMap<K, V>
where
    K: fmt::Display,
    V: fmt::Display,
{
    /// Formats the map as `frozendict({k: v, ...})` using the `Display` format
    /// for both keys and values.
    ///
    /// Entries are emitted in insertion (sorted-key) order.  O(n).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "frozendict({{")?;
        for (i, (k, v)) in self.keys.iter().zip(self.vals.iter()).enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{k}: {v}")?;
        }
        write!(f, "}}")
    }
}

impl<K, V> FrozenMap<K, V>
where
    K: Ord + core::hash::Hash + fmt::Debug,
    V: core::hash::Hash + fmt::Debug,
{
    /// Returns an indented multi-line representation of the map.
    ///
    /// The output format is:
    ///
    /// ```text
    /// frozendict({
    ///     "key1": value1,
    ///     "key2": value2,
    /// })
    /// ```
    ///
    /// Each key-value pair is prefixed with `num_spaces` space characters.
    /// The keys and values are formatted using their [`fmt::Debug`]
    /// implementation.
    ///
    /// # Arguments
    ///
    /// * `num_spaces`: Number of leading spaces for each entry line.
    ///
    /// # Complexity
    ///
    /// - **Time**: O(n).
    /// - **Space**: O(n × num_spaces).
    pub fn pretty_repr(&self, num_spaces: usize) -> String {
        let indent = " ".repeat(num_spaces);
        let mut out = String::with_capacity(14 + self.len() * (num_spaces + 4));
        out.push_str("frozendict({\n");
        for (k, v) in self.items() {
            out.push_str(&indent);
            let _ = writeln!(out, "{k:?}: {v:?},");
        }
        out.push_str("})");
        out
    }
}

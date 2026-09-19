// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # Probe-Table Internals
//!
//! Defines all primitives for the open-addressing hash probe table used inside
//! [`FrozenMap`](super::FrozenMap):
//!
//! - [`MIX_KEY`] / [`MIX_VAL`]: multiplicative hash-mixing constants.
//! - [`EMPTY`]: sentinel that marks an unused probe slot.
//! - [`Slot`]: a single `(hash, index)` probe-table cell.
//! - [`probe_capacity`]: next power-of-two capacity for a given entry count.
//! - [`table_insert`]: linear-probing insertion into a mutable slot slice.
//!
//! None of the items in this module are part of the public API; they are
//! re-exported selectively by [`super`].

/// Multiplicative constant for the key-hash contribution to the aggregate map
/// hash.
///
/// Derived from the golden-ratio approximation `φ × 2⁶⁴`.
pub(super) const MIX_KEY: u64 = 0x9e3779b97f4a7c15;

/// Multiplicative constant for the value-hash contribution to the aggregate
/// map hash.
pub(super) const MIX_VAL: u64 = 0x517cc1b727220a95;

/// Sentinel placed in an unused probe-table slot.
///
/// `u64::MAX` is used because `AHasher` output is uniformly distributed and
/// the probability of a real key producing this exact hash is `2⁻⁶⁴` per key.
pub(super) const EMPTY: u64 = u64::MAX;

/// One cell in the open-addressing probe table.
///
/// `hash` stores the full 64-bit key hash, or [`EMPTY`] when the slot is
/// vacant.  `idx` is the index into the parallel `keys` / `vals` slices for
/// the entry that maps to this slot.
///
/// The `#[repr(C)]` layout ensures the fields are packed without internal
/// padding, keeping the per-slot footprint at 12 bytes on all targets.
#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct Slot {
    /// Full 64-bit key hash, or [`EMPTY`] for a vacant slot.
    pub(super) hash: u64,
    /// Index into the parallel `keys` / `vals` arrays.
    pub(super) idx: u32,
}

/// Returns the smallest power-of-two capacity `≥ max(n × 2, 8)`.
///
/// The minimum of 8 avoids degenerate tables for tiny maps.  The `× 2` factor
/// keeps the load factor at or below 0.5, which bounds the expected probe
/// length to ≤ 2 slots.
///
/// # Complexity
///
/// - **Time**: O(1).
/// - **Space**: O(1).
#[inline]
pub(super) fn probe_capacity(n: usize) -> usize {
    let target = (n * 2).max(8);
    target.next_power_of_two()
}

/// Inserts `(key_hash, idx)` into `table` using linear probing.
///
/// `table.len()` must be a power-of-two so that the bitwise mask
/// `cap - 1` wraps the slot index correctly.  The function scans forward
/// until it finds an [`EMPTY`] slot and writes the new entry there.
///
/// # Panics
///
/// Panics only if the table is completely full (load factor = 1.0), which
/// cannot happen when [`probe_capacity`] is used correctly.
///
/// # Complexity
///
/// - **Time**: O(1) amortised; O(n) worst case under adversarial hash inputs.
/// - **Space**: O(1).
#[inline]
pub(super) fn table_insert(table: &mut [Slot], key_hash: u64, idx: u32) {
    let cap = table.len();
    let mask = cap - 1;
    let mut slot = (key_hash as usize) & mask;
    loop {
        if table[slot].hash == EMPTY {
            table[slot] = Slot {
                hash: key_hash,
                idx,
            };
            return;
        }
        slot = (slot + 1) & mask;
    }
}

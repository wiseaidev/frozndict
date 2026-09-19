// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # FrozenMap: State-of-the-art immutable key-value store
//!
//! [`FrozenMap<K, V>`] backs both the Python and Node.js bindings.
//!
//! ## Design Goals (in priority order)
//!
//! 1. **O(1) amortised lookup**: open-addressing probe table at load factor ≤ 0.5.
//! 2. **O(n) sequential iteration**: separate sorted `Box<[K]>` / `Box<[V]>` slices
//!    give perfect cache-line prefetch with zero pointer-chasing.
//! 3. **O(1) hash/equality short-circuit**: aggregate hash pre-computed at construction.
//! 4. **`no_std` + `forbid(unsafe_code)`**: usable in bare-metal, WASM, and embedded.
//!
//! ## Memory Layout
//!
//! ```text
//! keys:  Box<[K]>      ← insertion-ordered; used for sequential iteration
//! vals:  Box<[V]>      ← parallel to keys
//! table: Box<[Slot]>   ← open-addressing probe table; capacity power-of-two
//! hash:  u64           ← pre-computed aggregate; O(1) equals/hash
//! ```
//!
//! ## Probe Table
//!
//! A compact linear-probe table maps `key_hash → index into keys[]`.
//! Each slot is a `(u64, u32)` (8+4 = 12 bytes packed with `#[repr(C)]`).
//! Capacity is always the next power-of-two ≥ `2n` (load factor ≤ 0.5).
//! An empty slot carries `hash = EMPTY_SENTINEL = u64::MAX`.
//!
//! Lookup: `h = key.hash(); slot = h % cap; linear probe until match or empty`.
//!
//! ## Hash Mixing
//!
//! ```text
//! combined ^= k_hash × MIX_KEY ^ v_hash × MIX_VAL
//! ```
//!
//! ## Module Structure
//!
//! | Sub-module | Contents |
//! |---|---|
//! | [`slot`]  | [`slot::Slot`], constants, `probe_capacity`, `table_insert` |
//! | [`map`]   | [`FrozenMap`] struct, construction, lookup, functional updates |
//! | [`iter`]  | `IntoIterator`, `FromIterator` |
//! | [`fmt`]   | `Debug`, `Display`, `pretty_repr` |
//! | [`ops`]   | `Default`, `Hash`, `PartialEq`, `Eq`, `Index` |
//!
//! ## no_std
//!
//! The module uses only `alloc::{boxed::Box, vec::Vec}`: no `std` dependency.

extern crate alloc;

pub(crate) mod slot;

mod fmt;
mod iter;
mod map;
mod ops;

pub use map::FrozenMap;

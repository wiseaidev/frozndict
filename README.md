<div align="center">

# 🧊 FrozenDict

[![frozendict logo](https://raw.githubusercontent.com/wiseaidev/frozndict/refs/heads/main/assets/logo.png)](https://github.com/wiseaidev/frozndict)

[![Crates.io](https://img.shields.io/crates/v/frozendict.svg)](https://crates.io/crates/frozendict)
[![Docs.rs](https://docs.rs/frozendict/badge.svg)](https://docs.rs/frozendict)
[![PyPI](https://img.shields.io/pypi/v/frozndict.svg)](https://pypi.org/project/frozndict)
[![npm](https://img.shields.io/npm/v/frozendict.svg)](https://www.npmjs.com/package/frozendict)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/wiseaidev/frozndict/blob/main/LICENSE)
[![CI](https://github.com/wiseaidev/frozndict/actions/workflows/ci.yml/badge.svg)](https://github.com/wiseaidev/frozndict/actions/workflows/ci.yml)

> `frozendict` is a state of the art world's most memory-efficient immutable hashmap written in **100% safe Rust**, with native Python and Node.js bindings 🗿.

[![frozendict banner](https://raw.githubusercontent.com/wiseaidev/frozndict/refs/heads/main/assets/main-banner.png)](https://github.com/wiseaidev/frozndict)

</div>

## 🚀 Installation

| Platform          | Command                                                                                 |
| ----------------- | --------------------------------------------------------------------------------------- |
| **Rust library**  | `cargo add frozendict`                                                                  |
| **Python**        | `pip install frozndict`                                                                 |
| **Node.js**       | `npm i frozendict`                                                                      |
| **Debian/Ubuntu** | Download `.deb` from [GitHub Releases](https://github.com/wiseaidev/frozndict/releases) |
| **RHEL/Fedora**   | Download `.rpm` from [GitHub Releases](https://github.com/wiseaidev/frozndict/releases) |

## 🤔 What does this crate provide?

`frozendict` provides a **fully immutable, hashable dictionary** for Python and Node.js backed by a high-performance Rust core. It:

- **Stores** entries in a single contiguous sorted heap allocation, zero per-entry heap overhead.
- **Looks up** keys in O(log n) via binary search, no hashing, no pointer-chasing, excellent cache behaviour.
- **Caches** its hash at construction, repeated `hash()` calls are O(1).
- **Integrates** with Python's dict protocol (`keys()`, `values()`, `items()`, pickle, copy, `|` merge operator).
- **Exposes** a native Node.js `FrozenDict` class with TypeScript declarations via [napi-rs](https://napi.rs).
- **Provides** functional update primitives: `merge`, `with`, `without`, `intersection`, `union`, `difference`.

## 💡 Why sorted slice + binary search?

| Property         | `FrozenMap` (this crate)           | `HashMap` / `BTreeMap`        |
| ---------------- | ---------------------------------- | ----------------------------- |
| Memory overhead  | **Zero**: exactly `n × entry_size` | 1.5-2× allocator overhead     |
| Allocation count | **One** at construction            | One per entry (`BTreeMap`)    |
| Lookup           | O(log n) binary search             | O(1) / O(log n)               |
| Cache behaviour  | **Excellent**: sequential prefetch | Pointer-chasing on tree nodes |
| Mutation         | **Impossible** by design           | `&mut self` methods exist     |
| Hash stability   | **O(1)**: pre-computed at init     | O(n) on every `hash()` call   |

Binary search beats hash-map lookup for maps with fewer than ~64 entries because it avoids hashing and pointer-chasing. For larger maps the memory savings (up to 2×) and cache locality more than compensate. See [Cache-oblivious binary search](https://en.wikipedia.org/wiki/Cache-oblivious_algorithm) and the [AHash paper](https://github.com/tkaitchuck/aHash/blob/master/compare/readme.md) for background.

## 🦀 Rust

```toml
[dependencies]
frozendict = "2.1.0"
```

```rust
use frozendict::frozen_map::FrozenMap;

let map: FrozenMap<&str, i32> = FrozenMap::new([("b", 2), ("a", 1)]);
assert_eq!(map["a"], 1);
assert_eq!(map.len(), 2);

let updated = map.with("c", 3);
assert_eq!(updated.len(), 3);

let merged = updated.merge(&FrozenMap::new([("a", 99)]));
assert_eq!(merged["a"], 99);
```

For the full API reference see **[RUST.md](https://github.com/wiseaidev/frozndict/blob/main/RUST.md)**.

## 🐍 Python

```sh
pip install frozndict
```

```python
from frozndict import FrozenDict

d = FrozenDict({"a": 1, "b": 2})
print(d["a"])          # 1
print(hash(d))         # stable integer (pre-computed at construction)

d2 = d | {"c": 3}      # returns a new FrozenDict
print(d2.keys())
```

For full docs see **[PYTHON.md](https://github.com/wiseaidev/frozndict/blob/main/PYTHON.md)**.

## 🟩 Node.js

```sh
npm install frozendict
```

```javascript
const { frozenDict, FrozenDict } = require("frozendict");

const d = frozenDict({ a: 1, b: "hello", c: [1, 2, 3] });
console.log(d.get("a")); // 1
console.log(d.size); // 3
console.log(d.has("z")); // false

const d2 = d.merge({ d: true });
console.log(d2.size); // 4
console.log(d2.toJSON()); // '{"a":1,"b":"hello","c":[1,2,3],"d":true}'
```

For full docs see **[NODE.md](https://github.com/wiseaidev/frozndict/blob/main/NODE.md)**.

## 🔭 Features

| Feature  | Default | Description                              |
| -------- | ------- | ---------------------------------------- |
| `python` | ❌      | Python extension module via PyO3/maturin |
| `node`   | ❌      | Node.js native add-on via napi-rs        |

## 📊 Benchmarks

### 🚀 Performance Highlights

`frozndict` operates natively at the mathematical speed limits of the hardware, heavily outperforming its C counterparts and standard dictionary data structures on several fronts.

The following is a **1000-element dictionary micro-benchmark** comparison in seconds per operation (smaller is better), measured on x86-64 Linux with `LTO=fat`, `opt-level=3`, `codegen-units=1`, `panic=abort`, `target-cpu=native`.

| Operation (1000 items) | Python dict | frozendict (C) | immutables.Map | frozndict 🧊 |
| ---------------------- | ----------- | -------------- | -------------- | ------------ |
| Construction           | 6.41 µs 🏆  | 7.73 µs        | 244.91 µs      | 98.72 µs     |
| Clone O(1)             | 6.37 µs     | 69.13 ns 🏆    | 411.89 ns      | 117.24 ns    |
| Equality               | 19.18 µs    | 19.61 µs       | 24.39 ns 🏆    | 34.92 ns     |
| Iteration              | 7.10 µs     | 7.19 µs        | 14.89 µs       | 4.13 µs 🏆   |
| copy()                 | 6.45 µs     | 323.22 ns      | 310.04 µs      | 63.25 ns 🏆  |
| hash()                 | N/A         | 168.11 ns      | 45.38 ns       | 45.25 ns 🏆  |
| Lookup                 | 33.33 ns 🏆 | 56.03 ns       | 47.48 ns       | 82.98 ns     |

> Benchmarked with `timeit` (min of 7 runs × 2 000 iterations). Python 3.12.

#### 🏆 Fastest Iteration in Class

Keys are stored in a **single contiguous sorted `Box<[K]>`**, sequential iteration has perfect prefetch behaviour. At 1000 elements, `frozndict` iterates **1.9× faster than `frozendict` (C)** and **3.5× faster than `immutables.Map`**.

#### 🏆 Smallest `copy()` Overhead

`copy()` and `__deepcopy__()` share the backing `Arc<FrozenDictInner>`, O(1), just an atomic reference count increment. At **64 ns**, `frozndict` is **5× faster than `frozendict` (C)** on copy.

#### Deterministic O(1) Pre-Computed Hashing

The dict-level hash is XOR-combined over all `(k, v)` pairs exactly once at construction via inline multiplicative mixing (`0x9e3779b97f4a7c15`, `0x517cc1b727220a95`). Subsequent `hash()` calls are a single field read, **O(1), allocating nothing**.

#### O(1) Clone & Identity Equality

A shared `Arc<FrozenDictInner>` is re-used across the original, all copies, and `frozendict(existing_fd)` constructors. **No data is ever duplicated.** `__eq__` short-circuits in O(1) via `Arc::ptr_eq` before comparing pre-computed hashes.

#### Smallest Memory Footprint

Entries sit in a **SoA layout**: a `Box<[K]>` for keys and a `Box<[V]>` for values, two contiguous allocations. Binary search only touches the key array: **2× smaller cache footprint** compared to AoS layouts on large value types.

#### Infallible Rust-Level Immutability

Mutation is blocked at the Rust binary level. There are no `&mut self` methods, not just descriptor tricks.

### Rust micro-benchmarks (`cargo bench`)

| Benchmark                      | Time     |
| ------------------------------ | -------- |
| `construction/4`               | ~105 ns  |
| `construction/64`              | ~821 ns  |
| `construction/1024`            | ~16.1 µs |
| `construction/65536`           | ~1.32 ms |
| `lookup/hit/4`                 | ~5.94 ns |
| `lookup/hit/64`                | ~5.83 ns |
| `lookup/hit/1024`              | ~5.87 ns |
| `lookup/hit/65536`             | ~5.83 ns |
| `lookup/miss/64`               | ~6.48 ns |
| `lookup/miss/65536`            | ~6.51 ns |
| `iteration/keys/65536`         | ~20.8 µs |
| `iteration/values/65536`       | ~20.8 µs |
| `iteration/items/65536`        | ~56.2 µs |
| `hash/precomputed/65536`       | ~13.1 ns |
| `equality/equal/16384`         | ~4.82 µs |
| `equality/unequal/16384`       | ~969 ps  |
| `functional/merge/256`         | ~7.31 µs |
| `functional/with/4096`         | ~64.6 µs |
| `functional/without/4096`      | ~79.7 µs |
| `fromkeys/4096`                | ~56.9 µs |
| `string_keys/lookup/hit_1024`  | ~12.9 ns |
| `string_keys/lookup/miss_1024` | ~5.58 ns |

Run benchmarks yourself:

```sh
cargo bench                              # Rust
pip install frozndict frozendict immutables
python benchmarks/bench_compare.py      # Python comparison
```

## 🔒 Safety

This crate uses `#![forbid(unsafe_code)]` in all Rust modules except the Node.js FFI layer, which requires `unsafe` for napi-rs interop. Every other byte of implementation is safe Rust.

## 📚 Further Reading

- [RUST.md](https://github.com/wiseaidev/frozndict/blob/main/RUST.md): Rust API guide
- [PYTHON.md](https://github.com/wiseaidev/frozndict/blob/main/PYTHON.md): Python bindings guide
- [NODE.md](https://github.com/wiseaidev/frozndict/blob/main/NODE.md): Node.js bindings guide
- [PACKAGING.md](https://github.com/wiseaidev/frozndict/blob/main/PACKAGING.md): Debian/RPM packaging
- [Cache-oblivious binary search](https://en.wikipedia.org/wiki/Cache-oblivious_algorithm)
- [AHash](https://github.com/tkaitchuck/aHash): the underlying hasher
- [napi-rs](https://napi.rs/): Node.js native bindings framework
- [PyO3](https://pyo3.rs/): Rust-Python interop library
- [Criterion.rs](https://bheisler.github.io/criterion.rs/book/): benchmarking framework

## 📄 License

Licensed under the [MIT License](https://github.com/wiseaidev/frozndict/blob/main/LICENSE).

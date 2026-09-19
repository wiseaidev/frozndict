# frozendict Rust Documentation 🦀

The `frozendict` Rust crate provides a fully immutable, hashable, sorted-slice
key-value store available as both a Rust library and the core backing all
language bindings.

## 📦 Installation

```toml
[dependencies]
frozendict = "2.1.1"
```

## 🗂 Module Structure

The `frozen_map` public module is split into focused sub-modules:

| Sub-module            | Contents                                                          |
| --------------------- | ----------------------------------------------------------------- |
| `frozen_map::slot`    | `Slot`, `EMPTY`, `MIX_KEY/VAL`, `probe_capacity`, `table_insert`  |
| `frozen_map` (`map`)  | `FrozenMap` struct: construction, O(1) lookup, functional updates |
| `frozen_map` (`iter`) | `IntoIterator` (owned + borrowed), `FromIterator`                 |
| `frozen_map` (`fmt`)  | `Debug`, `Display`, `pretty_repr`                                 |
| `frozen_map` (`ops`)  | `Default`, `Hash`, `PartialEq`, `Eq`, `Index`                     |

## 🛠 Core Data Structure: `FrozenMap<K, V>`

### Construction

```rust
use frozendict::frozen_map::FrozenMap;

let map: FrozenMap<&str, i32> = FrozenMap::new([("b", 2), ("a", 1)]);
assert_eq!(map["a"], 1);
assert_eq!(map.len(), 2);
```

### `fromkeys` (Python-compatible)

```rust
use frozendict::frozen_map::FrozenMap;

let map = FrozenMap::fromkeys(["x", "y", "z"], 0_i32);
assert_eq!(map["x"], 0);
```

### Lookup

```rust
use frozendict::frozen_map::FrozenMap;

let map: FrozenMap<&str, i32> = FrozenMap::new([("a", 1)]);
assert_eq!(map.get("a"), Some(&1));
assert_eq!(map.get("b"), None);

let (key, val) = map.get_key_value("a").unwrap();
assert_eq!(key, &"a");
```

### Iteration

```rust
use frozendict::frozen_map::FrozenMap;

let map: FrozenMap<&str, i32> = FrozenMap::new([("b", 2), ("a", 1)]);

let keys: Vec<_>   = map.keys().collect();    // [&"a", &"b"]
let vals: Vec<_>   = map.values().collect();  // [&1, &2]
let items: Vec<_>  = map.items().collect();   // [(&"a", &1), (&"b", &2)]

for (k, v) in &map {
    println!("{k}: {v}");
}
```

### Hashing

```rust
use frozendict::frozen_map::FrozenMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

let map: FrozenMap<&str, i32> = FrozenMap::new([("a", 1)]);
println!("{}", map.precomputed_hash()); // O(1): pre-computed at construction
```

### Equality

```rust
use frozendict::frozen_map::FrozenMap;

let m1: FrozenMap<&str, i32> = FrozenMap::new([("a", 1), ("b", 2)]);
let m2: FrozenMap<&str, i32> = FrozenMap::new([("b", 2), ("a", 1)]);
assert_eq!(m1, m2); // order-independent; short-circuits on hash mismatch
```

### Functional Updates

All these methods return a **new** `FrozenMap`; the original is never modified.

```rust
use frozendict::frozen_map::FrozenMap;

let a: FrozenMap<&str, i32> = FrozenMap::new([("a", 1), ("b", 2)]);

// merge: other's values win on key collision
let merged = a.merge(&FrozenMap::new([("b", 99), ("c", 3)]));
assert_eq!(merged["b"], 99);
assert_eq!(merged["c"], 3);

// with: add or overwrite a single key
let extended = a.with("c", 3);
assert_eq!(extended.len(), 3);

// without: remove a single key
let reduced = a.without("a");
assert!(!reduced.contains_key("a"));

// intersection: keep keys present in both, values from self
let b: FrozenMap<&str, i32> = FrozenMap::new([("b", 99)]);
let inter = a.intersection(&b);
assert_eq!(inter.len(), 1);
assert_eq!(inter["b"], 2);

// union: all keys, self has priority on collision
let u = a.union(&FrozenMap::new([("b", 99), ("c", 3)]));
assert_eq!(u["b"], 2); // self wins
assert_eq!(u["c"], 3);

// difference: remove all keys present in other
let b2: FrozenMap<&str, i32> = FrozenMap::new([("b", 0)]);
let diff = a.difference(&b2);
assert_eq!(diff.len(), 1);
assert_eq!(diff["a"], 1);
```

### Index operator

```rust
use frozendict::frozen_map::FrozenMap;

let map: FrozenMap<&str, i32> = FrozenMap::new([("a", 1)]);
assert_eq!(map["a"], 1);
// map["z"] would panic
```

### Display and Debug

```rust
use frozendict::frozen_map::FrozenMap;

let map: FrozenMap<&str, i32> = FrozenMap::new([("a", 1)]);
println!("{map:?}");     // frozendict({"a": 1})
println!("{map}");       // frozendict({a: 1})
println!("{}", map.pretty_repr(4));
```

### `FromIterator` / `IntoIterator`

```rust
use frozendict::frozen_map::FrozenMap;

let map: FrozenMap<&str, i32> = [("a", 1), ("b", 2)].into_iter().collect();

// Consuming iteration
for (k, v) in map {
    println!("{k}: {v}");
}
```

## 📖 API Complexity Table

| Method                          | Time              | Notes                                |
| ------------------------------- | ----------------- | ------------------------------------ |
| `new(entries)`                  | O(n log n)        | Sort + linear dedup, no BTreeMap     |
| `fromkeys(keys, value)`         | O(n log n)        |                                      |
| `len()`                         | O(1)              |                                      |
| `is_empty()`                    | O(1)              |                                      |
| `get(key)`                      | O(1) amortised    | Open-addressing probe table          |
| `get_key_value(key)`            | O(1) amortised    |                                      |
| `contains_key(key)`             | O(1) amortised    |                                      |
| `keys()` / `values()`/`items()` | O(n)              | Full iteration                       |
| `precomputed_hash()`            | O(1)              | Cached at construction               |
| `hash(state)` (trait)           | O(1)              |                                      |
| `eq()` (PartialEq)              | O(1) / O(n)       | O(1) if hash differs, O(n) otherwise |
| `merge(other)`                  | O((m+n) log(m+n)) |                                      |
| `with(key, val)`                | O(n log n)        |                                      |
| `without(key)`                  | O(n log n)        |                                      |
| `intersection(other)`           | O(n log m)        |                                      |
| `union(other)`                  | O((m+n) log(m+n)) |                                      |
| `difference(other)`             | O(n log m)        |                                      |
| `pretty_repr(n)`                | O(n chars)        |                                      |
| `clone()`                       | O(n)              |                                      |
| `IntoIterator` (owned)          | O(n)              |                                      |
| `IntoIterator` (ref)            | O(n)              |                                      |

## 📊 Running Benchmarks

```sh
# All benchmark groups
cargo bench

# Targeted group
cargo bench -- construction
cargo bench -- lookup
cargo bench -- functional
```

Results written to `target/criterion/` as interactive HTML reports.

## 🔒 Safety

All Rust code outside the napi-rs FFI layer uses `#![forbid(unsafe_code)]`.

## 🔗 See Also

- [docs.rs/frozendict](https://docs.rs/frozendict)
- [GitHub](https://github.com/wiseaidev/frozndict)
- [NODE.md](https://github.com/wiseaidev/frozndict/blob/main/NODE.md)
- [PYTHON.md](https://github.com/wiseaidev/frozndict/blob/main/PYTHON.md)
- [Cache-oblivious binary search](https://en.wikipedia.org/wiki/Cache-oblivious_algorithm)
- [AHash crate](https://crates.io/crates/ahash)

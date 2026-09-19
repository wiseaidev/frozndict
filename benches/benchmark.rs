// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # Criterion Benchmarks for `frozendict`
//!
//! Measures construction, lookup, iteration, hashing, equality, and functional
//! update performance of [`frozendict::frozen_map::FrozenMap`] across multiple
//! map sizes.
//!
//! Run with:
//!
//! ```sh
//! cargo bench
//! # or a targeted group:
//! cargo bench -- construction
//! ```
//!
//! ## References
//!
//! - [Criterion user guide](https://bheisler.github.io/criterion.rs/book/)
//! - [Cache-oblivious binary search](https://en.wikipedia.org/wiki/Cache-oblivious_algorithm)
//! - [AHash crate](https://crates.io/crates/ahash)

use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;
use frozendict::frozen_map::FrozenMap;
use std::hint::black_box;

fn make_pairs(n: usize) -> Vec<(i32, i32)> {
    (0..n as i32).map(|i| (i, i * i)).collect()
}

fn bench_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("construction");
    for size in [4usize, 16, 64, 256, 1024, 4096, 16384, 65536] {
        let pairs = make_pairs(size);
        group.bench_with_input(BenchmarkId::from_parameter(size), &pairs, |b, pairs| {
            b.iter(|| FrozenMap::new(black_box(pairs.clone())));
        });
    }
    group.finish();
}

fn bench_lookup_hit(c: &mut Criterion) {
    let mut group = c.benchmark_group("lookup/hit");
    for size in [4usize, 64, 1024, 65536] {
        let map: FrozenMap<i32, i32> = FrozenMap::new(make_pairs(size));
        let mid = (size / 2) as i32;
        group.bench_with_input(BenchmarkId::from_parameter(size), &mid, |b, &key| {
            b.iter(|| black_box(map.get(black_box(&key))));
        });
    }
    group.finish();
}

fn bench_lookup_miss(c: &mut Criterion) {
    let mut group = c.benchmark_group("lookup/miss");
    for size in [4usize, 64, 1024, 65536] {
        let map: FrozenMap<i32, i32> = FrozenMap::new(make_pairs(size));
        let missing = size as i32 + 1;
        group.bench_with_input(BenchmarkId::from_parameter(size), &missing, |b, &key| {
            b.iter(|| black_box(map.get(black_box(&key))));
        });
    }
    group.finish();
}

fn bench_iteration_keys(c: &mut Criterion) {
    let mut group = c.benchmark_group("iteration/keys");
    for size in [64usize, 1024, 65536] {
        let map: FrozenMap<i32, i32> = FrozenMap::new(make_pairs(size));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| {
                map.keys().for_each(|k| {
                    black_box(k);
                })
            });
        });
    }
    group.finish();
}

fn bench_iteration_values(c: &mut Criterion) {
    let mut group = c.benchmark_group("iteration/values");
    for size in [64usize, 1024, 65536] {
        let map: FrozenMap<i32, i32> = FrozenMap::new(make_pairs(size));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| {
                map.values().for_each(|v| {
                    black_box(v);
                })
            });
        });
    }
    group.finish();
}

fn bench_iteration_items(c: &mut Criterion) {
    let mut group = c.benchmark_group("iteration/items");
    for size in [64usize, 1024, 65536] {
        let map: FrozenMap<i32, i32> = FrozenMap::new(make_pairs(size));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| {
                map.items().for_each(|(k, v)| {
                    black_box((k, v));
                })
            });
        });
    }
    group.finish();
}

fn bench_hash_precomputed(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash/precomputed");
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    for size in [64usize, 1024, 65536] {
        let map: FrozenMap<i32, i32> = FrozenMap::new(make_pairs(size));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| {
                let mut h = DefaultHasher::new();
                map.hash(&mut h);
                black_box(h.finish())
            });
        });
    }
    group.finish();
}

fn bench_equality_equal(c: &mut Criterion) {
    let mut group = c.benchmark_group("equality/equal");
    for size in [64usize, 1024, 16384] {
        let m1: FrozenMap<i32, i32> = FrozenMap::new(make_pairs(size));
        let m2: FrozenMap<i32, i32> = FrozenMap::new(make_pairs(size));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| black_box(&m1) == black_box(&m2));
        });
    }
    group.finish();
}

fn bench_equality_unequal(c: &mut Criterion) {
    let mut group = c.benchmark_group("equality/unequal");
    for size in [64usize, 1024, 16384] {
        let m1: FrozenMap<i32, i32> = FrozenMap::new(make_pairs(size));
        let m2: FrozenMap<i32, i32> = FrozenMap::new(make_pairs(size + 1));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| black_box(&m1) == black_box(&m2));
        });
    }
    group.finish();
}

fn bench_functional_merge(c: &mut Criterion) {
    let mut group = c.benchmark_group("functional/merge");
    for size in [16usize, 256, 4096] {
        let m1: FrozenMap<i32, i32> = FrozenMap::new(make_pairs(size));
        let m2: FrozenMap<i32, i32> =
            FrozenMap::new((size as i32..(2 * size) as i32).map(|i| (i, i)));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| black_box(m1.merge(black_box(&m2))));
        });
    }
    group.finish();
}

fn bench_functional_with(c: &mut Criterion) {
    let mut group = c.benchmark_group("functional/with");
    for size in [16usize, 256, 4096] {
        let map: FrozenMap<i32, i32> = FrozenMap::new(make_pairs(size));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &sz| {
            b.iter(|| black_box(map.with(black_box(sz as i32 + 1), black_box(0))));
        });
    }
    group.finish();
}

fn bench_functional_without(c: &mut Criterion) {
    let mut group = c.benchmark_group("functional/without");
    for size in [16usize, 256, 4096] {
        let map: FrozenMap<i32, i32> = FrozenMap::new(make_pairs(size));
        let mid = (size / 2) as i32;
        group.bench_with_input(BenchmarkId::from_parameter(size), &mid, |b, &key| {
            b.iter(|| black_box(map.without(black_box(&key))));
        });
    }
    group.finish();
}

fn bench_fromkeys(c: &mut Criterion) {
    let mut group = c.benchmark_group("fromkeys");
    for size in [16usize, 256, 4096] {
        let keys: Vec<i32> = (0..size as i32).collect();
        group.bench_with_input(BenchmarkId::from_parameter(size), &keys, |b, keys| {
            b.iter(|| FrozenMap::fromkeys(black_box(keys.clone()), black_box(0_i32)));
        });
    }
    group.finish();
}

fn bench_string_keys_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_keys/lookup");
    let pairs: Vec<(String, i32)> = (0..1024).map(|i| (format!("key_{i:04}"), i)).collect();
    let map: FrozenMap<String, i32> = FrozenMap::new(pairs);
    group.bench_function("hit_1024", |b| {
        b.iter(|| black_box(map.get(black_box("key_0512"))));
    });
    group.bench_function("miss_1024", |b| {
        b.iter(|| black_box(map.get(black_box("key_9999"))));
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_construction,
    bench_lookup_hit,
    bench_lookup_miss,
    bench_iteration_keys,
    bench_iteration_values,
    bench_iteration_items,
    bench_hash_precomputed,
    bench_equality_equal,
    bench_equality_unequal,
    bench_functional_merge,
    bench_functional_with,
    bench_functional_without,
    bench_fromkeys,
    bench_string_keys_lookup,
);
criterion_main!(benches);

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

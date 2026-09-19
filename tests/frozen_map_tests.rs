// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use frozendict::frozen_map::FrozenMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

fn hash_of<T: Hash>(val: &T) -> u64 {
    let mut h = DefaultHasher::new();
    val.hash(&mut h);
    h.finish()
}

#[test]
fn test_new_empty() {
    let m: FrozenMap<&str, i32> = FrozenMap::new([]);
    assert!(m.is_empty());
    assert_eq!(m.len(), 0);
}

#[test]
fn test_new_single_entry() {
    let m: FrozenMap<&str, i32> = FrozenMap::new([("a", 1)]);
    assert_eq!(m.len(), 1);
    assert_eq!(m.get("a"), Some(&1));
}

#[test]
fn test_new_multiple_entries() {
    let m: FrozenMap<&str, i32> = FrozenMap::new([("b", 2), ("a", 1), ("c", 3)]);
    assert_eq!(m.len(), 3);
    assert_eq!(m.get("a"), Some(&1));
    assert_eq!(m.get("b"), Some(&2));
    assert_eq!(m.get("c"), Some(&3));
}

#[test]
fn test_new_deduplication_last_write_wins() {
    let m: FrozenMap<&str, i32> = FrozenMap::new([("a", 1), ("a", 99)]);
    assert_eq!(m.len(), 1, "duplicate key should be deduplicated");
    assert_eq!(m.get("a"), Some(&99), "last value must win");
}

#[test]
fn test_new_deduplication_three_values_last_wins() {
    let m: FrozenMap<&str, i32> = FrozenMap::new([("x", 1), ("x", 2), ("x", 3)]);
    assert_eq!(m.len(), 1);
    assert_eq!(m.get("x"), Some(&3));
}

#[test]
fn test_new_entries_are_sorted() {
    let m: FrozenMap<&str, i32> = FrozenMap::new([("z", 26), ("a", 1), ("m", 13)]);
    let keys: Vec<&str> = m.keys().copied().collect();
    assert_eq!(keys, vec!["a", "m", "z"]);
}

#[test]
fn test_new_many_duplicates_and_uniques() {
    let m: FrozenMap<i32, i32> = FrozenMap::new([(1, 10), (2, 20), (1, 11), (3, 30), (2, 21)]);
    assert_eq!(m.len(), 3);
    assert_eq!(m.get(&1), Some(&11));
    assert_eq!(m.get(&2), Some(&21));
    assert_eq!(m.get(&3), Some(&30));
}

#[test]
fn test_fromkeys_basic() {
    let m = FrozenMap::fromkeys(["x", "y", "z"], 0_i32);
    assert_eq!(m.len(), 3);
    assert_eq!(m["x"], 0);
    assert_eq!(m["y"], 0);
    assert_eq!(m["z"], 0);
}

#[test]
fn test_fromkeys_duplicate_keys_deduplicated() {
    let m = FrozenMap::fromkeys(["a", "a", "b"], 1_i32);
    assert_eq!(m.len(), 2);
}

#[test]
fn test_from_iterator_trait() {
    let pairs = vec![("a", 1_i32), ("b", 2)];
    let m: FrozenMap<&str, i32> = pairs.into_iter().collect();
    assert_eq!(m.len(), 2);
    assert_eq!(m["a"], 1);
}

#[test]
fn test_get_existing_key() {
    let m: FrozenMap<&str, i32> = FrozenMap::new([("a", 42)]);
    assert_eq!(m.get("a"), Some(&42));
}

#[test]
fn test_get_missing_key_returns_none() {
    let m: FrozenMap<&str, i32> = FrozenMap::new([("a", 1)]);
    assert_eq!(m.get("z"), None);
}

#[test]
fn test_get_key_value_found() {
    let m: FrozenMap<String, i32> = FrozenMap::new([("hello".to_string(), 42)]);
    let (k, v) = m.get_key_value("hello").unwrap();
    assert_eq!(k, "hello");
    assert_eq!(*v, 42);
}

#[test]
fn test_get_key_value_not_found() {
    let m: FrozenMap<String, i32> = FrozenMap::new([("a".to_string(), 1)]);
    assert!(m.get_key_value("z").is_none());
}

#[test]
fn test_index_operator() {
    let m: FrozenMap<&str, i32> = FrozenMap::new([("a", 7)]);
    assert_eq!(m["a"], 7);
}

#[test]
#[should_panic(expected = "key not found")]
fn test_index_operator_panics_on_missing() {
    let m: FrozenMap<&str, i32> = FrozenMap::new([("a", 1)]);
    let _ = m["z"];
}

#[test]
fn test_contains_key_found() {
    let m: FrozenMap<&str, i32> = FrozenMap::new([("hello", 1)]);
    assert!(m.contains_key("hello"));
}

#[test]
fn test_contains_key_not_found() {
    let m: FrozenMap<&str, i32> = FrozenMap::new([("hello", 1)]);
    assert!(!m.contains_key("world"));
}

#[test]
fn test_keys_sorted_order() {
    let m: FrozenMap<&str, i32> = FrozenMap::new([("c", 3), ("a", 1), ("b", 2)]);
    let keys: Vec<&str> = m.keys().copied().collect();
    assert_eq!(keys, vec!["a", "b", "c"]);
}

#[test]
fn test_values_in_key_sorted_order() {
    let m: FrozenMap<&str, i32> = FrozenMap::new([("c", 3), ("a", 1), ("b", 2)]);
    let vals: Vec<i32> = m.values().copied().collect();
    assert_eq!(vals, vec![1, 2, 3]);
}

#[test]
fn test_items_sorted_by_key() {
    let m: FrozenMap<&str, i32> = FrozenMap::new([("b", 2), ("a", 1)]);
    let items: Vec<(&str, i32)> = m.items().map(|(k, v)| (*k, *v)).collect();
    assert_eq!(items, vec![("a", 1), ("b", 2)]);
}

#[test]
fn test_by_ref_into_iterator() {
    let m: FrozenMap<&str, i32> = FrozenMap::new([("a", 1), ("b", 2)]);
    let pairs: Vec<_> = (&m).into_iter().collect();
    assert_eq!(pairs.len(), 2);
}

#[test]
fn test_owned_into_iterator_consumes_map() {
    let m: FrozenMap<String, i32> = FrozenMap::new([("a".to_string(), 1)]);
    let pairs: Vec<_> = m.into_iter().collect();
    assert_eq!(pairs.len(), 1);
    assert_eq!(pairs[0].0, "a");
}

#[test]
fn test_equality_same_order() {
    let m1: FrozenMap<&str, i32> = FrozenMap::new([("a", 1), ("b", 2)]);
    let m2: FrozenMap<&str, i32> = FrozenMap::new([("a", 1), ("b", 2)]);
    assert_eq!(m1, m2);
}

#[test]
fn test_equality_different_insertion_order() {
    let m1: FrozenMap<&str, i32> = FrozenMap::new([("a", 1), ("b", 2)]);
    let m2: FrozenMap<&str, i32> = FrozenMap::new([("b", 2), ("a", 1)]);
    assert_eq!(m1, m2);
}

#[test]
fn test_inequality_different_values() {
    let m1: FrozenMap<&str, i32> = FrozenMap::new([("a", 1)]);
    let m2: FrozenMap<&str, i32> = FrozenMap::new([("a", 99)]);
    assert_ne!(m1, m2);
}

#[test]
fn test_inequality_different_keys() {
    let m1: FrozenMap<&str, i32> = FrozenMap::new([("a", 1)]);
    let m2: FrozenMap<&str, i32> = FrozenMap::new([("b", 1)]);
    assert_ne!(m1, m2);
}

#[test]
fn test_inequality_different_lengths() {
    let m1: FrozenMap<&str, i32> = FrozenMap::new([("a", 1)]);
    let m2: FrozenMap<&str, i32> = FrozenMap::new([("a", 1), ("b", 2)]);
    assert_ne!(m1, m2);
}

#[test]
fn test_empty_maps_equal() {
    let m1: FrozenMap<&str, i32> = FrozenMap::new([]);
    let m2: FrozenMap<&str, i32> = FrozenMap::new([]);
    assert_eq!(m1, m2);
}

#[test]
fn test_hash_order_independence() {
    let m1: FrozenMap<&str, i32> = FrozenMap::new([("a", 1), ("b", 2)]);
    let m2: FrozenMap<&str, i32> = FrozenMap::new([("b", 2), ("a", 1)]);
    assert_eq!(hash_of(&m1), hash_of(&m2));
}

#[test]
fn test_equal_maps_have_equal_hashes() {
    let m1: FrozenMap<&str, i32> = FrozenMap::new([("x", 10)]);
    let m2: FrozenMap<&str, i32> = FrozenMap::new([("x", 10)]);
    assert_eq!(hash_of(&m1), hash_of(&m2));
}

#[test]
fn test_empty_map_hash_is_consistent() {
    let m1: FrozenMap<&str, i32> = FrozenMap::new([]);
    let m2: FrozenMap<&str, i32> = FrozenMap::new([]);
    assert_eq!(hash_of(&m1), hash_of(&m2));
}

#[test]
fn test_hash_inequality_different_values() {
    let m1: FrozenMap<&str, i32> = FrozenMap::new([("a", 1)]);
    let m2: FrozenMap<&str, i32> = FrozenMap::new([("a", 2)]);
    assert_ne!(hash_of(&m1), hash_of(&m2));
}

#[test]
fn test_clone_is_equal() {
    let m: FrozenMap<&str, i32> = FrozenMap::new([("a", 1), ("b", 2)]);
    let cloned = m.clone();
    assert_eq!(m, cloned);
}

#[test]
fn test_clone_is_independent() {
    let m: FrozenMap<String, i32> = FrozenMap::new([("a".to_string(), 1)]);
    let cloned = m.clone();
    assert_eq!(cloned["a"], 1);
}

#[test]
fn test_debug_contains_frozendict() {
    let m: FrozenMap<&str, i32> = FrozenMap::new([("a", 1)]);
    let s = format!("{m:?}");
    assert!(s.contains("frozendict"));
}

#[test]
fn test_debug_contains_key() {
    let m: FrozenMap<&str, i32> = FrozenMap::new([("mykey", 42)]);
    let s = format!("{m:?}");
    assert!(s.contains("mykey"));
}

#[test]
fn test_display_format() {
    let m: FrozenMap<&str, i32> = FrozenMap::new([("a", 1)]);
    let s = format!("{m}");
    assert!(s.contains("frozendict"));
    assert!(s.contains("a"));
}

#[test]
fn test_pretty_repr_indented() {
    let m: FrozenMap<&str, i32> = FrozenMap::new([("a", 1)]);
    let r = m.pretty_repr(4);
    assert!(r.starts_with("frozendict({\n"));
    assert!(r.contains("    "));
    assert!(r.ends_with(")"));
}

#[test]
fn test_large_map_lookup() {
    let pairs: Vec<(i32, i32)> = (0..1000).map(|i| (i, i * i)).collect();
    let m: FrozenMap<i32, i32> = FrozenMap::new(pairs);
    assert_eq!(m.len(), 1000);
    assert_eq!(m.get(&0), Some(&0));
    assert_eq!(m.get(&999), Some(&(999 * 999)));
    assert_eq!(m.get(&1000), None);
}

#[test]
fn test_large_map_iteration_count() {
    let pairs: Vec<(i32, i32)> = (0..500).map(|i| (i, i)).collect();
    let m: FrozenMap<i32, i32> = FrozenMap::new(pairs);
    assert_eq!(m.keys().count(), 500);
    assert_eq!(m.values().count(), 500);
    assert_eq!(m.items().count(), 500);
}

#[test]
fn test_unicode_string_keys() {
    let m: FrozenMap<String, i32> = FrozenMap::new([
        ("日本語".to_string(), 1),
        ("中文".to_string(), 2),
        ("한국어".to_string(), 3),
        ("العربية".to_string(), 4),
        ("Ελληνικά".to_string(), 5),
    ]);
    assert_eq!(m.len(), 5);
    assert_eq!(m.get("日本語"), Some(&1));
    assert_eq!(m.get("العربية"), Some(&4));
    assert!(m.contains_key("한국어"));
}

#[test]
fn test_unicode_values() {
    let m: FrozenMap<i32, String> = FrozenMap::new([
        (1, "🦀 Rust".to_string()),
        (2, "🐍 Python".to_string()),
        (3, "🟩 Node.js".to_string()),
    ]);
    assert_eq!(m.get(&1), Some(&"🦀 Rust".to_string()));
    assert_eq!(m.len(), 3);
}

#[test]
fn test_u128_value_type() {
    let m: FrozenMap<i32, u128> = FrozenMap::new([(1, u128::MAX), (2, 0_u128), (3, u128::MAX / 2)]);
    assert_eq!(m.get(&1), Some(&u128::MAX));
    assert_eq!(m.get(&2), Some(&0_u128));
}

#[test]
fn test_i64_key_boundaries() {
    let m: FrozenMap<i64, &str> =
        FrozenMap::new([(i64::MIN, "min"), (i64::MAX, "max"), (0_i64, "zero")]);
    assert_eq!(m.get(&i64::MIN), Some(&"min"));
    assert_eq!(m.get(&i64::MAX), Some(&"max"));
    assert_eq!(m.get(&0_i64), Some(&"zero"));
    assert_eq!(m.len(), 3);
}

#[test]
fn test_default_is_empty() {
    let m: FrozenMap<&str, i32> = FrozenMap::default();
    assert!(m.is_empty());
    assert_eq!(m.len(), 0);
}

#[test]
fn test_precomputed_hash_stable() {
    let m: FrozenMap<&str, i32> = FrozenMap::new([("a", 1), ("b", 2)]);
    let h1 = m.precomputed_hash();
    let h2 = m.precomputed_hash();
    assert_eq!(h1, h2);
}

#[test]
fn test_merge_distinct_keys() {
    let a: FrozenMap<&str, i32> = FrozenMap::new([("a", 1), ("b", 2)]);
    let b: FrozenMap<&str, i32> = FrozenMap::new([("c", 3), ("d", 4)]);
    let merged = a.merge(&b);
    assert_eq!(merged.len(), 4);
    assert_eq!(merged["a"], 1);
    assert_eq!(merged["b"], 2);
    assert_eq!(merged["c"], 3);
    assert_eq!(merged["d"], 4);
}

#[test]
fn test_merge_overlapping_keys_other_wins() {
    let a: FrozenMap<&str, i32> = FrozenMap::new([("a", 1), ("b", 2)]);
    let b: FrozenMap<&str, i32> = FrozenMap::new([("b", 99), ("c", 3)]);
    let merged = a.merge(&b);
    assert_eq!(merged["b"], 99);
    assert_eq!(merged["a"], 1);
    assert_eq!(merged["c"], 3);
}

#[test]
fn test_merge_into_empty() {
    let empty: FrozenMap<&str, i32> = FrozenMap::new([]);
    let b: FrozenMap<&str, i32> = FrozenMap::new([("x", 1)]);
    let merged = empty.merge(&b);
    assert_eq!(merged.len(), 1);
    assert_eq!(merged["x"], 1);
}

#[test]
fn test_merge_from_empty() {
    let a: FrozenMap<&str, i32> = FrozenMap::new([("x", 1)]);
    let empty: FrozenMap<&str, i32> = FrozenMap::new([]);
    let merged = a.merge(&empty);
    assert_eq!(merged.len(), 1);
    assert_eq!(merged["x"], 1);
}

#[test]
fn test_merge_both_empty() {
    let a: FrozenMap<&str, i32> = FrozenMap::new([]);
    let b: FrozenMap<&str, i32> = FrozenMap::new([]);
    let merged = a.merge(&b);
    assert!(merged.is_empty());
}

#[test]
fn test_merge_preserves_originals() {
    let a: FrozenMap<&str, i32> = FrozenMap::new([("a", 1)]);
    let b: FrozenMap<&str, i32> = FrozenMap::new([("b", 2)]);
    let _ = a.merge(&b);
    assert_eq!(a.len(), 1);
    assert_eq!(b.len(), 1);
}

#[test]
fn test_with_new_key() {
    let a: FrozenMap<&str, i32> = FrozenMap::new([("a", 1)]);
    let b = a.with("b", 2);
    assert_eq!(b.len(), 2);
    assert_eq!(b["a"], 1);
    assert_eq!(b["b"], 2);
}

#[test]
fn test_with_replaces_existing_key() {
    let a: FrozenMap<&str, i32> = FrozenMap::new([("a", 1)]);
    let b = a.with("a", 99);
    assert_eq!(b.len(), 1);
    assert_eq!(b["a"], 99);
}

#[test]
fn test_with_does_not_mutate_original() {
    let a: FrozenMap<&str, i32> = FrozenMap::new([("a", 1)]);
    let _ = a.with("b", 2);
    assert_eq!(a.len(), 1);
    assert!(!a.contains_key("b"));
}

#[test]
fn test_without_existing_key() {
    let a: FrozenMap<&str, i32> = FrozenMap::new([("a", 1), ("b", 2)]);
    let b = a.without("a");
    assert_eq!(b.len(), 1);
    assert!(!b.contains_key("a"));
    assert_eq!(b["b"], 2);
}

#[test]
fn test_without_missing_key_is_noop() {
    let a: FrozenMap<&str, i32> = FrozenMap::new([("a", 1)]);
    let b = a.without("z");
    assert_eq!(b.len(), 1);
    assert_eq!(b["a"], 1);
}

#[test]
fn test_without_does_not_mutate_original() {
    let a: FrozenMap<&str, i32> = FrozenMap::new([("a", 1), ("b", 2)]);
    let _ = a.without("a");
    assert_eq!(a.len(), 2);
    assert!(a.contains_key("a"));
}

#[test]
fn test_without_all_keys() {
    let a: FrozenMap<&str, i32> = FrozenMap::new([("a", 1)]);
    let b = a.without("a");
    assert!(b.is_empty());
}

#[test]
fn test_intersection_common_keys() {
    let a: FrozenMap<&str, i32> = FrozenMap::new([("a", 1), ("b", 2), ("c", 3)]);
    let b: FrozenMap<&str, i32> = FrozenMap::new([("b", 99), ("c", 100), ("d", 4)]);
    let inter = a.intersection(&b);
    assert_eq!(inter.len(), 2);
    assert_eq!(inter["b"], 2);
    assert_eq!(inter["c"], 3);
}

#[test]
fn test_intersection_values_from_self() {
    let a: FrozenMap<&str, i32> = FrozenMap::new([("x", 10)]);
    let b: FrozenMap<&str, i32> = FrozenMap::new([("x", 999)]);
    let inter = a.intersection(&b);
    assert_eq!(inter["x"], 10);
}

#[test]
fn test_intersection_no_common_keys() {
    let a: FrozenMap<&str, i32> = FrozenMap::new([("a", 1)]);
    let b: FrozenMap<&str, i32> = FrozenMap::new([("z", 26)]);
    let inter = a.intersection(&b);
    assert!(inter.is_empty());
}

#[test]
fn test_intersection_with_empty() {
    let a: FrozenMap<&str, i32> = FrozenMap::new([("a", 1)]);
    let empty: FrozenMap<&str, i32> = FrozenMap::new([]);
    assert!(a.intersection(&empty).is_empty());
}

#[test]
fn test_union_combines_all_keys() {
    let a: FrozenMap<&str, i32> = FrozenMap::new([("a", 1), ("b", 2)]);
    let b: FrozenMap<&str, i32> = FrozenMap::new([("b", 99), ("c", 3)]);
    let u = a.union(&b);
    assert_eq!(u.len(), 3);
    assert!(u.contains_key("a"));
    assert!(u.contains_key("b"));
    assert!(u.contains_key("c"));
}

#[test]
fn test_union_self_takes_priority() {
    let a: FrozenMap<&str, i32> = FrozenMap::new([("x", 1)]);
    let b: FrozenMap<&str, i32> = FrozenMap::new([("x", 999)]);
    let u = a.union(&b);
    assert_eq!(u["x"], 1);
}

#[test]
fn test_difference_removes_other_keys() {
    let a: FrozenMap<&str, i32> = FrozenMap::new([("a", 1), ("b", 2), ("c", 3)]);
    let b: FrozenMap<&str, i32> = FrozenMap::new([("b", 0), ("c", 0)]);
    let diff = a.difference(&b);
    assert_eq!(diff.len(), 1);
    assert_eq!(diff["a"], 1);
}

#[test]
fn test_difference_no_overlap() {
    let a: FrozenMap<&str, i32> = FrozenMap::new([("a", 1), ("b", 2)]);
    let b: FrozenMap<&str, i32> = FrozenMap::new([("c", 3), ("d", 4)]);
    let diff = a.difference(&b);
    assert_eq!(diff.len(), 2);
}

#[test]
fn test_difference_removes_all() {
    let a: FrozenMap<&str, i32> = FrozenMap::new([("a", 1), ("b", 2)]);
    let diff = a.difference(&a.clone());
    assert!(diff.is_empty());
}

#[test]
fn test_difference_with_empty_other() {
    let a: FrozenMap<&str, i32> = FrozenMap::new([("a", 1)]);
    let empty: FrozenMap<&str, i32> = FrozenMap::new([]);
    let diff = a.difference(&empty);
    assert_eq!(diff.len(), 1);
}

#[test]
fn test_stress_large_construction_and_lookup() {
    let n = 10_000_usize;
    let pairs: Vec<(i32, i32)> = (0..n as i32).map(|i| (i, i.wrapping_mul(7))).collect();
    let m: FrozenMap<i32, i32> = FrozenMap::new(pairs);
    assert_eq!(m.len(), n);
    for i in 0..n as i32 {
        assert_eq!(m.get(&i), Some(&i.wrapping_mul(7)));
    }
    assert_eq!(m.get(&(n as i32)), None);
}

#[test]
fn test_stress_many_duplicate_keys() {
    let n = 1_000_usize;
    let pairs: Vec<(i32, i32)> = (0..n as i32)
        .flat_map(|i| [(i % 10, i), (i % 10, i + 1)])
        .collect();
    let m: FrozenMap<i32, i32> = FrozenMap::new(pairs);
    assert_eq!(m.len(), 10);
}

#[test]
fn test_nested_string_values() {
    let inner: FrozenMap<&str, i32> = FrozenMap::new([("x", 1)]);
    let outer: FrozenMap<&str, i32> = FrozenMap::new([("a", 1)]);
    assert_eq!(inner["x"], 1);
    assert_eq!(outer["a"], 1);
}

#[test]
fn test_bool_value_type() {
    let m: FrozenMap<&str, bool> = FrozenMap::new([("t", true), ("f", false)]);
    assert_eq!(m.get("t"), Some(&true));
    assert_eq!(m.get("f"), Some(&false));
}

#[test]
fn test_f64_as_bits_value_type() {
    let m: FrozenMap<&str, u64> = FrozenMap::new([
        ("pi", std::f64::consts::PI.to_bits()),
        ("e", std::f64::consts::E.to_bits()),
    ]);
    assert_eq!(f64::from_bits(m["pi"]), std::f64::consts::PI);
    assert_eq!(f64::from_bits(m["e"]), std::f64::consts::E);
}

#[test]
fn test_tuple_value_type() {
    let m: FrozenMap<i32, (String, bool)> = FrozenMap::new([
        (1, ("one".to_string(), true)),
        (2, ("two".to_string(), false)),
    ]);
    assert_eq!(m.get(&1), Some(&("one".to_string(), true)));
}

#[test]
fn test_empty_string_key() {
    let m: FrozenMap<&str, i32> = FrozenMap::new([("", 0), ("a", 1)]);
    assert_eq!(m.get(""), Some(&0));
    assert_eq!(m.len(), 2);
}

#[test]
fn test_is_empty_after_without_all() {
    let m: FrozenMap<&str, i32> = FrozenMap::new([("a", 1)]);
    let empty = m.without("a");
    assert!(empty.is_empty());
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

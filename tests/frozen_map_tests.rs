// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use frozndict::frozen_map::FrozenMap;
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
fn test_new_entries_are_sorted() {
    let m: FrozenMap<&str, i32> = FrozenMap::new([("z", 26), ("a", 1), ("m", 13)]);
    let keys: Vec<&str> = m.keys().copied().collect();
    assert_eq!(keys, vec!["a", "m", "z"]);
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

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

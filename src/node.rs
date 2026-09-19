// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # Node.js Bindings
//!
//! Exposes an immutable `FrozenDict` class and a `frozenDict()` factory
//! function to Node.js via [napi-rs](https://napi.rs).
//!
//! The backing store is a [`crate::frozen_map::FrozenMap<String, serde_json::Value>`],
//! giving O(log n) lookups with a single contiguous heap allocation.
//! All methods are **synchronous**: no Promise boilerplate required.
//!
//! ## Installation
//!
//! ```sh
//! npm install frozendict
//! ```
//!
//! Build locally:
//!
//! ```sh
//! npm install -g @napi-rs/cli
//! napi build --platform --release --features node
//! ```
//!
//! ## Usage
//!
//! ```javascript
//! const { FrozenDict, frozenDict } = require('frozendict');
//!
//! const d = frozenDict({ a: 1, b: 'hello', c: [1, 2, 3] });
//! console.log(d.get('a'));   // 1
//! console.log(d.size);       // 3
//! console.log(d.has('z'));   // false
//!
//! const d2 = d.merge({ d: true });
//! console.log(d2.size);      // 4
//! ```
//!
//! ## See Also
//!
//! - [`crate::frozen_map::FrozenMap`]: the underlying Rust data structure.
//! - [napi-rs documentation](https://napi.rs/)
//! - [serde_json](https://docs.rs/serde_json): JSON value type used for values.

use crate::frozen_map::FrozenMap;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde_json::Value as JsonValue;

/// An immutable, hashable dictionary backed by a sorted flat array.
///
/// `FrozenDict` wraps a [`FrozenMap<String, serde_json::Value>`] and exposes
/// it to Node.js with O(log n) lookups and O(1) size/hash queries.  The instance
/// is completely immutable after construction: no method modifies state.
/// Methods that "update" the map (e.g. [`merge`](FrozenDict::merge)) return a
/// brand-new `FrozenDict` instance.
///
/// All JSON-serialisable values (booleans, numbers, strings, arrays, objects,
/// and `null`) are accepted as values.
///
/// ```javascript
/// const { FrozenDict, frozenDict } = require('frozendict');
///
/// const d = frozenDict({ name: 'rust', version: 2 });
/// console.log(d.get('name'));    // 'rust'
/// console.log(d.size);           // 2
/// console.log(d.keys());         // ['name', 'version']
/// ```
#[napi(js_name = "FrozenDict")]
pub struct FrozenDict {
    inner: FrozenMap<String, JsonValue>,
}

#[napi]
impl FrozenDict {
    /// Constructs a `FrozenDict` from a plain JavaScript object.
    ///
    /// All own enumerable string properties of `obj` are included.  Values
    /// are recursively converted via serde_json.
    ///
    /// ```javascript
    /// const d = FrozenDict.fromObject({ a: 1, b: [2, 3] });
    /// console.log(d.get('a'));  // 1
    /// ```
    #[napi(factory, js_name = "fromObject")]
    pub fn from_object(obj: serde_json::Map<String, JsonValue>) -> Self {
        Self {
            inner: FrozenMap::new(obj),
        }
    }

    /// Constructs a `FrozenDict` from an array of `[key, value]` pairs.
    ///
    /// Mirrors the behaviour of `new Map(entries)` in JavaScript.
    /// Duplicate keys are resolved by last-write-wins.
    ///
    /// ```javascript
    /// const d = FrozenDict.fromEntries([['a', 1], ['b', 2]]);
    /// console.log(d.get('a'));  // 1
    /// ```
    #[napi(factory, js_name = "fromEntries")]
    pub fn from_entries(entries: Vec<Vec<JsonValue>>) -> Result<Self> {
        let mut pairs: Vec<(String, JsonValue)> = Vec::with_capacity(entries.len());
        for (i, pair) in entries.into_iter().enumerate() {
            if pair.len() < 2 {
                return Err(Error::from_reason(format!(
                    "entry at index {i} must have exactly 2 elements [key, value]"
                )));
            }
            let key = match &pair[0] {
                JsonValue::String(s) => s.clone(),
                other => {
                    return Err(Error::from_reason(format!(
                        "entry at index {i}: key must be a string, got {other:?}"
                    )));
                }
            };
            pairs.push((key, pair[1].clone()));
        }
        Ok(Self {
            inner: FrozenMap::new(pairs),
        })
    }

    /// Returns the value for `key`, or `undefined` if the key is absent.
    ///
    /// Uses binary search: O(log n).
    ///
    /// ```javascript
    /// const d = frozenDict({ x: 42 });
    /// console.log(d.get('x'));  // 42
    /// console.log(d.get('y'));  // undefined
    /// ```
    #[napi]
    pub fn get(&self, key: String) -> Option<serde_json::Value> {
        self.inner.get(key.as_str()).cloned()
    }

    /// Returns `true` if the dictionary contains `key`.
    ///
    /// Uses binary search: O(log n).
    ///
    /// ```javascript
    /// const d = frozenDict({ a: 1 });
    /// console.log(d.has('a'));  // true
    /// console.log(d.has('b'));  // false
    /// ```
    #[napi]
    pub fn has(&self, key: String) -> bool {
        self.inner.contains_key(key.as_str())
    }

    /// The number of key-value pairs in the dictionary.
    ///
    /// O(1).
    ///
    /// ```javascript
    /// console.log(frozenDict({ a: 1, b: 2 }).size);  // 2
    /// ```
    #[napi(getter)]
    pub fn size(&self) -> u32 {
        self.inner.len() as u32
    }

    /// Returns a sorted array of all keys.
    ///
    /// O(n).
    ///
    /// ```javascript
    /// frozenDict({ b: 2, a: 1 }).keys();  // ['a', 'b']
    /// ```
    #[napi]
    pub fn keys(&self) -> Vec<String> {
        self.inner.keys().cloned().collect()
    }

    /// Returns an array of all values in key-sorted order.
    ///
    /// O(n).
    ///
    /// ```javascript
    /// frozenDict({ b: 2, a: 1 }).values();  // [1, 2]
    /// ```
    #[napi]
    pub fn values(&self) -> Vec<serde_json::Value> {
        self.inner.values().cloned().collect()
    }

    /// Returns an array of `[key, value]` pairs in key-sorted order.
    ///
    /// O(n).
    ///
    /// ```javascript
    /// frozenDict({ b: 2, a: 1 }).entries();
    /// // [['a', 1], ['b', 2]]
    /// ```
    #[napi]
    pub fn entries(&self) -> Vec<Vec<serde_json::Value>> {
        self.inner
            .items()
            .map(|(k, v)| vec![JsonValue::String(k.clone()), v.clone()])
            .collect()
    }

    /// Converts the dictionary to a plain JavaScript object.
    ///
    /// O(n).
    ///
    /// ```javascript
    /// const obj = frozenDict({ a: 1 }).toObject();
    /// console.log(obj.a);  // 1
    /// ```
    #[napi(js_name = "toObject")]
    pub fn to_object(&self) -> serde_json::Map<String, JsonValue> {
        self.inner
            .items()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    /// Returns the JSON string representation of the dictionary.
    ///
    /// O(n).
    ///
    /// ```javascript
    /// frozenDict({ a: 1 }).toJSON();  // '{"a":1}'
    /// ```
    #[napi(js_name = "toJSON")]
    pub fn to_json(&self) -> Result<String> {
        let map: serde_json::Map<String, JsonValue> = self.to_object();
        serde_json::to_string(&JsonValue::Object(map))
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    /// Returns a new `FrozenDict` that is the merge of `self` and `other`.
    ///
    /// When both dictionaries contain the same key, the value from `other`
    /// wins (last-write-wins).  Neither original is modified.
    ///
    /// O((m + n) log(m + n)).
    ///
    /// ```javascript
    /// const a = frozenDict({ a: 1, b: 2 });
    /// const b = a.merge({ b: 99, c: 3 });
    /// console.log(b.get('b'));  // 99
    /// console.log(b.get('a'));  // 1
    /// ```
    #[napi]
    pub fn merge(&self, other: serde_json::Map<String, JsonValue>) -> Self {
        let mut pairs: Vec<(String, JsonValue)> =
            Vec::with_capacity(self.inner.len() + other.len());
        for (k, v) in self.inner.items() {
            if !other.contains_key(k) {
                pairs.push((k.clone(), v.clone()));
            }
        }
        for (k, v) in other {
            pairs.push((k, v));
        }
        Self {
            inner: FrozenMap::new(pairs),
        }
    }

    /// Returns `true` if `other` has the same keys and values.
    ///
    /// Short-circuits on pre-computed hash mismatch (O(1)), then falls back to
    /// full O(n) comparison.
    ///
    /// ```javascript
    /// const a = frozenDict({ a: 1 });
    /// const b = frozenDict({ a: 1 });
    /// console.log(a.equals(b));  // true
    /// ```
    #[napi]
    pub fn equals(&self, other: &FrozenDict) -> bool {
        self.inner == other.inner
    }

    /// Returns the `frozendict({...})` string representation.
    ///
    /// O(n).
    ///
    /// ```javascript
    /// frozenDict({ a: 1 }).toString();
    /// // "frozendict({"a": 1})"
    /// ```
    #[napi(js_name = "toString")]
    pub fn to_string_repr(&self) -> String {
        format!("{:?}", self.inner)
    }
}

/// Constructs a [`FrozenDict`] from a plain JavaScript object.
///
/// This is a convenience factory equivalent to `FrozenDict.fromObject(obj)`.
///
/// ```javascript
/// const { frozenDict } = require('frozendict');
///
/// const d = frozenDict({ lang: 'rust', fast: true });
/// console.log(d.get('lang'));  // 'rust'
/// console.log(d.size);         // 2
/// ```
#[napi(js_name = "frozenDict")]
pub fn frozen_dict_factory(obj: serde_json::Map<String, JsonValue>) -> FrozenDict {
    FrozenDict::from_object(obj)
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

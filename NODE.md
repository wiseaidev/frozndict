<div align="center">

# 🟩 frozendict Node.js Documentation

[![frozendict logo](https://raw.githubusercontent.com/wiseaidev/frozndict/refs/heads/main/assets/logo.png)](https://github.com/wiseaidev/frozndict)

[![npm](https://img.shields.io/npm/v/frozendict.svg)](https://www.npmjs.com/package/frozendict)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/wiseaidev/frozndict/blob/main/LICENSE)

</div>

The **`frozendict`** package provides a blazingly fast, native immutable
hashmap for Node.js via [napi-rs](https://napi.rs). All methods are
**synchronous**, no Promises required.

## 📦 Installation

```sh
npm install frozendict
```

Build locally:

```sh
git clone https://github.com/wiseaidev/frozndict.git
cd frozndict
npm install -g @napi-rs/cli
npm run build   # napi build --platform --release --features node
```

## 🛠 Usage

### Create a frozen dictionary

```javascript
// If installed via npm: const { frozenDict, FrozenDict } = require('frozendict');
// For local development:
const { frozenDict, FrozenDict } = require(".");

const d = frozenDict({ a: 1, b: "hello", c: [1, 2, 3] });
console.log(d.get("a")); // 1
console.log(d.size); // 3
console.log(d.has("z")); // false
```

### From entries

```javascript
const d = FrozenDict.fromEntries([
  ["a", 1],
  ["b", 2],
]);
console.log(d.get("a")); // 1
```

### Iterate

```javascript
const d = frozenDict({ b: 2, a: 1 });
console.log(d.keys()); // ['a', 'b'], sorted
console.log(d.values()); // [1, 2]
console.log(d.entries()); // [['a', 1], ['b', 2]]
```

### Convert back to a plain object

```javascript
const obj = frozenDict({ x: 1 }).toObject();
console.log(obj.x); // 1

const json = frozenDict({ x: 1 }).toJSON();
console.log(json); // '{"x":1}'
```

### Merge (returns new FrozenDict, other wins on collision)

```javascript
const a = frozenDict({ a: 1, b: 2 });
const b = a.merge({ b: 99, c: 3 });
console.log(b.get("b")); // 99
console.log(b.get("a")); // 1
console.log(a.get("b")); // 2  (original unchanged)
```

### Equality

```javascript
const a = frozenDict({ a: 1 });
const b = frozenDict({ a: 1 });
console.log(a.equals(b)); // true
```

### TypeScript

```typescript
// If installed via npm: const { frozenDict, FrozenDict, JsonValue } = require('frozendict');
// For local development:
const { frozenDict, FrozenDict, JsonValue } = require(".");

const d: FrozenDict = frozenDict({ count: 42, tags: ["rust", "napi"] });
const count: JsonValue | undefined = d.get("count"); // 42
```

## 📖 API Reference

### `frozenDict(obj)`

| Parameter | Type                        | Required | Description   |
| --------- | --------------------------- | -------- | ------------- |
| `obj`     | `Record<string, JsonValue>` | ✅       | Source object |

Returns `FrozenDict`.

### `FrozenDict.fromObject(obj)`

Same as `frozenDict(obj)`.

### `FrozenDict.fromEntries(entries)`

| Parameter | Type                    | Required | Description                   |
| --------- | ----------------------- | -------- | ----------------------------- |
| `entries` | `[string, JsonValue][]` | ✅       | Array of `[key, value]` pairs |

Returns `FrozenDict`.

### Instance methods

| Method / Property | Returns                     | Complexity    | Description                                 |
| ----------------- | --------------------------- | ------------- | ------------------------------------------- |
| `.get(key)`       | `JsonValue \| undefined`    | O(log n)      | Binary-search lookup                        |
| `.has(key)`       | `boolean`                   | O(log n)      | Membership test                             |
| `.size`           | `number`                    | O(1)          | Entry count                                 |
| `.keys()`         | `string[]`                  | O(n)          | Sorted array of keys                        |
| `.values()`       | `JsonValue[]`               | O(n)          | Values in key-sorted order                  |
| `.entries()`      | `[string, JsonValue][]`     | O(n)          | Key-sorted `[key, value]` pairs             |
| `.toObject()`     | `Record<string, JsonValue>` | O(n)          | Convert to plain object                     |
| `.toJSON()`       | `string`                    | O(n)          | JSON string representation                  |
| `.merge(other)`   | `FrozenDict`                | O((m+n)log n) | New dict merged with `other`; other wins    |
| `.equals(other)`  | `boolean`                   | O(1)/O(n)     | Structural equality with hash short-circuit |
| `.toString()`     | `string`                    | O(n)          | `frozendict({...})` representation          |

### `JsonValue` type

```typescript
type JsonValue =
  null | boolean | number | string | JsonValue[] | { [key: string]: JsonValue };
```

## 📄 License

Licensed under the [MIT License](https://github.com/wiseaidev/frozndict/blob/main/LICENSE).

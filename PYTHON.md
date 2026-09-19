<div align="center">

# 🐍 frozndict Python Documentation

[![frozndict logo](https://raw.githubusercontent.com/wiseaidev/frozndict/refs/heads/main/assets/logo.png)](https://github.com/wiseaidev/frozndict)

[![PyPI](https://img.shields.io/pypi/v/frozndict.svg)](https://pypi.org/project/frozndict)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/wiseaidev/frozndict/blob/main/LICENSE)

</div>

The **`frozndict`** package provides a blazingly fast, fully immutable
dictionary for Python backed by a 100% safe Rust core. All operations are
**synchronous**, no `asyncio` required.

## 📦 Installation

```sh
pip install frozndict
```

Build locally (requires [maturin](https://github.com/PyO3/maturin)):

```sh
git clone https://github.com/wiseaidev/frozndict.git
cd frozndict
python3 -m venv .venv && source .venv/bin/activate
pip install maturin
maturin develop --features python
```

## 🛠 Usage

### Create a frozen dictionary

```python
>>> from frozndict import FrozenDict
>>>
>>> d = FrozenDict({"a": 1, "b": 2})
>>> print(d["a"])
1
>>> print(len(d))
2
>>> print(hash(d))
2342407593872605277
```

### From keyword arguments

```python
d = FrozenDict(x=1, y=2)
print(d["x"])  # 1
```

### From another FrozenDict (O(1) Arc clone)

```python
>>> d1 = FrozenDict({"a": 1})
>>> d2 = FrozenDict(d1)
>>> assert d1 == d2
```

### Membership test

```python
>>> d = FrozenDict({"a": 1})
>>> print("a" in d)
True
>>> print("z" in d)
False

```

### Keys, values, items

```python
>>> d = FrozenDict({"b": 2, "a": 1})
>>> print(list(d.keys()))
['b', 'a']
>>> print(list(d.values()))
[2, 1]
>>> print(list(d.items()))
[('b', 2), ('a', 1)]
```

### Get with default

```python
>>> d = FrozenDict({"a": 1})
>>> print(d.get("b", 99))
99
>>> print(d.get("a"))
1
```

### Merge operator

```python
>>> d1 = FrozenDict({"a": 1, "b": 2})
>>> d2 = FrozenDict({"b": 99, "c": 3})
>>>
>>> merged = d1 | d2
>>> print(merged["b"])
99
```

### Copy / deepcopy

```python
>>> import copy
>>>
>>> d = FrozenDict({"a": 1})
>>> d2 = copy.copy(d)      # O(1) Arc clone
>>> d3 = copy.deepcopy(d)  # O(1): immutable, so deep == shallow
```

### Pickle

```python
>>> import pickle
>>>
>>> d = FrozenDict({"a": 1})
>>> blob = pickle.dumps(d)
>>> d2 = pickle.loads(blob)
>>> assert d == d2
```

### Immutability

```python
>>> d = FrozenDict({"a": 1})
>>> try:
...     d["a"] = 99          # raises TypeError
... except TypeError as e:
...     print(e)             # 'frozendict' object does not support mutation
...
'frozendict' object does not support mutation
```

### Nested auto-freeze

When constructing from a dict containing lists, sets, or nested dicts, values
are automatically frozen:

```python
>>> d = FrozenDict({"nums": [1, 2, 3], "tags": {"rust", "python"}})
>>> print(type(d["nums"]))  # <class 'tuple'>
<class 'tuple'>
>>> print(type(d["tags"]))  # <class 'frozenset'>
<class 'frozenset'>
```

## 📖 API Reference

| Method / Special Method       | Returns      | Complexity     | Description                                |
| ----------------------------- | ------------ | -------------- | ------------------------------------------ |
| `FrozenDict(*args, **kwargs)` | `FrozenDict` | O(n log n)     | Constructor; O(1) when arg is `FrozenDict` |
| `d[key]`                      | value        | O(log n)       | Item access                                |
| `key in d`                    | `bool`       | O(log n)       | Membership test                            |
| `len(d)`                      | `int`        | O(1)           | Entry count                                |
| `hash(d)`                     | `int`        | O(1)           | Pre-computed aggregate hash                |
| `d == other`                  | `bool`       | O(1)/O(n)      | Hash short-circuit then element-wise       |
| `iter(d)`                     | iterator     | O(1)/O(n)      | Iterates keys in insertion order           |
| `reversed(d)`                 | iterator     | O(n)           | Reversed key iteration                     |
| `repr(d)`                     | `str`        | O(n)           | `frozendict({...})`                        |
| `d.get(key, default=None)`    | value / None | O(log n)       | Safe lookup with default                   |
| `d.keys()`                    | `KeysView`   | O(1)           | Returns lazy view                          |
| `d.values()`                  | `ValuesView` | O(1)           | Returns lazy view                          |
| `d.items()`                   | `ItemsView`  | O(1)           | Returns lazy view                          |
| `d.copy()`                    | `FrozenDict` | O(1)           | Arc clone                                  |
| `d.__or__(other)`             | `FrozenDict` | O((m+n) log n) | `d \| other` merge; other wins             |
| `d.__ror__(other)`            | `FrozenDict` | O((m+n) log n) | `other \| d`                               |
| `d.__reduce__()`              | tuple        | O(n)           | Pickle support                             |
| `d.__dir__()`                 | -            | -              | Raises `AttributeError` (mutation guard)   |

## 📊 Running Benchmarks

```sh
pip install frozndict frozendict
python benchmarks/benchmark.py
```

## 📄 License

Licensed under the [MIT License](https://github.com/wiseaidev/frozndict/blob/main/LICENSE).

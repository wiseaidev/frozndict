#!/usr/bin/env python3
# Copyright 2026 Mahmoud Harmouch.
#
# Licensed under the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

"""
Run::

    source .venv/bin/activate
    python examples/usage.py
"""

import collections.abc
import copy
import pickle

from frozndict import FrozenItemsView, FrozenKeysView, FrozenValuesView, frozendict

SEP = "-" * 60


def section(title: str) -> None:
    """Print a formatted section header."""
    print(f"\n{SEP}")
    print(f"  {title}")
    print(SEP)


def example_construction() -> None:
    """Demonstrate all supported construction forms."""
    section("1. Construction")

    d1 = frozendict(b=2, a=1, c=3)
    print(f"  kwargs:          {d1}")

    d2 = frozendict({"x": 10, "y": 20})
    print(f"  from dict:       {d2}")

    d3 = frozendict([("p", 100), ("q", 200)])
    print(f"  from pairs:      {d3}")

    d4 = frozendict(d1)
    print(f"  from frozendict: {d4}")

    d5 = frozendict({"a": 0}, a=99, d=4)
    print(f"  merged:          {d5}   # 'a' comes from kwargs override")

    d6 = frozendict.fromkeys(["alpha", "beta", "gamma"], 0)
    print(f"  fromkeys:        {d6}")

    print(f"  empty:           {frozendict()}")

    d7 = frozendict(nested={"inner": [1, 2, 3]})
    print(f"  auto-freeze:     {d7}   # inner list → tuple, dict → frozendict")


def example_insertion_order() -> None:
    """Demonstrate insertion-order iteration (Python 3.7+ dict semantics)."""
    section("2. Insertion Order")

    d = frozendict(b=2, a=1, c=3)
    print(f"  dict:         {d}")
    print(f"  list(keys):   {list(d.keys())}")
    print(f"  list(values): {list(d.values())}")
    print(f"  list(items):  {list(d.items())}")
    print(f"  list(iter):   {list(d)}")
    print(f"  reversed:     {list(reversed(d))}")

    d2 = frozendict(
        {"a": 1, "b": 2, "a": 99}  # noqa: F601 # pylint: disable=duplicate-key
    )
    print(f"  dup key:      {d2}  # 'a' stays first, value updated to 99")


def example_lookup() -> None:
    """Demonstrate lookup, get, and contains."""
    section("3. Lookup")

    d = frozendict(name="Mahmoud", age=25, city="Beirut")
    print(f"  d['name']:               {d['name']}")
    print(f"  d.get('age'):            {d.get('age')}")
    print(f"  d.get('missing', -1):    {d.get('missing', -1)}")
    print(f"  'city' in d:             {'city' in d}")
    print(f"  'country' in d:          {'country' in d}")


def example_hashing() -> None:
    """Demonstrate order-independent, pre-computed O(1) hashing."""
    section("4. Hashing")

    d1 = frozendict(a=1, b=2)
    d2 = frozendict(b=2, a=1)
    print(f"  hash(d1):          {hash(d1)}")
    print(f"  hash(d2):          {hash(d2)}")
    print(f"  d1 == d2:          {d1 == d2}")
    print(f"  hash(d1) == hash(d2): {hash(d1) == hash(d2)}")

    lookup = {frozendict(a=1): "found it"}
    print(f"  as dict key:       {lookup[frozendict(a=1)]}")

    s = {frozendict(a=1, b=2), frozendict(b=2, a=1)}
    print(f"  in set (dedup):    {len(s)} unique element")


def example_views() -> None:
    """Demonstrate FrozenKeysView, FrozenValuesView, FrozenItemsView."""
    section("5. Views")

    d = frozendict(a=1, b=2, c=3)
    kv = d.keys()
    vv = d.values()
    iv = d.items()

    print(f"  keys():   {kv!r}")
    print(f"  values(): {vv!r}")
    print(f"  items():  {iv!r}")

    print(f"\n  isinstance(keys(), FrozenKeysView):   {isinstance(kv, FrozenKeysView)}")
    print(f"  isinstance(vals(), FrozenValuesView): {isinstance(vv, FrozenValuesView)}")
    print(f"  isinstance(items(),FrozenItemsView):  {isinstance(iv, FrozenItemsView)}")

    print(f"\n  'a' in keys():          {'a' in kv}")
    print(f"  ('b', 2) in items():   {('b', 2) in iv}")
    print(f"  99 in values():        {99 in vv}")

    other = {"b", "c", "x"}
    print(f"\n  keys() & {other}:   {kv & other}")
    print(f"  keys() | {other}:   {kv | other}")
    print(f"  keys() - {other}:   {kv - other}")
    print(f"  keys() ^ {other}:   {kv ^ other}")
    print(f"  keys().isdisjoint({{'x', 'y'}}): {kv.isdisjoint({'x', 'y'})}")

    print(f"\n  items() & {{('a',1)}}: {iv & {('a', 1)}}")
    print(f"  items().isdisjoint([('z',0)]): {iv.isdisjoint([('z', 0)])}")


def example_merge() -> None:
    """Demonstrate | merge operator."""
    section("6. Merge (| operator)")

    d1 = frozendict(a=1, b=2)
    d2 = frozendict(b=99, c=3)
    print(f"  d1:       {d1}")
    print(f"  d2:       {d2}")
    print(f"  d1 | d2:  {d1 | d2}   # d2 overrides on 'b'")
    print(f"  d2 | d1:  {d2 | d1}   # d1 overrides on 'b'")

    plain = {"x": 10}
    print(f"  dict | fd:{plain | d1}")

    ref = d1
    d1 |= d2
    print(f"  d1 |= d2: {d1}  (d1 is now a NEW object: {d1 is not ref})")


def example_copy_deepcopy() -> None:
    """Demonstrate O(1) copy and deepcopy."""
    section("7. copy / deepcopy")

    d = frozendict(a=1, b=(1, 2, 3))
    c1 = copy.copy(d)
    c2 = copy.deepcopy(d)
    m1 = d.copy()

    print(f"  original:               {d}")
    print(f"  copy.copy(d) == d:      {c1 == d}")
    print(f"  copy.copy(d) is d:      {c1 is d}  (new Python object, shared Arc)")
    print(f"  copy.deepcopy(d) == d:  {c2 == d}")
    print(f"  d.copy() == d:          {m1 == d}")
    print(f"  hash preserved:         {hash(c1) == hash(d)}")


def example_pickle() -> None:
    """Demonstrate pickle round-trips across all protocols."""
    section("8. Pickle")

    d = frozendict(language="Python", version=3.12, nested=(1, 2, 3))
    print(f"  original: {d}")

    for protocol in range(0, pickle.HIGHEST_PROTOCOL + 1):
        loaded = pickle.loads(pickle.dumps(d, protocol=protocol))
        ok = "✓" if loaded == d else "✗"
        print(f"  protocol {protocol}: {ok}  {loaded}")


def example_immutability() -> None:
    """Demonstrate that every mutation attempt raises TypeError."""
    section("9. Immutability (all raise TypeError)")

    d = frozendict(a=1)

    def attempt(label: str, fn) -> None:
        """Try a mutation and print the raised exception."""
        try:
            fn()
            print(f"  {label}: NO ERROR (unexpected!)")
        except (TypeError, AttributeError) as e:
            print(f"  {label}: {type(e).__name__}: {e}")

    attempt("d['a'] = 2        ", lambda: d.__setitem__("a", 2))
    attempt("del d['a']        ", lambda: d.__delitem__("a"))
    attempt("d.attribute = 1   ", lambda: setattr(d, "attribute", 1))
    attempt("del d.foo         ", lambda: d.__delattr__("foo"))
    attempt(
        "d.update({})      ",
        lambda: d.update(),  # pylint: disable=unnecessary-lambda
    )
    attempt("d.pop('a')        ", lambda: d.pop("a"))
    attempt(
        "d.clear()         ",
        lambda: d.clear(),  # pylint: disable=unnecessary-lambda
    )
    attempt("d.setdefault('x') ", lambda: d.setdefault("x"))


def example_mapping_abc() -> None:
    """Demonstrate Mapping ABC registration."""
    section("10. Mapping ABC")

    d = frozendict(a=1, b=2)
    print(
        f"  isinstance(d, Mapping):             "
        f"{isinstance(d, collections.abc.Mapping)}"
    )
    print(
        f"  issubclass(frozendict, Mapping):    "
        f"{issubclass(frozendict, collections.abc.Mapping)}"
    )
    print(
        f"  isinstance(d.keys(), Mapping):      "
        f"{isinstance(d.keys(), collections.abc.Mapping)}"
    )

    print(
        "  Mapping methods available: get, keys, values, items, "
        "__contains__, __len__, __iter__"
    )


def example_inheritance() -> None:
    """Demonstrate subclassing and __init_subclass__ hook."""
    section("11. Inheritance / Subclassing")

    class TypedFrozen(frozendict):
        """A frozendict subclass that enforces uniform value types."""

        def __init_subclass__(cls, strict: bool = False, **kwargs):
            """Propagate the strict flag to the subclass at class creation time."""
            super().__init_subclass__(**kwargs)
            cls._strict = strict

        def validate(self, expected_type: type) -> bool:
            """Return True if all values are instances of expected_type."""
            return all(isinstance(v, expected_type) for v in self.values())

        def key_list(self) -> list:
            """Return sorted list of keys."""
            return sorted(self.keys())

    class StrictFrozen(TypedFrozen, strict=True):
        """An even stricter subclass."""

    d = TypedFrozen(x=1, y=2, z=3)
    print(f"  TypedFrozen({{'x':1,'y':2,'z':3}}): {d}")
    print(f"  isinstance(d, frozendict):         {isinstance(d, frozendict)}")
    print(f"  d.validate(int):                   {d.validate(int)}")
    print(f"  d.validate(str):                   {d.validate(str)}")
    print(f"  d.key_list():                      {d.key_list()}")
    print(f"  d['x']:                            {d['x']}")
    print(f"  StrictFrozen._strict:              {StrictFrozen._strict}")

    try:
        d["x"] = 99
    except TypeError as e:
        print(f"  Mutation blocked in subclass:      TypeError: {e}")


def example_generic_subscript() -> None:
    """Demonstrate FrozenDict[K, V] generic alias."""
    section("12. Generic Subscript")

    import types

    alias = frozendict[str, int]
    print(f"  frozendict[str, int]:                    {alias}")
    print(
        f"  isinstance(alias, types.GenericAlias):   "
        f"{isinstance(alias, types.GenericAlias)}"
    )

    def process(mapping: frozendict[str, int]) -> int:
        """Sum all values."""
        return sum(mapping.values())

    d = frozendict(a=1, b=2, c=3)
    print(f"  process(frozendict(a=1, b=2, c=3)):      {process(d)}")


def example_repr() -> None:
    """Demonstrate repr and pretty_repr."""
    section("13. Representation")

    d = frozendict(name="Mahmoud", scores=[99, 88, 72], active=True)
    print(f"  repr:        {d!r}")
    print(f"  pretty_repr:\n{d.pretty_repr()}")
    print(f"  pretty (2):  \n{d.pretty_repr(num_spaces=2)}")


def main() -> None:
    """Run all usage examples."""
    print("\n" + "═" * 60)
    print("  frozndict v2.1; Usage Examples")
    print("═" * 60)

    example_construction()
    example_insertion_order()
    example_lookup()
    example_hashing()
    example_views()
    example_merge()
    example_copy_deepcopy()
    example_pickle()
    example_immutability()
    example_mapping_abc()
    example_inheritance()
    example_generic_subscript()
    example_repr()

    print(f"\n{SEP}")
    print("  All examples complete.")
    print(SEP + "\n")


if __name__ == "__main__":
    main()

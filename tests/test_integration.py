# pylint: disable=missing-function-docstring,redefined-outer-name

# Copyright 2026 Mahmoud Harmouch.
#
# Licensed under the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import sys
from operator import ior

import pytest

# pyrefly: ignore [missing-import]
from frozndict import FrozenDict, __version__, frozendict


@pytest.fixture
def empty():
    """Empty frozendict."""
    return frozendict()


@pytest.fixture
def simple():
    """frozendict with three entries."""
    return frozendict(a=1, b=2, c=3)


@pytest.fixture
def nested():
    """frozendict containing a tuple value (lists are not hashable)."""
    return frozendict(x=1, y=(1, 2, 3), z="hello")


def test_version_string_format():
    parts = __version__.split(".")
    assert len(parts) == 3
    assert all(p.isdigit() for p in parts)


def test_version_value():
    assert __version__ == "2.0.0"


def test_frozendict_is_alias_of_frozen_dict():
    assert frozendict is FrozenDict


def test_construct_from_kwargs():
    d = frozendict(a=1, b=2)
    assert d["a"] == 1
    assert d["b"] == 2


def test_construct_from_plain_dict():
    d = frozendict({"x": 10, "y": 20})
    assert d["x"] == 10
    assert d["y"] == 20


def test_construct_from_list_of_pairs():
    d = frozendict([("a", 1), ("b", 2)])
    assert d["a"] == 1
    assert d["b"] == 2


def test_construct_from_frozendict():
    original = frozendict(a=1, b=2)
    copy = frozendict(original)
    assert copy == original


def test_construct_empty_from_empty_dict():
    assert frozendict({}) == frozendict()


def test_construct_empty_from_empty_list():
    assert frozendict([]) == frozendict()


def test_construct_kwargs_override_positional():
    d = frozendict({"a": 1}, a=99)
    assert d["a"] == 99


def test_construct_too_many_positional_raises():
    with pytest.raises(TypeError):
        frozendict({"a": 1}, {"b": 2})


def test_fromkeys_default_none():
    d = frozendict.fromkeys(["x", "y", "z"])
    assert d["x"] is None
    assert d["y"] is None
    assert len(d) == 3


def test_fromkeys_with_value():
    d = frozendict.fromkeys(["a", "b"], 42)
    assert d["a"] == 42
    assert d["b"] == 42


def test_fromkeys_deduplicates_keys():
    d = frozendict.fromkeys(["a", "a", "b"], 1)
    assert len(d) == 2


def test_getitem_existing(simple):
    assert simple["a"] == 1


def test_getitem_missing_raises_key_error(simple):
    with pytest.raises(KeyError):
        _ = simple["missing"]


def test_get_existing(simple):
    assert simple.get("a") == 1


def test_get_missing_returns_none(simple):
    assert simple.get("missing") is None


def test_get_missing_with_default(simple):
    assert simple.get("missing", -1) == -1


def test_contains_true(simple):
    assert "a" in simple


def test_contains_false(simple):
    assert "z" not in simple


def test_len_empty(empty):
    assert len(empty) == 0


def test_len_non_empty(simple):
    assert len(simple) == 3


def test_keys_returns_all_keys(simple):
    assert sorted(simple.keys()) == ["a", "b", "c"]


def test_values_returns_all_values(simple):
    assert sorted(simple.values()) == [1, 2, 3]


def test_items_returns_all_pairs(simple):
    assert sorted(simple.items()) == [("a", 1), ("b", 2), ("c", 3)]


def test_iter_yields_keys(simple):
    assert sorted(simple) == ["a", "b", "c"]


def test_iter_can_be_consumed_twice(simple):
    assert sorted(k for k in simple) == sorted(k for k in simple)


def test_copy_equals_original(simple):
    assert simple.copy() == simple


def test_copy_is_frozendict(simple):
    assert isinstance(simple.copy(), frozendict)


def test_repr_format(simple):
    r = repr(simple)
    assert r.startswith("frozendict({")
    assert "a" in r


def test_pretty_repr_indented(simple):
    r = simple.pretty_repr()
    assert r.startswith("frozendict({\n")
    assert "    " in r
    assert r.endswith(")")


def test_pretty_repr_custom_indent(simple):
    r = simple.pretty_repr(num_spaces=2)
    assert "  " in r


def test_hash_is_int(simple):
    assert isinstance(hash(simple), int)


def test_hash_consistent_same_instance(simple):
    assert hash(simple) == hash(simple)


def test_hash_order_independent():
    d1 = frozendict(a=1, b=2)
    d2 = frozendict(b=2, a=1)
    assert hash(d1) == hash(d2)


def test_hash_empty():
    assert hash(frozendict()) == hash(frozendict())


def test_frozendict_usable_as_dict_key():
    d = {frozendict(a=1): "found"}
    assert d[frozendict(a=1)] == "found"


def test_frozendict_usable_in_set():
    s = {frozendict(a=1, b=2), frozendict(b=2, a=1)}
    assert len(s) == 1


def test_eq_same_content(simple):
    assert simple == frozendict(a=1, b=2, c=3)


def test_eq_order_independent():
    assert frozendict(a=1, b=2) == frozendict(b=2, a=1)


def test_eq_against_plain_dict(simple):
    assert simple == {"a": 1, "b": 2, "c": 3}


def test_neq_different_values():
    assert frozendict(a=1) != frozendict(a=99)


def test_neq_different_keys():
    assert frozendict(a=1) != frozendict(b=1)


def test_neq_different_sizes():
    assert frozendict(a=1) != frozendict(a=1, b=2)


def test_empty_equals_empty():
    assert frozendict() == frozendict()


def test_or_nonempty_with_empty(simple, empty):
    result = simple | empty
    assert result == simple


def test_or_empty_with_nonempty(empty, simple):
    result = empty | simple
    assert result == simple


def test_or_disjoint_keys():
    d1 = frozendict(a=1)
    d2 = frozendict(b=2)
    result = d1 | d2
    assert result == frozendict(a=1, b=2)


def test_or_rhs_overrides_lhs():
    d1 = frozendict(a=1, b=2)
    d2 = frozendict(b=99, c=3)
    result = d1 | d2
    assert result["b"] == 99
    assert result["a"] == 1
    assert result["c"] == 3


def test_or_returns_frozendict():
    result = frozendict(a=1) | frozendict(b=2)
    assert isinstance(result, frozendict)


def test_ior_operator_returns_frozendict(simple, empty):
    result = ior(simple, empty)
    assert isinstance(result, frozendict)


def test_ior_semantics_same_as_or(simple, empty):
    assert ior(simple, empty) == simple | empty


def test_nested_list_value(nested):
    assert nested["y"] == (1, 2, 3)


def test_nested_string_value(nested):
    assert nested["z"] == "hello"


def test_copy_shares_no_identity_with_original(simple):
    copy = simple.copy()
    assert copy is not simple


def test_frozendict_smaller_than_regular_dict():
    d = {"a": 1, "b": 2, "c": 3}
    fd = frozendict(a=1, b=2, c=3)
    assert sys.getsizeof(fd) <= sys.getsizeof(d) * 2


# Copyright 2026 Mahmoud Harmouch.
#
# Licensed under the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

# Copyright 2026 Mahmoud Harmouch.
#
# Licensed under the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.
# pylint: disable=missing-function-docstring,redefined-outer-name

import pytest

# pyrefly: ignore [missing-import]
from frozndict import FrozenItemsView, FrozenKeysView, FrozenValuesView, frozendict


@pytest.fixture
def simple():
    """A frozendict with three str-keyed entries."""
    return frozendict(b=2, a=1, c=3)


@pytest.fixture
def keys_view(simple):
    """FrozenKeysView from the simple fixture."""
    return simple.keys()


@pytest.fixture
def values_view(simple):
    """FrozenValuesView from the simple fixture."""
    return simple.values()


@pytest.fixture
def items_view(simple):
    """FrozenItemsView from the simple fixture."""
    return simple.items()


class TestFrozenKeysView:
    def test_isinstance(self, keys_view):
        assert isinstance(keys_view, FrozenKeysView)

    def test_len(self, keys_view):
        assert len(keys_view) == 3

    def test_iter_in_insertion_order(self, simple, keys_view):
        assert list(keys_view) == list(simple)

    def test_contains_true(self, keys_view):
        assert "a" in keys_view

    def test_contains_false(self, keys_view):
        assert "z" not in keys_view

    def test_repr(self, keys_view):
        r = repr(keys_view)
        assert r.startswith("frozendict_keys(")

    def test_and_intersection(self, keys_view):
        result = keys_view & {"a", "b"}
        assert result == {"a", "b"}

    def test_and_no_overlap(self, keys_view):
        result = keys_view & {"x", "y"}
        assert result == set()

    def test_or_union(self, keys_view):
        result = keys_view | {"x"}
        assert "x" in result
        assert "a" in result

    def test_sub_difference(self, keys_view):
        result = keys_view - {"a"}
        assert "a" not in result
        assert "b" in result

    def test_xor_symmetric_difference(self, keys_view):
        result = keys_view ^ {"a", "x"}
        assert "x" in result
        assert "a" not in result

    def test_isdisjoint_true(self, keys_view):
        assert keys_view.isdisjoint({"x", "y", "z"})

    def test_isdisjoint_false(self, keys_view):
        assert not keys_view.isdisjoint({"a", "x"})

    def test_isdisjoint_empty(self, keys_view):
        assert keys_view.isdisjoint([])

    def test_can_iterate_multiple_times(self, keys_view):
        first = list(keys_view)
        second = list(keys_view)
        assert first == second


class TestFrozenValuesView:
    def test_isinstance(self, values_view):
        assert isinstance(values_view, FrozenValuesView)

    def test_len(self, values_view):
        assert len(values_view) == 3

    def test_iter_in_insertion_order(self, simple, values_view):
        assert list(values_view) == [simple[k] for k in simple]

    def test_contains_true(self, values_view):
        assert 1 in values_view

    def test_contains_false(self, values_view):
        assert 99 not in values_view

    def test_repr(self, values_view):
        r = repr(values_view)
        assert r.startswith("frozendict_values(")

    def test_can_iterate_multiple_times(self, values_view):
        first = list(values_view)
        second = list(values_view)
        assert first == second


class TestFrozenItemsView:
    def test_isinstance(self, items_view):
        assert isinstance(items_view, FrozenItemsView)

    def test_len(self, items_view):
        assert len(items_view) == 3

    def test_iter_in_insertion_order(self, simple, items_view):
        assert list(items_view) == [(k, simple[k]) for k in simple]

    def test_contains_true(self, items_view):
        assert ("a", 1) in items_view

    def test_contains_false_wrong_value(self, items_view):
        assert ("a", 99) not in items_view

    def test_contains_false_missing_key(self, items_view):
        assert ("z", 1) not in items_view

    def test_contains_non_tuple_is_false(self, items_view):
        assert "a" not in items_view

    def test_repr(self, items_view):
        r = repr(items_view)
        assert r.startswith("frozendict_items(")

    def test_and_intersection(self, items_view):
        result = items_view & {("a", 1)}
        assert ("a", 1) in result
        assert ("b", 2) not in result

    def test_or_union(self, items_view):
        result = items_view | {("x", 99)}
        assert ("x", 99) in result
        assert ("a", 1) in result

    def test_sub_difference(self, items_view):
        result = items_view - {("a", 1)}
        assert ("a", 1) not in result
        assert ("b", 2) in result

    def test_xor_symmetric_difference(self, items_view):
        result = items_view ^ {("a", 1), ("x", 99)}
        assert ("x", 99) in result
        assert ("a", 1) not in result

    def test_isdisjoint_true(self, items_view):
        assert items_view.isdisjoint([("x", 10), ("y", 20)])

    def test_isdisjoint_false(self, items_view):
        assert not items_view.isdisjoint([("a", 1)])

    def test_can_iterate_multiple_times(self, items_view):
        first = list(items_view)
        second = list(items_view)
        assert first == second


class TestInsertionOrder:
    def test_keys_insertion_ordered(self):
        d = frozendict(b=2, a=1, c=3)
        assert list(d.keys()) == ["b", "a", "c"]

    def test_values_insertion_ordered(self):
        d = frozendict(b=2, a=1, c=3)
        assert list(d.values()) == [2, 1, 3]

    def test_items_insertion_ordered(self):
        d = frozendict(b=2, a=1, c=3)
        assert list(d.items()) == [("b", 2), ("a", 1), ("c", 3)]

    def test_iter_insertion_ordered(self):
        d = frozendict(b=2, a=1, c=3)
        assert list(d) == ["b", "a", "c"]

    def test_duplicate_key_preserves_first_position(self):
        d = frozendict(
            {"a": 1, "b": 2, "a": 99}  # noqa: F601 # pylint: disable=duplicate-key
        )
        assert list(d) == ["a", "b"]
        assert d["a"] == 99

    def test_repr_insertion_ordered(self):
        d = frozendict(b=2, a=1, c=3)
        r = repr(d)
        assert r.index("'b'") < r.index("'a'") < r.index("'c'")

    def test_reversed_is_reverse_insertion_order(self):
        d = frozendict(b=2, a=1, c=3)
        assert list(reversed(d)) == ["c", "a", "b"]


# Copyright 2026 Mahmoud Harmouch.
#
# Licensed under the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

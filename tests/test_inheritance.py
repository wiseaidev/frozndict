# Copyright 2026 Mahmoud Harmouch.
#
# Licensed under the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.
# pylint: disable=missing-function-docstring,redefined-outer-name

import collections.abc

import pytest

# pyrefly: ignore [missing-import]
from frozndict import frozendict


class ExtendedDict(frozendict):
    """A frozendict subclass that adds a `summary()` method."""

    def summary(self):
        """Return a human-readable summary string."""
        return f"ExtendedDict({len(self)} keys)"

    def key_list(self):
        """Return keys as a sorted list."""
        return sorted(self.keys())


class TypeTaggedDict(frozendict):
    """A frozendict subclass that validates all values share the same type."""

    def __new__(cls, *args, _value_type=None, **kwargs):
        instance = super().__new__(cls, *args, **kwargs)
        return instance

    def validate(self, value_type):
        """Return True if all values are instances of value_type."""
        return all(isinstance(v, value_type) for v in self.values())


@pytest.fixture
def ext():
    """An ExtendedDict with three entries."""
    return ExtendedDict(b=2, a=1, c=3)


@pytest.fixture
def tagged():
    """A TypeTaggedDict with integer values."""
    return TypeTaggedDict(x=1, y=2, z=3)


class TestSubclassing:
    def test_subclass_is_frozendict(self, ext):
        assert isinstance(ext, frozendict)

    def test_subclass_isinstance_of_subclass(self, ext):
        assert isinstance(ext, ExtendedDict)

    def test_subclass_custom_method(self, ext):
        assert "3 keys" in ext.summary()

    def test_subclass_key_list(self, ext):
        assert ext.key_list() == ["a", "b", "c"]

    def test_subclass_inherits_getitem(self, ext):
        assert ext["a"] == 1

    def test_subclass_inherits_len(self, ext):
        assert len(ext) == 3

    def test_subclass_inherits_contains(self, ext):
        assert "a" in ext
        assert "z" not in ext

    def test_subclass_inherits_hash(self, ext):
        assert isinstance(hash(ext), int)

    def test_subclass_inherits_immutability_setitem(self, ext):
        with pytest.raises(TypeError):
            ext["a"] = 99

    def test_subclass_inherits_immutability_delitem(self, ext):
        with pytest.raises(TypeError):
            del ext["a"]

    def test_subclass_inherits_repr(self, ext):
        assert repr(ext).startswith("frozendict({")

    def test_subclass_inherits_keys(self, ext):
        assert set(ext.keys()) == {"a", "b", "c"}

    def test_subclass_inherits_values(self, ext):
        assert set(ext.values()) == {1, 2, 3}

    def test_subclass_inherits_items(self, ext):
        assert set(ext.items()) == {("a", 1), ("b", 2), ("c", 3)}

    def test_subclass_inherits_or(self, ext):
        result = ext | frozendict(d=4)
        assert result["d"] == 4
        assert result["a"] == 1

    def test_subclass_inherits_copy(self, ext):
        c = ext.copy()
        assert c == ext

    def test_subclass_inherits_fromkeys(self):
        d = ExtendedDict.fromkeys(["x", "y"], 0)
        assert d["x"] == 0
        assert isinstance(d, frozendict)

    def test_two_subclasses_equal_same_content(self):
        d1 = ExtendedDict(a=1)
        d2 = ExtendedDict(a=1)
        assert d1 == d2

    def test_subclass_equal_to_frozendict_same_content(self, ext):
        fd = frozendict(b=2, a=1, c=3)
        assert ext == fd

    def test_validate_method(self, tagged):
        assert tagged.validate(int)
        assert not tagged.validate(str)


class TestMappingABC:
    def test_frozendict_is_mapping(self):
        assert isinstance(frozendict(a=1), collections.abc.Mapping)

    def test_frozendict_class_is_registered(self):
        assert issubclass(frozendict, collections.abc.Mapping)

    def test_subclass_is_mapping(self, ext):
        assert isinstance(ext, collections.abc.Mapping)

    def test_class_getitem_generic_alias(self):
        alias = frozendict[str, int]
        assert alias is not None

    def test_class_getitem_is_generic_alias(self):
        import types

        alias = frozendict[str, int]
        assert isinstance(alias, types.GenericAlias)


# Copyright 2026 Mahmoud Harmouch.
#
# Licensed under the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

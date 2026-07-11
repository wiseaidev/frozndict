# Copyright 2026 Mahmoud Harmouch.
#
# Licensed under the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.
# pylint: disable=missing-function-docstring,redefined-outer-name

from operator import ior

import pytest

# pyrefly: ignore [missing-import]
from frozndict import __version__, frozendict


def test_version():
    assert __version__ == "2.0.0"


@pytest.fixture
def frozen_empty_dict():
    return frozendict()


@pytest.fixture
def frozen_dict():
    return frozendict.fromkeys(("foo",), 1)


def test_empty_dict(frozen_empty_dict):
    assert frozen_empty_dict == frozendict({}) == frozendict([]) == frozendict({}, **{})


def test_vars(frozen_dict):
    with pytest.raises(TypeError):
        vars(frozen_dict)


def test_copy(frozen_dict):
    assert frozen_dict.copy() == frozen_dict


def test__dir(frozen_dict):
    with pytest.raises(AttributeError):
        dir(frozen_dict)


def test__del(frozen_dict):
    with pytest.raises(TypeError):
        del frozen_dict.foo


def test__delitem(frozen_dict):
    with pytest.raises(TypeError):
        del frozen_dict["foo"]


def test_setattr(frozen_dict):
    with pytest.raises(TypeError):
        frozen_dict.foo = 2


def test_setitem(frozen_dict):
    with pytest.raises(TypeError):
        frozen_dict["foo"] = 2


def test_pop(frozen_dict):
    with pytest.raises(TypeError):
        frozen_dict.pop()


def test__update(frozen_dict):
    with pytest.raises(TypeError):
        frozen_dict.update()


def test__clear(frozen_dict):
    with pytest.raises(TypeError):
        frozen_dict.clear()


def test__setdefault(frozen_dict):
    with pytest.raises(TypeError):
        frozen_dict.setdefault("foo")


def test_creation_inception(frozen_dict):
    frozen_dict = frozendict(frozen_dict)
    assert len(frozen_dict) == 1
    assert frozen_dict["foo"] == 1


def test_or(frozen_dict, frozen_empty_dict):
    result = frozen_dict | frozen_empty_dict
    assert result is not None
    assert result == frozen_dict
    assert result != frozen_empty_dict


def test_ior(frozen_dict, frozen_empty_dict):
    result = ior(frozen_dict, frozen_empty_dict)
    assert result is not None
    assert result == frozen_dict
    assert result != frozen_empty_dict


def test_hash_consistency():
    d1 = frozendict(a=1, b=2)
    d2 = frozendict(b=2, a=1)
    assert hash(d1) == hash(d2)
    assert d1 == d2


def test_fromkeys():
    d = frozendict.fromkeys(["a", "b"], 69)
    assert d["a"] == 69
    assert d["b"] == 69
    assert len(d) == 2


def test_pretty_repr():
    d = frozendict(a=1)
    assert "a" in d.pretty_repr()


# Copyright 2026 Mahmoud Harmouch.
#
# Licensed under the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

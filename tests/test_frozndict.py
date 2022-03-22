"""
frozndict is a python package that acts as an alternative to frozenset,
but for dictionaries.
Copyright (C) 2022  Mahmoud Harmouch

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.
"""


import pytest

from operator import (
    ior,
)

from frozndict import (
    __version__,
    frozendict,
)


def test_version():
    assert __version__ == "1.0.1"


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

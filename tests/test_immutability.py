# pylint: disable=missing-function-docstring,redefined-outer-name

# Copyright 2026 Mahmoud Harmouch.
#
# Licensed under the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import pytest

# pyrefly: ignore [missing-import]
from frozndict import frozendict


@pytest.fixture
def fd():
    """Return a single-entry frozendict: {'foo': 1}."""
    return frozendict(foo=1)


@pytest.fixture
def multi_fd():
    """Return a multi-entry frozendict."""
    return frozendict(a=1, b=2, c=3)


def test_setitem_raises_type_error(fd):
    with pytest.raises(TypeError):
        fd["foo"] = 99


def test_setitem_does_not_change_value(fd):
    try:
        fd["foo"] = 99
    except TypeError:
        pass
    assert fd["foo"] == 1


def test_delitem_raises_type_error(fd):
    with pytest.raises(TypeError):
        del fd["foo"]


def test_delitem_key_still_present_after_attempt(fd):
    try:
        del fd["foo"]
    except TypeError:
        pass
    assert "foo" in fd


def test_setattr_raises_type_error(fd):
    with pytest.raises(TypeError):
        fd.foo = 42


def test_delattr_raises_type_error(fd):
    with pytest.raises(TypeError):
        del fd.foo


def test_update_raises_type_error(fd):
    with pytest.raises(TypeError):
        fd.update(foo=2)


def test_clear_raises_type_error(fd):
    with pytest.raises(TypeError):
        fd.clear()


def test_pop_raises_type_error(fd):
    with pytest.raises(TypeError):
        fd.pop("foo")


def test_popitem_raises_type_error(fd):
    with pytest.raises(TypeError):
        fd.popitem()


def test_setdefault_raises_type_error(fd):
    with pytest.raises(TypeError):
        fd.setdefault("foo", 42)


def test_dir_raises_attribute_error(fd):
    with pytest.raises(AttributeError):
        dir(fd)


def test_vars_raises_type_error(fd):
    with pytest.raises(TypeError):
        vars(fd)


def test_length_unchanged_after_failed_setitem(multi_fd):
    original_len = len(multi_fd)
    try:
        multi_fd["new_key"] = 999
    except TypeError:
        pass
    assert len(multi_fd) == original_len


def test_contents_unchanged_after_multiple_failed_mutations(multi_fd):
    expected = dict(multi_fd.items())
    for _ in range(5):
        try:
            multi_fd["a"] = 999
        except TypeError:
            pass
        try:
            multi_fd.update(a=999)
        except TypeError:
            pass
    assert dict(multi_fd.items()) == expected


# Copyright 2026 Mahmoud Harmouch.
#
# Licensed under the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

# Copyright 2026 Mahmoud Harmouch.
#
# Licensed under the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.
# pylint: disable=missing-function-docstring,redefined-outer-name

import copy
import pickle

import pytest

# pyrefly: ignore [missing-import]
from frozndict import frozendict


@pytest.fixture
def simple():
    """frozendict with three entries."""
    return frozendict(a=1, b=2, c=3)


@pytest.fixture
def nested():
    """frozendict with a tuple value."""
    return frozendict(x=1, y=(1, 2, 3), z="hello")


@pytest.fixture
def empty():
    """Empty frozendict."""
    return frozendict()


class TestPickle:
    @pytest.mark.parametrize("protocol", range(0, pickle.HIGHEST_PROTOCOL + 1))
    def test_roundtrip_simple(self, simple, protocol):
        data = pickle.dumps(simple, protocol=protocol)
        loaded = pickle.loads(data)
        assert loaded == simple

    @pytest.mark.parametrize("protocol", range(0, pickle.HIGHEST_PROTOCOL + 1))
    def test_roundtrip_empty(self, empty, protocol):
        data = pickle.dumps(empty, protocol=protocol)
        loaded = pickle.loads(data)
        assert loaded == empty

    @pytest.mark.parametrize("protocol", range(0, pickle.HIGHEST_PROTOCOL + 1))
    def test_roundtrip_nested(self, nested, protocol):
        data = pickle.dumps(nested, protocol=protocol)
        loaded = pickle.loads(data)
        assert loaded == nested

    def test_pickled_is_frozendict(self, simple):
        loaded = pickle.loads(pickle.dumps(simple))
        assert isinstance(loaded, frozendict)

    def test_hash_preserved_after_pickle(self, simple):
        loaded = pickle.loads(pickle.dumps(simple))
        assert hash(loaded) == hash(simple)

    def test_pickled_is_not_same_object(self, simple):
        loaded = pickle.loads(pickle.dumps(simple))
        assert loaded is not simple

    def test_dumps_with_default_protocol(self, simple):
        data = pickle.dumps(simple, protocol=-1)
        loaded = pickle.loads(data)
        assert loaded == simple


class TestCopy:
    def test_copy_dot_copy_returns_equal(self, simple):
        c = copy.copy(simple)
        assert c == simple

    def test_copy_dot_copy_is_frozendict(self, simple):
        c = copy.copy(simple)
        assert isinstance(c, frozendict)

    def test_copy_dot_copy_is_not_same_object(self, simple):
        c = copy.copy(simple)
        assert c is not simple

    def test_copy_dot_copy_same_hash(self, simple):
        c = copy.copy(simple)
        assert hash(c) == hash(simple)

    def test_copy_method_equals_copy_module(self, simple):
        assert simple.copy() == copy.copy(simple)

    def test_copy_empty(self, empty):
        c = copy.copy(empty)
        assert c == empty


class TestDeepcopy:
    def test_deepcopy_returns_equal(self, simple):
        c = copy.deepcopy(simple)
        assert c == simple

    def test_deepcopy_is_frozendict(self, simple):
        c = copy.deepcopy(simple)
        assert isinstance(c, frozendict)

    def test_deepcopy_is_not_same_object(self, simple):
        c = copy.deepcopy(simple)
        assert c is not simple

    def test_deepcopy_same_hash(self, simple):
        c = copy.deepcopy(simple)
        assert hash(c) == hash(simple)

    def test_deepcopy_empty(self, empty):
        c = copy.deepcopy(empty)
        assert c == empty

    def test_deepcopy_values_preserved(self, nested):
        c = copy.deepcopy(nested)
        assert c["y"] == (1, 2, 3)


# Copyright 2026 Mahmoud Harmouch.
#
# Licensed under the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

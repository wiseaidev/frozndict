# Copyright 2026 Mahmoud Harmouch.
#
# Licensed under the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

"""
frozndict - A memory-efficient, fully immutable dictionary for Python,
powered by Rust.

This package exposes a single public mapping type, :class:`FrozenDict`,
compiled from Rust via PyO3/maturin, along with three view types:
:class:`FrozenKeysView`, :class:`FrozenValuesView`, and
:class:`FrozenItemsView`.

The ``frozendict`` name is provided as a convenience lowercase alias of
:class:`FrozenDict`.

New in 2.1
----------
- :class:`FrozenKeysView` and :class:`FrozenItemsView` support full set
  algebra: ``&``, ``|``, ``-``, ``^``, and ``isdisjoint``.
- ``copy.copy()`` and ``copy.deepcopy()`` work correctly (O(1), return self).
- Pickle is fully supported across all protocols 0-5.
- :class:`FrozenDict` is registered as a :class:`collections.abc.Mapping`.
- :class:`FrozenDict` supports subclassing for polymorphism.
- ``FrozenDict[str, int]`` generic subscript syntax is supported.
- ``__reversed__`` is implemented.
- Insertion order is preserved exactly as in CPython ``dict`` (Python 3.7+).
- ``|`` and ``__ror__`` are supported; ``|=`` falls back to ``__or__``.

Example
-------
>>> from frozndict import frozendict
>>> d = frozendict(a=1, b=2)
>>> d["a"]
1
>>> hash(d) == hash(frozendict(b=2, a=1))
True
>>> list(d.keys())
['a', 'b']
>>> import pickle
>>> pickle.loads(pickle.dumps(d)) == d
True
>>> import copy
>>> copy.copy(d) == d
True
"""

import collections.abc
import copyreg

# pylint: disable=no-name-in-module
from frozndict._frozndict import (
    FrozenDict,
    FrozenItemsView,
    FrozenKeysView,
    FrozenValuesView,
)

__author__ = "Mahmoud Harmouch"
__email__ = "oss@wiseai.dev"
__version__ = "2.1.0"

frozendict = FrozenDict

FrozenDict.__module__ = "frozndict"
FrozenDict.__qualname__ = "FrozenDict"
FrozenKeysView.__module__ = "frozndict"
FrozenValuesView.__module__ = "frozndict"
FrozenItemsView.__module__ = "frozndict"


def _frozendict_reduce(obj):
    """Return a ``(constructor, args)`` tuple for pickle serialisation.

    Uses the ``frozndict.frozendict`` constructor so that the pickled
    representation is importable from the public package rather than the
    internal ``_frozndict`` C extension.

    Parameters
    ----------
    obj:
        The :class:`FrozenDict` instance to pickle.

    Returns
    -------
    tuple
        A ``(callable, args)`` pair accepted by :mod:`pickle`.
    """
    return (frozendict, (dict(obj.items()),))


copyreg.dispatch_table[FrozenDict] = _frozendict_reduce

collections.abc.Mapping.register(FrozenDict)
collections.abc.Mapping.register(FrozenKeysView)
collections.abc.Mapping.register(FrozenValuesView)
collections.abc.Mapping.register(FrozenItemsView)

__all__ = [
    "FrozenDict",
    "FrozenItemsView",
    "FrozenKeysView",
    "FrozenValuesView",
    "frozendict",
]

# Copyright 2026 Mahmoud Harmouch.
#
# Licensed under the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

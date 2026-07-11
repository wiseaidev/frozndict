# Copyright 2026 Mahmoud Harmouch.
#
# Licensed under the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

"""
frozndict - A memory-efficient, fully immutable dictionary for Python,
powered by Rust.

This package exposes a single public type, :class:`FrozenDict`, compiled
from Rust via PyO3/maturin.  The ``frozendict`` name is provided as a
convenience lowercase alias.

Example:
    >>> from frozndict import frozendict
    >>> d = frozendict(a=1, b=2)
    >>> d["a"]
    1
    >>> hash(d) == hash(frozendict(b=2, a=1))
    True
"""

# pylint: disable=no-name-in-module
from frozndict._frozndict import FrozenDict

__author__ = "Mahmoud Harmouch"
__email__ = "oss@wiseai.dev"
__version__ = "2.0.0"

frozendict = FrozenDict

__all__ = [
    "FrozenDict",
    "frozendict",
]

# Copyright 2026 Mahmoud Harmouch.
#
# Licensed under the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

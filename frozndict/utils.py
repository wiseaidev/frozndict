"""
This is a utility script for frozndict
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

from typing import (
    Any,
    Callable,
    Hashable,
    NoReturn,
)


def block_function(func: Callable) -> Callable:
    def wrapper(cls, *args: Any, **kwags: Any) -> NoReturn:
        raise TypeError(
            f"'{cls.__class__.__name__.lower()}' is immutable -- create a copy instead of modifying."
        )

    return wrapper


def deny_access(func: Callable) -> Callable:
    def wrapper(*args: Any, **kwags: Any) -> NoReturn:
        raise AttributeError("Access is Denied!")

    return wrapper


def hash_args(args: Any) -> int:
    hash_value: int = 0
    # case 1: args have type dict
    # example: FrozenDict({'foo': 1},bar=2)
    if isinstance(args[0], dict):
        for key, value in args[0].items():
            if isinstance(value, Hashable):
                hash_value ^= hash((key, value))
            else:
                hash_value ^= id((key, value))
    # case 2: args have type tuple
    # example: FrozenDict((('foo', 1),),bar=2)
    elif isinstance(args[0], tuple):
        for key, value in args[0]:
            if isinstance(value, Hashable):
                hash_value ^= hash((key, value))
            else:
                hash_value ^= id((key, value))
    return hash_value


def hash_kwargs(kwargs: Any) -> int:
    hash_value: int = 0
    for key, value in kwargs.items():
        if isinstance(value, Hashable):
            hash_value ^= hash((key, value))
        else:
            hash_value ^= id((key, value))
    return hash_value


def indent(string: str, num_spaces: int) -> str:
    indent_str: str = " " * num_spaces
    return "\n".join(indent_str + line for line in string.split("\n")[:-1]) + "\n"

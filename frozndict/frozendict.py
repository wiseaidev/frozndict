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


from collections.abc import (
    Hashable,
)
from typing import (
    Any,
    Dict,
    Iterator,
    NoReturn,
    Optional,
    Tuple,
    TypeVar,
)

from frozndict.utils import (
    block_function,
    deny_access,
    hash_args,
    hash_kwargs,
    indent,
)

T = TypeVar("T", bound=Any)
TKey = TypeVar("TKey", bound=Hashable)
TValue = TypeVar("TValue")


class FrozenDict(Dict[TKey, TValue]):
    """
    This class implements an immutable Dict: God-tier immutability. It inherits most of its
    functionalities from the built-in Python dict. It mostly behaves like an usual dict, except
    the following:
        - Once an object is instantiated, its items and attributes cannot be altered.
        - Removing the __dict__ attribute and use __slots__ to reduce memory usage.
        - Deny the access to __dir__ on a given instance.
    These class have been implemented to avoid item's accidental overwrite/deletion.
    """

    __slots__: Tuple[str, ...] = ("__hash",)

    def __new__(cls, *args: T, **kwargs: T) -> "FrozenDict[TKey, TValue]":
        cls.__hash: int = 0
        # case 1: both args and kwargs are provided:
        # example: FrozenDict((('foo', 1),),bar=2)
        if args and kwargs:
            cls.__hash = hash_args(args)
        # case 2: args is provided and kwargs is empty:
        # example: FrozenDict((('foo', 1),))
        elif args:
            cls.__hash = hash_args(args)
        # case 3: kwargs is provided and args is empty:
        # example: FrozenDict(foo = 1, bar= 2)
        elif kwargs:
            cls.__hash = hash_kwargs(kwargs)
        return super().__new__(cls, args, kwargs)

    def __hash__(self) -> int:  # type: ignore
        if self.__hash:
            return self.__hash
        return hash(tuple(sorted(self.items())))

    def __iter__(self) -> Iterator[TKey]:
        yield from self.keys()

    def __repr__(self) -> str:
        return f"{self.__class__.__name__.lower()}({dict(self.items())!r})"

    def pretty_repr(self, num_spaces=4) -> str:
        """Returns an indented representation of the nested dictionary."""

        def pretty_dict(dict_):
            if not isinstance(dict_, dict):
                return repr(dict_)
            rep = []
            for key, val in dict_.items():
                rep.append(f"{key}: {pretty_dict(val)},\n")
            if rep:
                return f'{{\n{indent("".join(rep), num_spaces)}}}'
            else:
                return "{}"

        return f"{(self.__class__.__name__).lower()}({pretty_dict(dict(self.items()))})"

    @deny_access
    def __dir__(self) -> None:
        ...

    @block_function
    def __delattr__(self, name: str) -> NoReturn:
        """
        Deny del on an attribute of this instance.

        Example: del FrozenDict(foo = 1, bar= 2).foo
        """
        ...

    @block_function
    def __delitem__(self, key: TKey, value: TValue) -> NoReturn:
        """
        Deny del on an item of this instance.

        Example: del FrozenDict(foo = 1, bar= 2)['foo']
        """
        ...

    @block_function
    def __setattr__(self, name: str, value: TValue) -> NoReturn:
        """
        Deny attribute setter for this instance.

        Example: FrozenDict(foo = 1, bar= 2).foo = 2
        """
        ...

    @block_function
    def __setitem__(self, key: TKey, value: TValue) -> NoReturn:
        """
        Deny item setter for this instance.

        Example: FrozenDict(foo = 1, bar= 2)['foo'] = 2
        """
        ...

    @block_function
    def pop(self) -> TValue:
        """
        Deny pop of an item for this instance.

        Example: FrozenDict(foo = 1, bar= 2).pop()
        """
        ...

    @block_function
    def update(self) -> NoReturn:
        """
        Deny update of items for this instance.

        Example: FrozenDict(foo = 1, bar= 2).update({'foo': 2})
        """
        ...

    @block_function
    def setdefault(self, key: TKey, default: Optional[TValue] = None) -> TValue:
        """
        Deny setdefault for this instance.

        Example: FrozenDict(bar= 2).setdefault('foo')
        """
        ...

    @block_function
    def clear(self) -> NoReturn:
        """
        Deny clear for this instance.

        Example: FrozenDict(foo = 1, bar= 2).clear()
        """
        ...

    @block_function
    def popitem(self) -> Tuple[TKey, TValue]:
        """
        Deny popitem for this instance.

        Example: FrozenDict(foo = 1, bar= 2).popitem()
        """
        ...

    @classmethod
    def fromkeys(cls, *args, **kwargs):
        return cls(dict.fromkeys(*args, **kwargs))


frozendict = FrozenDict

__all__ = [
    "frozendict",
]

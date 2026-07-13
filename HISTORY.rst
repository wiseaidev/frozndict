=======
History
=======

2.1.0 (2026-07-13)
------------------

* **Insertion ordering**: keys, values, and items now iterate in
  first-insertion order, matching CPython ``dict`` (Python 3.7+).
* **View types with set algebra**: ``keys()`` and ``items()`` return
  ``FrozenKeysView`` / ``FrozenItemsView`` with ``&``, ``|``, ``-``,
  ``^``, and ``isdisjoint``; ``values()`` returns ``FrozenValuesView``.
* **Pickle**: full support for all protocols 0-5 via
  ``__reduce__`` + ``copyreg.dispatch_table``.
* **copy / deepcopy**: ``copy.copy()`` and ``copy.deepcopy()`` work
  in O(1) (immutable-object semantics, same as ``frozenset``).
* **Mapping ABC**: ``isinstance(fd, collections.abc.Mapping)`` is
  ``True``; all view types are registered too.
* **Subclassing**: ``FrozenDict`` can be subclassed from Python for
  polymorphism; ``__init_subclass__`` hook is provided.
* **Generic subscript**: ``frozendict[str, int]`` returns a
  ``types.GenericAlias``.
* **Reversed iteration**: ``reversed(fd)`` is supported.
* **``|`` / ``__ror__``**: ``dict | frozendict`` and ``frozendict | dict``
  both work; ``fd |= other`` rebinds via ``__or__``.
* **Arc inner**: all views share a single ``Arc<FrozenDictInner>``
  so ``keys()``, ``values()``, ``items()`` are O(1) with no copy.
* **O(1) ``__hash__``**: pre-computed at construction; order-independent.
* **O(1) short-circuit ``__eq__``**: hash mismatch exits without scan.

2.0.0 (2026-07-11)
------------------

* Complete rewrite in Rust with PyO3 / maturin.
* Added: O(1) cached hashing, ``fromkeys``, ``pretty_repr``, ``__or__`` merge.
* Strictly immutable: enforced at the Rust level.

1.0.0 (2022-03-21)
------------------

* First release on PyPI.

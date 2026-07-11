=========
frozndict
=========

.. image:: https://raw.githubusercontent.com/wiseaidev/frozndict/main/assets/banner.png
   :target: https://github.com/wiseaidev/frozndict/
   :alt: frozndict - Rust-Powered Immutable Dictionary
   :align: center

.. image:: https://img.shields.io/badge/License-MIT-blue.svg
   :target: https://github.com/wiseaidev/frozndict/blob/main/LICENSE
   :alt: License

.. image:: https://img.shields.io/pypi/v/frozndict.svg
   :target: https://pypi.org/project/frozndict/
   :alt: PyPI version

.. image:: https://img.shields.io/github/repo-size/wiseaidev/frozndict
   :target: https://github.com/wiseaidev/frozndict/
   :alt: Repo Size

.. image:: https://circleci.com/gh/wiseaidev/frozndict/tree/main.svg?style=svg
   :target: https://circleci.com/gh/wiseaidev/frozndict/tree/main
   :alt: CircleCI Build Status

**frozndict** is a production-grade, fully immutable Python dictionary powered by **Rust** and **PyO3**.
It is a drop-in companion to ``frozenset`` for dictionaries - hashable, thread-safe, and built for performance.

🚀 The Fastest, Most Efficient Immutable Dictionary
----------------------------------------------------

**frozndict** pushes immutable mappings to the absolute mathematical limits.
While every other library wraps a standard Python hash-table, **frozndict** is engineered ground-up in
**Rust** as a hash-sorted flat slice - a single contiguous heap block, binary-searched by native
``isize`` keys with zero FFI crossings during the search phase.

1. **🏆 Fastest Iteration in Class**
   Keys, values, and items lists are **pre-built once at construction time** as cached Python objects.
   Every subsequent ``keys()`` / ``values()`` / ``items()`` / iteration call is an O(1) reference
   clone - **no per-call allocation at all**.

2. **Deterministic O(1) Pre-Computed Hashing**
   The dict-level hash is XOR-combined over all ``(k, v)`` pairs exactly once at construction and
   stored as a native ``isize``. Every ``hash(d)`` call costs only a Python integer return - ~0.3 µs
   regardless of size. Regular ``dict`` *cannot be hashed at all*.

3. **🏆 Smallest Memory Footprint in Class**
   Entries sit in a single flat ``Box<[(isize, K, V)]>`` allocation - no bucket arrays, no load-factor
   slack, no pointer tables. Python sees exactly **64 bytes** regardless of element count vs 6,584 bytes
   for ``frozendict`` at n=200.

4. **Infallible Rust-Level Immutability**
   Mutation is blocked inside the Rust binary - not via Python descriptor tricks. There is no
   monkey-patchable ``__setattr__`` escape path.

Run the full benchmark suite locally::

    $ python3 -m venv .venv
    $ source .venv/bin/activate
    $ pip install maturin
    $ maturin develop --features python
    $ pip install frozendict
    $ python3 benchmarks/benchmark.py

+----------------------------------+-------------------+----------------+----------------+
| Metric (200 Elements)            | frozndict (Rust)  | frozendict (C) | dict           |
+==================================+===================+================+================+
| Memory Size **SMALLEST** 🏆      | **64 B**          | 6,584 B        | 6,576 B        |
+----------------------------------+-------------------+----------------+----------------+
| Lookup (Worst-Case)              | 0.756 µs          | **0.057 µs**   | 0.035 µs       |
+----------------------------------+-------------------+----------------+----------------+
| Iteration (keys) **FASTEST** 🏆  | **0.877 µs**      | 1.895 µs       | 1.977 µs       |
+----------------------------------+-------------------+----------------+----------------+
| Cached Hashing                   | 0.280 µs          | **0.199 µs**   | not hashable   |
+----------------------------------+-------------------+----------------+----------------+

.. note::

   ``dict`` cannot be used as a dictionary key or set member - it raises ``TypeError: unhashable type: 'dict'``.
   **frozndict** is fully hashable out of the box.

🛠️ Requirements
---------------

- Python 3.12+
- Rust 1.89+ (only when building from source)
- Maturin 1.14+ (only when building from source)

🚨 Installation
---------------

With ``pip``::

    python3 -m pip install frozndict

Build from source for development::

    git clone https://github.com/wiseaidev/frozndict.git
    cd frozndict
    python3 -m venv .venv
    source .venv/bin/activate
    pip install maturin
    maturin develop --features python

🚸 Usage
--------

.. code-block:: python3

   >>> from frozndict import frozendict

   # Empty immutable dictionary.
   >>> frozendict({})
   frozendict({})

   # From a plain dict.
   >>> frozen_dict = frozendict({"Greetings": "Hello World!"})
   >>> frozen_dict["Greetings"]
   'Hello World!'

   # From keyword arguments.
   >>> frozendict(a=1, b=2)
   frozendict({'a': 1, 'b': 2})

   # Pretty-printed representation (all values must be hashable).
   >>> print(frozendict(x=3, y=4, z="nested-str", c=1).pretty_repr())
   frozendict({
       'c': 1,
       'y': 4,
       'z': 'nested-str',
       'x': 3,
   })

   # Values must be hashable - frozndict hashes every (key, value) pair at construction.
   # Use frozendict/frozenset for nested immutable structures:
   >>> frozendict(x=1, tags=frozenset(["a", "b"]))
   frozendict({'x': 1, 'tags': frozenset({'b', 'a'})})

   # fromkeys constructor.
   >>> frozendict.fromkeys(["x", "y"], "5")
   frozendict({'x': '5', 'y': '5'})

   # Fully hashable - use as dictionary keys or set members.
   >>> set([frozendict(a=1, b=2), frozendict(a=5), frozendict(b=2, a=1)])
   {frozendict({'a': 1, 'b': 2}), frozendict({'a': 5})}

   # Hash is order-independent and O(1).
   >>> hash(frozendict(a=1, b=2)) == hash(frozendict(b=2, a=1))
   True

   # Mutation raises TypeError - enforced at the Rust binary level.
   >>> frozen_dict["x"] = 1
   TypeError: 'frozendict' object does not support mutation

🎉 Credits
----------

Built with: `python`_ · `PyO3`_ · `maturin`_ · `pytest`_ · `black`_ · `isort`_ · `flake8`_ · `precommit`_

👋 Contribute
-------------

Please refer to the `Guideline`_ for contribution instructions.

📝 License
----------

Released under the `MIT License`_.

.. _MIT License: https://opensource.org/licenses/MIT
.. _frozendict: https://pypi.org/project/frozendict/
.. _Marco Sulla: https://github.com/Marco-Sulla
.. _Guideline: https://github.com/wiseaidev/frozndict/blob/main/CONTRIBUTING.rst
.. _PyO3: https://pyo3.rs/
.. _maturin: https://github.com/PyO3/maturin
.. _python: https://www.python.org/
.. _pytest: https://docs.pytest.org/en/7.1.x/
.. _flake8: https://flake8.pycqa.org/en/latest/
.. _black: https://black.readthedocs.io/en/stable/
.. _isort: https://github.com/PyCQA/isort
.. _precommit: https://pre-commit.com/

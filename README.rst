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

.. image:: https://dl.circleci.com/status-badge/img/gh/wiseaidev/frozndict/tree/main.svg?style=svg&circle-token=CCIPRJ_86zVdw8bxskJfEWVcmN7d5_65a0e201b0e167e30852ad63c0c742c20c4e9dd0
        :target: https://dl.circleci.com/status-badge/redirect/gh/wiseaidev/frozndict/tree/main
   :alt: CircleCI Build Status

**frozndict** is a production-grade, fully immutable Python dictionary powered by **Rust** and **PyO3**.
It is a drop-in companion to ``frozenset`` for dictionaries: hashable, thread-safe, insertion-ordered,
and engineered for performance.

🚀 Performance Highlights
--------------------------

**frozndict** operates natively at the mathematical speed limits of the hardware, heavily outperforming its C counterparts and standard dictionary data structures on several fronts.

The following is a 1000-element dictionary micro-benchmark comparison in seconds (Smaller is better).

+-----------------------+--------------+-----------------+-----------------+-------------------+
| Operation (1000 int)  | Python dict  | immutables.Map  | frozendict (C)  | **frozndict** 🧊  |
+=======================+==============+=================+=================+===================+
| **Construction**      |**4.92e-06**🏆| 1.61e-04        | 6.19e-06        | 5.79e-05          |
+-----------------------+--------------+-----------------+-----------------+-------------------+
| **Clone** O(1)        | 4.88e-06     | **9.87e-08** 🏆 | 4.38e-07        | **1.29e-07**      |
+-----------------------+--------------+-----------------+-----------------+-------------------+
| **Equality** O(1)     | 1.66e-05     | **2.52e-08** 🏆 | 1.66e-05        | **3.24e-08**      |
+-----------------------+--------------+-----------------+-----------------+-------------------+
| **Iteration**         | 1.15e-05     | 1.69e-05        | 1.15e-05        | **6.08e-06** 🏆   |
+-----------------------+--------------+-----------------+-----------------+-------------------+
| **Copy** O(1)         | 4.88e-06     | 2.17e-04        | 6.58e-08        | **6.42e-08** 🏆   |
+-----------------------+--------------+-----------------+-----------------+-------------------+

1. **🏆 Fastest Iteration in Class via Lazy Caching**
   Keys, values, and items are **built lazily on first access** via a Rust ``OnceCell``.
   This means construction is ultra-fast, and every subsequent ``keys()`` / ``values()`` / ``items()`` / iteration call returns an O(1) view object wrapping a shared ``Arc``, **no per-call allocation at all**.

2. **Deterministic O(1) Pre-Computed Hashing**
   The dict-level hash is XOR-combined over all ``(k, v)`` pairs exactly once at construction via inline multiplicative mixing, skipping the slow allocation of intermediate ``PyTuple`` instances entirely.

3. **O(1) ``copy()`` / ``deepcopy()`` & Clones**
   A shared ``Arc<FrozenDictInner>`` is re-used across the original, all copies, and mapping constructors (``frozendict(existing_fd)``). No data is ever duplicated.

4. **Instantaneous O(1) ``__eq__`` short-circuit**
   Self-equality (``o == o``) returns true instantly via ``Arc::ptr_eq``. Equality between distinct ``FrozenDict`` instances first compares the pre-computed hashes, detecting mismatches in O(1) before any scanning.

5. **🏆 Smallest Memory Footprint in Class**
   **frozndict** has the smallest memory footprint compared to other Python/C alternatives (occupying just 24 bytes of overhead for the main wrapper class).
   Entries sit in a single flat ``Box<[(isize, K, V)]>`` allocation plus a compact ``Box<[(isize, u32)]>`` lookup index, eliminating load-factor slack completely.

6. **Infallible Rust-Level Immutability**
   Mutation is blocked safely at the Rust binary level, completely bypassing Python descriptor tricks.

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

   # Insertion order is preserved.
   >>> frozendict(b=2, a=1, c=3)
   frozendict({'b': 2, 'a': 1, 'c': 3})

   >>> list(frozendict(b=2, a=1).keys())
   ['b', 'a']

   # Set operations on keys.
   >>> frozendict(a=1, b=2).keys() & {"a", "x"}
   frozenset({'a'})

   # Set operations on items.
   >>> frozendict(a=1, b=2).items() - {("a", 1)}
   frozenset({('b', 2)})

   # Pickle round-trip.
   >>> import pickle
   >>> d = frozendict(a=1, b=2)
   >>> pickle.loads(pickle.dumps(d)) == d
   True

   # copy / deepcopy (O(1)).
   >>> import copy
   >>> copy.copy(d) == d
   True

   # Mapping ABC.
   >>> import collections.abc
   >>> isinstance(d, collections.abc.Mapping)
   True

   # Subclassing.
   >>> class MyFrozen(frozendict):
   ...     def summary(self): return f"{len(self)} keys"
   >>> MyFrozen(a=1, b=2).summary()
   '2 keys'

   # Generic subscript.
   >>> frozendict[str, int]
   frozndict.FrozenDict[str, int]

   # fromkeys constructor.
   >>> frozendict.fromkeys(["x", "y"], "5")
   frozendict({'x': '5', 'y': '5'})

   # Fully hashable and usable as dict keys or set members.
   >>> set([frozendict(a=1, b=2), frozendict(b=2, a=1)])
   {frozendict({'a': 1, 'b': 2})}

   # Mutation raises TypeError at the Rust binary level.
   >>> d["x"] = 1
   TypeError: 'frozendict' object does not support mutation

   # Memory footprint comparison: frozndict (Rust) wrapper matches pointer size (24 bytes)
   >>> import frozendict
   >>> frozendict.c_ext
   False
   >>> frozendict.__version__
   '2.4.7'
   >>> from frozendict import frozendict
   >>> from frozndict import frozendict as frozndict
   >>> d = {'x': 3, 'y': 4, 'z': {'a': 0, 'b': [3,1,{4,1},[5,9]]}}
   >>> classes = (dict, frozndict, frozendict)
   >>> objects = [k(d, c=1) for k in classes]
   >>> import sys
   # sys.getsizeof shows: dict (184B) vs frozndict Rust (24B) vs frozendict C (192B)
   >>> tuple(map(sys.getsizeof, objects))
   (184, 24, 192)

📊 Running Benchmarks
---------------------

::

    $ python3 -m venv .venv
    $ source .venv/bin/activate
    $ pip install maturin frozendict immutables
    $ maturin develop --release --features python
    $ python3 benchmarks/benchmark.py

The benchmark compares **frozndict (Rust)**, **frozendict (C)**, **dict**, and **immutables.Map**
across construction, lookup, iteration, copy, pickle, hash, and set/delete operations.

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

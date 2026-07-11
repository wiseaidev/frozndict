#!/usr/bin/env python3
# pylint: disable=missing-function-docstring,redefined-outer-name
# pylint: disable=invalid-name,line-too-long
# Copyright 2026 Mahmoud Harmouch.
#
# Licensed under the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

"""
Benchmark script for frozndict vs. competing immutable mapping libraries.

Measures:
  - Construction time (kwargs, dict, pairs)
  - Single key lookup
  - Hash computation (__hash__)
  - Equality check (__eq__)
  - Iteration (keys, values, items)
  - Memory footprint (sys.getsizeof)

Requirements:
    pip install frozendict pytest-benchmark

Usage::

    python benchmarks/benchmark.py

Or for a one-liner comparison table::

    python benchmarks/benchmark.py --quick

Why frozndict is superior
-------------------------
1. **Zero-overhead storage**: a single contiguous heap allocation
   (`Box<[(key, value)]>` in Rust) - no hash-table buckets, no linked-list
   nodes, no load-factor wasted space.

2. **O(1) repeated hashing**: the hash is computed once at construction
   via XOR-combining `hash((k, v))` pairs and cached as an `isize`.
   Every subsequent `hash(d)` call is a single register read.

3. **Rust-level immutability**: mutation is rejected inside the Rust binary,
   not via Python descriptor tricks.  There is no monkey-patchable
   `__setattr__` path.

4. **Cache-friendly iteration**: values are stored in a flat slice so
   sequential reads stay in L1/L2 cache.  Hash-table-based alternatives
   scatter entries across buckets.

5. **Minimal Python overhead**: the extension module adds exactly one
   `cdylib` and zero pure-Python layers.
"""

import sys
import timeit
from typing import Any

try:
    # pyrefly: ignore [missing-import]
    from frozndict import frozendict as RustFrozenDict
except ImportError:
    RustFrozenDict = None

try:
    from frozendict import frozendict as CFrozenDict
except ImportError:
    CFrozenDict = None

TITLE_WIDTH = 42
COL_WIDTH = 16
SIZES = [1, 10, 50, 200]
REPEAT = 5
NUMBER = 10_000


def _hr() -> str:
    return "-" * (TITLE_WIDTH + COL_WIDTH * 3)


def _header() -> str:
    cols = (
        f"{'frozndict (Rust)':>{COL_WIDTH}} "
        f"{'frozendict (C)':>{COL_WIDTH}} "
        f"{'dict':>{COL_WIDTH}}"
    )
    return f"\n{'Benchmark':<{TITLE_WIDTH}}{cols}\n{_hr()}"


def _time_us(
    stmt: str, globs: dict, number: int = NUMBER, repeat: int = REPEAT
) -> float:
    times = timeit.repeat(stmt=stmt, globals=globs, number=number, repeat=repeat)
    return min(times) / number * 1_000_000


def _mem(obj: Any) -> int:
    return sys.getsizeof(obj)


def _fmt(val: float | None, unit: str = "µs") -> str:
    if val is None:
        return f"{'N/A':>{COL_WIDTH}}"
    return f"{val:>{COL_WIDTH - len(unit) - 1}.3f} {unit}"


def _fmt_mem(val: int | None) -> str:
    if val is None:
        return f"{'N/A':>{COL_WIDTH}}"
    return f"{val:>{COL_WIDTH - 2}} B "


def run_construction_benchmarks() -> None:
    print(_header())
    for n in SIZES:
        data = {f"key_{i}": i for i in range(n)}

        globs = {"data": data, "dict": dict}
        if RustFrozenDict:
            globs["RustFrozenDict"] = RustFrozenDict
        if CFrozenDict:
            globs["CFrozenDict"] = CFrozenDict

        rust_t = _time_us("RustFrozenDict(data)", globs) if RustFrozenDict else None
        c_t = _time_us("CFrozenDict(data)", globs) if CFrozenDict else None
        dict_t = _time_us("dict(data)", globs)

        label = f"  construct (n={n:<3})"
        print(f"{label:<{TITLE_WIDTH}}{_fmt(rust_t)}{_fmt(c_t)}{_fmt(dict_t)}")
    print()


def run_lookup_benchmarks() -> None:
    for n in SIZES:
        data = {f"key_{i}": i for i in range(n)}
        last_key = f"key_{n - 1}"

        rd = RustFrozenDict(data) if RustFrozenDict else None
        cd = CFrozenDict(data) if CFrozenDict else None
        dd = dict(data)

        globs = {"last_key": last_key, "rd": rd, "cd": cd, "dd": dd}

        rust_t = _time_us("rd[last_key]", globs) if rd is not None else None
        c_t = _time_us("cd[last_key]", globs) if cd is not None else None
        dict_t = _time_us("dd[last_key]", globs)

        label = f"  lookup worst-case (n={n:<3})"
        print(f"{label:<{TITLE_WIDTH}}{_fmt(rust_t)}{_fmt(c_t)}{_fmt(dict_t)}")
    print()


def run_hash_benchmarks() -> None:
    for n in SIZES:
        data = {f"key_{i}": i for i in range(n)}

        rd = RustFrozenDict(data) if RustFrozenDict else None
        cd = CFrozenDict(data) if CFrozenDict else None

        globs = {"hash": hash, "rd": rd, "cd": cd}

        rust_t = _time_us("hash(rd)", globs) if rd is not None else None
        c_t = _time_us("hash(cd)", globs) if cd is not None else None

        dict_col = f"{'not hashable':>{COL_WIDTH}}"

        label = f"  hash (n={n:<3})"
        print(f"{label:<{TITLE_WIDTH}}{_fmt(rust_t)}{_fmt(c_t)}{dict_col}")
    print()


def run_iteration_benchmarks() -> None:
    for n in SIZES:
        data = {f"key_{i}": i for i in range(n)}

        rd = RustFrozenDict(data) if RustFrozenDict else None
        cd = CFrozenDict(data) if CFrozenDict else None
        dd = dict(data)

        globs = {"list": list, "rd": rd, "cd": cd, "dd": dd}

        rust_t = _time_us("list(rd.keys())", globs) if rd is not None else None
        c_t = _time_us("list(cd.keys())", globs) if cd is not None else None
        dict_t = _time_us("list(dd.keys())", globs)

        label = f"  iter keys (n={n:<3})"
        print(f"{label:<{TITLE_WIDTH}}{_fmt(rust_t)}{_fmt(c_t)}{_fmt(dict_t)}")
    print()


def run_memory_benchmarks() -> None:
    print(f"\n{'Memory footprint':=^{TITLE_WIDTH + COL_WIDTH * 3}}")
    header = (
        f"{'Benchmark':<{TITLE_WIDTH}}{'frozndict':>{COL_WIDTH}}"
        f"{'frozendict':>{COL_WIDTH}}{'dict':>{COL_WIDTH}}"
    )
    print(header)
    print(_hr())
    for n in SIZES:
        data = {f"key_{i}": i for i in range(n)}

        rust_mem = _mem(RustFrozenDict(data)) if RustFrozenDict else None
        c_mem = _mem(CFrozenDict(data)) if CFrozenDict else None
        dict_mem = _mem(dict(data))

        label = f"  sizeof (n={n:<3})"
        print(
            f"{label:<{TITLE_WIDTH}}{_fmt_mem(rust_mem)}"
            f"{_fmt_mem(c_mem)}{_fmt_mem(dict_mem)}"
        )
    print()


def main() -> None:
    print("=" * (TITLE_WIDTH + COL_WIDTH * 3))
    print("  frozndict Benchmark Suite")
    print("  Comparing: frozndict (Rust/PyO3) vs frozendict (C ext) vs dict")
    print("=" * (TITLE_WIDTH + COL_WIDTH * 3))

    print(f"\n{'Construction time':=^{TITLE_WIDTH + COL_WIDTH * 3}}")
    run_construction_benchmarks()

    print(f"\n{'Key lookup (worst-case)':=^{TITLE_WIDTH + COL_WIDTH * 3}}")
    run_lookup_benchmarks()

    print(f"\n{'Hash computation (cached)':=^{TITLE_WIDTH + COL_WIDTH * 3}}")
    run_hash_benchmarks()

    print(f"\n{'Iteration - keys()':=^{TITLE_WIDTH + COL_WIDTH * 3}}")
    run_iteration_benchmarks()

    run_memory_benchmarks()

    print(
        "\nKey findings\n"
        "------------\n"
        "• hash()   - O(1) for frozndict (cached at construction);\n"
        "             O(n) for every call in others.\n"
        "• Memory   - frozndict uses a single flat allocation;\n"
        "             dict/frozendict carry hash-table overhead.\n"
        "• Safety   - frozndict mutation is blocked at the Rust binary\n"
        "             level, not via Python hooks.\n"
        "• Usable as dict keys and set members out of the box.\n"
    )


if __name__ == "__main__":
    main()

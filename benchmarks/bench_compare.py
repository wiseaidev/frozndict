#!/usr/bin/env python3
"""
bench_compare.py: Cross-library micro-benchmark comparison.

Compares:
  - Python built-in dict
  - frozendict (C extension)
  - immutables.Map (Rust-backed)
  - frozndict (our Rust via PyO3)

Usage:
  pip install frozendict immutables frozndict
  python benchmarks/bench_compare.py

Outputs a Markdown table ready to paste into README.md.
"""
from __future__ import annotations

import copy
import sys
import timeit
from typing import Any

N = 1000
REPEAT = 7
NUMBER = 2000

DATA: dict[str, Any] = {str(i): i for i in range(N)}
KEYS = list(DATA.keys())

results: dict[str, dict[str, float]] = {}


def bench(stmt: str, setup: str = "", globals_: dict | None = None) -> float:
    g = globals_ or {}
    times = timeit.repeat(stmt, setup=setup, repeat=REPEAT, number=NUMBER, globals=g)
    return min(times) / NUMBER


py_dict = dict(DATA)

results["Python dict"] = {
    "Construction": bench("dict(DATA)", globals_={"DATA": DATA}),
    "Clone O(1)":   bench("d.copy()", globals_={"d": py_dict}),
    "Equality":     bench("d == d", globals_={"d": py_dict}),
    "Iteration":    bench("list(d)", globals_={"d": py_dict}),
    "copy()":       bench("copy.copy(d)", globals_={"d": py_dict, "copy": copy}),
    "hash()":       None,  # dict is not hashable
    "Lookup":       bench("d['500']", globals_={"d": py_dict}),
}

try:
    from frozendict import frozendict as FD_C

    fd_c = FD_C(DATA)
    results["frozendict (C)"] = {
        "Construction": bench("FD_C(DATA)", globals_={"FD_C": FD_C, "DATA": DATA}),
        "Clone O(1)":   bench("fd.copy()", globals_={"fd": fd_c}),
        "Equality":     bench("fd == fd", globals_={"fd": fd_c}),
        "Iteration":    bench("list(fd)", globals_={"fd": fd_c}),
        "copy()":       bench("copy.copy(fd)", globals_={"fd": fd_c, "copy": copy}),
        "hash()":       bench("hash(fd)", globals_={"fd": fd_c}),
        "Lookup":       bench("fd['500']", globals_={"fd": fd_c}),
    }
except ImportError:
    results["frozendict (C)"] = {}
    print("  [skipped] frozendict not installed", file=sys.stderr)

try:
    from immutables import Map as IMap

    im = IMap(DATA)
    results["immutables.Map"] = {
        "Construction": bench("IMap(DATA)", globals_={"IMap": IMap, "DATA": DATA}),
        "Clone O(1)":   bench("m.set('_tmp', 1)", globals_={"m": im}),  # cheapest mutation → new map
        "Equality":     bench("m == m", globals_={"m": im}),
        "Iteration":    bench("list(m.keys())", globals_={"m": im}),
        "copy()":       bench("copy.copy(m)", globals_={"m": im, "copy": copy}),
        "hash()":       bench("hash(m)", globals_={"m": im}),
        "Lookup":       bench("m['500']", globals_={"m": im}),
    }
except ImportError:
    results["immutables.Map"] = {}
    print("  [skipped] immutables not installed", file=sys.stderr)

try:
    from frozndict import FrozenDict as FD_RS

    fd_rs = FD_RS(DATA)
    results["frozndict 🧊"] = {
        "Construction": bench("FD_RS(DATA)", globals_={"FD_RS": FD_RS, "DATA": DATA}),
        "Clone O(1)":   bench("FD_RS(fd)", globals_={"FD_RS": FD_RS, "fd": fd_rs}),
        "Equality":     bench("fd == fd", globals_={"fd": fd_rs}),
        "Iteration":    bench("list(fd)", globals_={"fd": fd_rs}),
        "copy()":       bench("fd.copy()", globals_={"fd": fd_rs}),
        "hash()":       bench("hash(fd)", globals_={"fd": fd_rs}),
        "Lookup":       bench("fd['500']", globals_={"fd": fd_rs}),
    }
except ImportError:
    results["frozndict 🧊"] = {}
    print("  [skipped] frozndict not installed", file=sys.stderr)


def fmt(v: float | None) -> str:
    if v is None:
        return "N/A"
    if v < 1e-6:
        return f"{v * 1e9:.2f} ns"
    if v < 1e-3:
        return f"{v * 1e6:.2f} µs"
    return f"{v * 1e3:.2f} ms"


OPS = ["Construction", "Clone O(1)", "Equality", "Iteration", "copy()", "hash()", "Lookup"]
LIBS = list(results.keys())


def trophy(op: str) -> str:
    """Return the library name with the best (lowest) time for `op`."""
    best_lib = None
    best_t = float("inf")
    for lib, rr in results.items():
        t = rr.get(op)
        if t is not None and t < best_t:
            best_t = t
            best_lib = lib
    return best_lib or ""


print(f"\n### Performance: {N}-element dictionary (smaller is better)\n")
header = "| Operation" + "".join(f" | {lib}" for lib in LIBS) + " |"
sep    = "|" + "|".join(["-" * (len(s) + 2) for s in ["Operation"] + LIBS]) + "|"
print(header)
print(sep)

for op in OPS:
    winner = trophy(op)
    row = f"| {op}"
    for lib in LIBS:
        t = results[lib].get(op)
        cell = fmt(t)
        if lib == winner and t is not None:
            cell += " 🏆"
        row += f" | {cell}"
    row += " |"
    print(row)

print()
print("Benchmarked with `timeit` (min of 7 runs × 2 000 iterations).")
print(f"N = {N}, Python {sys.version.split()[0]}")

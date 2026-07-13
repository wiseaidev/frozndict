#!/usr/bin/env python3
# Copyright 2026 Mahmoud Harmouch.
#
# Licensed under the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

"""
Comprehensive benchmark for frozndict vs frozendict (C) vs dict vs immutables.Map.

Measures timing with statistical filtering (minimum + 3-sigma outlier removal)
and reports time-per-operation alongside the standard deviation around the
minimum.  Each benchmark is run for at least ``bench_time`` seconds to ensure
reliable results.

Usage::

    python benchmarks/benchmark.py          # full run (~10 s per benchmark)
    python benchmarks/benchmark.py 1        # one-shot (faster, less accurate)

Requirements::

    pip install frozendict immutables
"""

import sys
import timeit
import uuid
from copy import copy
from math import sqrt
from time import time

try:
    from frozndict import frozendict as RustFrozenDict
except ImportError:
    RustFrozenDict = None

try:
    from frozendict import frozendict as CFrozenDict
except ImportError:
    CFrozenDict = None

try:
    # pyrefly: ignore [missing-import]
    import immutables

    ImmutableMap = immutables.Map  # pylint: disable=invalid-name
except ImportError:
    ImmutableMap = None  # pylint: disable=invalid-name


def mindev(data, xbar=None):
    """
    Compute the standard deviation relative to the **minimum** of ``data``.

    Unlike a classical standard deviation (which uses the arithmetic mean),
    this function uses ``xbar`` (defaulting to ``min(data)``) as the reference
    point.  This is appropriate for benchmark timing data where the minimum
    represents the best achievable time and the distribution is right-skewed.

    Parameters
    ----------
    data:
        Non-empty sequence of floats.
    xbar:
        Reference value.  Defaults to ``min(data)``.

    Returns
    -------
    float
        Standard deviation around ``xbar``.
    """
    if not data:
        raise ValueError("No data")
    if xbar is None:
        xbar = min(data)
    sigma2 = sum((x - xbar) ** 2 for x in data)
    n = max(len(data) - 1, 1)
    return sqrt(sigma2 / n)


def autorange(stmt, setup="pass", globals=None, ratio=1000, bench_time=2, number=None):
    # pylint: disable=too-many-arguments,too-many-locals,redefined-builtin
    """
    Adaptively time ``stmt``, filtering outliers via 3-sigma rejection.

    The number of inner loop iterations is chosen so that the total time of
    one repeat is roughly ``1/ratio`` of the ``autorange`` estimate.  The
    benchmark then repeats until ``bench_time`` seconds elapse.  Data points
    more than 3 standard deviations above the current minimum are removed
    iteratively.

    Parameters
    ----------
    stmt:
        String statement to benchmark (forwarded to :class:`timeit.Timer`).
    setup:
        Setup code run once per :class:`timeit.Timer` instance.
    globals:
        Global namespace for both ``stmt`` and ``setup``.
    ratio:
        Divisor applied to the :meth:`Timer.autorange` loop count to derive
        the inner loop count used for each data point.
    bench_time:
        Minimum wall-clock seconds to collect data.
    number:
        If provided, forces the inner loop count and runs only one repeat.

    Returns
    -------
    tuple[float, float]
        ``(min_per_call, sigma_per_call)`` both in seconds.
    """
    if setup is None:
        setup = "pass"

    t = timeit.Timer(stmt=stmt, setup=setup, globals=globals)
    break_immediately = False

    if number is None:
        a = t.autorange()
        number_temp = a[0]
        number = max(int(number_temp / ratio), 1)
        repeat = max(int(number_temp / number), 1)
    else:
        repeat = 1
        break_immediately = True

    data_min = list(t.repeat(number=number, repeat=repeat))
    bench_start = time()

    while True:
        data_min.extend(t.repeat(number=number, repeat=repeat))
        if break_immediately or time() - bench_start > bench_time:
            break

    data_min.sort()
    xbar = data_min[0]
    i = 0
    while i < len(data_min):
        i = len(data_min)
        sigma = mindev(data_min, xbar=xbar)
        for i2 in range(2, len(data_min)):
            if data_min[i2] - xbar > 3 * sigma:
                break
        k = max(i, 5)
        del data_min[k:]

    return (min(data_min) / number, mindev(data_min, xbar=xbar) / number)


def get_uuid():
    """Return a new random UUID string."""
    return str(uuid.uuid4())


PRINT_TPL = (
    "Name: {name: <26} Size: {size: >4}; Keys: {keys: >3}; "
    "Type: {type: >12}; Time: {time:.2e}; Sigma: {sigma:.0e}"
)

BENCH_HASH_NAME = "hash"
BENCH_COPY_NAME = "copy"
BENCH_SET_NAME = "set"
BENCH_DELETE_NAME = "delete"
BENCH_FROMKEYS_NAME = "fromkeys"
BENCH_CONSTR_KWARGS_NAME = "constructor(kwargs)"

BENCHMARKS = (
    {
        "name": "constructor(d)",
        "code": "klass(d)",
        "setup": "klass = type(o)",
    },
    {
        "name": BENCH_CONSTR_KWARGS_NAME,
        "code": "klass(**d)",
        "setup": "klass = type(o)",
    },
    {
        "name": "constructor(seq2)",
        "code": "klass(v)",
        "setup": "klass = type(o); v = tuple(d.items())",
    },
    {
        "name": "constructor(o)",
        "code": "klass(o)",
        "setup": "klass = type(o)",
    },
    {
        "name": BENCH_COPY_NAME,
        "code": None,
        "setup": "pass",
    },
    {
        "name": "o == o",
        "code": "o == o",
        "setup": "pass",
    },
    {
        "name": "for x in o",
        "code": "for _ in o: pass",
        "setup": "pass",
    },
    {
        "name": "for x in o.values()",
        "code": "for _ in values: pass",
        "setup": "values = o.values()",
    },
    {
        "name": "for x in o.items()",
        "code": "for _ in items: pass",
        "setup": "items = o.items()",
    },
    {
        "name": "pickle.dumps",
        "code": "dumps(o, protocol=-1)",
        "setup": "from pickle import dumps",
    },
    {
        "name": "pickle.loads",
        "code": "loads(dump)",
        "setup": "from pickle import loads, dumps; dump = dumps(o, protocol=-1)",
    },
    {
        "name": BENCH_FROMKEYS_NAME,
        "code": "fromkeys(keys)",
        "setup": "fromkeys = type(o).fromkeys; keys = list(o.keys())",
    },
    {
        "name": BENCH_SET_NAME,
        "code": None,
        "setup": "val = get_uuid()",
    },
    {
        "name": BENCH_DELETE_NAME,
        "code": None,
        "setup": "pass",
    },
    {
        "name": BENCH_HASH_NAME,
        "code": "hash(o)",
        "setup": "pass",
    },
)


def main(number):
    # pylint: disable=too-many-locals,too-many-branches
    # pylint: disable=too-many-statements,too-many-nested-blocks
    """
    Run the full benchmark matrix and print results to stdout.

    Iterates over all benchmark definitions, dictionary sizes, and key types.
    Each combination is timed for at least ``bench_time`` seconds using
    :func:`autorange` with outlier filtering.

    Parameters
    ----------
    number:
        Fixed inner-loop count passed to :func:`autorange`.  ``None`` means
        the count is determined automatically.
    """
    dictionary_sizes = (5, 1000)
    sep_n = 80
    sep_major = "#"
    sep_minor = "/"

    str_key = "12323f29-c31f-478c-9b15-e7acc5354df9"
    int_key = max(dictionary_sizes[0] - 2, 0)

    dict_collection = []
    for n in dictionary_sizes:
        d1 = {}
        d2 = {}
        for i in range(n - 1):
            d1[get_uuid()] = get_uuid()
            d2[i] = i
        d1[str_key] = get_uuid()
        d2[999] = 999

        entries = [(d1, str_key, "str"), (d2, int_key, "int")]
        dict_collection.append((n, entries))

    for benchmark in BENCHMARKS:
        print(sep_major * sep_n)

        for _n, entries in dict_collection:
            for plain_dict, one_key, key_label in entries:
                if benchmark["name"] == BENCH_CONSTR_KWARGS_NAME and key_label == "int":
                    continue

                candidates = []
                if dict is not None:
                    candidates.append((plain_dict, "dict"))
                if RustFrozenDict is not None:
                    candidates.append((RustFrozenDict(plain_dict), "FrozenDict"))
                if CFrozenDict is not None:
                    candidates.append((CFrozenDict(plain_dict), "frozendict"))
                if ImmutableMap is not None:
                    candidates.append((ImmutableMap(plain_dict), "Map"))

                print(sep_minor * sep_n)
                for o, _ in candidates:
                    if benchmark["name"] == BENCH_HASH_NAME and isinstance(o, dict):
                        continue

                    if benchmark["name"] == BENCH_SET_NAME:
                        if isinstance(o, dict):
                            benchmark["code"] = "o.copy()[one_key] = val"
                        else:
                            benchmark["code"] = (
                                "o.set(one_key, val)"
                                if type(o).__name__ == "Map"
                                else "type(o)({**o, one_key: val})"
                            )

                    if benchmark["name"] == BENCH_DELETE_NAME:
                        if isinstance(o, dict):
                            benchmark["code"] = "del o.copy()[one_key]"
                        else:
                            benchmark["code"] = (
                                "o.delete(one_key)"
                                if type(o).__name__ == "Map"
                                else "{k: v for k, v in o.items() if k != one_key}"
                            )

                    if benchmark["name"] == BENCH_COPY_NAME:
                        if type(o).__name__ == "Map":
                            benchmark["code"] = "copy(o)"
                        else:
                            benchmark["code"] = "o.copy()"

                    if (
                        benchmark["name"] == BENCH_FROMKEYS_NAME
                        and type(o).__name__ == "Map"
                    ):
                        continue

                    if benchmark["name"] in (
                        "pickle.dumps",
                        "pickle.loads",
                    ) and isinstance(o, dict):
                        continue

                    bench_res = autorange(
                        stmt=benchmark["code"],
                        setup=benchmark["setup"],
                        globals={
                            "o": copy(o) if isinstance(o, dict) else o,
                            "get_uuid": get_uuid,
                            "d": plain_dict.copy(),
                            "one_key": one_key,
                            "copy": copy,
                        },
                        number=number,
                    )

                    print(
                        PRINT_TPL.format(
                            name=f"`{benchmark['name']}`; ",
                            keys=key_label,
                            size=len(o),
                            type=type(o).__name__,
                            time=bench_res[0],
                            sigma=bench_res[1],
                        )
                    )

    print(sep_major * sep_n)


if __name__ == "__main__":
    NUM = None
    ARGV = sys.argv
    LEN_ARGV = len(ARGV)
    MAX_POSITIONAL_ARGS = 1
    MAX_LEN_ARGV = MAX_POSITIONAL_ARGS + 1

    if LEN_ARGV > MAX_LEN_ARGV:
        raise ValueError(
            f"{__name__} must not accept more than "
            f"{MAX_POSITIONAL_ARGS} positional command-line parameters"
        )

    if LEN_ARGV == MAX_LEN_ARGV:
        NUM = int(ARGV[MAX_POSITIONAL_ARGS])

    main(NUM)

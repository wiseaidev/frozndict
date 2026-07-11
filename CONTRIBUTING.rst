.. highlight:: shell

.. Copyright 2026 Mahmoud Harmouch.
..
.. Licensed under the MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>.
.. This file may not be copied, modified, or distributed except according to those terms.

============
Contributing
============

Contributions are welcome, and they are greatly appreciated!  Every little bit
helps, and credit will always be given.

Get Started!
------------

Ready to contribute?  Here is how to set up **frozndict** for local development.

Prerequisites
~~~~~~~~~~~~~

You will need the following tools installed:

* `Rust <https://www.rust-lang.org/tools/install>`_ (stable toolchain via ``rustup``)
* `maturin <https://github.com/PyO3/maturin>`_ (``pip install maturin``)
* Python 3.12 or above
* ``git``

Setup
~~~~~

1. Fork the ``frozndict`` repository on GitHub.

2. Clone your fork locally::

    $ git clone git@github.com:your_name_here/frozndict.git
    $ cd frozndict

3. Create a virtual environment and install the package in editable (development) mode::

    $ python3 -m venv .venv
    $ source .venv/bin/activate          # On Windows: .venv\Scripts\activate
    $ pip install maturin pytest
    $ maturin develop --features python

4. Create a branch for your bugfix or feature::

    $ git checkout -b name-of-your-bugfix-or-feature

   Now you can make your changes locally.

Running Tests
~~~~~~~~~~~~~

After making changes, run the full test suite::

    $ pytest tests/ -v
    $ cargo test --all-features

Running Linters
~~~~~~~~~~~~~~~

Keep the code clean before submitting::

    $ cargo clippy --all-targets --all-features -- -D warnings
    $ cargo fmt --all -- --check
    $ pre-commit run --all-files

Pull Request Guidelines
-----------------------

Before you submit a pull request, check that it meets these guidelines:

1. The pull request **must include tests**.
2. If the pull request adds functionality, update the docs accordingly.
   Add a doc comment with a ``# Complexity`` section to every new Rust function.
3. The pull request should work for **Python 3.12** and above.
4. All four CI checks must pass:

   - **🦀 Rust Tests** - ``cargo test --all-features``
   - **🧪 Python Tests** - ``pytest tests/ -v``
   - **🧹 Clippy** - zero warnings
   - **🦀 Fmt** - ``cargo fmt --all -- --check``

5. All public Rust items must be documented with ``///`` doc comments following
   the style established in ``src/frozen_map.rs`` and ``src/python.rs``:

   * Module-level ``//!`` banner
   * Per-function ``# Complexity`` section (Time + Space)
   * No inline comments inside function bodies

Deploying a Release
-------------------

A reminder for the maintainers.

1. Make sure all changes are committed and ``HISTORY.rst`` is updated.

2. Bump the version::

    $ bump2version patch   # or major / minor

   This updates ``Cargo.toml``, ``python/frozndict/__init__.py``, and
   ``tests/test_frozndict.py`` atomically.

3. Push the commit and tag::

    $ git push
    $ git push --tags

4. The ``publish.yml`` GitHub Actions workflow will automatically:

   * Build multi-platform wheels with ``maturin``
   * Publish to PyPI via OIDC trusted publishing (no token needed)
   * Publish the Rust crate to ``crates.io``

Tips
----

To run a subset of the Python tests::

    $ pytest tests/test_immutability.py -v
    $ pytest tests/test_integration.py -v
    $ pytest tests/ -k "hash" -v

To open Rust API documentation locally::

    $ cargo doc --open

.. Copyright 2026 Mahmoud Harmouch.
..
.. Licensed under the MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>.

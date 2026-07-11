// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # frozndict
//!
//! A memory-efficient, fully immutable dictionary for Python, powered by Rust.
//!
//! ## Overview
//!
//! `frozndict` provides a zero-overhead, hashable, immutable mapping type that
//! can be used wherever a frozen alternative to Python's built-in `dict` is
//! needed.
//!
//! The core data structure, [`frozen_map::FrozenMap`], stores entries in a
//! single sorted heap allocation.  Lookups run in O(log n) via binary search;
//! the hash is computed once at construction and cached, making repeated
//! `hash()` calls O(1).
//!
//! The Python class [`python::FrozenDict`] wraps [`frozen_map::FrozenMap`] and
//! is exposed to Python as the `FrozenDict` (and its `frozendict` alias) type.
//!
//! ## Modules
//!
//! - [`frozen_map`] - The core Rust data structure.
//! - [`python`] - PyO3 Python bindings (gated on the `python` feature flag).

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod frozen_map;

#[cfg(feature = "python")]
pub mod python;

#[cfg(feature = "python")]
use pyo3::prelude::*;

/// Entry point for the `_frozndict` Python extension module.
///
/// This function is called by Python's import machinery when the compiled
/// extension is imported.  It registers the [`python::FrozenDict`] class and
/// sets the module version string.
#[cfg(feature = "python")]
#[pymodule]
fn _frozndict(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    crate::python::register_python_module(py, m)?;
    Ok(())
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

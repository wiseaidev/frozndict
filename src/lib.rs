// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![cfg_attr(feature = "std", doc = include_str!("../README.md"))]
#![cfg_attr(feature = "std", doc = include_str!("../RUST.md"))]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/wiseaidev/frozndict/refs/heads/main/assets/banner.png",
    html_favicon_url = "https://raw.githubusercontent.com/wiseaidev/frozndict/refs/heads/main/assets/favicon.png"
)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), doc = "")]
#![cfg_attr(feature = "std", doc = include_str!("../README.md"))]
#![cfg_attr(feature = "std", doc = include_str!("../RUST.md"))]
#![cfg_attr(not(feature = "node"), forbid(unsafe_code))]
#![cfg_attr(docsrs, feature(doc_cfg))]

extern crate alloc;

pub mod frozen_map;

#[cfg(all(feature = "python", feature = "std"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "python", feature = "std"))))]
pub mod python;

#[cfg(all(feature = "node", feature = "std"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "node", feature = "std"))))]
pub mod node;

#[cfg(all(feature = "python", feature = "std"))]
use pyo3::prelude::*;

#[cfg(all(feature = "python", feature = "std"))]
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

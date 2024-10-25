// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

extern crate arbolta as arbol;

pub mod conversion;
pub mod design;

use pyo3::prelude::*;

#[pymodule]
fn arbolta(m: &Bound<'_, PyModule>) -> PyResult<()> {
  m.add_class::<design::PyDesign>()?;

  Ok(())
}

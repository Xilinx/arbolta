// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

use arbol::bit::BitVec;
use num_traits::PrimInt;
use numpy::{PyReadonlyArray1, PyReadwriteArray1};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

pub fn bits_to_bool_numpy(bits: &BitVec, numpy_array: &Bound<'_, PyAny>) -> PyResult<()> {
  let mut buffer = numpy_array.extract::<PyReadwriteArray1<bool>>()?;
  match bits.to_bool_ndarray_buffer(buffer.as_array_mut()) {
    Ok(()) => Ok(()),
    Err(err) => Err(PyValueError::new_err(format!("{err}"))),
  }
}

pub fn bool_numpy_to_bits(numpy_array: &Bound<'_, PyAny>) -> PyResult<BitVec> {
  let buffer = numpy_array.extract::<PyReadonlyArray1<bool>>()?;

  match BitVec::from_bool_ndarray(buffer.as_array()) {
    Ok(bits) => Ok(bits),
    Err(err) => Err(PyValueError::new_err(format!("{err}"))),
  }
}

pub fn bits_to_int_numpy<T: PrimInt + std::ops::BitXorAssign + numpy::Element>(
  bits: &BitVec,
  elem_size: usize,
  numpy_array: &Bound<'_, PyAny>,
) -> PyResult<()> {
  let mut buffer = numpy_array.extract::<PyReadwriteArray1<T>>()?;

  match bits.to_int_ndarray_sized_buffer(elem_size, buffer.as_array_mut()) {
    Ok(()) => Ok(()),
    Err(err) => Err(PyValueError::new_err(format!("{err}"))),
  }
}

pub fn int_numpy_to_bits<T: PrimInt + numpy::Element>(
  numpy_array: &Bound<'_, PyAny>,
  elem_size: usize,
) -> PyResult<BitVec> {
  let buffer = numpy_array.extract::<PyReadonlyArray1<T>>()?;

  match BitVec::from_int_ndarray_sized(buffer.as_array(), elem_size) {
    Ok(bits) => Ok(bits),
    Err(err) => Err(PyValueError::new_err(format!("{err}"))),
  }
}

// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

use crate::bit::{Bit, BitVec};
use crate::signal::{AccessSignal, SignalIndexList, SignalList};
use ndarray::{Array1, ArrayView1};
use num_traits::PrimInt;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum PortDirection {
  Input,
  Output,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Port {
  pub signal_idx_list: SignalIndexList,
  pub shape: [usize; 2],
  pub direction: PortDirection,
  pub signed: bool,
}

#[derive(Debug, Error)]
pub enum PortError {
  #[error("tried to set input port")]
  Direction,
  #[error("couldn't convert port to type")]
  Conversion,
  #[error("incompatible shapes: requested={requested:?}, actual={actual:?}")]
  Shape {
    requested: [usize; 2],
    actual: [usize; 2],
  },
}

impl Port {
  pub fn set_shape(&mut self, shape: &[usize; 2]) -> Result<(), PortError> {
    if shape[0] * shape[1] != self.signal_idx_list.len() {
      return Err(PortError::Shape {
        requested: *shape,
        actual: self.shape,
      });
    }

    (self.shape[0], self.shape[1]) = (shape[0], shape[1]);

    Ok(())
  }

  pub fn get_shape(&self) -> [usize; 2] {
    self.shape
  }

  pub fn get_bits(&self, signals: &SignalList) -> BitVec {
    BitVec::from(
      self
        .signal_idx_list
        .iter()
        .map(|idx| signals[*idx].get_value())
        .collect::<Vec<Bit>>(),
    )
  }

  pub fn set_bits(&self, vals: &BitVec, signals: &mut SignalList) -> Result<(), PortError> {
    if self.direction == PortDirection::Output {
      return Err(PortError::Direction);
    }

    let stop_idx = vals.bits.len();

    for (i, val) in vals
      .bits
      .iter()
      .enumerate()
      .take(stop_idx.clamp(0, self.signal_idx_list.len()))
    {
      signals[self.signal_idx_list[i]].set_value(*val);
    }

    Ok(())
  }

  pub fn get_int<T: PrimInt + std::ops::BitXorAssign>(&self, signals: &SignalList) -> T {
    self.get_bits(signals).to_int()
  }

  pub fn set_int<T: PrimInt + std::fmt::Display>(
    &self,
    val: T,
    signals: &mut SignalList,
  ) -> Result<(), PortError> {
    if self.direction == PortDirection::Output {
      return Err(PortError::Direction);
    }

    let Ok(bits) = BitVec::from_int(val) else {
      return Err(PortError::Direction);
    };

    self.set_bits(&bits, signals)
  }

  pub fn get_int_vec<T: PrimInt + std::ops::BitXorAssign>(&self, signals: &SignalList) -> Vec<T> {
    let elem_size = self.shape[1];
    self.get_bits(signals).to_ints_sized(elem_size)
  }

  pub fn set_int_vec<T: PrimInt>(
    &self,
    vals: &[T],
    signals: &mut SignalList,
  ) -> Result<(), PortError> {
    if vals.len() != self.shape[0] {
      return Err(PortError::Shape {
        requested: [vals.len(), std::mem::size_of::<T>() * 8],
        actual: self.shape,
      });
    }

    let elem_size = self.shape[1];

    match BitVec::from_ints_sized(vals, elem_size) {
      Ok(bits) => self.set_bits(&bits, signals),
      Err(_) => Err(PortError::Conversion),
    }
  }

  pub fn get_ndarray<T: PrimInt + std::ops::BitXorAssign>(
    &self,
    signals: &SignalList,
  ) -> Array1<T> {
    let elem_size = self.shape[1];
    self.get_bits(signals).to_int_ndarray_sized(elem_size)
  }

  pub fn set_ndarray<T: PrimInt>(
    &self,
    vals: ArrayView1<T>,
    signals: &mut SignalList,
  ) -> Result<(), PortError> {
    if vals.len() != self.shape[0] {
      return Err(PortError::Shape {
        requested: [vals.len(), std::mem::size_of::<T>() * 8],
        actual: self.shape,
      });
    }

    let elem_size = self.shape[1];

    match BitVec::from_int_ndarray_sized(vals, elem_size) {
      Ok(bits) => self.set_bits(&bits, signals),
      Err(_) => Err(PortError::Conversion),
    }
  }

  pub fn get_string(&self, signals: &SignalList) -> String {
    self.get_bits(signals).to_string()
  }
}

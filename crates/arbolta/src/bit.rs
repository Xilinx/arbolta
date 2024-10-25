// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

use core::fmt;
use ndarray::{Array1, ArrayView1, ArrayViewMut1};
use num_traits::PrimInt;
use serde::{Deserialize, Serialize};
use std::convert::{From, Into};
use std::fmt::Debug;
use std::ops::{BitAnd, BitOr, BitXor, Not};
use std::str::FromStr;
use thiserror::Error;

/// Primitive signal value
#[derive(Debug, Clone, Eq, Copy, PartialEq, Deserialize, Serialize, Default)]
pub enum Bit {
  #[default]
  Zero,
  One,
}

#[derive(Debug, PartialEq, Eq, Error)]
#[error("error converting bits")]
pub struct ParseBitError;

impl From<bool> for Bit {
  fn from(val: bool) -> Self {
    if val {
      Self::One
    } else {
      Self::Zero
    }
  }
}

impl From<Bit> for bool {
  fn from(val: Bit) -> Self {
    match val {
      Bit::Zero => false,
      Bit::One => true,
    }
  }
}

impl TryFrom<char> for Bit {
  type Error = ParseBitError;
  fn try_from(val: char) -> Result<Self, Self::Error> {
    match val {
      '0' => Ok(Self::Zero),
      '1' => Ok(Self::One),
      _ => Err(ParseBitError),
    }
  }
}

impl From<Bit> for char {
  fn from(bit: Bit) -> Self {
    match bit {
      Bit::Zero => '0',
      Bit::One => '1',
    }
  }
}

impl Bit {
  pub fn from_int<T: PrimInt>(val: T) -> Result<Self, ParseBitError> {
    if val == T::zero() {
      Ok(Self::Zero)
    } else if val == T::one() {
      Ok(Self::One)
    } else {
      Err(ParseBitError)
    }
  }

  pub fn to_int<T: PrimInt>(self) -> T {
    match self {
      Self::Zero => T::zero(),
      Self::One => T::one(),
    }
  }
}

impl FromStr for Bit {
  type Err = ParseBitError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let int_val = s.parse::<usize>().or(Err(ParseBitError))?;
    Self::from_int(int_val)
  }
}

impl Not for Bit {
  type Output = Self;

  fn not(self) -> Self::Output {
    match self {
      Bit::Zero => Bit::One,
      Bit::One => Bit::Zero,
    }
  }
}

impl BitAnd for Bit {
  type Output = Self;

  fn bitand(self, rhs: Self) -> Self::Output {
    match &[self, rhs] {
      [Bit::Zero, Bit::Zero] | [Bit::Zero, Bit::One] | [Bit::One, Bit::Zero] => Bit::Zero,
      [Bit::One, Bit::One] => Bit::One,
    }
  }
}

impl BitOr for Bit {
  type Output = Self;

  fn bitor(self, rhs: Self) -> Self::Output {
    match &[self, rhs] {
      [Bit::Zero, Bit::Zero] => Bit::Zero,
      [Bit::Zero, Bit::One] | [Bit::One, Bit::Zero] | [Bit::One, Bit::One] => Bit::One,
    }
  }
}

impl BitXor for Bit {
  type Output = Self;

  fn bitxor(self, rhs: Self) -> Self::Output {
    match &[self, rhs] {
      [Bit::Zero, Bit::Zero] | [Bit::One, Bit::One] => Bit::Zero,
      [Bit::Zero, Bit::One] | [Bit::One, Bit::Zero] => Bit::One,
    }
  }
}

impl fmt::Display for Bit {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", <Self as Into<char>>::into(*self))
  }
}

/// Structure for storing+manipulating a vector of `Bit`s
#[derive(Debug, PartialEq, Eq)]
pub struct BitVec {
  pub bits: Vec<Bit>,
}

impl fmt::Display for BitVec {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let bit_string: String = self
      .bits
      .iter()
      .rev()
      .map(|b| <Bit as Into<char>>::into(*b))
      .collect();
    write!(f, "{bit_string}")
  }
}

impl FromStr for BitVec {
  type Err = ParseBitError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut bits: Vec<Bit> = vec![];
    for c in s.chars().rev() {
      bits.push(Bit::try_from(c)?);
    }
    Ok(Self { bits })
  }
}

impl From<Vec<Bit>> for BitVec {
  fn from(bits: Vec<Bit>) -> Self {
    Self { bits }
  }
}

impl From<BitVec> for Vec<Bit> {
  fn from(val: BitVec) -> Self {
    val.bits
  }
}

impl From<&BitVec> for Vec<bool> {
  fn from(val: &BitVec) -> Self {
    val.bits.iter().rev().map(|b| (*b).into()).collect()
  }
}

impl From<BitVec> for Vec<bool> {
  fn from(val: BitVec) -> Self {
    val.bits.iter().rev().map(|b| (*b).into()).collect()
  }
}

impl From<&[bool]> for BitVec {
  fn from(vals: &[bool]) -> Self {
    let bits: Vec<Bit> = vals.iter().rev().map(|b| (*b).into()).collect();
    Self::from(bits)
  }
}

impl From<Vec<bool>> for BitVec {
  fn from(vals: Vec<bool>) -> Self {
    Self::from(vals.as_slice())
  }
}

impl TryFrom<&str> for BitVec {
  type Error = ParseBitError;
  fn try_from(value: &str) -> Result<Self, Self::Error> {
    let mut bits: Vec<Bit> = vec![];
    for c in value.chars().rev() {
      bits.push(Bit::try_from(c)?);
    }
    Ok(Self { bits })
  }
}

impl From<BitVec> for String {
  fn from(val: BitVec) -> Self {
    val
      .bits
      .iter()
      .rev()
      .map(|b| <Bit as Into<char>>::into(*b))
      .collect()
  }
}
impl Default for BitVec {
  fn default() -> Self {
    Self::new()
  }
}

impl BitVec {
  pub fn new() -> Self {
    Self { bits: vec![] }
  }

  /// Convert int to vector of `Bit`s.
  ///
  /// # Arguments
  /// * `val` - Int to convert.
  /// * `size` - Number of bits to use.
  fn int_to_bits_sized<T: PrimInt>(val: T, size: usize) -> Result<Vec<Bit>, ParseBitError> {
    let mut bits: Vec<Bit> = vec![];
    for n in 0..size {
      bits.push(Bit::from_int((val >> n) & T::one())?);
    }

    Ok(bits)
  }

  /// Convert `Bit`s to int.
  /// Automatically extends sign if target int type is signed.
  fn bits_to_int<T: PrimInt + std::ops::BitXorAssign>(bits: &[Bit]) -> T {
    let mut val: T;
    let bit_ints: Vec<T> = bits.iter().map(|b| b.to_int()).collect();

    // Signed bits, need to sign extend
    if *bit_ints.last().unwrap() == T::one() && T::min_value() != T::zero() {
      val = !T::zero();
      bit_ints
        .iter()
        .enumerate()
        .for_each(|(i, b)| val ^= (*b ^ T::one()) << i);
    } else {
      val = T::zero();
      bit_ints
        .iter()
        .enumerate()
        .for_each(|(i, b)| val ^= *b << i);
    }

    val
  }

  /// Convert slice of `Bit`s to vector of ints.
  ///
  /// # Arguments
  /// * `bits` - Slice of `Bit`s to convert.
  /// * `elem_size` - Number of bits per int.
  fn bits_to_ints<T: PrimInt + std::ops::BitXorAssign>(bits: &[Bit], elem_size: usize) -> Vec<T> {
    bits
      .chunks(elem_size)
      .map(|chunk| Self::bits_to_int(chunk))
      .collect()
  }

  /// Convert slice of `Bit`s to vector of ints and store in buffer.
  ///
  /// # Arguments
  /// * `bits` - Slice of `Bit`s to convert.
  /// * `elem_size` - Number of bits per int.
  /// * `buffer` - Buffer to store ints.
  fn bits_to_ints_buffer<T: PrimInt + std::ops::BitXorAssign>(
    bits: &[Bit],
    elem_size: usize,
    buffer: &mut [T],
  ) {
    bits
      .chunks(elem_size)
      .enumerate()
      .for_each(|(i, chunk)| buffer[i] = Self::bits_to_int(chunk));
  }
  // --- Integer Conversion Helpers ---

  /// Create from int.
  ///
  /// # Arguments
  /// * `val` - Int to convert.
  /// * `size` - Number of bits to use.
  pub fn from_int_sized<T: PrimInt>(val: T, size: usize) -> Result<Self, ParseBitError> {
    let bits = Self::int_to_bits_sized(val, size)?;
    Ok(Self::from(bits))
  }

  /// Create from int.
  ///
  /// # Arguments
  /// * `val` - Int to convert.
  pub fn from_int<T: PrimInt>(val: T) -> Result<Self, ParseBitError> {
    let type_size = std::mem::size_of::<T>() * 8; // bytes to bits
    let bits = Self::int_to_bits_sized(val, type_size)?;
    Ok(Self::from(bits))
  }

  /// Convert to int.
  pub fn to_int<T: PrimInt + std::ops::BitXorAssign>(&self) -> T {
    Self::bits_to_int(&self.bits)
  }

  /// Create from slice of ints.
  ///
  /// # Arguments
  /// * `vals` - Ints to convert.
  /// * `elem_size` - Number of bits per int.
  pub fn from_ints_sized<T: PrimInt>(vals: &[T], elem_size: usize) -> Result<Self, ParseBitError> {
    let mut bits: Vec<Bit> = vec![];
    for val in vals {
      bits.append(&mut Self::int_to_bits_sized(*val, elem_size)?);
    }

    Ok(Self::from(bits))
  }

  /// Create from slice of ints.
  ///
  /// # Arguments
  /// * `vals` - Ints to convert.
  pub fn from_ints<T: PrimInt>(vals: &[T]) -> Result<Self, ParseBitError> {
    let type_size = std::mem::size_of::<T>() * 8; // bytes to bits
    Self::from_ints_sized(vals, type_size)
  }

  /// Convert to vector of ints.
  pub fn to_ints<T: PrimInt + std::ops::BitXorAssign>(&self) -> Vec<T> {
    let type_size = std::mem::size_of::<T>() * 8; // bytes to bits
    Self::bits_to_ints(&self.bits, type_size)
  }

  /// Convert to ints and store in buffer.
  ///
  /// # Arguments
  /// * `buffer` - Buffer to store ints.
  pub fn to_ints_buffer<T: PrimInt + std::ops::BitXorAssign>(&self, buffer: &mut [T]) {
    let type_size = std::mem::size_of::<T>() * 8; // bytes to bits
    Self::bits_to_ints_buffer(&self.bits, type_size, buffer);
  }

  /// Convert to vector of ints.
  ///
  /// # Arguments
  /// * `elem_size` - Number of bits per int.
  pub fn to_ints_sized<T: PrimInt + std::ops::BitXorAssign>(&self, elem_size: usize) -> Vec<T> {
    Self::bits_to_ints(&self.bits, elem_size)
  }

  /// Convert to ints and store in buffer.
  ///
  /// # Arguments
  /// * `elem_size` - Number of bits per int.
  /// * `buffer` - Buffer to store ints.
  pub fn to_ints_sized_buffer<T: PrimInt + std::ops::BitXorAssign>(
    &self,
    elem_size: usize,
    buffer: &mut [T],
  ) {
    Self::bits_to_ints_buffer(&self.bits, elem_size, buffer);
  }

  /// Create from `ndarray` of ints.
  ///
  /// # Arguments
  /// * `vals` - Ints to convert.
  /// * `elem_size` - Number of bits per int.
  pub fn from_int_ndarray_sized<T: PrimInt>(
    vals: ArrayView1<T>,
    elem_size: usize,
  ) -> Result<Self, ParseBitError> {
    let mut bits: Vec<Bit> = vec![];
    for val in vals {
      bits.append(&mut Self::int_to_bits_sized(*val, elem_size)?);
    }
    Ok(Self::from(bits))
  }

  /// Create from `ndarray` of ints.
  ///
  /// # Arguments
  /// * `vals` - Ints to convert.
  pub fn from_int_ndarray<T: PrimInt>(vals: ArrayView1<T>) -> Result<Self, ParseBitError> {
    let type_size = std::mem::size_of::<T>() * 8; // bytes to bits
    Self::from_int_ndarray_sized(vals, type_size)
  }

  /// Create from `ndarray` of bools.
  ///
  /// # Arguments
  /// * `vals` - Bools to convert.
  pub fn from_bool_ndarray(vals: ArrayView1<bool>) -> Result<Self, ParseBitError> {
    match vals.as_slice() {
      None => Err(ParseBitError),
      Some(buffer_slice) => Ok(Self::from(buffer_slice)),
    }
  }

  /// Convert to `ndarray` of ints.
  ///
  /// # Arguments
  /// * `elem_size` - Number of bits per int.
  pub fn to_int_ndarray_sized<T: PrimInt + std::ops::BitXorAssign>(
    &self,
    elem_size: usize,
  ) -> Array1<T> {
    Array1::from_vec(Self::bits_to_ints(&self.bits, elem_size))
  }

  /// Convert to ints and store in `ndarray`.
  ///
  /// # Arguments
  /// * `elem_size` - Number of bits per int.
  /// * `buffer` - `ndarray` buffer to store ints.
  pub fn to_int_ndarray_sized_buffer<T: PrimInt + std::ops::BitXorAssign>(
    &self,
    elem_size: usize,
    mut buffer: ArrayViewMut1<T>,
  ) -> Result<(), ParseBitError> {
    match buffer.as_slice_mut() {
      None => Err(ParseBitError),
      Some(buffer_slice) => {
        Self::bits_to_ints_buffer(&self.bits, elem_size, buffer_slice);
        Ok(())
      }
    }
  }

  /// Convert to ints and store in `ndarray`.
  ///
  /// # Arguments
  /// * `buffer` - `ndarray` buffer to store ints.
  pub fn to_int_ndarray_buffer<T: PrimInt + std::ops::BitXorAssign>(
    &self,
    mut buffer: ArrayViewMut1<T>,
  ) -> Result<(), ParseBitError> {
    let type_size = std::mem::size_of::<T>() * 8; // bytes to bits
    match buffer.as_slice_mut() {
      None => Err(ParseBitError),
      Some(buffer_slice) => {
        Self::bits_to_ints_buffer(&self.bits, type_size, buffer_slice);
        Ok(())
      }
    }
  }

  /// Convert to `ndarray` of ints.
  pub fn to_int_ndarray<T: PrimInt + std::ops::BitXorAssign>(&self) -> Array1<T> {
    let type_size = std::mem::size_of::<T>() * 8; // bytes to bits
    Array1::from_vec(Self::bits_to_ints(&self.bits, type_size))
  }

  /// Convert to bools and store in `ndarray`.
  ///
  /// # Arguments
  /// * `buffer` - `ndarray` buffer to store bools.
  pub fn to_bool_ndarray_buffer(
    &self,
    mut buffer: ArrayViewMut1<bool>,
  ) -> Result<(), ParseBitError> {
    match buffer.as_slice_mut() {
      None => Err(ParseBitError),
      Some(buffer_slice) => {
        self
          .bits
          .iter()
          .enumerate()
          .for_each(|(i, b)| buffer_slice[i] = (*b).into());
        Ok(())
      }
    }
  }
}

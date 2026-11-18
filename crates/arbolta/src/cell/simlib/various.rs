// Copyright (c) 2026 Alexander Redding
// SPDX-License-Identifier: MIT

use super::{CellFn, copy_nets};
use crate::{
  bit::{Bit, BitVec},
  signal::Signals,
};
use derive_more::Constructor;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct ProcBuffer {
  a_nets: Box<[usize]>,
  y_nets: Box<[usize]>,
}

impl CellFn for ProcBuffer {
  #[inline]
  fn eval(&mut self, signals: &mut Signals) {
    copy_nets(signals, &self.a_nets, &self.y_nets);
  }

  fn reset(&mut self) {}
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct Pos {
  signed: bool,
  a_nets: Box<[usize]>,
  y_nets: Box<[usize]>,
}

impl CellFn for Pos {
  #[inline]
  fn eval(&mut self, signals: &mut Signals) {
    // Need to pad w/ possible sign extension
    let pad_size = self.y_nets.len().saturating_sub(self.a_nets.len());
    let pad_net = self.a_nets.last().copied().unwrap_or(0) * self.signed as usize;

    let a_nets = self
      .a_nets
      .iter()
      .copied()
      .chain(std::iter::repeat_n(pad_net, pad_size));

    copy_nets(signals, a_nets, &self.y_nets);
  }

  fn reset(&mut self) {}
}

#[derive(Debug, Clone, Serialize, Deserialize, Constructor)]
pub struct Mux {
  pub select_net: usize,
  pub a_nets: Box<[usize]>,
  pub b_nets: Box<[usize]>,
  pub y_nets: Box<[usize]>,
}

impl CellFn for Mux {
  #[inline]
  fn eval(&mut self, signals: &mut Signals) {
    let src_nets = match signals.get_net(self.select_net) {
      Bit::ZERO => &self.a_nets,
      Bit::ONE => &self.b_nets,
    };
    copy_nets(signals, src_nets, &self.y_nets);
  }

  fn reset(&mut self) {}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BMux {
  // Nets
  select_nets: Box<[usize]>,
  a_nets: Box<[usize]>,
  y_nets: Box<[usize]>,
  // Bits
  select_bits: BitVec,
}

impl BMux {
  pub fn new(select_nets: Box<[usize]>, a_nets: Box<[usize]>, y_nets: Box<[usize]>) -> Self {
    Self {
      select_bits: BitVec::new(select_nets.len()),
      select_nets,
      a_nets,
      y_nets,
    }
  }
}

// "Selects between 'slices' of A where each value of S corresponds to a unique"
impl CellFn for BMux {
  #[inline]
  fn eval(&mut self, signals: &mut Signals) {
    // Read select
    self
      .select_bits
      .set_bits(self.select_nets.iter().map(|&n| signals.get_net(n)));

    let select = self.select_bits.to_int::<usize>();

    let start_net = (select * self.y_nets.len()) + self.a_nets[0];
    let end_net = start_net + self.y_nets.len();

    copy_nets(signals, start_net..end_net, &self.y_nets);
  }

  fn reset(&mut self) {}
}

// "Selects between 'slices' of B where each slice corresponds to a single bit
// of S. Outputs A when all bits of S are low."
#[derive(Debug, Clone, Serialize, Deserialize, Constructor)]
pub struct PMux {
  select_nets: Box<[usize]>,
  a_nets: Box<[usize]>, // output when S all low
  b_nets: Box<[usize]>, // slices source
  y_nets: Box<[usize]>, // slice size
}

impl CellFn for PMux {
  #[inline]
  fn eval(&mut self, signals: &mut Signals) {
    let mut out_nets = self.a_nets.iter();
    for (i, &select_net) in self.select_nets.iter().enumerate() {
      if signals.get_net(select_net) == Bit::ONE {
        let index = i * self.a_nets.len();
        out_nets = self.b_nets[index..index + self.a_nets.len()].iter();
      }
    }

    copy_nets(signals, out_nets, &self.y_nets);
  }

  fn reset(&mut self) {}
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::cell::{simlib::copy_bits, test_helpers::*};
  use rstest::rstest;

  #[rstest]
  #[case("0", "0")]
  #[case("1", "1")]
  #[case("0000", "0000")]
  #[case("1111", "1111")]
  #[case("1101", "1101")]
  #[case("1010", "1010")]
  #[case("0110", "0110")]
  #[case("1000", "1000")]
  #[case("0001", "0001")]
  #[case("0010110", "0010110")]
  #[case("11011110", "11011110")]
  fn buf(#[case] a: BitVec, #[case] expected: BitVec) {
    run_unary_cell_case!(ProcBuffer, a, expected);
  }

  #[rstest]
  #[case(false, "1101", "001101")]
  #[case(false, "1010", "001010")]
  #[case(false, "0110", "000110")]
  #[case(false, "11011110", "011110")]
  #[case(false, "11111101", "111101")]
  #[case(false, "1", "1")]
  #[case(false, "0", "0")]
  #[case(false, "1111", "1111")]
  #[case(false, "0000", "0000")]
  #[case(true, "1101", "111101")]
  #[case(true, "1010", "111010")]
  #[case(true, "0110", "000110")]
  #[case(true, "11011110", "011110")]
  #[case(true, "11111101", "111101")]
  #[case(true, "1", "1")]
  #[case(true, "0", "0")]
  #[case(true, "1111", "1111")]
  #[case(true, "0000", "0000")]
  fn pos(#[case] signed: bool, #[case] a: BitVec, #[case] expected: BitVec) {
    run_unary_cell_case_signed!(Pos, signed, a, expected);
  }

  #[rstest]
  #[case(Bit::ZERO, "000", "001", "000")]
  #[case(Bit::ONE, "000", "001", "001")]
  #[case(Bit::ZERO, "111", "000", "111")]
  #[case(Bit::ONE, "111", "000", "000")]
  #[case(Bit::ZERO, "000", "000", "000")]
  #[case(Bit::ONE, "000", "000", "000")]
  #[case(Bit::ZERO, "111", "111", "111")]
  #[case(Bit::ONE, "111", "111", "111")]
  #[case(Bit::ZERO, "101", "010", "101")]
  #[case(Bit::ONE, "101", "010", "010")]
  #[case(Bit::ZERO, "010", "101", "010")]
  #[case(Bit::ONE, "010", "101", "101")]
  #[case(Bit::ZERO, "100", "001", "100")]
  #[case(Bit::ONE, "100", "001", "001")]
  #[case(Bit::ZERO, "011", "110", "011")]
  #[case(Bit::ONE, "011", "110", "110")]
  #[case(Bit::ZERO, "1", "0", "1")]
  #[case(Bit::ONE, "1", "0", "0")]
  #[case(Bit::ZERO, "0", "1", "0")]
  #[case(Bit::ONE, "0", "1", "1")]
  #[case(Bit::ZERO, "10101010", "01010101", "10101010")]
  #[case(Bit::ONE, "10101010", "01010101", "01010101")]
  #[case(Bit::ZERO, "11110000", "00001111", "11110000")]
  #[case(Bit::ONE, "11110000", "00001111", "00001111")]
  #[case(Bit::ZERO, "10000000", "11111111", "10000000")]
  #[case(Bit::ONE, "10000000", "11111111", "11111111")]
  #[case(Bit::ZERO, "00000001", "11111110", "00000001")]
  #[case(Bit::ONE, "00000001", "11111110", "11111110")]
  fn mux(#[case] select: Bit, #[case] a: BitVec, #[case] b: BitVec, #[case] expected: BitVec) {
    let nets = allocate_nets(Some(NET_OFFSET + 1), &[&a, &b, &expected]);

    let select_net: usize = NET_OFFSET;
    let (a_nets, b_nets, y_nets) = (&nets[0], &nets[1], &nets[2]);
    let mut signals = Signals::new(y_nets.last().unwrap() + 1);
    let mut cell = Mux::new(select_net, a_nets.clone(), b_nets.clone(), y_nets.clone());

    signals.set_net(select_net, select);
    copy_bits(&mut signals, a_nets, &a);
    copy_bits(&mut signals, b_nets, &b);

    cell.eval(&mut signals);

    let actual = BitVec::from_iter(y_nets.iter().map(|&n| signals.get_net(n)));
    assert_eq!(actual, expected);
  }

  #[rstest]
  #[case("0", "101010", "010")]
  #[case("1", "101010", "101")]
  #[case("0", "111000", "000")]
  #[case("1", "111000", "111")]
  #[case("00", "000001010111", "111")]
  #[case("01", "000001010111", "010")]
  #[case("10", "000001010111", "001")]
  #[case("11", "000001010111", "000")]
  #[case("00", "111000101010", "010")]
  #[case("01", "111000101010", "101")]
  #[case("10", "111000101010", "000")]
  #[case("11", "111000101010", "111")]
  #[case("000", "000001010011100111101010", "010")]
  #[case("001", "000001010011100111101010", "101")]
  #[case("010", "000001010011100111101010", "111")]
  #[case("011", "000001010011100111101010", "100")]
  #[case("100", "000001010011100111101010", "011")]
  #[case("101", "000001010011100111101010", "010")]
  #[case("110", "000001010011100111101010", "001")]
  #[case("111", "000001010011100111101010", "000")]
  fn bmux(#[case] select: BitVec, #[case] a: BitVec, #[case] expected: BitVec) {
    let nets = allocate_nets(Some(NET_OFFSET), &[&select, &a, &expected]);

    let (select_nets, a_nets, y_nets) = (&nets[0], &nets[1], &nets[2]);
    let mut signals = Signals::new(y_nets.last().unwrap() + 1);
    let mut cell = BMux::new(select_nets.clone(), a_nets.clone(), y_nets.clone());

    copy_bits(&mut signals, select_nets, &select);
    copy_bits(&mut signals, a_nets, &a);
    cell.eval(&mut signals);

    let actual = BitVec::from_iter(y_nets.iter().map(|&n| signals.get_net(n)));
    assert_eq!(actual, expected);
  }

  #[rstest]
  #[case("00", "000", "111001", "000")]
  #[case("01", "000", "111001", "001")]
  #[case("10", "000", "111001", "111")]
  #[case("11", "000", "111001", "111")]
  fn pmux(#[case] select: BitVec, #[case] a: BitVec, #[case] b: BitVec, #[case] expected: BitVec) {
    let nets = allocate_nets(Some(NET_OFFSET), &[&select, &a, &b, &expected]);

    let select_nets = &nets[0];
    let a_nets = &nets[1];
    let b_nets = &nets[2];
    let y_nets = &nets[3];

    let mut signals = Signals::new(y_nets.last().unwrap() + 1);
    let mut cell = PMux::new(
      select_nets.clone(),
      a_nets.clone(),
      b_nets.clone(),
      y_nets.clone(),
    );

    copy_bits(&mut signals, select_nets, &select);
    copy_bits(&mut signals, a_nets, &a);
    copy_bits(&mut signals, b_nets, &b);

    cell.eval(&mut signals);

    let actual = BitVec::from_iter(y_nets.iter().map(|&n| signals.get_net(n)));
    assert_eq!(actual, expected);
  }
}

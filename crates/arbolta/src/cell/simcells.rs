// Copyright (c) 2026 Alexander Redding
// SPDX-License-Identifier: MIT

use super::CellFn;
use crate::{bit::Bit, cell::CellRegistration, signal::Signals};
use derive_more::Constructor;
use paste::paste;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, env};

macro_rules! create_cell {
  // 1-output, no inputs
  ($rtl_names:expr, $cell_type:ident, $out_net:ident, $body:expr) => {
    #[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
    pub struct $cell_type {
      $out_net: usize
    }

    impl CellFn for $cell_type {
       #[inline]
      fn eval(&mut self, signals: &mut Signals) {
        signals.set_net(self.$out_net, $body());
      }

      fn reset(&mut self) {}
    }

    paste! {
      inventory::submit! {CellRegistration::new($rtl_names,
        |connections: &BTreeMap<&str, Box<[usize]>>, _parameters: &BTreeMap<&str, usize>| {
          if env::var("ARBOLTA_DEBUG").is_ok() {
            println!("Parsing connections: {:#?}", connections);
          }

          $cell_type::new(
            connections[stringify!([<$out_net:upper>])][0]
          ).into()
      })}
    }
  };

  // 1-output, N-inputs
  ($rtl_names:expr, $cell_type:ident { $($in_netn:ident),* $(,)?}, $out_net:ident, $body:expr) => {
    #[allow(clippy::too_many_arguments)]
    #[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
    pub struct $cell_type {
      $(
        $in_netn: usize,
      )*

      $out_net: usize
    }

    impl CellFn for $cell_type {
       #[inline]
      fn eval(&mut self, signals: &mut Signals) {
        $(
          let $in_netn: Bit = signals.get_net(self.$in_netn);
        )*
        signals.set_net(self.$out_net, $body($($in_netn,)*));
      }

      fn reset(&mut self) {}
    }

    paste! {
      inventory::submit! {CellRegistration::new($rtl_names,
        |connections: &BTreeMap<&str, Box<[usize]>>, _parameters: &BTreeMap<&str, usize>| {
          if env::var("ARBOLTA_DEBUG").is_ok() {
            println!("Parsing connections: {:#?}", connections);
          }

          // Special case for buffers
          let output_net = match connections.get(stringify!([<$out_net:upper>])) {
            Some(nets) => nets[0],
            None => 0 // Write to zero does nothing
          };

          $cell_type::new(
            $(
              connections[stringify!([<$in_netn:upper>])][0],
            )*
            output_net
          ).into()
      })}
    }
  };

  // N-output, N-inputs
  ($rtl_names:expr, $cell_type:ident { $($in_netn:ident),* $(,)?}, { $($out_netn:ident),+ $(,)?}, $body:expr) => {
    #[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
    pub struct $cell_type {
      $(
        $in_netn: usize,
      )*

      $(
        $out_netn: usize,
      )*
    }

    impl CellFn for $cell_type {
       #[inline]
      fn eval(&mut self, signals: &mut Signals) {
        $(
          let $in_netn: Bit = signals.get_net(self.$in_netn);
        )*

        let ( $($out_netn),+ ) = $body($($in_netn,)*);

        $(
          signals.set_net(self.$out_netn, $out_netn);
        )+
      }

      fn reset(&mut self) {}
    }

    paste! {
      inventory::submit! {CellRegistration::new($rtl_names,
        |connections: &BTreeMap<&str, Box<[usize]>>, _parameters: &BTreeMap<&str, usize>| {
          if env::var("ARBOLTA_DEBUG").is_ok() {
            println!("Parsing connections: {:#?}", connections);
          }

          $cell_type::new(
            $(
              connections[stringify!([<$in_netn:upper>])][0],
            )*
            $(
              connections[stringify!([<$out_netn:upper>])][0],
            )*
          ).into()
      })}
    }
  };
}

/* ++++++++ Sim Cells ++++++++ */
// Unary
create_cell!(&["$_BUF_"], Buffer { a }, y, |a: Bit| a);
create_cell!(&["$_NOT_"], Inverter { a }, y, |a: Bit| !a);

// Binary
create_cell!(&["$_AND_"], And2 { a, b }, y, |a: Bit, b: Bit| a & b);
create_cell!(&["$_ANDNOT_"], AndNot2 { a, b }, y, |a: Bit, b: Bit| a & !b);
create_cell!(&["$_NAND_"], Nand2 { a, b }, y, |a: Bit, b: Bit| !(a & b));
create_cell!(&["$_NOR_"], Nor2 { a, b }, y, |a: Bit, b: Bit| !(a | b));
create_cell!(&["$_OR_"], Or2 { a, b }, y, |a: Bit, b: Bit| a | b);
create_cell!(&["$_ORNOT_"], OrNot2 { a, b }, y, |a: Bit, b: Bit| a | !b);
create_cell!(&["$_XNOR_"], Xnor2 { a, b }, y, |a: Bit, b: Bit| !(a ^ b));
create_cell!(&["$_XOR_"], Xor2 { a, b }, y, |a: Bit, b: Bit| a ^ b);

// Ternary
create_cell!(
  &["$_AOI3_"],
  AndOrInvert3 { a, b, c },
  y,
  |a: Bit, b: Bit, c: Bit| { !((a & b) | c) }
);
create_cell!(
  &["$_MUX_"],
  Mux2 { a, b, s },
  y,
  |a: Bit, b: Bit, select: Bit| if select.into() { b } else { a }
);
create_cell!(
  &["$_NMUX_"],
  NMux2 { a, b, s },
  y,
  |a: Bit, b: Bit, select: Bit| if select.into() { !b } else { !a }
);
create_cell!(
  &["$_OAI3_"],
  OrAndInvert3 { a, b, c },
  y,
  |a: Bit, b: Bit, c: Bit| { !((a | b) & c) }
);

// Memory
#[derive(Debug, Clone, Serialize, Deserialize, derive_new::new)]
pub struct Dff {
  polarity: Bit,
  clock_net: usize,
  data_in_net: usize,
  data_out_net: usize,
  #[new(default)]
  last_clock: Bit,
}

impl CellFn for Dff {
  #[inline]
  fn eval(&mut self, signals: &mut Signals) {
    // Check if clock active for any polarity
    let clock = !(signals.get_net(self.clock_net) ^ self.polarity);

    // Rising edge
    if clock == Bit::ONE && self.last_clock == Bit::ZERO {
      let data_in = signals.get_net(self.data_in_net);
      signals.set_net(self.data_out_net, data_in);
    }

    self.last_clock = clock;
  }

  fn reset(&mut self) {
    self.last_clock = Bit::ZERO;
  }
}

inventory::submit! {
  CellRegistration::new(&["$_DFF_P_"],
  |connections: &BTreeMap<&str, Box<[usize]>>, _parameters: &BTreeMap<&str, usize>| {
    Dff::new(
      Bit::ONE,
      connections["C"][0],
      connections["D"][0],
      connections["Q"][0],
    ).into()
  })
}

#[derive(Debug, Clone, Serialize, Deserialize, derive_new::new)]
pub struct DffReset {
  clock_polarity: Bit,
  reset_polarity: Bit,
  reset_val: Bit,
  clock_net: usize,
  reset_net: usize,
  data_in_net: usize,
  data_out_net: usize,
  #[new(default)]
  last_clock: Bit,
}

impl CellFn for DffReset {
  #[inline]
  fn eval(&mut self, signals: &mut Signals) {
    // Check if clock active for any polarity
    let clock = !(signals.get_net(self.clock_net) ^ self.clock_polarity);

    // Rising edge
    if clock == Bit::ONE && self.last_clock == Bit::ZERO {
      // Check if reset active for any polarity
      if signals.get_net(self.reset_net) == self.reset_polarity {
        signals.set_net(self.data_out_net, self.reset_val);
      } else {
        let data_in = signals.get_net(self.data_in_net);
        signals.set_net(self.data_out_net, data_in);
      }
    }

    self.last_clock = clock;
  }

  fn reset(&mut self) {
    self.last_clock = Bit::ZERO;
  }
}

inventory::submit! {
  CellRegistration::new(&["$_SDFF_PP0_"],
  |connections: &BTreeMap<&str, Box<[usize]>>, _parameters: &BTreeMap<&str, usize>| {
    DffReset::new(
      Bit::ONE,
      Bit::ONE,
      Bit::ZERO,
      connections["C"][0],
      connections["R"][0],
      connections["D"][0],
      connections["Q"][0],
    ).into()
  })
}

/* ++++++++ ASAP7 Cells ++++++++ */
create_cell!(
  &[
    "BUFx10_ASAP7_75t_R",
    "BUFx12_ASAP7_75t_R",
    "BUFx12f_ASAP7_75t_R",
    "BUFx16f_ASAP7_75t_R",
    "BUFx24_ASAP7_75t_R",
    "BUFx2_ASAP7_75t_R",
    "BUFx3_ASAP7_75t_R",
    "BUFx4_ASAP7_75t_R",
    "BUFx4f_ASAP7_75t_R",
    "BUFx5_ASAP7_75t_R",
    "BUFx6f_ASAP7_75t_R",
    "BUFx8_ASAP7_75t_R",
    "HB1xp67_ASAP7_75t_R",
    "HB2xp67_ASAP7_75t_R",
    "HB3xp67_ASAP7_75t_R",
    "HB4xp67_ASAP7_75t_R",
  ],
  Asap7Buf { a },
  y,
  |a: Bit| a
);
create_cell!(
  &[
    "CKINVDCx10_ASAP7_75t_R",
    "CKINVDCx11_ASAP7_75t_R",
    "CKINVDCx12_ASAP7_75t_R",
    "CKINVDCx14_ASAP7_75t_R",
    "CKINVDCx16_ASAP7_75t_R",
    "CKINVDCx20_ASAP7_75t_R",
    "CKINVDCx5p33_ASAP7_75t_R",
    "CKINVDCx6p67_ASAP7_75t_R",
    "CKINVDCx8_ASAP7_75t_R",
    "CKINVDCx9p33_ASAP7_75t_R",
    "INVx11_ASAP7_75t_R",
    "INVx13_ASAP7_75t_R",
    "INVx1_ASAP7_75t_R",
    "INVx2_ASAP7_75t_R",
    "INVx3_ASAP7_75t_R",
    "INVx4_ASAP7_75t_R",
    "INVx5_ASAP7_75t_R",
    "INVx6_ASAP7_75t_R",
    "INVx8_ASAP7_75t_R",
    "INVxp33_ASAP7_75t_R",
    "INVxp67_ASAP7_75t_R",
  ],
  Asap7Inv { a },
  y,
  |a: Bit| !a
);
create_cell!(
  &[
    "AND2x2_ASAP7_75t_R",
    "AND2x4_ASAP7_75t_R",
    "AND2x6_ASAP7_75t_R",
  ],
  Asap7And2 { a, b },
  y,
  |a: Bit, b: Bit| a & b
);
create_cell!(
  &[
    "AND3x1_ASAP7_75t_R",
    "AND3x2_ASAP7_75t_R",
    "AND3x4_ASAP7_75t_R",
  ],
  Asap7And3 { a, b, c },
  y,
  |a: Bit, b: Bit, c: Bit| a & b & c
);
create_cell!(
  &["AND4x1_ASAP7_75t_R", "AND4x2_ASAP7_75t_R",],
  Asap7And4 { a, b, c, d },
  y,
  |a: Bit, b: Bit, c: Bit, d: Bit| a & b & c & d
);
create_cell!(
  &["AND5x1_ASAP7_75t_R", "AND5x2_ASAP7_75t_R",],
  Asap7And5 { a, b, c, d, e },
  y,
  |a: Bit, b: Bit, c: Bit, d: Bit, e: Bit| a & b & c & d & e
);
create_cell!(
  &["FAx1_ASAP7_75t_R"],
  Asap7FullAdderInv {a, b, ci}, {sn, con},
  |a: Bit, b: Bit, carry_in: Bit| {
    let sum = a ^ b;
    (!(sum ^ carry_in), !((a & b) | (sum & carry_in)))
  }
);
create_cell!(
  &["HAxp5_ASAP7_75t_R"],
  Asap7HalfAdderInv { a, b }, {sn, con}, |a: Bit, b: Bit| (
  !(a ^ b),
  !(a & b)
));
create_cell!(
  &["MAJIxp5_ASAP7_75t_R"],
  Asap7MajorityInv { a, b, c },
  y,
  |a: Bit, b: Bit, c: Bit| (!a & !b) | (!a & !c) | (!b & !c)
);
create_cell!(
  &["MAJx2_ASAP7_75t_R", "MAJx3_ASAP7_75t_R"],
  Asap7Majority { a, b, c },
  y,
  |a: Bit, b: Bit, c: Bit| (a & b) | (a & c) | (b & c)
);
create_cell!(
  &[
    "NAND2x1_ASAP7_75t_R",
    "NAND2x1p5_ASAP7_75t_R",
    "NAND2x2_ASAP7_75t_R",
    "NAND2xp33_ASAP7_75t_R",
    "NAND2xp5_ASAP7_75t_R",
    "NAND2xp67_ASAP7_75t_R",
  ],
  Asap7Nand2 { a, b },
  y,
  |a: Bit, b: Bit| !(a & b)
);
create_cell!(
  &[
    "NAND3x1_ASAP7_75t_R",
    "NAND3x2_ASAP7_75t_R",
    "NAND3xp33_ASAP7_75t_R",
  ],
  Asap7Nand3 { a, b, c },
  y,
  |a: Bit, b: Bit, c: Bit| !(a & b & c)
);
create_cell!(
  &["NAND4xp25_ASAP7_75t_R", "NAND4xp75_ASAP7_75t_R",],
  Asap7Nand4 { a, b, c, d },
  y,
  |a: Bit, b: Bit, c: Bit, d: Bit| !(a & b & c & d)
);
create_cell!(
  &["NAND5xp2_ASAP7_75t_R",],
  Asap7Nand5 { a, b, c, d, e },
  y,
  |a: Bit, b: Bit, c: Bit, d: Bit, e: Bit| !(a & b & c & d & e)
);
create_cell!(
  &[
    "NOR2x1_ASAP7_75t_R",
    "NOR2x1p5_ASAP7_75t_R",
    "NOR2x2_ASAP7_75t_R",
    "NOR2xp33_ASAP7_75t_R",
    "NOR2xp67_ASAP7_75t_R",
  ],
  Asap7Nor2 { a, b },
  y,
  |a: Bit, b: Bit| !(a | b)
);
create_cell!(
  &[
    "NOR3x1_ASAP7_75t_R",
    "NOR3x2_ASAP7_75t_R",
    "NOR3xp33_ASAP7_75t_R",
  ],
  Asap7Nor3 { a, b, c },
  y,
  |a: Bit, b: Bit, c: Bit| !(a | b | c)
);
create_cell!(
  &["NOR4xp25_ASAP7_75t_R", "NOR4xp75_ASAP7_75t_R"],
  Asap7Nor4 { a, b, c, d },
  y,
  |a: Bit, b: Bit, c: Bit, d: Bit| !(a | b | c | d)
);
create_cell!(
  &["NOR5xp2_ASAP7_75t_R"],
  Asap7Nor5 { a, b, c, d, e },
  y,
  |a: Bit, b: Bit, c: Bit, d: Bit, e: Bit| !(a | b | c | d | e)
);
create_cell!(
  &[
    "OR2x2_ASAP7_75t_R",
    "OR2x4_ASAP7_75t_R",
    "OR2x6_ASAP7_75t_R",
  ],
  Asap7Or2 { a, b },
  y,
  |a: Bit, b: Bit| a | b
);
create_cell!(
  &[
    "OR3x1_ASAP7_75t_R",
    "OR3x2_ASAP7_75t_R",
    "OR3x4_ASAP7_75t_R",
  ],
  Asap7Or3 { a, b, c },
  y,
  |a: Bit, b: Bit, c: Bit| a | b | c
);
create_cell!(
  &["OR4x1_ASAP7_75t_R", "OR4x2_ASAP7_75t_R"],
  Asap7Or4 { a, b, c, d },
  y,
  |a: Bit, b: Bit, c: Bit, d: Bit| a | b | c | d
);
create_cell!(
  &["OR5x1_ASAP7_75t_R", "OR5x2_ASAP7_75t_R"],
  Asap7Or5 { a, b, c, d, e },
  y,
  |a: Bit, b: Bit, c: Bit, d: Bit, e: Bit| a | b | c | d | e
);
create_cell!(&["TIEHIx1_ASAP7_75t_R"], Asap7TieHigh, h, || Bit::ONE);
create_cell!(&["TIELOx1_ASAP7_75t_R"], Asap7TieLow, l, || Bit::ZERO);
create_cell!(
  &[
    "XNOR2x1_ASAP7_75t_R",
    "XNOR2x2_ASAP7_75t_R",
    "XNOR2xp5_ASAP7_75t_R",
  ],
  Asap7Xnor2 { a, b },
  y,
  |a: Bit, b: Bit| !(a ^ b)
);
create_cell!(
  &[
    "XOR2x1_ASAP7_75t_R",
    "XOR2x2_ASAP7_75t_R",
    "XOR2xp5_ASAP7_75t_R",
  ],
  Asap7Xor2 { a, b },
  y,
  |a: Bit, b: Bit| a ^ b
);
create_cell!(
  &["O2A1O1Ixp33_ASAP7_75t_R", "O2A1O1Ixp5_ASAP7_75t_R"],
  Asap7Or2And1Or1Inv { a1, a2, b, c },
  y,
  |a1: Bit, a2: Bit, b: Bit, c: Bit| (!a1 & !a2 & !c) | (!b & !c)
);
create_cell!(
  &["OA211x2_ASAP7_75t_R"],
  Asap7OrAnd211 { a1, a2, b, c },
  y,
  |a1: Bit, a2: Bit, b: Bit, c: Bit| (a1 & b & c) | (a2 & b & c)
);
create_cell!(
  &["OA21x2_ASAP7_75t_R"],
  Asap7OrAnd21 { a1, a2, b },
  y,
  |a1: Bit, a2: Bit, b: Bit| (a1 & b) | (a2 & b)
);
create_cell!(
  &["OA221x2_ASAP7_75t_R"],
  Asap7OrAnd221 { a1, a2, b1, b2, c },
  y,
  |a1: Bit, a2: Bit, b1: Bit, b2: Bit, c: Bit| (a2 & b2 & c)
    | (a2 & b1 & c)
    | (a1 & b2 & c)
    | (a1 & b1 & c)
);
create_cell!(
  &["OA222x2_ASAP7_75t_R"],
  Asap7OrAnd222 {
    a1,
    a2,
    b1,
    b2,
    c1,
    c2
  },
  y,
  |a1: Bit, a2: Bit, b1: Bit, b2: Bit, c1: Bit, c2: Bit| (a2 & b2 & c2)
    | (a2 & b2 & c1)
    | (a2 & b1 & c2)
    | (a2 & b1 & c1)
    | (a1 & b2 & c2)
    | (a1 & b2 & c1)
    | (a1 & b1 & c2)
    | (a1 & b1 & c1)
);
create_cell!(
  &["OA22x2_ASAP7_75t_R"],
  Asap7OrAnd22 { a1, a2, b1, b2 },
  y,
  |a1: Bit, a2: Bit, b1: Bit, b2: Bit| (a2 & b2) | (a2 & b1) | (a1 & b2) | (a1 & b1)
);
create_cell!(
  &["OA31x2_ASAP7_75t_R"],
  Asap7OrAnd31 { a1, a2, a3, b1 },
  y,
  |a1: Bit, a2: Bit, a3: Bit, b1: Bit| (a3 & b1) | (a2 & b1) | (a1 & b1)
);
create_cell!(
  &["OA331x1_ASAP7_75t_R", "OA331x2_ASAP7_75t_R"],
  Asap7OrAnd331 {
    a1,
    a2,
    a3,
    b1,
    b2,
    b3,
    c1,
  },
  y,
  |a1: Bit, a2: Bit, a3: Bit, b1: Bit, b2: Bit, b3: Bit, c1: Bit| (a3 & b3 & c1)
    | (a3 & b2 & c1)
    | (a3 & b1 & c1)
    | (a2 & b3 & c1)
    | (a2 & b2 & c1)
    | (a2 & b1 & c1)
    | (a1 & b3 & c1)
    | (a1 & b2 & c1)
    | (a1 & b1 & c1)
);
create_cell!(
  &["OA332x1_ASAP7_75t_R", "OA332x2_ASAP7_75t_R",],
  Asap7OrAnd332 {
    a1,
    a2,
    a3,
    b1,
    b2,
    b3,
    c1,
    c2,
  },
  y,
  |a1: Bit, a2: Bit, a3: Bit, b1: Bit, b2: Bit, b3: Bit, c1: Bit, c2: Bit| (a3 & b3 & c2)
    | (a3 & b3 & c1)
    | (a3 & b2 & c2)
    | (a3 & b2 & c1)
    | (a3 & b1 & c2)
    | (a3 & b1 & c1)
    | (a2 & b3 & c2)
    | (a2 & b3 & c1)
    | (a2 & b2 & c2)
    | (a2 & b2 & c1)
    | (a2 & b1 & c2)
    | (a2 & b1 & c1)
    | (a1 & b3 & c2)
    | (a1 & b3 & c1)
    | (a1 & b2 & c2)
    | (a1 & b2 & c1)
    | (a1 & b1 & c2)
    | (a1 & b1 & c1)
);
create_cell!(
  &["OA333x1_ASAP7_75t_R", "OA333x2_ASAP7_75t_R",],
  Asap7OrAnd333 {
    a1,
    a2,
    a3,
    b1,
    b2,
    b3,
    c1,
    c2,
    c3,
  },
  y,
  |a1: Bit, a2: Bit, a3: Bit, b1: Bit, b2: Bit, b3: Bit, c1: Bit, c2: Bit, c3: Bit| (a3 & b3 & c3)
    | (a3 & b3 & c2)
    | (a3 & b3 & c1)
    | (a3 & b2 & c3)
    | (a3 & b2 & c2)
    | (a3 & b2 & c1)
    | (a3 & b1 & c3)
    | (a3 & b1 & c2)
    | (a3 & b1 & c1)
    | (a2 & b3 & c3)
    | (a2 & b3 & c2)
    | (a2 & b3 & c1)
    | (a2 & b2 & c3)
    | (a2 & b2 & c2)
    | (a2 & b2 & c1)
    | (a2 & b1 & c3)
    | (a2 & b1 & c2)
    | (a2 & b1 & c1)
    | (a1 & b3 & c3)
    | (a1 & b3 & c2)
    | (a1 & b3 & c1)
    | (a1 & b2 & c3)
    | (a1 & b2 & c2)
    | (a1 & b2 & c1)
    | (a1 & b1 & c3)
    | (a1 & b1 & c2)
    | (a1 & b1 & c1)
);
create_cell!(
  &["OA33x2_ASAP7_75t_R"],
  Asap7OrAnd33 {
    a1,
    a2,
    a3,
    b1,
    b2,
    b3
  },
  y,
  |a1: Bit, a2: Bit, a3: Bit, b1: Bit, b2: Bit, b3: Bit| (a3 & b3)
    | (a3 & b2)
    | (a3 & b1)
    | (a2 & b3)
    | (a2 & b2)
    | (a2 & b1)
    | (a1 & b3)
    | (a1 & b2)
    | (a1 & b1)
);
create_cell!(
  &["OAI211xp5_ASAP7_75t_R"],
  Asap7OrAndInv211 { a1, a2, b, c },
  y,
  |a1: Bit, a2: Bit, b: Bit, c: Bit| (!a1 & !a2) | !b | !c
);
create_cell!(
  &[
    "OAI21x1_ASAP7_75t_R",
    "OAI21xp33_ASAP7_75t_R",
    "OAI21xp5_ASAP7_75t_R",
  ],
  Asap7OrAndInv21 { a1, a2, b },
  y,
  |a1: Bit, a2: Bit, b: Bit| (!a1 & !a2) | !b
);
create_cell!(
  &["OAI221xp5_ASAP7_75t_R"],
  Asap7OrAndInv221 { a1, a2, b1, b2, c },
  y,
  |a1: Bit, a2: Bit, b1: Bit, b2: Bit, c: Bit| { (!a1 & !a2) | (!b1 & !b2) | !c }
);
create_cell!(
  &["OAI222xp33_ASAP7_75t_R"],
  Asap7OrAndInv222 {
    a1,
    a2,
    b1,
    b2,
    c1,
    c2
  },
  y,
  |a1: Bit, a2: Bit, b1: Bit, b2: Bit, c1: Bit, c2: Bit| (!a1 & !a2) | (!b1 & !b2) | (!c1 & !c2)
);
create_cell!(
  &[
    "OAI22x1_ASAP7_75t_R",
    "OAI22xp33_ASAP7_75t_R",
    "OAI22xp5_ASAP7_75t_R"
  ],
  Asap7OrAndInv22 { a1, a2, b1, b2 },
  y,
  |a1: Bit, a2: Bit, b1: Bit, b2: Bit| (!a1 & !a2) | (!b1 & !b2)
);
create_cell!(
  &["OAI311xp33_ASAP7_75t_R"],
  Asap7OrAndInv311 { a1, a2, a3, b1, c1 },
  y,
  |a1: Bit, a2: Bit, a3: Bit, b1: Bit, c1: Bit| (!a1 & !a2 & !a3) | !b1 | !c1
);
create_cell!(
  &["OAI31xp33_ASAP7_75t_R", "OAI31xp67_ASAP7_75t_R"],
  Asap7OrAndInv31 { a1, a2, a3, b },
  y,
  |a1: Bit, a2: Bit, a3: Bit, b: Bit| (!a1 & !a2 & !a3) | !b
);
create_cell!(
  &["OAI321xp33_ASAP7_75t_R"],
  Asap7OrAndInv321 {
    a1,
    a2,
    a3,
    b1,
    b2,
    c
  },
  y,
  |a1: Bit, a2: Bit, a3: Bit, b1: Bit, b2: Bit, c: Bit| (!a1 & !a2 & !a3) | (!b1 & !b2) | !c
);
create_cell!(
  &["OAI322xp33_ASAP7_75t_R"],
  Asap7OrAndInv322 {
    a1,
    a2,
    a3,
    b1,
    b2,
    c1,
    c2
  },
  y,
  |a1: Bit, a2: Bit, a3: Bit, b1: Bit, b2: Bit, c1: Bit, c2: Bit| (!a1 & !a2 & !a3)
    | (!b1 & !b2)
    | (!c1 & !c2)
);
create_cell!(
  &["OAI32xp33_ASAP7_75t_R"],
  Asap7OrAndInv32 { a1, a2, a3, b1, b2 },
  y,
  |a1: Bit, a2: Bit, a3: Bit, b1: Bit, b2: Bit| (!a1 & !a2 & !a3) | (!b1 & !b2)
);
create_cell!(
  &["OAI331xp33_ASAP7_75t_R"],
  Asap7OrAndInv331 {
    a1,
    a2,
    a3,
    b1,
    b2,
    b3,
    c1
  },
  y,
  |a1: Bit, a2: Bit, a3: Bit, b1: Bit, b2: Bit, b3: Bit, c1: Bit| (!a1 & !a2 & !a3)
    | (!b1 & !b2 & !b3)
    | !c1
);
create_cell!(
  &["OAI332xp33_ASAP7_75t_R"],
  Asap7OrAndInv332 {
    a1,
    a2,
    a3,
    b1,
    b2,
    b3,
    c1,
    c2
  },
  y,
  |a1: Bit, a2: Bit, a3: Bit, b1: Bit, b2: Bit, b3: Bit, c1: Bit, c2: Bit| (!a1 & !a2 & !a3)
    | (!b1 & !b2 & !b3)
    | (!c1 & !c2)
);
create_cell!(
  &["OAI333xp33_ASAP7_75t_R"],
  Asap7OrAndInv333 {
    a1,
    a2,
    a3,
    b1,
    b2,
    b3,
    c1,
    c2,
    c3
  },
  y,
  |a1: Bit, a2: Bit, a3: Bit, b1: Bit, b2: Bit, b3: Bit, c1: Bit, c2: Bit, c3: Bit| (!a1
    & !a2
    & !a3)
    | (!b1 & !b2 & !b3)
    | (!c1 & !c2 & !c3)
);
create_cell!(
  &["OAI33xp33_ASAP7_75t_R"],
  Asap7OrAndInv33 {
    a1,
    a2,
    a3,
    b1,
    b2,
    b3
  },
  y,
  |a1: Bit, a2: Bit, a3: Bit, b1: Bit, b2: Bit, b3: Bit| (!a1 & !a2 & !a3) | (!b1 & !b2 & !b3)
);
create_cell!(
  &["A2O1A1Ixp33_ASAP7_75t_R"],
  Asap7And2Or1And1Inv { a1, a2, b, c },
  y,
  |a1: Bit, a2: Bit, b: Bit, c: Bit| (!a1 & !b) | (!a2 & !b) | !c
);
create_cell!(
  &["A2O1A1O1Ixp25_ASAP7_75t_R"],
  Asap7And2Or1And1Or1Inv { a1, a2, b, c, d },
  y,
  |a1: Bit, a2: Bit, b: Bit, c: Bit, d: Bit| (!a1 & !b & !d) | (!a2 & !b & !d) | (!c & !d)
);
create_cell!(
  &["AO211x2_ASAP7_75t_R"],
  Asap7AndOr211 { a1, a2, b, c },
  y,
  |a1: Bit, a2: Bit, b: Bit, c: Bit| (a1 & a2) | b | c
);
create_cell!(
  &["AO21x1_ASAP7_75t_R", "AO21x2_ASAP7_75t_R",],
  Asap7AndOr21 { a1, a2, b },
  y,
  |a1: Bit, a2: Bit, b: Bit| (a1 & a2) | b
);
create_cell!(
  &["AO221x1_ASAP7_75t_R", "AO221x2_ASAP7_75t_R",],
  Asap7AndOr221 { a1, a2, b1, b2, c },
  y,
  |a1: Bit, a2: Bit, b1: Bit, b2: Bit, c: Bit| (a1 & a2) | (b1 & b2) | c
);
create_cell!(
  &["AO222x2_ASAP7_75t_R",],
  Asap7AndOr222 {
    a1,
    a2,
    b1,
    b2,
    c1,
    c2
  },
  y,
  |a1: Bit, a2: Bit, b1: Bit, b2: Bit, c1: Bit, c2: Bit| (a1 & a2) | (b1 & b2) | (c1 & c2)
);
create_cell!(
  &["AO22x1_ASAP7_75t_R", "AO22x2_ASAP7_75t_R",],
  Asap7AndOr22 { a1, a2, b1, b2 },
  y,
  |a1: Bit, a2: Bit, b1: Bit, b2: Bit| (a1 & a2) | (b1 & b2)
);

create_cell!(
  &["AO31x2_ASAP7_75t_R"],
  Asap7AndOr31 { a1, a2, a3, b },
  y,
  |a1, a2, a3, b| (a1 & a2 & a3) | b
);
create_cell!(
  &["AO322x2_ASAP7_75t_R"],
  Asap7AndOr322 {
    a1,
    a2,
    a3,
    b1,
    b2,
    c1,
    c2
  },
  y,
  |a1, a2, a3, b1, b2, c1, c2| (a1 & a2 & a3) | (b1 & b2) | (c1 & c2)
);
create_cell!(
  &["AO32x1_ASAP7_75t_R", "AO32x2_ASAP7_75t_R"],
  Asap7AndOr32 { a1, a2, a3, b1, b2 },
  y,
  |a1, a2, a3, b1, b2| (a1 & a2 & a3) | (b1 & b2)
);
create_cell!(
  &["AO331x1_ASAP7_75t_R", "AO331x2_ASAP7_75t_R"],
  Asap7AndOr331 {
    a1,
    a2,
    a3,
    b1,
    b2,
    b3,
    c
  },
  y,
  |a1, a2, a3, b1, b2, b3, c| (a1 & a2 & a3) | (b1 & b2 & b3) | c
);
create_cell!(
  &["AO332x1_ASAP7_75t_R", "AO332x2_ASAP7_75t_R"],
  Asap7AndOr332 {
    a1,
    a2,
    a3,
    b1,
    b2,
    b3,
    c1,
    c2
  },
  y,
  |a1, a2, a3, b1, b2, b3, c1, c2| (a1 & a2 & a3) | (b1 & b2 & b3) | (c1 & c2)
);
create_cell!(
  &["AO333x1_ASAP7_75t_R", "AO333x2_ASAP7_75t_R"],
  Asap7AndOr333 {
    a1,
    a2,
    a3,
    b1,
    b2,
    b3,
    c1,
    c2,
    c3
  },
  y,
  |a1, a2, a3, b1, b2, b3, c1, c2, c3| (a1 & a2 & a3) | (b1 & b2 & b3) | (c1 & c2 & c3)
);
create_cell!(
  &["AO33x2_ASAP7_75t_R"],
  Asap7AndOr33 {
    a1,
    a2,
    a3,
    b1,
    b2,
    b3
  },
  y,
  |a1, a2, a3, b1, b2, b3| (a1 & a2 & a3) | (b1 & b2 & b3)
);
create_cell!(
  &["AOI211x1_ASAP7_75t_R", "AOI211xp5_ASAP7_75t_R"],
  Asap7AndOrInv211 { a1, a2, b, c },
  y,
  |a1: Bit, a2: Bit, b: Bit, c: Bit| (!a1 & !b & !c) | (!a2 & !b & !c)
);
create_cell!(
  &[
    "AOI21x1_ASAP7_75t_R",
    "AOI21xp33_ASAP7_75t_R",
    "AOI21xp5_ASAP7_75t_R"
  ],
  Asap7AndOrInv21 { a1, a2, b },
  y,
  |a1: Bit, a2: Bit, b: Bit| (!a1 & !b) | (!a2 & !b)
);
create_cell!(
  &["AOI221x1_ASAP7_75t_R", "AOI221xp5_ASAP7_75t_R"],
  Asap7AndOrInv221 { a1, a2, b1, b2, c },
  y,
  |a1: Bit, a2: Bit, b1: Bit, b2: Bit, c: Bit| (!a1 & !b1 & !c)
    | (!a1 & !b2 & !c)
    | (!a2 & !b1 & !c)
    | (!a2 & !b2 & !c)
);
create_cell!(
  &["AOI222xp33_ASAP7_75t_R"],
  Asap7AndOrInv222 {
    a1,
    a2,
    b1,
    b2,
    c1,
    c2
  },
  y,
  |a1: Bit, a2: Bit, b1: Bit, b2: Bit, c1: Bit, c2: Bit| (!a1 & !b1 & !c1)
    | (!a1 & !b1 & !c2)
    | (!a1 & !b2 & !c1)
    | (!a1 & !b2 & !c2)
    | (!a2 & !b1 & !c1)
    | (!a2 & !b1 & !c2)
    | (!a2 & !b2 & !c1)
    | (!a2 & !b2 & !c2)
);
create_cell!(
  &[
    "AOI22x1_ASAP7_75t_R",
    "AOI22xp33_ASAP7_75t_R",
    "AOI22xp5_ASAP7_75t_R"
  ],
  Asap7AndOrInv22 { a1, a2, b1, b2 },
  y,
  |a1: Bit, a2: Bit, b1: Bit, b2: Bit| (!a1 & !b1) | (!a1 & !b2) | (!a2 & !b1) | (!a2 & !b2)
);
create_cell!(
  &["AOI311xp33_ASAP7_75t_R"],
  Asap7AndOrInv311 { a1, a2, a3, b, c },
  y,
  |a1: Bit, a2: Bit, a3: Bit, b: Bit, c: Bit| (!a1 & !b & !c) | (!a2 & !b & !c) | (!a3 & !b & !c)
);
create_cell!(
  &["AOI31xp33_ASAP7_75t_R", "AOI31xp67_ASAP7_75t_R"],
  Asap7AndOrInv31 { a1, a2, a3, b },
  y,
  |a1: Bit, a2: Bit, a3: Bit, b: Bit| (!a1 & !b) | (!a2 & !b) | (!a3 & !b)
);
create_cell!(
  &["AOI321xp33_ASAP7_75t_R"],
  Asap7AndOrInv321 {
    a1,
    a2,
    a3,
    b1,
    b2,
    c
  },
  y,
  |a1: Bit, a2: Bit, a3: Bit, b1: Bit, b2: Bit, c: Bit| (a1 & b1 & c)
    | (a1 & b2 & c)
    | (a2 & b1 & c)
    | (a2 & b2 & c)
    | (a3 & b1 & c)
    | (a3 & b2 & c)
);
create_cell!(
  &["AOI322xp5_ASAP7_75t_R"],
  Asap7AndOrInv322 {
    a1,
    a2,
    a3,
    b1,
    b2,
    c1,
    c2
  },
  y,
  |a1: Bit, a2: Bit, a3: Bit, b1: Bit, b2: Bit, c1: Bit, c2: Bit| (!a1 & !b1 & !c1)
    | (!a1 & !b1 & !c2)
    | (!a1 & !b2 & !c1)
    | (!a1 & !b2 & !c2)
    | (!a2 & !b1 & !c1)
    | (!a2 & !b1 & !c2)
    | (!a2 & !b2 & !c1)
    | (!a2 & !b2 & !c2)
    | (!a3 & !b1 & !c1)
    | (!a3 & !b1 & !c2)
    | (!a3 & !b2 & !c1)
    | (!a3 & !b2 & !c2)
);
create_cell!(
  &["AOI32xp33_ASAP7_75t_R"],
  Asap7AndOrInv32 { a1, a2, a3, b1, b2 },
  y,
  |a1: Bit, a2: Bit, a3: Bit, b1: Bit, b2: Bit| (!a1 & !b1)
    | (!a1 & !b2)
    | (!a2 & !b1)
    | (!a2 & !b2)
    | (!a3 & !b1)
    | (!a3 & !b2)
);
create_cell!(
  &["AOI331xp33_ASAP7_75t_R"],
  Asap7AndOrInv331 {
    a1,
    a2,
    a3,
    b1,
    b2,
    b3,
    c1
  },
  y,
  |a1: Bit, a2: Bit, a3: Bit, b1: Bit, b2: Bit, b3: Bit, c1: Bit| (!a1 & !b1 & !c1)
    | (!a1 & !b2 & !c1)
    | (!a1 & !b3 & !c1)
    | (!a2 & !b1 & !c1)
    | (!a2 & !b2 & !c1)
    | (!a2 & !b3 & !c1)
    | (!a3 & !b1 & !c1)
    | (!a3 & !b2 & !c1)
    | (!a3 & !b3 & !c1)
);
create_cell!(
  &["AOI332xp33_ASAP7_75t_R"],
  Asap7AndOrInv332 {
    a1,
    a2,
    a3,
    b1,
    b2,
    b3,
    c1,
    c2
  },
  y,
  |a1: Bit, a2: Bit, a3: Bit, b1: Bit, b2: Bit, b3: Bit, c1: Bit, c2: Bit| (!a1 & !b1 & !c1)
    | (!a1 & !b1 & !c2)
    | (!a1 & !b2 & !c1)
    | (!a1 & !b2 & !c2)
    | (!a1 & !b3 & !c1)
    | (!a1 & !b3 & !c2)
    | (!a2 & !b1 & !c1)
    | (!a2 & !b1 & !c2)
    | (!a2 & !b2 & !c1)
    | (!a2 & !b2 & !c2)
    | (!a2 & !b3 & !c1)
    | (!a2 & !b3 & !c2)
    | (!a3 & !b1 & !c1)
    | (!a3 & !b1 & !c2)
    | (!a3 & !b2 & !c1)
    | (!a3 & !b2 & !c2)
    | (!a3 & !b3 & !c1)
    | (!a3 & !b3 & !c2)
);
create_cell!(
  &["AOI333xp33_ASAP7_75t_R"],
  Asap7AndOrInv333 {
    a1,
    a2,
    a3,
    b1,
    b2,
    b3,
    c1,
    c2,
    c3
  },
  y,
  |a1: Bit, a2: Bit, a3: Bit, b1: Bit, b2: Bit, b3: Bit, c1: Bit, c2: Bit, c3: Bit| (!a1
    & !b1
    & !c1)
    | (!a1 & !b1 & !c2)
    | (!a1 & !b1 & !c3)
    | (!a1 & !b2 & !c1)
    | (!a1 & !b2 & !c2)
    | (!a1 & !b2 & !c3)
    | (!a1 & !b3 & !c1)
    | (!a1 & !b3 & !c2)
    | (!a1 & !b3 & !c3)
    | (!a2 & !b1 & !c1)
    | (!a2 & !b1 & !c2)
    | (!a2 & !b1 & !c3)
    | (!a2 & !b2 & !c1)
    | (!a2 & !b2 & !c2)
    | (!a2 & !b2 & !c3)
    | (!a2 & !b3 & !c1)
    | (!a2 & !b3 & !c2)
    | (!a2 & !b3 & !c3)
    | (!a3 & !b1 & !c1)
    | (!a3 & !b1 & !c2)
    | (!a3 & !b1 & !c3)
    | (!a3 & !b2 & !c1)
    | (!a3 & !b2 & !c2)
    | (!a3 & !b2 & !c3)
    | (!a3 & !b3 & !c1)
    | (!a3 & !b3 & !c2)
    | (!a3 & !b3 & !c3)
);
create_cell!(
  &["AOI33xp33_ASAP7_75t_R"],
  Asap7AndOrInv33 {
    a1,
    a2,
    a3,
    b1,
    b2,
    b3
  },
  y,
  |a1: Bit, a2: Bit, a3: Bit, b1: Bit, b2: Bit, b3: Bit| (!a1 & !b1)
    | (!a1 & !b2)
    | (!a1 & !b3)
    | (!a2 & !b1)
    | (!a2 & !b2)
    | (!a2 & !b3)
    | (!a3 & !b1)
    | (!a3 & !b2)
    | (!a3 & !b3)
);

#[derive(Debug, Clone, Serialize, Deserialize, derive_new::new)]
pub struct Asap7DffInv {
  clock_net: usize,
  data_in_net: usize,
  data_out_net: usize,
  #[new(default)]
  last_clock: Bit,
}

impl CellFn for Asap7DffInv {
  #[inline]
  fn eval(&mut self, signals: &mut Signals) {
    // Check if clock active for any polarity
    let clock = signals.get_net(self.clock_net);

    // Rising edge
    if clock == Bit::ONE && self.last_clock == Bit::ZERO {
      let data_in = signals.get_net(self.data_in_net);
      signals.set_net(self.data_out_net, !data_in); // Inverse
    }

    self.last_clock = clock;
  }

  fn reset(&mut self) {
    self.last_clock = Bit::ZERO;
  }
}

inventory::submit! {
  CellRegistration::new(&[
    "DFFHQNx1_ASAP7_75t_R",
    "DFFHQNx2_ASAP7_75t_R",
    "DFFHQNx3_ASAP7_75t_R",
  ],
  |connections: &BTreeMap<&str, Box<[usize]>>, _parameters: &BTreeMap<&str, usize>| {
    Asap7DffInv::new(
      connections["CLK"][0],
      connections["D"][0],
      connections["QN"][0],
    ).into()
  })
}

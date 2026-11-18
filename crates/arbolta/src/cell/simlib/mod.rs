// Copyright (c) 2026 Alexander Redding
// SPDX-License-Identifier: MIT

use super::{Cell, CellFn, CellRegistration};
use crate::{
  bit::{Bit, BitVec},
  signal::Signals,
};
use paste::paste;
use std::{borrow::Borrow, collections::BTreeMap, env};
mod arithmetic;
mod bool_ops;
mod logic_reduce_ops;
mod registers;
mod shift_ops;
mod various;

#[inline]
fn copy_nets<S, D, SI, DI>(signals: &mut Signals, src_nets: S, dst_nets: D)
where
  S: IntoIterator<Item = SI>,
  D: IntoIterator<Item = DI>,
  SI: Borrow<usize>,
  DI: Borrow<usize>,
{
  for (src, dst) in src_nets.into_iter().zip(dst_nets) {
    signals.set_net(*dst.borrow(), signals.get_net(*src.borrow()));
  }
}

#[inline]
fn copy_bits<'a>(
  signals: &mut Signals,
  dst_nets: &[usize],
  bits: impl IntoIterator<Item = &'a Bit>,
) {
  dst_nets
    .iter()
    .zip(bits)
    .for_each(|(&n, b)| signals.set_net(n, *b));
}

#[macro_export]
macro_rules! define_arithmetic_cell {
  ($rtl_names:expr, $cell_type:ident { $($in_port:ident),* $(,)?}, $out_port:ident, $body:expr) => {
    paste! {
      #[derive(Debug, Clone, Serialize, Deserialize)]
      pub struct $cell_type {
        pub signed: bool,
        // Nets
        $(
          [<$in_port _nets>]: Box<[usize]>,
        )*

        [<$out_port _nets>]: Box<[usize]>,
        // Temp bits
        $(
          [<$in_port _bits>]: BitVec,
        )*

        [<$out_port _bits>]: BitVec
      }
    }
    paste! {
      impl $cell_type {
        pub fn new(
          signed: bool,
          $(
            [<$in_port _nets>]: Box<[usize]>,
          )*
          [<$out_port _nets>]: Box<[usize]>,
        ) -> Self {
          Self {
            signed,
            $(
              [<$in_port _bits>]: BitVec::new([<$in_port _nets>].len()),
              [<$in_port _nets>],
            )*
            [<$out_port _bits>]: BitVec::new([<$out_port _nets>].len()),
            [<$out_port _nets>],
          }
        }
      }
    }

    paste! {
      impl CellFn for $cell_type {
        #[inline]
        fn eval(&mut self, signals: &mut Signals) {
          $(
            self.[<$in_port _bits>].set_bits(self.[<$in_port _nets>].iter().map(|&n| signals.get_net(n)));
          )*

          let output_size = self.[<$out_port _nets>].len();

          if self.signed {
            if output_size <= 64 {
              let ($($in_port,)*) = ($(self.[<$in_port _bits>].to_int::<i64>(),)*);
              self.[<$out_port _bits>].set_int( $body as i64 );
            } else {
              let ($($in_port,)*) = ($(self.[<$in_port _bits>].to_int::<i128>(),)*);
              self.[<$out_port _bits>].set_int( $body as i64 );
            }
          } else {
            if output_size <= 64 {
              let ($($in_port,)*) = ($(self.[<$in_port _bits>].to_int::<u64>(),)*);
              self.[<$out_port _bits>].set_int( $body as i64 );
            } else {
              let ($($in_port,)*) = ($(self.[<$in_port _bits>].to_int::<u128>(),)*);
              self.[<$out_port _bits>].set_int( $body as i64 );
            }
          };

          copy_bits(signals, &self.[<$out_port _nets>], &self.[<$out_port _bits>]);
        }

        fn reset(&mut self) {}
      }
    }
    paste! {
      inventory::submit! {CellRegistration::new($rtl_names,
        |connections: &BTreeMap<&str, Box<[usize]>>, parameters: &BTreeMap<&str, usize>| {
          if env::var("ARBOLTA_DEBUG").is_ok() {
            println!("Parsing connections: {:#?}", connections);
          }

          let mut signed = false;
          $(
            if let Some(&net_signed) = parameters.get(stringify!([<$in_port:upper _SIGNED>])) {
              signed |= (net_signed != 0);
            }
          )*

          $cell_type::new(
            signed,
            $(
              connections[stringify!([<$in_port:upper>])].clone(),
            )*
            connections[stringify!([<$out_port:upper>])].clone()
          ).into()
      })}
    }
  };
}

#[allow(unused)]
pub(crate) use define_arithmetic_cell;

// Re-export
pub use arithmetic::*;
pub use bool_ops::*;
pub use logic_reduce_ops::*;
pub use registers::*;
pub use shift_ops::*;
pub use various::*;

// TODO: clean up temp functions
fn make_not(
  connections: &BTreeMap<&str, Box<[usize]>>,
  parameters: &BTreeMap<&str, usize>,
) -> Cell {
  Not::new(
    parameters["A_SIGNED"] != 0,
    connections["A"].clone(),
    connections["Y"].clone(),
  )
  .into()
}

fn make_buffer(
  connections: &BTreeMap<&str, Box<[usize]>>,
  _parameters: &BTreeMap<&str, usize>,
) -> Cell {
  ProcBuffer::new(connections["A"].clone(), connections["Y"].clone()).into()
}

fn make_pos(
  connections: &BTreeMap<&str, Box<[usize]>>,
  parameters: &BTreeMap<&str, usize>,
) -> Cell {
  Pos::new(
    parameters["A_SIGNED"] != 0,
    connections["A"].clone(),
    connections["Y"].clone(),
  )
  .into()
}

fn make_dff(
  connections: &BTreeMap<&str, Box<[usize]>>,
  parameters: &BTreeMap<&str, usize>,
) -> Cell {
  Reg::new(
    (parameters["CLK_POLARITY"] != 0).into(),
    connections["CLK"][0],
    connections["D"].clone(),
    connections["Q"].clone(),
  )
  .into()
}

fn make_aldff(
  connections: &BTreeMap<&str, Box<[usize]>>,
  parameters: &BTreeMap<&str, usize>,
) -> Cell {
  DffAsyncLoad::new(
    (parameters["CLK_POLARITY"] != 0).into(),
    (parameters["ALOAD_POLARITY"] != 0).into(),
    connections["CLK"][0],
    connections["ALOAD"][0],
    connections["AD"].clone(),
    connections["D"].clone(),
    connections["Q"].clone(),
  )
  .into()
}

fn make_adffe(
  connections: &BTreeMap<&str, Box<[usize]>>,
  parameters: &BTreeMap<&str, usize>,
) -> Cell {
  let reset_val = BitVec::from_int(parameters["ARST_VALUE"], Some(parameters["WIDTH"]));

  DffAsyncResetEnable::new(
    (parameters["CLK_POLARITY"] != 0).into(),
    (parameters["EN_POLARITY"] != 0).into(),
    (parameters["ARST_POLARITY"] != 0).into(),
    reset_val.bits.into(),
    connections["CLK"][0],
    connections["ARST"][0],
    connections["EN"][0],
    connections["D"].clone(),
    connections["Q"].clone(),
  )
  .into()
}

fn make_mux(
  connections: &BTreeMap<&str, Box<[usize]>>,
  _parameters: &BTreeMap<&str, usize>,
) -> Cell {
  Mux::new(
    connections["S"][0],
    connections["A"].clone(),
    connections["B"].clone(),
    connections["Y"].clone(),
  )
  .into()
}

fn make_bmux(
  connections: &BTreeMap<&str, Box<[usize]>>,
  _parameters: &BTreeMap<&str, usize>,
) -> Cell {
  BMux::new(
    connections["S"].clone(),
    connections["A"].clone(),
    connections["Y"].clone(),
  )
  .into()
}

fn make_pmux(
  connections: &BTreeMap<&str, Box<[usize]>>,
  _parameters: &BTreeMap<&str, usize>,
) -> Cell {
  PMux::new(
    connections["S"].clone(),
    connections["A"].clone(),
    connections["B"].clone(),
    connections["Y"].clone(),
  )
  .into()
}

inventory::submit! {CellRegistration::new(&["$not"], make_not)}
inventory::submit! {CellRegistration::new(&["$buf"], make_buffer)}
inventory::submit! {CellRegistration::new(&["$pos"], make_pos)}
inventory::submit! {CellRegistration::new(&["$dff"], make_dff)}
inventory::submit! {CellRegistration::new(&["$aldff"], make_aldff)}
inventory::submit! {CellRegistration::new(&["$adffe"], make_adffe)}
inventory::submit! {CellRegistration::new(&["$mux"], make_mux)}
inventory::submit! {CellRegistration::new(&["$bmux"], make_bmux)}
inventory::submit! {CellRegistration::new(&["$pmux"], make_pmux)}

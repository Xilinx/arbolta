// Copyright (c) 2026 Alexander Redding
// SPDX-License-Identifier: MIT

mod simcells;
mod simlib;
mod test_helpers;

// Re-export
use crate::signal::Signals;
use enum_dispatch::enum_dispatch;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
pub use simcells::*;
pub use simlib::*;
use std::{
  collections::{BTreeMap, HashMap},
  env,
};
use thiserror::Error;

#[enum_dispatch]
pub trait CellFn {
  fn eval(&mut self, signals: &mut Signals);
  fn reset(&mut self);
}

// connections, parameters
pub type CellCtor = fn(&BTreeMap<&str, Box<[usize]>>, &BTreeMap<&str, usize>) -> Cell;
pub type CellDispatchMap = HashMap<&'static str, CellCtor>;

#[derive(derive_more::Constructor)]
pub struct CellRegistration {
  pub aliases: &'static [&'static str],
  pub ctor: CellCtor,
}

inventory::collect!(CellRegistration);

pub static CELL_DISPATCH: Lazy<HashMap<&'static str, CellCtor>> = Lazy::new(|| {
  let mut cell_map = HashMap::new();
  for registration in inventory::iter::<CellRegistration> {
    for name in registration.aliases {
      cell_map.insert(*name, registration.ctor);
    }
  }
  cell_map
});

#[enum_dispatch(CellFn)]
#[derive(Debug, Serialize, Deserialize, Clone)]
/// Proxy for a standard-cell and basic unit of 'compute'.
pub enum Cell {
  // ASAP7
  Asap7Buf,
  Asap7Inv,
  Asap7And2,
  Asap7And3,
  Asap7And4,
  Asap7And5,
  Asap7FullAdderInv,
  Asap7HalfAdderInv,
  Asap7MajorityInv,
  Asap7Majority,
  Asap7Nand2,
  Asap7Nand3,
  Asap7Nand4,
  Asap7Nand5,
  Asap7Nor2,
  Asap7Nor3,
  Asap7Nor4,
  Asap7Nor5,
  Asap7Or2,
  Asap7Or3,
  Asap7Or4,
  Asap7Or5,
  Asap7TieHigh,
  Asap7TieLow,
  Asap7Xnor2,
  Asap7Xor2,
  Asap7Or2And1Or1Inv,
  Asap7OrAnd211,
  Asap7OrAnd21,
  Asap7OrAnd221,
  Asap7OrAnd222,
  Asap7OrAnd22,
  Asap7OrAnd31,
  Asap7OrAnd331,
  Asap7OrAnd332,
  Asap7OrAnd333,
  Asap7OrAnd33,
  Asap7OrAndInv211,
  Asap7OrAndInv21,
  Asap7OrAndInv221,
  Asap7OrAndInv222,
  Asap7OrAndInv22,
  Asap7OrAndInv311,
  Asap7OrAndInv31,
  Asap7OrAndInv321,
  Asap7OrAndInv322,
  Asap7OrAndInv32,
  Asap7OrAndInv331,
  Asap7OrAndInv332,
  Asap7OrAndInv333,
  Asap7OrAndInv33,
  Asap7And2Or1And1Inv,
  Asap7And2Or1And1Or1Inv,
  Asap7AndOr211,
  Asap7AndOr21,
  Asap7AndOr221,
  Asap7AndOr222,
  Asap7AndOr22,
  Asap7AndOr31,
  Asap7AndOr322,
  Asap7AndOr32,
  Asap7AndOr331,
  Asap7AndOr332,
  Asap7AndOr333,
  Asap7AndOr33,
  Asap7AndOrInv211,
  Asap7AndOrInv21,
  Asap7AndOrInv221,
  Asap7AndOrInv222,
  Asap7AndOrInv22,
  Asap7AndOrInv311,
  Asap7AndOrInv31,
  Asap7AndOrInv321,
  Asap7AndOrInv322,
  Asap7AndOrInv32,
  Asap7AndOrInv331,
  Asap7AndOrInv332,
  Asap7AndOrInv333,
  Asap7AndOrInv33,
  Asap7DffInv,
  // Sim Cells
  // - Unary
  Buffer,
  Inverter,
  // - Binary
  And2,
  AndNot2,
  Nand2,
  Nor2,
  Or2,
  OrNot2,
  Xnor2,
  Xor2,
  // - Ternary
  AndOrInvert3,
  Mux2,
  NMux2,
  OrAndInvert3,
  // - Memory
  Dff,
  DffReset,
  // Sim Lib
  // - Arithmetic
  Add,
  Div,
  Equal,
  GreaterEqual,
  GreaterThan,
  LessEqual,
  LessThan,
  Modulus,
  Mul,
  Negate,
  NotEqual,
  Sub,
  //
  DffAsyncLoad,
  DffAsyncResetEnable,
  BMux,
  LogicAnd,
  LogicNot,
  LogicOr,
  Mux,
  Not,
  PMux,
  ProcBuffer,
  Pos,
  ProcAnd,
  ProcOr,
  ProcXor,
  ReduceAnd,
  ReduceOr,
  Reg,
  Shl,
  Shr,
}

#[derive(Debug, Error)]
pub enum CellError {
  #[error("Unsupported cell `{0}`")]
  Unsupported(String),
  #[error("Cell `{0}` not found in netlist")]
  NotFound(String),
  #[error("Direction `{0}` not supported")]
  Direction(String),
}

pub type CellMapping = HashMap<String, (String, Option<HashMap<String, String>>)>;

pub fn create_cell(
  cell_type: &str,
  connections: &BTreeMap<&str, Box<[usize]>>,
  parameters: &BTreeMap<&str, usize>,
  mapping: Option<&CellMapping>,
) -> Result<Cell, CellError> {
  if env::var("ARBOLTA_DEBUG").is_ok() {
    println!("Parsing cell `{cell_type}`")
  }

  let (cell_type, connections) = if let Some(mapping) = mapping
    && let Some((mapped_cell_type, mapped_connections)) = mapping.get(cell_type)
  {
    let connections = match mapped_connections {
      Some(mapped_connections) => connections
        .iter()
        .map(|(&port_name, nets)| (mapped_connections[port_name].as_str(), nets.clone()))
        .collect(),
      None => connections.clone(),
    };

    (mapped_cell_type.as_str(), connections)
  } else {
    (cell_type, connections.clone())
  };

  let ctor = CELL_DISPATCH
    .get(cell_type)
    .ok_or_else(|| CellError::Unsupported(cell_type.to_string()))?;

  Ok(ctor(&connections, parameters))
}

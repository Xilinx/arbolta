// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

use crate::bit::Bit;
use crate::signal::{AccessSignal, SignalIndex, SignalList};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

pub const CONNECTION_SIZE: usize = 8;
pub const STATE_SIZE: usize = 2;

/// Basic logic gate functions.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum Function {
  Inverter,
  And,
  Nor,
  Nand,
  Xor,
  Xnor,
  Or,
  DffPosEdge,
  Buf,
}

/// Proxy for entry in a Liberty Cell Library.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CellInfo {
  pub name: String,
  pub function: Function,
  pub area: f64,
  pub num_inputs: usize,
}

/// Proxy for a standard-cell and basic unit of 'compute'.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Cell {
  /// Name of cell.
  pub name: String,
  /// Cell's function.
  pub function: Function,
  /// For storing cell state (ex, last clock value).
  pub state: [Bit; STATE_SIZE],
  /// Number of inputs the cell has.
  pub num_inputs: usize,
  /// Input signal indices.
  pub input_connections: [SignalIndex; CONNECTION_SIZE], // Put this on stack
  /// Output signal index.
  pub output_connection: SignalIndex,
}

#[derive(Debug, Error)]
pub enum CellError {
  #[error("couldn't find cell `{0}`")]
  NotFound(String),
}

impl From<&CellInfo> for Cell {
  fn from(value: &CellInfo) -> Self {
    Self {
      name: value.name.clone(),
      function: value.function.clone(),
      state: [Bit::Zero; STATE_SIZE],
      num_inputs: value.num_inputs,
      input_connections: [0; CONNECTION_SIZE],
      output_connection: 0,
    }
  }
}

impl From<CellInfo> for Cell {
  fn from(value: CellInfo) -> Self {
    Cell::from(&value)
  }
}

/// Proxy for a Liberty Cell Library
/// User can define their own cells
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CellLibrary {
  pub cells: HashMap<String, CellInfo>,
}

impl CellLibrary {
  /// Generate a cell given its name
  /// # Arguments
  /// * `cell_name` - Name of cell to generate
  pub fn generate_cell(&self, cell_name: &str) -> Result<Cell, CellError> {
    match self.cells.get(cell_name) {
      Some(cell_info) => Ok(Cell::from(cell_info)),
      None => Err(CellError::NotFound(cell_name.to_string())),
    }
  }

  pub fn get_cell_area(&self, cell_name: &str) -> Result<f64, CellError> {
    match self.cells.get(cell_name) {
      Some(cell_info) => Ok(cell_info.area),
      None => Err(CellError::NotFound(cell_name.to_string())),
    }
  }

  pub fn get_cell_breakdown_area(
    &self,
    breakdown: &HashMap<String, usize>,
  ) -> Result<f64, CellError> {
    let mut total_area: f64 = 0.0;
    for (cell_name, count) in breakdown {
      total_area += (*count as f64) * self.get_cell_area(cell_name)?;
    }

    Ok(total_area)
  }
}

impl Cell {
  pub fn empty_from_function(function: Function) -> Self {
    Self {
      name: String::new(),
      function,
      state: [Bit::Zero; STATE_SIZE],
      num_inputs: 0,
      input_connections: [0; CONNECTION_SIZE],
      output_connection: 0,
    }
  }

  pub fn eval(&mut self, signals: &mut SignalList) {
    let mut output_bit: Bit;

    match &self.function {
      Function::Buf => {
        output_bit = signals[self.input_connections[0]].get_value();
      }
      Function::Inverter => {
        output_bit = !signals[self.input_connections[0]].get_value();
      }
      Function::And => {
        output_bit = signals[self.input_connections[0]].get_value();
        for bit in self.input_connections[1..self.num_inputs]
          .iter()
          .map(|i| signals[*i].get_value())
        {
          output_bit = output_bit & bit;
        }
      }
      Function::Or => {
        output_bit = signals[self.input_connections[0]].get_value();
        for bit in self.input_connections[1..self.num_inputs]
          .iter()
          .map(|i| signals[*i].get_value())
        {
          output_bit = output_bit | bit;
        }
      }
      Function::Nor => {
        output_bit = signals[self.input_connections[0]].get_value();
        for bit in self.input_connections[1..self.num_inputs]
          .iter()
          .map(|i| signals[*i].get_value())
        {
          output_bit = output_bit | bit;
        }
        output_bit = !output_bit;
      }
      Function::Nand => {
        output_bit = signals[self.input_connections[0]].get_value();
        for bit in self.input_connections[1..self.num_inputs]
          .iter()
          .map(|i| signals[*i].get_value())
        {
          output_bit = output_bit & bit;
        }
        output_bit = !output_bit;
      }
      Function::Xor => {
        output_bit = signals[self.input_connections[0]].get_value();
        for bit in self.input_connections[1..self.num_inputs]
          .iter()
          .map(|i| signals[*i].get_value())
        {
          output_bit = output_bit ^ bit;
        }
      }
      Function::Xnor => {
        output_bit = signals[self.input_connections[0]].get_value();
        for bit in self.input_connections[1..self.num_inputs]
          .iter()
          .map(|i| signals[*i].get_value())
        {
          output_bit = output_bit ^ bit;
        }
        output_bit = !output_bit;
      }
      Function::DffPosEdge => {
        let (clock, data) = (
          signals[self.input_connections[0]].get_value(),
          signals[self.input_connections[1]].get_value(),
        );
        let (last_data, last_clock) = (self.state[0], self.state[1]);
        // Detect rising edge, clock new data
        output_bit = if clock == Bit::One && last_clock == Bit::Zero {
          data
        } else {
          last_data
        };
        self.state = [output_bit, clock];
      }
    };
    signals[self.output_connection].set_value(output_bit);
  }

  pub fn reset(&mut self) {
    if self.function == Function::DffPosEdge {
      self.state = [Bit::Zero; 2]
    }
  }
}

pub fn default_cell_library() -> CellLibrary {
  let cells = HashMap::from([
    (
      "BUF".to_string(),
      CellInfo {
        name: "BUF".to_string(),
        function: Function::Buf,
        area: 4.0,
        num_inputs: 1,
      },
    ),
    (
      "NOT".to_string(),
      CellInfo {
        name: "NOT".to_string(),
        function: Function::Inverter,
        area: 2.0,
        num_inputs: 1,
      },
    ),
    (
      "NAND".to_string(),
      CellInfo {
        name: "NAND".to_string(),
        function: Function::Nand,
        area: 4.0,
        num_inputs: 2,
      },
    ),
    (
      "NOR".to_string(),
      CellInfo {
        name: "NOR".to_string(),
        function: Function::Nor,
        area: 4.0,
        num_inputs: 2,
      },
    ),
    (
      "DFF".to_string(),
      CellInfo {
        name: "DFF".to_string(),
        function: Function::DffPosEdge,
        area: 8.0,
        num_inputs: 2,
      },
    ),
  ]);

  CellLibrary { cells }
}

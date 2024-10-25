// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

use crate::bit::Bit;
use crate::cell::{Cell, CellLibrary};
use crate::module::hardware_module::{Component, ComponentIndexMap, HardwareModule, PortMap};
use crate::module::port::{Port, PortDirection};
use crate::signal::{AccessSignal, Signal, SignalIndexMap, SignalList};
use std::collections::BTreeMap;
use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SynthError {
  #[error("Module `{0}` does not exist")]
  MissingModule(String),
  #[error("Error opening netlist: {0}")]
  Netlist(String),
  #[error("{0}")]
  IoError(#[from] io::Error),
}

#[derive(Debug)]
pub enum SynthBit {
  Constant(Bit),
  NetIndex(usize),
}

#[derive(Debug)]
pub struct SynthPort {
  pub direction: PortDirection,
  pub bits: Vec<SynthBit>, // context of local module
  pub signed: bool,
}

#[derive(Debug)]
pub struct SynthCell {
  pub cell_type: String,
  pub connections: BTreeMap<String, Vec<SynthBit>>,
}

#[derive(Debug)]
pub struct SynthModule {
  // Preserve topological order of cells
  pub ports: BTreeMap<String, SynthPort>,
  pub cells: BTreeMap<String, SynthCell>,
  pub nets: BTreeMap<String, Vec<SynthBit>>, // context of local module
}

impl SynthModule {
  pub fn max_net_idx(&self) -> usize {
    self
      .nets
      .values()
      .map(|bits| {
        bits
          .iter()
          .map(|x| match x {
            SynthBit::Constant(_) => 0,
            SynthBit::NetIndex(idx) => *idx,
          })
          .max()
          .unwrap()
      })
      .max()
      .unwrap()
  }
}

impl From<&SynthBit> for Signal {
  fn from(value: &SynthBit) -> Self {
    match value {
      SynthBit::Constant(x) => Signal::new_constant(*x),
      SynthBit::NetIndex(x) => Signal::new_net(*x),
    }
  }
}

impl From<SynthBit> for Signal {
  fn from(value: SynthBit) -> Self {
    Self::from(&value)
  }
}

impl From<&SynthPort> for Port {
  fn from(value: &SynthPort) -> Self {
    let signal_idx_list: Vec<usize> = value
      .bits
      .iter()
      .map(|x| match x {
        SynthBit::Constant(bit) => match bit {
          Bit::Zero => 0,
          Bit::One => 1,
        },
        SynthBit::NetIndex(idx) => *idx,
      })
      .collect();

    let shape = [1, signal_idx_list.len()];

    Self {
      signal_idx_list,
      shape,
      direction: value.direction.clone(),
      signed: value.signed,
    }
  }
}

impl From<SynthPort> for Port {
  fn from(value: SynthPort) -> Self {
    Self::from(&value)
  }
}

#[derive(Debug)]
pub struct Netlist {
  pub modules: BTreeMap<String, SynthModule>,
}

impl Netlist {
  pub fn generate_module(
    &self,
    name: &str,
    cell_library: &CellLibrary,
  ) -> Result<HardwareModule, SynthError> {
    let top_module: &SynthModule = match self.modules.get(name) {
      Some(x) => x,
      None => return Err(SynthError::MissingModule(name.to_string())),
    };

    let ports: PortMap = top_module
      .ports
      .iter()
      .map(|(port_name, port_synth)| (port_name.clone(), Port::from(port_synth)))
      .collect();

    let mut signals: SignalList = (0..(top_module.max_net_idx() + 1))
      .map(|_| Signal::new_constant(Bit::Zero))
      .collect();
    // Bits 0 and 1 are unused by Yosys so we keep them as constant 0 and 1 respectively
    signals[1] = Signal::new_constant(Bit::One);

    let mut signal_map = SignalIndexMap::new();
    for (net_name, bits) in &top_module.nets {
      for (i, bit) in bits.iter().enumerate() {
        let signal_name = if bits.len() > 1 {
          format!("{net_name}[{i}]")
        } else {
          net_name.clone()
        };

        match bit {
          SynthBit::Constant(_) => (), // Do nothing
          SynthBit::NetIndex(idx) => {
            let mut signal = Signal::new_net(*idx);
            signal_map.insert(signal_name.clone(), *idx);
            signal.set_name(signal_name);
            signals[*idx] = signal;
          }
        }
      }
    }

    let mut components: Vec<Component> = vec![];
    let mut component_map = ComponentIndexMap::new();

    for (instance_name, synth_cell) in &top_module.cells {
      let new_component = match cell_library.cells.get(&synth_cell.cell_type) {
        Some(cell_info) => {
          let mut cell = Cell::from(cell_info);
          // flatten this for now, should only be 1 bit
          for (i, bits) in synth_cell.connections.values().enumerate() {
            let idx = match &bits[0] {
              SynthBit::Constant(bit) => match bit {
                Bit::Zero => 0,
                Bit::One => 1,
              },
              SynthBit::NetIndex(idx) => *idx,
            };
            cell.input_connections[i] = idx;
          }
          // this sets last input as output but, fix later
          cell.output_connection = cell.input_connections[cell.num_inputs];
          Component::Cell(cell)
        }
        None => {
          let mut submodule = self.generate_module(&synth_cell.cell_type, cell_library)?;
          for (port_name, bits) in &synth_cell.connections {
            let port = submodule.ports.get(port_name).unwrap();
            for (i, bit) in bits.iter().enumerate() {
              let idx = match bit {
                SynthBit::Constant(bit) => match bit {
                  Bit::Zero => 0,
                  Bit::One => 1,
                },
                SynthBit::NetIndex(idx) => *idx,
              };

              match port.direction {
                PortDirection::Input => submodule
                  .input_connections
                  .push((idx, port.signal_idx_list[i])),
                PortDirection::Output => submodule
                  .output_connections
                  .push((idx, port.signal_idx_list[i])),
              }
            }
          }
          Component::Module(submodule)
        }
      };
      component_map.insert(instance_name.clone(), components.len());
      components.push(new_component);
    }

    Ok(HardwareModule {
      name: name.to_string(),
      ports,
      signals,
      signal_map,
      components,
      component_map,
      input_connections: vec![],
      output_connections: vec![],
    })
  }
}

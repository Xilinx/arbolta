// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

use super::port::{Port, PortDirection, PortError};
use crate::bit::{Bit, BitVec};
use crate::cell::Cell;
use crate::signal::{AccessSignal, SignalIndex, SignalIndexMap, SignalList};
use ndarray::{Array1, ArrayView1};
use num_traits::PrimInt;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Debug;
use thiserror::Error;

pub type PortMap = BTreeMap<String, Port>;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum Component {
  Cell(Cell),
  Module(HardwareModule),
}

pub type ComponentIndex = usize;
pub type ComponentIndexMap = BTreeMap<String, ComponentIndex>;

#[derive(Default, Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct HardwareModule {
  pub name: String,
  pub ports: PortMap,
  pub signals: SignalList,
  pub signal_map: SignalIndexMap,
  pub components: Vec<Component>,
  pub component_map: ComponentIndexMap,
  pub input_connections: Vec<(SignalIndex, SignalIndex)>,
  pub output_connections: Vec<(SignalIndex, SignalIndex)>,
}

#[derive(Debug, Error)]
pub enum ModuleError {
  #[error("module does not have port `{0}`")]
  MissingPort(String),
  #[error("error accessing port `{0}`: {1}")]
  Port(String, PortError),
  #[error("module does not have signal `{0}`")]
  MissingSignal(String),
  #[error("module does not have signal index `{0}`")]
  MissingSignalIndex(SignalIndex),
  #[error("module `{0}` does not exist")]
  MissingModule(String),
}

impl HardwareModule {
  pub fn get_signal_idx(&self, name: &str) -> Result<SignalIndex, ModuleError> {
    match self.signal_map.get(name) {
      Some(idx) => Ok(*idx),
      None => Err(ModuleError::MissingSignal(name.to_string())),
    }
  }

  pub fn set_signal(&mut self, idx: SignalIndex, val: Bit) -> Result<(), ModuleError> {
    if idx > self.signals.len() {
      Err(ModuleError::MissingSignalIndex(idx))
    } else {
      self.signals[idx].set_value(val);
      Ok(())
    }
  }

  pub fn get_module_port_int<T: PrimInt + std::ops::BitXorAssign>(
    &self,
    path: Vec<&str>,
    name: &str,
  ) -> Result<T, ModuleError> {
    if path.is_empty() {
      return self.get_port_int(name);
    }

    for component in &self.components {
      match component {
        Component::Cell(_) => (),
        Component::Module(module) => {
          if path[0] == module.name {
            return module.get_module_port_int(path[1..].to_vec(), name);
          }
        }
      }
    }
    Err(ModuleError::MissingPort(name.to_string()))
  }

  pub fn search_signal(&mut self, name: &str) -> Option<Bit> {
    for signal in &self.signals {
      if signal.get_name() == name {
        return Some(signal.get_value());
      }
    }

    for component in &mut self.components {
      match component {
        Component::Cell(_) => (),
        Component::Module(module) => match module.search_signal(name) {
          Some(val) => return Some(val),
          None => continue,
        },
      }
    }
    None
  }

  pub fn eval(&mut self) {
    for component in &mut self.components {
      match component {
        Component::Cell(cell) => {
          cell.eval(&mut self.signals);
        }
        Component::Module(module) => {
          // Propagate input connections
          for (external_idx, internal_idx) in &module.input_connections {
            let bit = self.signals[*external_idx].get_value();
            module.signals[*internal_idx].set_value(bit);
          }
          module.eval();
          // Propagate output connections
          for (external_idx, internal_idx) in &module.output_connections {
            let bit = module.signals[*internal_idx].get_value();
            self.signals[*external_idx].set_value(bit);
          }
        }
      }
    }
  }

  pub fn reset(&mut self) {
    // Reset signals
    self.signals.iter_mut().for_each(|signal| signal.reset());

    // Reset components
    self
      .components
      .iter_mut()
      .for_each(|component| match component {
        Component::Cell(cell) => cell.reset(),
        Component::Module(module) => module.reset(),
      });
  }

  pub fn set_port_shape(&mut self, name: &str, shape: &[usize; 2]) -> Result<(), ModuleError> {
    match self.ports.get_mut(name) {
      Some(port) => match port.set_shape(shape) {
        Ok(()) => Ok(()),
        Err(err) => Err(ModuleError::Port(name.to_string(), err)),
      },
      None => Err(ModuleError::MissingPort(name.to_string())),
    }
  }

  pub fn get_port_shape(&self, name: &str) -> Result<[usize; 2], ModuleError> {
    match self.ports.get(name) {
      Some(port) => Ok(port.get_shape()),
      None => Err(ModuleError::MissingPort(name.to_string())),
    }
  }

  pub fn get_port_direction(&self, name: &str) -> Result<PortDirection, ModuleError> {
    match self.ports.get(name) {
      Some(port) => Ok(port.direction.clone()),
      None => Err(ModuleError::MissingPort(name.to_string())),
    }
  }

  pub fn get_port_bits(&self, name: &str) -> Result<BitVec, ModuleError> {
    match self.ports.get(name) {
      Some(port) => Ok(port.get_bits(&self.signals)),
      None => Err(ModuleError::MissingPort(name.to_string())),
    }
  }

  pub fn set_port_bits(&mut self, name: &str, vals: &BitVec) -> Result<(), ModuleError> {
    match self.ports.get_mut(name) {
      Some(port) => match port.set_bits(vals, &mut self.signals) {
        Ok(()) => Ok(()),
        Err(err) => Err(ModuleError::Port(name.to_string(), err)),
      },
      None => Err(ModuleError::MissingPort(name.to_string())),
    }
  }

  pub fn get_port_int<T: PrimInt + std::ops::BitXorAssign>(
    &self,
    name: &str,
  ) -> Result<T, ModuleError> {
    match self.ports.get(name) {
      Some(port) => Ok(port.get_int(&self.signals)),
      None => Err(ModuleError::MissingPort(name.to_string())),
    }
  }

  pub fn set_port_int<T: PrimInt + std::fmt::Display>(
    &mut self,
    name: &str,
    val: T,
  ) -> Result<(), ModuleError> {
    match self.ports.get_mut(name) {
      Some(port) => match port.set_int(val, &mut self.signals) {
        Ok(()) => Ok(()),
        Err(err) => Err(ModuleError::Port(name.to_string(), err)),
      },
      None => Err(ModuleError::MissingPort(name.to_string())),
    }
  }

  pub fn get_port_int_vec<T: PrimInt + std::ops::BitXorAssign>(
    &self,
    name: &str,
  ) -> Result<Vec<T>, ModuleError> {
    match self.ports.get(name) {
      Some(port) => Ok(port.get_int_vec(&self.signals)),
      None => Err(ModuleError::MissingPort(name.to_string())),
    }
  }

  pub fn set_port_int_vec<T: PrimInt>(
    &mut self,
    name: &str,
    vals: &[T],
  ) -> Result<(), ModuleError> {
    match self.ports.get_mut(name) {
      Some(port) => match port.set_int_vec(vals, &mut self.signals) {
        Ok(()) => Ok(()),
        Err(err) => Err(ModuleError::Port(name.to_string(), err)),
      },
      None => Err(ModuleError::MissingPort(name.to_string())),
    }
  }

  pub fn get_port_ndarray<T: PrimInt + std::ops::BitXorAssign>(
    &self,
    name: &str,
  ) -> Result<Array1<T>, ModuleError> {
    match self.ports.get(name) {
      Some(port) => Ok(port.get_ndarray(&self.signals)),
      None => Err(ModuleError::MissingPort(name.to_string())),
    }
  }

  pub fn set_port_ndarray<T: PrimInt + std::ops::BitXorAssign>(
    &mut self,
    name: &str,
    vals: ArrayView1<T>,
  ) -> Result<(), ModuleError> {
    match self.ports.get(name) {
      Some(port) => match port.set_ndarray(vals, &mut self.signals) {
        Ok(()) => Ok(()),
        Err(err) => Err(ModuleError::Port(name.to_string(), err)),
      },
      None => Err(ModuleError::MissingPort(name.to_string())),
    }
  }

  pub fn get_port_string(&self, name: &str) -> Result<String, ModuleError> {
    match self.ports.get(name) {
      Some(port) => Ok(port.get_string(&self.signals)),
      None => Err(ModuleError::MissingPort(name.to_string())),
    }
  }

  pub fn get_cell_breakdown(&self) -> HashMap<String, usize> {
    let mut breakdown = HashMap::<String, usize>::new();
    for component in &self.components {
      match component {
        Component::Cell(cell) => {
          if !breakdown.contains_key(&cell.name) {
            breakdown.insert(cell.name.clone(), 0);
          }

          *breakdown.get_mut(&cell.name).unwrap() += 1;
        }
        Component::Module(module) => {
          for (cell_name, count) in module.get_cell_breakdown() {
            if !breakdown.contains_key(&cell_name) {
              breakdown.insert(cell_name.clone(), 0);
            }
            *breakdown.get_mut(&cell_name).unwrap() += count;
          }
        }
      }
    }
    breakdown
  }

  pub fn search_module_cell_breakdown(
    &self,
    name: &str,
  ) -> Result<HashMap<String, usize>, ModuleError> {
    if name == self.name {
      Ok(self.get_cell_breakdown())
    } else {
      for component in &self.components {
        match component {
          Component::Cell(_) => continue,
          Component::Module(sub_module) => match sub_module.search_module_cell_breakdown(name) {
            Ok(breakdown) => return Ok(breakdown),
            Err(_) => continue,
          },
        }
      }
      Err(ModuleError::MissingModule(name.to_string()))
    }
  }

  // TODO: Add tests for these

  pub fn get_total_toggle_count(&self) -> usize {
    let mut total_toggles: usize = 0;
    let input_connections: HashSet<SignalIndex> = self
      .input_connections
      .iter()
      .map(|(_, internal_idx)| *internal_idx)
      .collect();
    self.signals.iter().for_each(|signal| {
      if !input_connections.contains(&signal.get_index()) {
        total_toggles += signal.get_total_toggle_count();
      }
    });
    self
      .components
      .iter()
      .for_each(|component| match component {
        Component::Cell(_) => (),
        Component::Module(module) => total_toggles += module.get_total_toggle_count(),
      });

    total_toggles
  }

  pub fn search_module_total_toggle_count(&self, name: &str) -> Result<usize, ModuleError> {
    if name == self.name {
      Ok(self.get_total_toggle_count())
    } else {
      for component in &self.components {
        match component {
          Component::Cell(_) => continue,
          Component::Module(sub_module) => {
            match sub_module.search_module_total_toggle_count(name) {
              Ok(count) => return Ok(count),
              Err(_) => continue,
            }
          }
        }
      }
      Err(ModuleError::MissingModule(name.to_string()))
    }
  }

  // need wrapper function to get input ports and not use connections
  pub fn get_module_bit_flips(&self, name: &str) -> usize {
    if self.name == name {
      let mut total_toggles = 0;

      let input_connections: HashSet<SignalIndex> = self
        .input_connections
        .iter()
        .map(|(_, internal_idx)| *internal_idx)
        .collect();
      self.signals.iter().for_each(|signal| {
        if !input_connections.contains(&signal.get_index()) {
          total_toggles += signal.get_total_toggle_count();
        }
      });

      return total_toggles;
    } else {
      for component in &self.components {
        match component {
          Component::Module(module) if module.name == name => {
            return module.get_module_bit_flips(name)
          }
          _ => continue,
        }
      }
    }
    0
  }
}

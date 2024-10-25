// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

use crate::bit::Bit;
use crate::cell::{CellError, CellLibrary};
use crate::module::hardware_module::{HardwareModule, ModuleError};
use crate::signal::SignalIndex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;
use std::io::Write;
use thiserror::Error;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Design {
  pub module: HardwareModule,
  pub clock: Option<SignalIndex>,
  pub reset: Option<SignalIndex>,
  pub cell_library: CellLibrary,
}

#[derive(Debug, Error)]
pub enum DesignError {
  #[error("{0}")]
  ModuleError(#[from] ModuleError),
  #[error("{0}")]
  CellError(#[from] CellError),
  #[error("{0}")]
  IoError(#[from] io::Error),
  #[error("{0}")]
  FlexReaderError(#[from] flexbuffers::ReaderError),
  #[error("{0}")]
  DeserializeError(#[from] flexbuffers::DeserializationError),
  #[error("{0}")]
  SerializeError(#[from] flexbuffers::SerializationError),
}

impl Design {
  pub fn load(path: &str) -> Result<Self, DesignError> {
    let serialized = std::fs::read(path)?;
    let reader = flexbuffers::Reader::get_root(serialized.as_slice())?;
    Ok(Self::deserialize(reader)?)
  }

  pub fn save(&self, path: &str) -> Result<(), DesignError> {
    let mut serializer = flexbuffers::FlexbufferSerializer::new();
    self.serialize(&mut serializer)?;
    let mut file_output = std::fs::File::create(path)?;
    _ = file_output.write(serializer.view())?;
    Ok(())
  }

  pub fn from_module(module: HardwareModule, cell_library: CellLibrary) -> Self {
    Self {
      module,
      clock: None,
      reset: None,
      cell_library,
    }
  }

  pub fn set_clock(&mut self, name: &str) -> Result<(), DesignError> {
    self.clock = Some(self.module.get_signal_idx(name)?);
    Ok(())
  }

  pub fn set_reset(&mut self, name: &str) -> Result<(), DesignError> {
    self.reset = Some(self.module.get_signal_idx(name)?);
    Ok(())
  }

  pub fn eval(&mut self) {
    self.module.eval();
  }

  pub fn eval_clocked(&mut self) -> Result<(), DesignError> {
    let Some(clock) = self.clock else {
      return Err(DesignError::ModuleError(ModuleError::MissingSignal(
        "clock".to_string(),
      )));
    };

    // Can we do this deterministically?
    self.module.eval();
    self.module.eval();
    self.module.eval();
    self.module.set_signal(clock, Bit::One)?;
    self.module.eval();
    self.module.set_signal(clock, Bit::Zero)?;
    self.module.eval();
    Ok(())
  }

  pub fn reset_clocked(&mut self) -> Result<(), DesignError> {
    let Some(reset) = self.reset else {
      return Err(DesignError::ModuleError(ModuleError::MissingSignal(
        "reset".to_string(),
      )));
    };

    self.module.set_signal(reset, Bit::One)?;
    self.eval_clocked()?;
    self.module.set_signal(reset, Bit::Zero)?;
    self.module.eval();

    Ok(())
  }

  pub fn get_module_area(&self, name: &str) -> Result<f64, DesignError> {
    let breakdown = self.get_module_breakdown(name)?;
    Ok(self.cell_library.get_cell_breakdown_area(&breakdown)?)
  }

  pub fn get_module_breakdown(&self, name: &str) -> Result<HashMap<String, usize>, DesignError> {
    Ok(self.module.search_module_cell_breakdown(name)?)
  }

  pub fn get_module_total_toggle_count(&self, name: &str) -> Result<usize, DesignError> {
    Ok(self.module.search_module_total_toggle_count(name)?)
  }
}

// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

use super::netlist::{Netlist, SynthBit, SynthCell, SynthError, SynthModule, SynthPort};
use crate::bit::Bit;
use crate::module::port::PortDirection;
use std::collections::BTreeMap;

impl From<yosys_netlist_json::PortDirection> for PortDirection {
  fn from(value: yosys_netlist_json::PortDirection) -> Self {
    match value {
      yosys_netlist_json::PortDirection::InOut => todo!("Inout not supported"),
      yosys_netlist_json::PortDirection::Input => Self::Input,
      yosys_netlist_json::PortDirection::Output => Self::Output,
    }
  }
}

impl From<yosys_netlist_json::BitVal> for SynthBit {
  fn from(value: yosys_netlist_json::BitVal) -> Self {
    match value {
      yosys_netlist_json::BitVal::N(idx) => Self::NetIndex(idx),
      yosys_netlist_json::BitVal::S(constant) => Self::Constant(match constant {
        yosys_netlist_json::SpecialBit::_0 => Bit::Zero,
        yosys_netlist_json::SpecialBit::_1 => Bit::One,
        yosys_netlist_json::SpecialBit::X => todo!("X not supported"),
        yosys_netlist_json::SpecialBit::Z => todo!("Z not supported"),
      }),
    }
  }
}

impl From<yosys_netlist_json::Port> for SynthPort {
  fn from(value: yosys_netlist_json::Port) -> Self {
    Self {
      direction: PortDirection::from(value.direction),
      bits: value.bits.iter().map(|x| SynthBit::from(*x)).collect(),
      signed: value.signed > 0,
    }
  }
}

impl From<yosys_netlist_json::Cell> for SynthCell {
  fn from(value: yosys_netlist_json::Cell) -> Self {
    let mut connections: BTreeMap<String, Vec<SynthBit>> = BTreeMap::new();
    for (key, vals) in value.connections {
      let bits: Vec<SynthBit> = vals.iter().map(|x| SynthBit::from(*x)).collect();
      connections.insert(key, bits);
    }

    Self {
      cell_type: value.cell_type,
      connections,
    }
  }
}

impl From<yosys_netlist_json::Module> for SynthModule {
  fn from(value: yosys_netlist_json::Module) -> Self {
    let ports: BTreeMap<String, SynthPort> = value
      .ports
      .into_iter()
      .map(|(key, val)| (key.clone(), SynthPort::from(val)))
      .collect();

    let cells: BTreeMap<String, SynthCell> = value
      .cells
      .into_iter()
      .map(|(key, val)| (key.clone(), SynthCell::from(val)))
      .collect();

    let nets: BTreeMap<String, Vec<SynthBit>> = value
      .netnames
      .into_iter()
      .map(|(key, vals)| {
        (
          key.clone(),
          vals.bits.iter().map(|x| SynthBit::from(*x)).collect(),
        )
      })
      .collect();

    Self { ports, cells, nets }
  }
}

impl From<yosys_netlist_json::Netlist> for Netlist {
  fn from(value: yosys_netlist_json::Netlist) -> Self {
    let mut modules: BTreeMap<String, SynthModule> = BTreeMap::new();
    for (key, val) in value.modules {
      modules.insert(key, SynthModule::from(val));
    }

    Self { modules }
  }
}

impl Netlist {
  pub fn from_yosys(json_path: &str) -> Result<Self, SynthError> {
    let raw_json = std::fs::read(json_path)?;
    let Ok(raw_netlist) = yosys_netlist_json::Netlist::from_slice(&raw_json) else {
      return Err(SynthError::Netlist(json_path.to_string()));
    };

    Ok(Netlist::from(raw_netlist))
  }

  pub fn from_yosys_raw(raw_json: &[u8]) -> Result<Self, SynthError> {
    let Ok(raw_netlist) = yosys_netlist_json::Netlist::from_slice(raw_json) else {
      return Err(SynthError::Netlist("raw_json".to_string()));
    };

    Ok(Netlist::from(raw_netlist))
  }
}

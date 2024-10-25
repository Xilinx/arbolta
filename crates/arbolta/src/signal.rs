// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

use crate::bit::Bit;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub type SignalIndex = usize;
pub type SignalList = Vec<Signal>;
pub type SignalIndexList = Vec<usize>;
pub type SignalIndexMap = BTreeMap<String, SignalIndex>;

/// Connection between cells/modules.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub struct Net {
  /// Name of net
  pub name: String,
  /// Index in netlist connections (proxy to Yosys bit)
  pub index: usize,
  /// Value of net
  pub value: Bit,
  /// Number of times net has transitioned from 0 -> 1
  pub toggle_count_rising: usize,
  /// Number of times net has transitioned from 1 -> 0
  pub toggle_count_falling: usize,
}

/// Connection between cells/modules or constant.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum Signal {
  Constant(Bit),
  Net(Net),
}

impl Signal {
  /// Create a new constant.
  ///
  /// # Arguments
  /// * `value` - Constant `Bit` value.
  pub fn new_constant(value: Bit) -> Self {
    Self::Constant(value)
  }

  /// Create a new net.
  ///
  /// # Arguments
  /// * `index` - Net index.
  pub fn new_net(index: usize) -> Self {
    Self::Net(Net {
      index,
      ..Default::default()
    })
  }

  /// Create a new net.
  ///
  /// # Arguments
  /// * `index` - Net index.
  /// * `value` - Value to initialize net with.
  pub fn new_net_from(index: usize, value: Bit) -> Self {
    Self::Net(Net {
      index,
      value,
      ..Default::default()
    })
  }

  /// Create a new list of `Signal`s.
  /// List initialized with zero `Constant`s.
  ///
  /// # Arguments
  /// * `size` - Size of list.
  pub fn new_list(size: usize) -> SignalList {
    let mut signal_list = SignalList::with_capacity(size);
    for _ in 0..size {
      signal_list.push(Self::new_constant(Bit::Zero));
    }
    signal_list
  }
}

pub trait AccessSignal {
  fn reset(&mut self);
  fn set_name(&mut self, name: String);
  fn get_name(&self) -> &str;
  fn get_index(&self) -> usize;
  fn get_value(&self) -> Bit;
  fn set_value(&mut self, val: Bit);
  fn get_total_toggle_count(&self) -> usize;
  fn get_toggle_count_rising(&self) -> usize;
  fn get_toggle_count_falling(&self) -> usize;
}

impl AccessSignal for Signal {
  /// Reset signal value to zero.
  /// Clear all signal statistics.
  fn reset(&mut self) {
    match self {
      Signal::Constant(_) => (), // Do nothing
      Signal::Net(net) => {
        net.toggle_count_rising = 0;
        net.toggle_count_falling = 0;
        net.value = Bit::Zero;
      }
    }
  }

  /// Set name of signal.
  fn set_name(&mut self, name: String) {
    match self {
      Signal::Constant(_) => (), // Do nothing
      Signal::Net(net) => net.name = name,
    }
  }

  /// Get name of signal.
  fn get_name(&self) -> &str {
    match self {
      Signal::Constant(_) => "const", // Do nothing
      Signal::Net(net) => &net.name,
    }
  }

  /// Get netlist index of signal.
  fn get_index(&self) -> usize {
    match self {
      Signal::Constant(_) => 0, // TODO: Improve constant handling
      Signal::Net(net) => net.index,
    }
  }

  /// Get value of signal.
  fn get_value(&self) -> Bit {
    match self {
      Signal::Constant(bit) => *bit,
      Signal::Net(net) => net.value,
    }
  }

  /// Set value of signal. Updates toggle statistics.
  fn set_value(&mut self, val: Bit) {
    match self {
      Signal::Constant(_) => (), // Do nothing
      Signal::Net(net) => {
        match &[net.value, val] {
          [Bit::Zero, Bit::One] => net.toggle_count_rising += 1,
          [Bit::One, Bit::Zero] => net.toggle_count_falling += 1,
          [Bit::Zero, Bit::Zero] | [Bit::One, Bit::One] => return,
        }
        net.value = val;
      }
    }
  }

  /// Get total signal toggle count (rising + falling).
  fn get_total_toggle_count(&self) -> usize {
    match self {
      Signal::Constant(_) => 0,
      Signal::Net(net) => net.toggle_count_falling + net.toggle_count_rising,
    }
  }

  /// Get total rising toggle count of signal.
  fn get_toggle_count_rising(&self) -> usize {
    match self {
      Signal::Constant(_) => 0,
      Signal::Net(net) => net.toggle_count_rising,
    }
  }

  /// Get total falling toggle count of signal.
  fn get_toggle_count_falling(&self) -> usize {
    match self {
      Signal::Constant(_) => 0,
      Signal::Net(net) => net.toggle_count_falling,
    }
  }
}

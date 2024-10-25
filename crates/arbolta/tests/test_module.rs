// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

use arbolta::bit::Bit;
use arbolta::cell::{Cell, Function};
use arbolta::module::hardware_module::{Component, HardwareModule};
use arbolta::module::port::{Port, PortDirection};
use arbolta::signal::Signal;
use once_cell::sync::Lazy;
use rstest::rstest;

static VARIABLE_ALPHABET: Lazy<Vec<String>> = Lazy::new(|| {
  (b'a'..=b'z')
    .map(|x| String::from_utf8(vec![x]).unwrap())
    .collect()
});

fn cell_module_from_function(function: Function, num_inputs: usize) -> HardwareModule {
  let mut cell_inputs_connections = [0; 8];
  let mut module = HardwareModule::default();

  for i in 0..num_inputs {
    module.signals.push(Signal::new_net(i));
    module.ports.insert(
      VARIABLE_ALPHABET[i].clone(),
      Port {
        signal_idx_list: vec![i],
        shape: [1, 1],
        direction: PortDirection::Input,
        signed: false,
      },
    );

    cell_inputs_connections[i] = i;
  }
  module.signals.push(Signal::new_net(num_inputs));
  module.ports.insert(
    VARIABLE_ALPHABET[num_inputs].clone(),
    Port {
      signal_idx_list: vec![num_inputs],
      shape: [1, 1],
      direction: PortDirection::Output,
      signed: false,
    },
  );

  module.components.push(Component::Cell(Cell {
    name: String::new(),
    function,
    state: [Bit::Zero; 2],
    input_connections: cell_inputs_connections,
    output_connection: num_inputs,
    num_inputs,
  }));

  module
}

#[rstest]
#[case(Function::Inverter, 0, 1)]
#[case(Function::Inverter, 1, 0)]
#[case(Function::Buf, 0, 0)]
#[case(Function::Buf, 1, 1)]
fn test_module_1_input_cell(#[case] function: Function, #[case] a: u8, #[case] expected: u8) {
  let mut cell_module = cell_module_from_function(function, 1);
  cell_module.set_port_int("a", a).unwrap();
  cell_module.eval();

  let actual: u8 = cell_module.get_port_int("b").unwrap();
  assert_eq!(actual, expected);
}

#[rstest]
#[case(Function::And, 0, 0, 0)]
#[case(Function::And, 0, 1, 0)]
#[case(Function::And, 1, 0, 0)]
#[case(Function::And, 1, 1, 1)]
#[case(Function::Nor, 0, 0, 1)]
#[case(Function::Nor, 0, 1, 0)]
#[case(Function::Nor, 1, 0, 0)]
#[case(Function::Nor, 1, 1, 0)]
#[case(Function::Nand, 0, 0, 1)]
#[case(Function::Nand, 0, 1, 1)]
#[case(Function::Nand, 1, 0, 1)]
#[case(Function::Nand, 1, 1, 0)]
#[case(Function::Or, 0, 0, 0)]
#[case(Function::Or, 0, 1, 1)]
#[case(Function::Or, 1, 0, 1)]
#[case(Function::Or, 1, 1, 1)]
#[case(Function::Xor, 0, 0, 0)]
#[case(Function::Xor, 0, 1, 1)]
#[case(Function::Xor, 1, 0, 1)]
#[case(Function::Xor, 1, 1, 0)]
#[case(Function::Xnor, 0, 0, 1)]
#[case(Function::Xnor, 0, 1, 0)]
#[case(Function::Xnor, 1, 0, 0)]
#[case(Function::Xnor, 1, 1, 1)]
fn test_module_2_input_cell(
  #[case] function: Function,
  #[case] a: u8,
  #[case] b: u8,
  #[case] expected: u8,
) {
  let mut cell_module = cell_module_from_function(function, 2);
  cell_module.set_port_int("a", a).unwrap();
  cell_module.set_port_int("b", b).unwrap();
  cell_module.eval();

  let actual: u8 = cell_module.get_port_int("c").unwrap();
  assert_eq!(actual, expected);
}

#[rstest]
#[case(Function::Or, 0, 0, 0, 0)]
#[case(Function::Or, 0, 0, 1, 1)]
#[case(Function::Or, 0, 1, 0, 1)]
#[case(Function::Or, 0, 1, 1, 1)]
#[case(Function::Or, 1, 0, 0, 1)]
#[case(Function::Or, 1, 0, 1, 1)]
#[case(Function::Or, 1, 1, 0, 1)]
#[case(Function::Or, 1, 1, 1, 1)]
fn test_module_3_input_cell(
  #[case] function: Function,
  #[case] a: u8,
  #[case] b: u8,
  #[case] c: u8,
  #[case] expected: u8,
) {
  let mut cell_module = cell_module_from_function(function, 3);
  cell_module.set_port_int("a", a).unwrap();
  cell_module.set_port_int("b", b).unwrap();
  cell_module.set_port_int("c", c).unwrap();
  cell_module.eval();

  let actual: u8 = cell_module.get_port_int("d").unwrap();
  assert_eq!(actual, expected);
}

#[rstest]
#[case(Function::DffPosEdge, 0, 0)]
#[case(Function::DffPosEdge, 1, 1)]
fn test_module_1_input_cell_clocked(
  #[case] function: Function,
  #[case] a: u8,
  #[case] expected: u8,
) {
  let mut cell_module = cell_module_from_function(function, 2);

  cell_module.set_port_int("a", 0).unwrap(); // clock
  cell_module.set_port_int("b", a).unwrap();
  cell_module.eval();

  cell_module.set_port_int("a", 1).unwrap();
  cell_module.eval();

  cell_module.set_port_int("a", 0).unwrap();
  cell_module.eval();

  let actual: u8 = cell_module.get_port_int("c").unwrap();
  assert_eq!(actual, expected);
}

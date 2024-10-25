// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

use arbolta::bit::Bit;
use arbolta::cell::{Cell, Function};
use arbolta::signal::{AccessSignal, Signal, SignalList};

use rstest::rstest;

#[rstest]
#[case(Function::Inverter, Bit::Zero, Bit::One)]
#[case(Function::Inverter, Bit::One, Bit::Zero)]
#[case(Function::Buf, Bit::Zero, Bit::Zero)]
#[case(Function::Buf, Bit::One, Bit::One)]
fn test_cell_1_input(#[case] function: Function, #[case] a: Bit, #[case] expected: Bit) {
  let mut cell = Cell::empty_from_function(function);
  let mut signals: SignalList = vec![Signal::new_constant(a), Signal::new_net(1)];
  cell.num_inputs = 1;
  cell.input_connections[0] = 0;
  cell.output_connection = 1;

  cell.eval(&mut signals);

  let actual = signals[1].get_value();
  assert_eq!(actual, expected);
}

#[rstest]
#[case(Function::And, Bit::Zero, Bit::Zero, Bit::Zero)]
#[case(Function::And, Bit::Zero, Bit::One, Bit::Zero)]
#[case(Function::And, Bit::One, Bit::Zero, Bit::Zero)]
#[case(Function::And, Bit::One, Bit::One, Bit::One)]
#[case(Function::Nor, Bit::Zero, Bit::Zero, Bit::One)]
#[case(Function::Nor, Bit::Zero, Bit::One, Bit::Zero)]
#[case(Function::Nor, Bit::One, Bit::Zero, Bit::Zero)]
#[case(Function::Nor, Bit::One, Bit::One, Bit::Zero)]
#[case(Function::Nand, Bit::Zero, Bit::Zero, Bit::One)]
#[case(Function::Nand, Bit::Zero, Bit::One, Bit::One)]
#[case(Function::Nand, Bit::One, Bit::Zero, Bit::One)]
#[case(Function::Nand, Bit::One, Bit::One, Bit::Zero)]
#[case(Function::Or, Bit::Zero, Bit::Zero, Bit::Zero)]
#[case(Function::Or, Bit::Zero, Bit::One, Bit::One)]
#[case(Function::Or, Bit::One, Bit::Zero, Bit::One)]
#[case(Function::Or, Bit::One, Bit::One, Bit::One)]
#[case(Function::Xor, Bit::Zero, Bit::Zero, Bit::Zero)]
#[case(Function::Xor, Bit::Zero, Bit::One, Bit::One)]
#[case(Function::Xor, Bit::One, Bit::Zero, Bit::One)]
#[case(Function::Xor, Bit::One, Bit::One, Bit::Zero)]
#[case(Function::Xnor, Bit::Zero, Bit::Zero, Bit::One)]
#[case(Function::Xnor, Bit::Zero, Bit::One, Bit::Zero)]
#[case(Function::Xnor, Bit::One, Bit::Zero, Bit::Zero)]
#[case(Function::Xnor, Bit::One, Bit::One, Bit::One)]
fn test_cell_2_input(
  #[case] function: Function,
  #[case] a: Bit,
  #[case] b: Bit,
  #[case] expected: Bit,
) {
  let mut cell = Cell::empty_from_function(function);
  let mut signals: SignalList = vec![
    Signal::new_constant(a),
    Signal::new_constant(b),
    Signal::new_net(2),
  ];

  cell.num_inputs = 2;
  cell.input_connections[0] = 0;
  cell.input_connections[1] = 1;
  cell.output_connection = 2;

  cell.eval(&mut signals);

  let actual = signals[2].get_value();
  assert_eq!(actual, expected);
}

#[rstest]
#[case(Function::Or, Bit::Zero, Bit::Zero, Bit::Zero, Bit::Zero)]
#[case(Function::Or, Bit::Zero, Bit::Zero, Bit::One, Bit::One)]
#[case(Function::Or, Bit::Zero, Bit::One, Bit::Zero, Bit::One)]
#[case(Function::Or, Bit::Zero, Bit::One, Bit::One, Bit::One)]
#[case(Function::Or, Bit::One, Bit::Zero, Bit::Zero, Bit::One)]
#[case(Function::Or, Bit::One, Bit::Zero, Bit::One, Bit::One)]
#[case(Function::Or, Bit::One, Bit::One, Bit::Zero, Bit::One)]
#[case(Function::Or, Bit::One, Bit::One, Bit::One, Bit::One)]
fn test_cell_3_input(
  #[case] function: Function,
  #[case] a: Bit,
  #[case] b: Bit,
  #[case] c: Bit,
  #[case] expected: Bit,
) {
  let mut cell = Cell::empty_from_function(function);
  let mut signals: SignalList = vec![
    Signal::new_constant(a),
    Signal::new_constant(b),
    Signal::new_constant(c),
    Signal::new_net(3),
  ];

  cell.num_inputs = 3;
  cell.input_connections[0] = 0;
  cell.input_connections[1] = 1;
  cell.input_connections[2] = 2;
  cell.output_connection = 3;

  cell.eval(&mut signals);

  let actual = signals[3].get_value();
  assert_eq!(actual, expected);
}

#[rstest]
#[case(Function::DffPosEdge, Bit::Zero, Bit::Zero)]
#[case(Function::DffPosEdge, Bit::One, Bit::One)]
fn test_cell_1_input_clocked(#[case] function: Function, #[case] a: Bit, #[case] expected: Bit) {
  let mut cell = Cell::empty_from_function(function);
  let mut signals: SignalList = vec![
    Signal::new_net(0),
    Signal::new_constant(a),
    Signal::new_net(2),
  ];

  cell.num_inputs = 2;
  cell.input_connections[0] = 0;
  cell.input_connections[1] = 1;
  cell.output_connection = 2;

  signals[0].set_value(Bit::Zero);
  cell.eval(&mut signals);
  signals[0].set_value(Bit::One);
  cell.eval(&mut signals);
  signals[0].set_value(Bit::Zero);
  cell.eval(&mut signals);

  let actual = signals[2].get_value();
  assert_eq!(actual, expected);
}

// TODO: Randomize input testing
// TODO: N-input gate tests

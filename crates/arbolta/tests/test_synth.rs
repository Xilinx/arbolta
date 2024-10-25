// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

use arbolta::cell::default_cell_library;
use arbolta::synth::netlist::Netlist;

static ADDER_RAW: &str = include_str!("test_netlists/4b_adder_netlist.json");
static NESTED_ADDER_RAW: &str = include_str!("test_netlists/4b_nested_adder_netlist.json");

#[test]
fn test_synth_4b_adder() {
  let netlist = Netlist::from_yosys_raw(ADDER_RAW.as_bytes()).unwrap();
  let mut adder_module = netlist
    .generate_module("adder", &default_cell_library())
    .unwrap();

  for a in 0..16_u8 {
    adder_module.set_port_int("op0_i", a).unwrap();
    for b in 0..16_u8 {
      adder_module.set_port_int("op1_i", b).unwrap();
      adder_module.eval();
      let actual_sum = adder_module.get_port_int::<u8>("sum_o").unwrap();
      let expected_sum = a + b;

      assert_eq!(actual_sum, expected_sum)
    }
  }
}

#[test]
fn test_synth_4b_nested_adder() {
  let netlist = Netlist::from_yosys_raw(NESTED_ADDER_RAW.as_bytes()).unwrap();
  let mut adder_module = netlist
    .generate_module("adder", &default_cell_library())
    .unwrap();

  for a in 0..16_u8 {
    adder_module.set_port_int("op0_i", a).unwrap();
    for b in 0..16_u8 {
      adder_module.set_port_int("op1_i", b).unwrap();
      adder_module.eval();
      let actual_sum = adder_module.get_port_int::<u8>("sum_o").unwrap();
      let expected_sum = a + b;

      assert_eq!(actual_sum, expected_sum)
    }
  }
}

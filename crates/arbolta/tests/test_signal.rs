// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

use arbolta::bit::Bit;
use arbolta::signal::{AccessSignal, Signal};

#[test]
fn test_signal_net_init() {
  let x = Signal::new_net(0);

  assert_eq!(x.get_value(), Bit::Zero);
  assert_eq!(x.get_toggle_count_falling(), 0);
  assert_eq!(x.get_toggle_count_rising(), 0);
  assert_eq!(x.get_total_toggle_count(), 0);
  assert_eq!(x.get_index(), 0);
}

#[test]
fn test_signal_net_set_value() {
  let mut x = Signal::new_net(0);

  assert_eq!(x.get_value(), Bit::Zero);
  x.set_value(Bit::One);
  assert_eq!(x.get_value(), Bit::One);
}

#[test]
fn test_signal_net_toggle_rising() {
  let mut x = Signal::new_net(0);

  assert_eq!(x.get_total_toggle_count(), 0);
  assert_eq!(x.get_toggle_count_falling(), 0);
  assert_eq!(x.get_toggle_count_rising(), 0);

  x.set_value(Bit::One);

  assert_eq!(x.get_total_toggle_count(), 1);
  assert_eq!(x.get_toggle_count_falling(), 0);
  assert_eq!(x.get_toggle_count_rising(), 1);
}

#[test]
fn test_signal_net_toggle_falling() {
  let mut x = Signal::new_net_from(0, Bit::One);

  assert_eq!(x.get_total_toggle_count(), 0);
  assert_eq!(x.get_toggle_count_falling(), 0);
  assert_eq!(x.get_toggle_count_rising(), 0);

  x.set_value(Bit::Zero);

  assert_eq!(x.get_total_toggle_count(), 1);
  assert_eq!(x.get_toggle_count_falling(), 1);
  assert_eq!(x.get_toggle_count_rising(), 0);
}

#[test]
fn test_signal_net_toggle_same_zero() {
  let mut x = Signal::new_net(0);

  assert_eq!(x.get_total_toggle_count(), 0);
  assert_eq!(x.get_toggle_count_falling(), 0);
  assert_eq!(x.get_toggle_count_rising(), 0);

  x.set_value(Bit::Zero);

  assert_eq!(x.get_total_toggle_count(), 0);
  assert_eq!(x.get_toggle_count_falling(), 0);
  assert_eq!(x.get_toggle_count_rising(), 0);
}

#[test]
fn test_signal_net_toggle_same_one() {
  let mut x = Signal::new_net_from(0, Bit::One);

  assert_eq!(x.get_total_toggle_count(), 0);
  assert_eq!(x.get_toggle_count_falling(), 0);
  assert_eq!(x.get_toggle_count_rising(), 0);

  x.set_value(Bit::One);

  assert_eq!(x.get_total_toggle_count(), 0);
  assert_eq!(x.get_toggle_count_falling(), 0);
  assert_eq!(x.get_toggle_count_rising(), 0);
}

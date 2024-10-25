// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

use arbolta::bit::Bit;
use std::str::FromStr;

use rstest::rstest;

#[rstest]
#[case("0", Bit::Zero)]
#[case("1", Bit::One)]
fn test_bit_from_str(#[case] val: String, #[case] expected: Bit) {
  assert_eq!(Bit::from_str(&val).unwrap(), expected);
}

#[rstest]
#[case(Bit::Zero, '0')]
#[case(Bit::One, '1')]
fn test_bit_to_char(#[case] bit: Bit, #[case] expected: char) {
  assert_eq!(<Bit as Into<char>>::into(bit), expected);
}

#[rstest]
#[case(false, Bit::Zero)]
#[case(true, Bit::One)]
fn test_bit_from_bool(#[case] val: bool, #[case] expected: Bit) {
  assert_eq!(Bit::from(val), expected);
}

#[rstest]
#[case(Bit::Zero, false)]
#[case(Bit::One, true)]
fn test_bit_to_bool(#[case] bit: Bit, #[case] expected: bool) {
  assert_eq!(<Bit as Into<bool>>::into(bit), expected);
}

#[rstest]
#[case(0, Bit::Zero)]
#[case(1, Bit::One)]
fn test_bit_from_int(#[case] val: usize, #[case] expected: Bit) {
  assert_eq!(Bit::from_int(val).unwrap(), expected);
}

#[test]
fn test_bit_not() {
  assert_eq!(!Bit::Zero, Bit::One);
  assert_eq!(!Bit::One, Bit::Zero);
}

#[test]
fn test_bit_and() {
  assert_eq!(Bit::Zero & Bit::Zero, Bit::Zero);
  assert_eq!(Bit::Zero & Bit::One, Bit::Zero);
  assert_eq!(Bit::One & Bit::Zero, Bit::Zero);
  assert_eq!(Bit::One & Bit::One, Bit::One);
}

#[test]
fn test_bit_or() {
  assert_eq!(Bit::Zero | Bit::Zero, Bit::Zero);
  assert_eq!(Bit::Zero | Bit::One, Bit::One);
  assert_eq!(Bit::One | Bit::Zero, Bit::One);
  assert_eq!(Bit::One | Bit::One, Bit::One);
}

#[test]
fn test_bit_xor() {
  assert_eq!(Bit::Zero ^ Bit::Zero, Bit::Zero);
  assert_eq!(Bit::Zero ^ Bit::One, Bit::One);
  assert_eq!(Bit::One ^ Bit::Zero, Bit::One);
  assert_eq!(Bit::One ^ Bit::One, Bit::Zero);
}

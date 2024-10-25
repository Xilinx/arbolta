// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

use arbolta::bit::{Bit, BitVec};
use ndarray::{array, Array1};

use rstest::rstest;

#[rstest] // TODO: Generate random bit patterns and check
#[case(vec![
  Bit::One,
  Bit::Zero,
  Bit::One,
  Bit::Zero,
  Bit::Zero,
  Bit::One,
  Bit::Zero,
  Bit::Zero,
], "00100101")]
fn test_bits_to_str(#[case] bits: Vec<Bit>, #[case] expected: String) {
  let bits = BitVec { bits: bits };
  assert_eq!(bits.to_string(), expected);
}

#[rstest]
#[case("00100101", vec![
  Bit::One,
  Bit::Zero,
  Bit::One,
  Bit::Zero,
  Bit::Zero,
  Bit::One,
  Bit::Zero,
  Bit::Zero,
]
)]
fn test_str_to_bits(#[case] val: String, #[case] expected: Vec<Bit>) {
  assert_eq!(BitVec::try_from(val.as_str()).unwrap().bits, expected)
}

#[rstest]
#[case(vec![
  false,
  false,
  true,
  false,
  false,
  true,
  false,
  true,
], vec![
  Bit::One,
  Bit::Zero,
  Bit::One,
  Bit::Zero,
  Bit::Zero,
  Bit::One,
  Bit::Zero,
  Bit::Zero,
]
)]
fn test_bools_to_bits(#[case] vals: Vec<bool>, #[case] expected: Vec<Bit>) {
  assert_eq!(BitVec::from(vals).bits, expected)
}

#[rstest]
#[case("0", u8::MIN)]
#[case("11111111", u8::MAX)]
#[case("1000110", 70)]
#[case("11001000", 200)]
#[case("11011", 27)]
#[case("1111011", 123)]
#[case("11011011", 219)]
#[case("1", 1)]
#[case("10101101", 173)]
#[case("1001110", 78)]
#[case("100", 4)]
#[case("1010010", 82)]
fn test_bits_to_u8(#[case] bits: BitVec, #[case] expected: u8) {
  let actual: u8 = bits.to_int();
  assert_eq!(actual, expected);
}

#[rstest]
#[case("0", u16::MIN)]
#[case("1111111111111111", u16::MAX)]
#[case("100010110001", 2225)]
#[case("1100111110101", 6645)]
#[case("100101100101100", 19244)]
#[case("1010111001000100", 44612)]
#[case("1011011010000011", 46723)]
#[case("1111100111101111", 63983)]
#[case("11000011011100", 12508)]
#[case("1101100101101", 6957)]
#[case("1001011010100", 4820)]
#[case("1001011000111111", 38463)]
fn test_bits_to_u16(#[case] bits: BitVec, #[case] expected: u16) {
  let actual: u16 = bits.to_int();
  assert_eq!(actual, expected);
}

#[rstest]
#[case("0", u32::MIN)]
#[case("11111111111111111111111111111111", u32::MAX)]
#[case("10001011010001001111000011100", 292068892)]
#[case("11010101110100110111110111100001", 3587407329)]
#[case("11110000000000100011110110010100", 4026678676)]
#[case("101000110001000100011001110010", 683951730)]
#[case("10000011001010000000100101101010", 2200439146)]
#[case("110000000010110110110101", 12594613)]
#[case("10111010111101000111101100110100", 3136584500)]
#[case("100010011000100100100100100000", 576866592)]
#[case("11101000100100101100100000111000", 3901933624)]
#[case("101101101110010001010000100101", 767104037)]
fn test_bits_to_u32(#[case] bits: BitVec, #[case] expected: u32) {
  let actual: u32 = bits.to_int();
  assert_eq!(actual, expected);
}

#[rstest]
#[case("0", u64::MIN)]
#[case(
  "1111111111111111111111111111111111111111111111111111111111111111",
  u64::MAX
)]
#[case(
  "1000110001111101011000010111000110010111101010000101000001111011",
  10123354677902332027
)]
#[case(
  "1010001001011101010010001100001101101001110010111000000101101",
  1462448426388647981
)]
#[case(
  "100010111111000110100000101100011000010111011010111010100001111",
  5042008862487311631
)]
#[case(
  "1100010110111110111111110111110011010010111100011001100110111110",
  14249107182626904510
)]
#[case(
  "1011100100111101111000111001000001000010011010001011100110000001",
  13348075079324973441
)]
#[case(
  "111001011101110111101100010010001110011111100100110001111011010",
  8281827401205441498
)]
#[case(
  "1011101110101010101111110100110111011100100101001010110010001101",
  13522831172267453581
)]
#[case(
  "1001110000101011110010101101100101011001110001110101111101110110",
  11253311128778268534
)]
#[case(
  "1010111100100000111101100010010000110100100101000101000110100001",
  12619356791253520801
)]
#[case(
  "1011001001011111010100111011001100111010110110110101111001000001",
  12853083890790391361
)]
fn test_bits_to_u64(#[case] bits: BitVec, #[case] expected: u64) {
  let actual: u64 = bits.to_int();
  assert_eq!(actual, expected);
}

#[rstest]
#[case("10000000", i8::MIN)]
#[case("01111111", i8::MAX)]
#[case("0100", 4)]
#[case("11111001",-7)]
#[case("0111111", 63)]
#[case("10000100",-124)]
#[case("10000",-16)]
#[case("011011", 27)]
#[case("10001",-15)]
#[case("11010010",-46)]
#[case("10011011",-101)]
#[case("01101010", 106)]
fn test_bits_to_i8(#[case] bits: BitVec, #[case] expected: i8) {
  let actual: i8 = bits.to_int();
  assert_eq!(actual, expected);
}

#[rstest]
#[case("1000000000000000", i16::MIN)]
#[case("0111111111111111", i16::MAX)]
#[case("0100100011110100", 18676)]
#[case("01000100011010", 4378)]
#[case("011100010011101", 14493)]
#[case("10110111110110",-4618)]
#[case("0110011111100011", 26595)]
#[case("0100110110100011", 19875)]
#[case("01100010111110", 6334)]
#[case("100100101001100",-14004)]
#[case("011011011011111", 14047)]
#[case("011110000001011", 15371)]
fn test_bits_to_i16(#[case] bits: BitVec, #[case] expected: i16) {
  let actual: i16 = bits.to_int();
  assert_eq!(actual, expected);
}

#[rstest]
#[case("10000000000000000000000000000000", i32::MIN)]
#[case("01111111111111111111111111111111", i32::MAX)]
#[case("01001110111111111001000000101100", 1325371436)]
#[case("00110101000010100101110111111001", 889871865)]
#[case("01001011000010011000101000110011", 1258916403)]
#[case("10000010110110000010011110010001", -2099763311)]
#[case("00111000011010111100100111100010", 946588130)]
#[case("01110010111100001101011010010000", 1928386192)]
#[case("00000111011010101000110010010111", 124423319)]
#[case("00100101111010010100101011010011", 636046035)]
#[case("00110111011101100001010011000001", 930485441)]
#[case("11001111010111010001111101110010", -815980686)]
fn test_bits_to_i32(#[case] bits: BitVec, #[case] expected: i32) {
  let actual: i32 = bits.to_int();
  assert_eq!(actual, expected);
}

#[rstest]
#[case(0, "00000000")]
#[case(255, "11111111")]
#[case(80, "01010000")]
#[case(178, "10110010")]
#[case(72, "01001000")]
#[case(123, "01111011")]
#[case(108, "01101100")]
#[case(84, "01010100")]
#[case(131, "10000011")]
#[case(181, "10110101")]
#[case(182, "10110110")]
#[case(171, "10101011")]
fn test_u8_to_bits(#[case] val: u8, #[case] expected: BitVec) {
  assert_eq!(BitVec::from_int(val).unwrap(), expected)
}

#[rstest]
#[case(u16::MIN, "0000000000000000")]
#[case(u16::MAX, "1111111111111111")]
#[case(31261, "0111101000011101")]
#[case(20632, "0101000010011000")]
#[case(24420, "0101111101100100")]
#[case(56791, "1101110111010111")]
#[case(51723, "1100101000001011")]
#[case(63801, "1111100100111001")]
#[case(59134, "1110011011111110")]
#[case(63868, "1111100101111100")]
#[case(39090, "1001100010110010")]
#[case(36192, "1000110101100000")]
fn test_u16_to_bits(#[case] val: u16, #[case] expected: BitVec) {
  assert_eq!(BitVec::from_int(val).unwrap(), expected)
}
// TODO: Test other data types

#[rstest] // Reversed element order for bits
#[case(&[124, 70], "0100011001111100")]
#[case(&[253, 43], "0010101111111101")]
#[case(&[114, 74], "0100101001110010")]
#[case(&[179, 61], "0011110110110011")]
#[case(&[27, 184], "1011100000011011")]
#[case(&[190, 97], "0110000110111110")]
#[case(&[205, 117], "0111010111001101")]
#[case(&[255, 111], "0110111111111111")]
#[case(&[253, 176], "1011000011111101")]
#[case(&[220, 231], "1110011111011100")]
fn test_u8_vec_to_bits(#[case] vals: &[u8], #[case] expected: BitVec) {
  assert_eq!(BitVec::from_ints(vals).unwrap(), expected);
}

#[rstest]
#[case(&[0, -114], "1000111000000000")]
#[case(&[-107, 89], "0101100110010101")]
#[case(&[59, -99], "1001110100111011")]
#[case(&[115, -117], "1000101101110011")]
#[case(&[-90, 87], "0101011110100110")]
#[case(&[-80, -49], "1100111110110000")]
#[case(&[-88, 51], "0011001110101000")]
#[case(&[-101, 62], "0011111010011011")]
#[case(&[15, -27], "1110010100001111")]
#[case(&[-58, -95], "1010000111000110")]
fn test_i8_vec_to_bits(#[case] vals: &[i8], #[case] expected: BitVec) {
  assert_eq!(BitVec::from_ints(vals).unwrap(), expected);
}

#[rstest]
#[case(&[30, -44], 7, "10101000011110")]
#[case(&[-19, -42], 7, "10101101101101")]
#[case(&[3, -4], 4, "11000011")]
#[case(&[1, -1], 2, "1101")]
fn test_i8_vec_to_bits_sized(
  #[case] vals: &[i8],
  #[case] elem_size: usize,
  #[case] expected: BitVec,
) {
  assert_eq!(BitVec::from_ints_sized(vals, elem_size).unwrap(), expected);
}

#[rstest]
#[case("0100011001111100", &[124, 70])]
#[case("0010101111111101", &[253, 43])]
#[case("0100101001110010", &[114, 74])]
#[case("0011110110110011", &[179, 61])]
#[case("1011100000011011", &[27, 184])]
#[case("0110000110111110", &[190, 97])]
#[case("0111010111001101", &[205, 117])]
#[case("0110111111111111", &[255, 111])]
#[case("1011000011111101", &[253, 176])]
#[case("1110011111011100", &[220, 231])]
fn test_bits_to_u8_vec(#[case] bits: BitVec, #[case] expected: &[u8]) {
  let actual: Vec<u8> = bits.to_ints();
  assert_eq!(actual, expected);
}

#[rstest]
#[case("0100011001111100", &[124, 70])]
#[case("0010101111111101", &[253, 43])]
#[case("0100101001110010", &[114, 74])]
#[case("0011110110110011", &[179, 61])]
#[case("1011100000011011", &[27, 184])]
#[case("0110000110111110", &[190, 97])]
#[case("0111010111001101", &[205, 117])]
#[case("0110111111111111", &[255, 111])]
#[case("1011000011111101", &[253, 176])]
#[case("1110011111011100", &[220, 231])]
fn test_bits_to_u8_vec_buffer(#[case] bits: BitVec, #[case] expected: &[u8]) {
  let mut buffer: Vec<u8> = vec![0; expected.len()];
  bits.to_ints_buffer(buffer.as_mut_slice());

  assert_eq!(buffer, expected);
}

#[rstest]
#[case("0100011001111100",  array![124, 70])]
#[case("0010101111111101", array![253, 43])]
#[case("0100101001110010", array![114, 74])]
#[case("0011110110110011", array![179, 61])]
#[case("1011100000011011", array![27, 184])]
#[case("0110000110111110", array![190, 97])]
#[case("0111010111001101", array![205, 117])]
#[case("0110111111111111", array![255, 111])]
#[case("1011000011111101", array![253, 176])]
#[case("1110011111011100", array![220, 231])]
fn test_bits_to_u8_ndarray(#[case] bits: BitVec, #[case] expected: Array1<u8>) {
  let actual: Array1<u8> = bits.to_int_ndarray();
  assert_eq!(actual, expected);
}

#[rstest]
#[case("0100011001111100",  array![124, 70])]
#[case("0010101111111101", array![253, 43])]
#[case("0100101001110010", array![114, 74])]
#[case("0011110110110011", array![179, 61])]
#[case("1011100000011011", array![27, 184])]
#[case("0110000110111110", array![190, 97])]
#[case("0111010111001101", array![205, 117])]
#[case("0110111111111111", array![255, 111])]
#[case("1011000011111101", array![253, 176])]
#[case("1110011111011100", array![220, 231])]
fn test_bits_to_u8_ndarray_buffer(#[case] bits: BitVec, #[case] expected: Array1<u8>) {
  let mut buffer: Array1<u8> = Array1::zeros([expected.len()]);
  bits.to_int_ndarray_buffer(buffer.view_mut()).unwrap();
  assert_eq!(buffer, expected);
}

#[rstest]
#[case("10101000011110", 7, &[30, -44])]
#[case("10101101101101", 7, &[-19, -42])]
#[case("11000011", 4, &[3, -4])]
#[case("1101", 2, &[1, -1])]
fn test_bits_sized_to_i8_vec(
  #[case] bits: BitVec,
  #[case] elem_size: usize,
  #[case] expected: &[i8],
) {
  let actual: Vec<i8> = bits.to_ints_sized(elem_size);
  assert_eq!(actual, expected);
}

#[rstest]
#[case("10101000011110", 7, &[30, -44])]
#[case("10101101101101", 7, &[-19, -42])]
#[case("11000011", 4, &[3, -4])]
#[case("1101", 2, &[1, -1])]
fn test_bits_sized_to_i8_vec_buffer(
  #[case] bits: BitVec,
  #[case] elem_size: usize,
  #[case] expected: &[i8],
) {
  let mut buffer: Vec<i8> = vec![0; expected.len()];
  bits.to_ints_sized_buffer(elem_size, buffer.as_mut_slice());
  assert_eq!(buffer, expected);
}

#[rstest]
#[case("10101000011110", 7, array![30, -44])]
#[case("10101101101101", 7, array![-19, -42])]
#[case("11000011", 4, array![3, -4])]
#[case("1101", 2, array![1, -1])]
fn test_bits_sized_to_i8_ndarray_buffer(
  #[case] bits: BitVec,
  #[case] elem_size: usize,
  #[case] expected: Array1<i8>,
) {
  let mut buffer: Array1<i8> = Array1::zeros([expected.len()]);
  bits
    .to_int_ndarray_sized_buffer(elem_size, buffer.view_mut())
    .unwrap();
  assert_eq!(buffer, expected);
}

#[rstest]
#[case("1000111000000000", &[0, -114])]
#[case("0101100110010101", &[-107, 89])]
#[case("1001110100111011", &[59, -99])]
#[case("1000101101110011", &[115, -117])]
#[case("0101011110100110", &[-90, 87])]
#[case("1100111110110000", &[-80, -49])]
#[case("0011001110101000", &[-88, 51])]
#[case("0011111010011011", &[-101, 62])]
#[case("1110010100001111", &[15, -27])]
#[case("1010000111000110", &[-58, -95])]
fn test_bits_to_i8_vec(#[case] bits: BitVec, #[case] expected: &[i8]) {
  let actual: Vec<i8> = bits.to_ints();
  assert_eq!(actual, expected);
}

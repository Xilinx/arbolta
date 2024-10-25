// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

module fma #(
  parameter  int unsigned ExpWidth                 = 4,
  parameter  int unsigned ManWidth                 = 3,
  parameter  int unsigned Size                     = 4,
  parameter  int unsigned GuardBits                = 3,
  localparam int unsigned FormatWidth              = 1 + ExpWidth + ManWidth,
  localparam int unsigned ManFixedWidth            = ManWidth + 1,
  localparam int unsigned SignedExpWidth           = ExpWidth + 2,
  // Fixed point mantissa product
  localparam int unsigned ManFixedProdWidth        = ManFixedWidth * 2,
  // Normalized fixed-point mantissa product
  localparam int unsigned NormManFixedProdWidth    = ManFixedProdWidth + GuardBits,
  // Stages for adding fixed-point mantissa products
  localparam int unsigned AdderStages              = $clog2(Size),
  // Normalized fixed-point mantissa product sum
  localparam int unsigned NormManFixedProdSumWidth = AdderStages + NormManFixedProdWidth
)(
  input  logic        [Size-1:0][FormatWidth-1:0]    op0_vec_i,
                                                     op1_vec_i,
  output logic                                       sign_o,
  output logic        [NormManFixedProdSumWidth-1:0] man_o,
  output logic signed [SignedExpWidth-1:0]           exp_o
);
  logic [Size-1:0] op0_sign_vec, op1_sign_vec;
  logic [Size-1:0][ManFixedWidth-1:0] op0_man_vec, op1_man_vec;
  logic signed [Size-1:0][SignedExpWidth-1:0] op0_exp_vec, op1_exp_vec;
  minifloat_unpack_signed_vector #(
    .ExpWidth ( ExpWidth ),
    .ManWidth ( ManWidth ),
    .Size     ( Size     )
  ) unpack_op0 (
    .op_vec_i   ( op0_vec_i    ),
    .sign_vec_o ( op0_sign_vec ),
    .man_vec_o  ( op0_man_vec  ),
    .exp_vec_o  ( op0_exp_vec  )
  );

  minifloat_unpack_signed_vector #(
    .ExpWidth ( ExpWidth ),
    .ManWidth ( ManWidth ),
    .Size     ( Size     )
  ) unpack_op1 (
    .op_vec_i   ( op1_vec_i    ),
    .sign_vec_o ( op1_sign_vec ),
    .man_vec_o  ( op1_man_vec  ),
    .exp_vec_o  ( op1_exp_vec  )
  );

  logic [Size-1:0] product_sign_vec;
  logic [Size-1:0][ManFixedProdWidth-1:0] product_mantissa_vec;
  logic signed [Size-1:0][SignedExpWidth-1:0] product_exponent_vec;
  minifloat_unpacked_signed_vector_multiplier #(
    .ManFixedWidth  ( ManFixedWidth  ),
    .SignedExpWidth ( SignedExpWidth ),
    .Size           ( Size           )
  ) multiplier (
    .op0_sign_vec_i ( op0_sign_vec            ),
    .op1_sign_vec_i ( op1_sign_vec            ),
    .op0_man_vec_i  ( op0_man_vec             ),
    .op1_man_vec_i  ( op1_man_vec             ),
    .op0_exp_vec_i  ( op0_exp_vec             ),
    .op1_exp_vec_i  ( op1_exp_vec             ),
    .sign_vec_o     ( product_sign_vec        ),
    .man_vec_o      ( product_mantissa_vec    ),
    .exp_vec_o      ( product_exponent_vec    )
  );

  logic [Size-1:0][NormManFixedProdWidth-1:0] aligned_product_mantissa_vec;
  logic signed [SignedExpWidth-1:0] max_product_exponent;
  align_max_mantissa_vector #(
    .ManFixedWidth  ( ManFixedProdWidth ),
    .SignedExpWidth ( SignedExpWidth    ),
    .Size           ( Size              ),
    .GuardBits      ( GuardBits         )
  ) align_product_mantissas (
    .man_vec_i ( product_mantissa_vec         ),
    .exp_vec_i ( product_exponent_vec         ),
    .man_vec_o ( aligned_product_mantissa_vec ),
    .max_exp_o ( max_product_exponent         )
  );

  logic [Size-1:0][NormManFixedProdWidth-1:0] neg_aligned_product_mantissa_vec,
                                              pos_aligned_product_mantissa_vec;
  split_sign_mag_vector #(
    .DataWidth ( NormManFixedProdWidth ),
    .Size      ( Size                  )
  ) split_aligned_product_mantissas (
    .sign_vec_i    ( prod_sign_vec_o                  ),
    .mag_vec_i     ( aligned_product_mantissa_vec     ),
    .neg_mag_vec_o ( neg_aligned_product_mantissa_vec ),
    .pos_mag_vec_o ( pos_aligned_product_mantissa_vec )
  );

  logic [NormManFixedProdSumWidth-1:0] neg_aligned_product_mantissa_sum,
                                       pos_aligned_product_mantissa_sum;
  int_vector_adder_tree #(
    .DataWidth ( NormManFixedProdWidth ),
    .Size      ( Size                  ),
    .Signed    ( 1'b0                  )
  ) neg_aligned_product_mantissa_vector_sum (
    .op_vec_i ( neg_aligned_product_mantissa_vec ),
    .sum_o    ( neg_aligned_product_mantissa_sum )
  );

  int_vector_adder_tree #(
    .DataWidth ( NormManFixedProdWidth ),
    .Size      ( Size                  ),
    .Signed    ( 1'b0                  )
  ) pos_aligned_product_mantissa_vector_sum (
    .op_vec_i ( pos_aligned_product_mantissa_vec ),
    .sum_o    ( pos_aligned_product_mantissa_sum )
  );

  logic aligned_product_mantissa_sum_sign;
  logic [NormManFixedProdSumWidth-1:0] aligned_product_mantissa_sum;
  sign_mag_adder #(
    .DataWidth ( NormManFixedProdSumWidth )
  ) combine_aligned_product_mantissa_sums (
    .neg_mag_i ( neg_aligned_product_mantissa_sum  ),
    .pos_mag_i ( pos_aligned_product_mantissa_sum  ),
    .sign_o    ( aligned_product_mantissa_sum_sign ),
    .sum_mag_o ( aligned_product_mantissa_sum      )
  );

  assign sign_o = aligned_product_mantissa_sum_sign;
  assign man_o = aligned_product_mantissa_sum;
  assign exp_o = max_product_exponent;

  // TODO: Add normalization here

endmodule

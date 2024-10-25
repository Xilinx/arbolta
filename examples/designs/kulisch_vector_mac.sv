// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

//  Contains following designs:
//    kulisch_mac_single_adder
//    kulisch_mac_dual_adder

module kulisch_mac_single_adder #(
  parameter  int unsigned ExpWidth          = 4,
  parameter  int unsigned ManWidth          = 3,
  parameter  int unsigned Size              = 64,
  parameter  int unsigned AccumulatorWidth  = 64,
  localparam int unsigned FormatWidth       = 1 + ExpWidth + ManWidth,
  localparam int unsigned ManProductWidth   = (ManWidth + 1) * 2,
  localparam int unsigned ExpSumWidth       = ExpWidth + 1,
  localparam int unsigned FixedWidth        = (1<<ExpWidth) + ManWidth,
  localparam int unsigned FixedProductWidth = 2 * FixedWidth,
  localparam int unsigned SumWidth          = $clog2(Size) + FixedProductWidth
)(
  input  logic                                    clock,
  input  logic                                    reset_i,
  input  logic        [Size-1:0][FormatWidth-1:0] op0_vec_i,
                                                  op1_vec_i,
  output logic signed [AccumulatorWidth-1:0]      mac_o
);
  logic [Size-1:0] product_sign_vec;
  logic [Size-1:0][ManProductWidth-1:0] mantissa_product_vec;
  logic [Size-1:0][ExpSumWidth-1:0] exponent_sum_vec;
  minifloat_vector_multiplier #(
    .ExpWidth ( ExpWidth ),
    .ManWidth ( ManWidth ),
    .Size     ( Size     )
  ) vector_multiplier (
    .op0_vec_i         ( op0_vec_i            ),
    .op1_vec_i         ( op1_vec_i            ),
    .sign_vec_o        ( product_sign_vec     ),
    .man_product_vec_o ( mantissa_product_vec ),
    .exp_sum_vec_o     ( exponent_sum_vec     )
  );

  logic [Size-1:0][FixedProductWidth-1:0] fixed_product_vec;
  unpacked_minifloat_to_fixed_point_vector #(
    .ExpWidth   ( ExpSumWidth       ),
    .ManWidth   ( ManProductWidth   ),
    .Size       ( Size              ),
    .FixedWidth ( FixedProductWidth )
  ) convert_fixed_point_vector (
    .man_vec_i   ( mantissa_product_vec ),
    .exp_vec_i   ( exponent_sum_vec     ),
    .fixed_vec_o ( fixed_product_vec    )
  );

  logic signed [Size-1:0][FixedProductWidth-1:0] product_vec;
  sign_mag_to_signed_int_vector #(
    .DataWidth ( FixedProductWidth ),
    .Size      ( Size              )
  ) product_vector_converter (
    .sign_vec_i ( product_sign_vec  ),
    .mag_vec_i  ( fixed_product_vec ),
    .int_vec_o  ( product_vec       )
  );

  logic signed [SumWidth-1:0] sum;
  int_vector_adder_tree #(
    .DataWidth ( FixedProductWidth ),
    .Size      ( Size              ),
    .Signed    ( 1'b1              )
  ) vector_sum (
    .op_vec_i ( product_vec ),
    .sum_o    ( sum         )
  );

  logic signed [AccumulatorWidth-1:0] combined_sum;
  int_adder #(
    .DataWidth ( AccumulatorWidth )
  ) adder (
    .op0_i ( sum ),
    .op1_i ( mac_o                  ),
    .sum_o ( combined_sum           )
  );

  clocked_register #(
    .DataWidth ( AccumulatorWidth )
  ) register (
    .clock   ( clock        ),
    .reset_i ( reset_i      ),
    .op_i    ( combined_sum ),
    .acc_o   ( mac_o        )
  );

endmodule


module kulisch_mac_dual_adder #(
  parameter  int unsigned ExpWidth          = 4,
  parameter  int unsigned ManWidth          = 3,
  parameter  int unsigned Size              = 64,
  parameter  int unsigned AccumulatorWidth  = 64,
  localparam int unsigned FormatWidth       = 1 + ExpWidth + ManWidth,
  localparam int unsigned ManProductWidth   = (ManWidth + 1) * 2,
  localparam int unsigned ExpSumWidth       = ExpWidth + 1,
  localparam int unsigned FixedWidth        = (1<<ExpWidth) + ManWidth,
  localparam int unsigned FixedProductWidth = 2 * FixedWidth,
  localparam int unsigned SumWidth          = $clog2(Size) + FixedProductWidth
)(
  input  logic                                    clock,
  input  logic                                    reset_i,
  input  logic        [Size-1:0][FormatWidth-1:0] op0_vec_i,
                                                  op1_vec_i,
  output logic signed [AccumulatorWidth-1:0]      mac_o
);
  logic [Size-1:0] product_sign_vec;
  logic [Size-1:0][ManProductWidth-1:0] mantissa_product_vec;
  logic [Size-1:0][ExpSumWidth-1:0] exponent_sum_vec;
  minifloat_vector_multiplier #(
    .ExpWidth ( ExpWidth ),
    .ManWidth ( ManWidth ),
    .Size     ( Size     )
  ) vector_multiplier (
    .op0_vec_i         ( op0_vec_i            ),
    .op1_vec_i         ( op1_vec_i            ),
    .sign_vec_o        ( product_sign_vec     ),
    .man_product_vec_o ( mantissa_product_vec ),
    .exp_sum_vec_o     ( exponent_sum_vec     )
  );

  logic [Size-1:0][FixedProductWidth-1:0] fixed_product_vec;
  unpacked_minifloat_to_fixed_point_vector #(
    .ExpWidth   ( ExpSumWidth       ),
    .ManWidth   ( ManProductWidth   ),
    .Size       ( Size              ),
    .FixedWidth ( FixedProductWidth )
  ) convert_fixed_point_vector (
    .man_vec_i   ( mantissa_product_vec ),
    .exp_vec_i   ( exponent_sum_vec     ),
    .fixed_vec_o ( fixed_product_vec    )
  );

  logic [Size-1:0][FixedProductWidth-1:0] neg_product_mag_vec, pos_product_mag_vec;
  split_sign_mag_vector #(
    .DataWidth ( FixedProductWidth ),
    .Size      ( Size              )
  ) product_splitter (
    .sign_vec_i    ( product_sign_vec    ),
    .mag_vec_i     ( fixed_product_vec   ),
    .neg_mag_vec_o ( neg_product_mag_vec ),
    .pos_mag_vec_o ( pos_product_mag_vec )
  );

  logic [SumWidth-1:0] negative_sum, positive_sum;
  int_vector_adder_tree #(
    .DataWidth ( FixedProductWidth ),
    .Size      ( Size              ),
    .Signed    ( 1'b0              )
  ) negative_vector_sum (
    .op_vec_i ( neg_product_mag_vec ),
    .sum_o    ( negative_sum        )
  );

  int_vector_adder_tree #(
    .DataWidth ( FixedProductWidth ),
    .Size      ( Size              ),
    .Signed    ( 1'b0              )
  ) positive_vector_sum (
    .op_vec_i ( pos_product_mag_vec ),
    .sum_o    ( positive_sum        )
  );

  logic signed [SumWidth-1:0] sum;
  int_subtractor #(
    .DataWidth( SumWidth )
  ) subtractor (
    .op0_i ( positive_sum ),
    .op1_i ( negative_sum ),
    .sum_o ( sum          )
  );

  logic signed [AccumulatorWidth-1:0] combined_sum;
  int_adder #(
    .DataWidth ( AccumulatorWidth )
  ) adder (
    .op0_i ( AccumulatorWidth'(sum) ),
    .op1_i ( mac_o                  ),
    .sum_o ( combined_sum           )
  );

  clocked_register #(
    .DataWidth ( AccumulatorWidth )
  ) register (
    .clock   ( clock        ),
    .reset_i ( reset_i      ),
    .op_i    ( combined_sum ),
    .acc_o   ( mac_o        )
  );

endmodule

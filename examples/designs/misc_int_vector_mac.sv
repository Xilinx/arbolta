// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

//  Contains following designs:
//    int_vector_mac_twos_mult_single_adder
//    int_vector_mac_twos_mult_dual_adder
//    int_vector_mac_sign_mag_mult_single_adder
//    int_vector_mac_sign_mag_mult_dual_adder


module int_vector_mac_twos_mult_single_adder #(
  parameter  int unsigned DataWidth        = 8,
  parameter  int unsigned Size             = 64,
  parameter  int unsigned AccumulatorWidth = 32,
  localparam int unsigned MultWidth        = DataWidth * 2,
  localparam int unsigned SumWidth         = $clog2(Size) + MultWidth
)(
  input  logic                                  clock,
  input  logic                                  reset_i,
  input  logic signed [Size-1:0][DataWidth-1:0] op0_vec_i,
                                                op1_vec_i,
  output logic signed [AccumulatorWidth-1:0]    mac_o
);
  logic signed [Size-1:0][MultWidth-1:0] product_vec;
  int_vector_multiplier #(
    .DataWidth ( DataWidth ),
    .Size      ( Size      ),
    .Signed    ( 1'b1      )
  ) vector_multiplier (
    .op0_vec_i  ( op0_vec_i   ),
    .op1_vec_i  ( op1_vec_i   ),
    .prod_vec_o ( product_vec )
  );

  logic signed [SumWidth-1:0] sum;
  int_vector_adder_tree #(
    .DataWidth ( MultWidth ),
    .Size      ( Size      ),
    .Signed    ( 1'b1      )
  ) vector_sum (
    .op_vec_i ( product_vec ),
    .sum_o    ( sum )
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


module int_vector_mac_twos_mult_dual_adder #(
  parameter  int unsigned DataWidth        = 8,
  parameter  int unsigned Size             = 64,
  parameter  int unsigned AccumulatorWidth = 32,
  localparam int unsigned MultWidth        = DataWidth * 2,
  localparam int unsigned SumWidth         = $clog2(Size) + MultWidth
)(
  input  logic                                  clock,
  input  logic                                  reset_i,
  input  logic signed [Size-1:0][DataWidth-1:0] op0_vec_i,
                                                op1_vec_i,
  output logic signed [AccumulatorWidth-1:0]    mac_o
);
  logic signed [Size-1:0][MultWidth-1:0] product_vec;
  int_vector_multiplier #(
    .DataWidth ( DataWidth ),
    .Size      ( Size      ),
    .Signed    ( 1'b1      )
  ) vector_multiplier (
    .op0_vec_i  ( op0_vec_i   ),
    .op1_vec_i  ( op1_vec_i   ),
    .prod_vec_o ( product_vec )
  );

  logic [Size-1:0] product_sign_vec;
  logic [Size-1:0][MultWidth-1:0] product_mag_vec;
  signed_int_to_sign_mag_vector #(
    .DataWidth ( MultWidth ),
    .Size      ( Size      )
  ) product_vector_converter (
    .op_vec_i   ( product_vec      ),
    .sign_vec_o ( product_sign_vec ),
    .mag_vec_o  ( product_mag_vec  )
  );

  logic [Size-1:0][MultWidth-1:0] neg_product_mag_vec, pos_product_mag_vec;
  split_sign_mag_vector #(
    .DataWidth ( MultWidth ),
    .Size      ( Size )
  ) product_splitter (
    .sign_vec_i    ( product_sign_vec ),
    .mag_vec_i     ( product_mag_vec ),
    .neg_mag_vec_o ( neg_product_mag_vec ),
    .pos_mag_vec_o ( pos_product_mag_vec )
  );

  logic [SumWidth-1:0] negative_sum, positive_sum;
  int_vector_adder_tree #(
    .DataWidth ( MultWidth ),
    .Size      ( Size      ),
    .Signed    ( 1'b0      )
  ) negative_vector_sum (
    .op_vec_i ( neg_product_mag_vec ),
    .sum_o    ( negative_sum        )
  );

  int_vector_adder_tree #(
    .DataWidth ( MultWidth ),
    .Size      ( Size      ),
    .Signed    ( 1'b0      )
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


module int_vector_mac_sign_mag_mult_single_adder #(
  parameter  int unsigned DataWidth        = 8,
  parameter  int unsigned Size             = 64,
  parameter  int unsigned AccumulatorWidth = 32,
  localparam int unsigned MultWidth        = DataWidth * 2,
  localparam int unsigned SumWidth         = $clog2(Size) + MultWidth
)(
  input  logic                                  clock,
  input  logic                                  reset_i,
  input  logic signed [Size-1:0][DataWidth-1:0] op0_vec_i,
                                                op1_vec_i,
  output logic signed [AccumulatorWidth-1:0]    mac_o
);
  logic [Size-1:0] op0_sign_vec, op1_sign_vec;
  logic [Size-1:0][DataWidth-1:0] op0_mag_vec, op1_mag_vec;

  signed_int_to_sign_mag_vector #(
    .DataWidth ( DataWidth ),
    .Size      ( Size      )
  ) op0_convert (
    .op_vec_i   ( op0_vec_i    ),
    .sign_vec_o ( op0_sign_vec ),
    .mag_vec_o  ( op0_mag_vec  )
  );

  signed_int_to_sign_mag_vector #(
    .DataWidth ( DataWidth ),
    .Size      ( Size      )
  ) op1_convert (
    .op_vec_i   ( op1_vec_i    ),
    .sign_vec_o ( op1_sign_vec ),
    .mag_vec_o  ( op1_mag_vec  )
  );

  logic [Size-1:0] product_sign_vec;
  logic [Size-1:0][MultWidth-1:0] product_mag_vec;
  sign_mag_vector_multiplier #(
    .DataWidth ( DataWidth ),
    .Size      ( Size      )
  ) vector_multiplier (
    .op0_sign_vec_i  ( op0_sign_vec     ),
    .op1_sign_vec_i  ( op1_sign_vec     ),
    .op0_mag_vec_i   ( op0_mag_vec      ),
    .op1_mag_vec_i   ( op1_mag_vec      ),
    .prod_sign_vec_o ( product_sign_vec ),
    .prod_mag_vec_o  ( product_mag_vec  )
  );

  logic signed [Size-1:0][MultWidth-1:0] product_vec;
  sign_mag_to_signed_int_vector #(
    .DataWidth ( MultWidth ),
    .Size      ( Size      )
  ) product_convert (
    .sign_vec_i ( product_sign_vec ),
    .mag_vec_i  ( product_mag_vec  ),
    .int_vec_o  ( product_vec      )
  );

  logic signed [SumWidth-1:0] sum;
  int_vector_adder_tree #(
    .DataWidth ( MultWidth ),
    .Size      ( Size      ),
    .Signed    ( 1'b1      )
  ) vector_sum (
    .op_vec_i ( product_vec ),
    .sum_o    ( sum )
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


module int_vector_mac_sign_mag_mult_dual_adder #(
  parameter  int unsigned DataWidth        = 8,
  parameter  int unsigned Size             = 64,
  parameter  int unsigned AccumulatorWidth = 32,
  localparam int unsigned MultWidth        = DataWidth * 2,
  localparam int unsigned SumWidth         = $clog2(Size) + MultWidth
)(
  input  logic                                  clock,
  input  logic                                  reset_i,
  input  logic signed [Size-1:0][DataWidth-1:0] op0_vec_i,
                                                op1_vec_i,
  output logic signed [AccumulatorWidth-1:0]    mac_o
);
  logic [Size-1:0] op0_sign_vec, op1_sign_vec;
  logic [Size-1:0][DataWidth-1:0] op0_mag_vec, op1_mag_vec;

  signed_int_to_sign_mag_vector #(
    .DataWidth ( DataWidth ),
    .Size      ( Size      )
  ) op0_convert (
    .op_vec_i   ( op0_vec_i    ),
    .sign_vec_o ( op0_sign_vec ),
    .mag_vec_o  ( op0_mag_vec  )
  );

  signed_int_to_sign_mag_vector #(
    .DataWidth ( DataWidth ),
    .Size      ( Size      )
  ) op1_convert (
    .op_vec_i   ( op1_vec_i    ),
    .sign_vec_o ( op1_sign_vec ),
    .mag_vec_o  ( op1_mag_vec  )
  );

  logic [Size-1:0] product_sign_vec;
  logic [Size-1:0][MultWidth-1:0] product_mag_vec;
  sign_mag_vector_multiplier #(
    .DataWidth ( DataWidth ),
    .Size      ( Size      )
  ) vector_multiplier (
    .op0_sign_vec_i  ( op0_sign_vec     ),
    .op1_sign_vec_i  ( op1_sign_vec     ),
    .op0_mag_vec_i   ( op0_mag_vec      ),
    .op1_mag_vec_i   ( op1_mag_vec      ),
    .prod_sign_vec_o ( product_sign_vec ),
    .prod_mag_vec_o  ( product_mag_vec  )
  );

  logic [Size-1:0][MultWidth-1:0] neg_product_mag_vec, pos_product_mag_vec;
  split_sign_mag_vector #(
    .DataWidth ( MultWidth ),
    .Size      ( Size )
  ) product_splitter (
    .sign_vec_i    ( product_sign_vec ),
    .mag_vec_i     ( product_mag_vec ),
    .neg_mag_vec_o ( neg_product_mag_vec ),
    .pos_mag_vec_o ( pos_product_mag_vec )
  );

  logic [SumWidth-1:0] negative_sum, positive_sum;
  int_vector_adder_tree #(
    .DataWidth ( MultWidth ),
    .Size      ( Size      ),
    .Signed    ( 1'b0      )
  ) negative_vector_sum (
    .op_vec_i ( neg_product_mag_vec ),
    .sum_o    ( negative_sum        )
  );

  int_vector_adder_tree #(
    .DataWidth ( MultWidth ),
    .Size      ( Size      ),
    .Signed    ( 1'b0      )
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

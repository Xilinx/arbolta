// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT


module int_vector_adder_tree #(
  parameter int unsigned  DataWidth,
  parameter int unsigned  Size,

  localparam int unsigned  SumWidth = $clog2(Size) + DataWidth
)(
  input  logic signed [Size-1:0][DataWidth-1:0] op_vec_i,
  output logic signed [SumWidth-1:0]            sum_o
);

  // Array of Tree Nodes (breadth-first indexing, root at zero)
  logic signed [SumWidth-1:0]  tree[2*Size-1];

  // Feed Leaves
  for(genvar  i = 0; i < Size; i++) begin : genLeaves
    assign  tree[Size-1 + i] = $signed(op_vec_i[i]);
  end : genLeaves

  // Reduce through Inner Nodes
  for(genvar  i = 0; i < Size-1; i++) begin : genInner
    localparam int unsigned  CHILD_WIDTH = SumWidth - $clog2(i+2);
    assign  tree[i] =
      $signed(tree[2*i+1][CHILD_WIDTH-1:0]) +
      $signed(tree[2*i+2][CHILD_WIDTH-1:0]);
  end : genInner

  // Root to Output
  assign  sum_o = tree[0];

endmodule : int_vector_adder_tree


module int_vector_multiplier #(
  parameter  int unsigned DataWidth,
  parameter  int unsigned Size,
  localparam int unsigned MultDataWidth = DataWidth * 2
)(
  input  logic signed [Size-1:0][DataWidth-1:0]     op0_vec_i,
                                                    op1_vec_i,
  output logic signed [Size-1:0][MultDataWidth-1:0] prod_vec_o
);
  always_comb begin
    for (int unsigned i = 0; i < Size; i++) begin
      prod_vec_o[i] = $signed(op0_vec_i[i]) * $signed(op1_vec_i[i]);
    end
  end
endmodule


module adder #(
  parameter int unsigned DataWidth = 8,
  parameter int unsigned SumWidth  = DataWidth + 1
)(
  input  logic signed [DataWidth-1:0] op0_i,
                                      op1_i,
  output logic signed [SumWidth-1:0]  sum_o
);
  assign sum_o = $signed(op0_i) + $signed(op1_i);
endmodule


module accumulator #(
  parameter int unsigned AccumulatorWidth = 32
)(
  input  logic            clock,
  input  logic            reset_i,
  input  logic signed [AccumulatorWidth-1:0] op_i,
  output logic signed [AccumulatorWidth-1:0] acc_o
);
  always_ff @(posedge clock) begin
    if (reset_i)
      acc_o <= '0;
    else
      acc_o <= $signed(op_i);
  end
endmodule


module int_vector_mac #(
  parameter  int unsigned DataWidth = 8,
  parameter  int unsigned Size = 16,
  parameter  int unsigned AccumulatorWidth = 32,
  // Derived
  localparam int unsigned MultDataWidth = DataWidth * 2,
  localparam int unsigned Stages        = $clog2(Size),
  localparam int unsigned SumWidth      = Stages + MultDataWidth
)(
  input  logic                                  clock,
  input  logic                                  reset_i,
  input  logic signed [Size-1:0][DataWidth-1:0] op0_vec_i,
                                                op1_vec_i,
  output logic signed [AccumulatorWidth-1:0]    mac_o
);
  logic signed [Size-1:0][MultDataWidth-1:0] product_vec;
  int_vector_multiplier #(
    .DataWidth ( DataWidth ),
    .Size      ( Size      )
  ) vector_multiplier (
    .op0_vec_i  ( op0_vec_i   ),
    .op1_vec_i  ( op1_vec_i   ),
    .prod_vec_o ( product_vec )
  );

  logic signed [SumWidth-1:0] sum;
  int_vector_adder_tree #(
    .DataWidth ( MultDataWidth ),
    .Size      ( Size          )
  ) vector_sum (
    .op_vec_i ( product_vec ),
    .sum_o    ( sum         )
  );

  logic signed [AccumulatorWidth-1:0] combined_sum;
  adder #(
    .DataWidth ( AccumulatorWidth ),
  ) add (
    .op0_i ( AccumulatorWidth'(sum) ),
    .op1_i ( mac_o                  ),
    .sum_o ( combined_sum           )
  );

  accumulator #(
    .AccumulatorWidth ( AccumulatorWidth )
  ) accumulate (
    .clock   ( clock        ),
    .reset_i ( reset_i      ),
    .op_i    ( combined_sum ),
    .acc_o   ( mac_o        )
  );

endmodule

// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

module clocked_register #(
  parameter int unsigned DataWidth = 32
)(
  input  logic                        clock,
  input  logic                        reset_i,
  input  logic signed [DataWidth-1:0] op_i,
  output logic signed [DataWidth-1:0] acc_o
);
  always_ff @(posedge clock) begin
    if (reset_i)
      acc_o <= '0;
    else
      acc_o <= $signed(op_i);
  end
endmodule

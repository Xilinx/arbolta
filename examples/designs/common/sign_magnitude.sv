// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

// ++++++++++++++++++++ Scalar ++++++++++++++++++++

module sign_mag_adder #(
  parameter int unsigned DataWidth = 8
)(
  input  logic [DataWidth-1:0] neg_mag_i,
                               pos_mag_i,
  output logic                 sign_o,
  output logic [DataWidth-1:0] sum_mag_o
);
  always_comb begin
    if (pos_mag_i >  neg_mag_i) begin
      sign_o = 1'b0;
      sum_mag_o = pos_mag_i - neg_mag_i;
    end else begin
      sign_o = 1'b1;
      sum_mag_o = neg_mag_i - pos_mag_i;
    end
  end
endmodule

// -------------------- Scalar --------------------

// ++++++++++++++++++++ Vector ++++++++++++++++++++

module signed_int_to_sign_mag_vector #(
  parameter int unsigned DataWidth = 8,
  parameter int unsigned Size      = 64
)(
  input  logic signed [Size-1:0][DataWidth-1:0] op_vec_i,
  output logic        [Size-1:0]                sign_vec_o,
  // + 1 bit for max negative -2^(DataWidth-1)
  output logic        [Size-1:0][DataWidth-1:0] mag_vec_o
);
  always_comb begin
    for (int unsigned i = 0; i < Size; i++) begin: gen_convertors
      sign_vec_o[i] = op_vec_i[i][DataWidth-1];
      mag_vec_o[i] = sign_vec_o[i] ?
          ~op_vec_i[i] + DataWidth'(1) : // Negative
          op_vec_i[i];                   // Positive
    end
  end
endmodule


module split_sign_mag_vector #(
  parameter int unsigned DataWidth = 8,
  parameter int unsigned Size      = 64
)(
  input  logic [Size-1:0]                sign_vec_i,
  input  logic [Size-1:0][DataWidth-1:0] mag_vec_i,
  output logic [Size-1:0][DataWidth-1:0] neg_mag_vec_o,
  output logic [Size-1:0][DataWidth-1:0] pos_mag_vec_o
);
  always_comb begin
    for (int i = 0; i < Size; i++) begin
      neg_mag_vec_o[i] = sign_vec_i[i] ? mag_vec_i[i] : '0;
      pos_mag_vec_o[i] = sign_vec_i[i] ? '0 : mag_vec_i[i];
    end
  end
endmodule


module sign_mag_vector_multiplier #(
  parameter  int unsigned DataWidth  = 8,
  parameter  int unsigned Size       = 64,
  localparam int unsigned MultWidth = DataWidth * 2
)(
  input  logic [Size-1:0]                op0_sign_vec_i,
                                         op1_sign_vec_i,
  input  logic [Size-1:0][DataWidth-1:0] op0_mag_vec_i,
                                         op1_mag_vec_i,
  output logic [Size-1:0]                prod_sign_vec_o,
  output logic [Size-1:0][MultWidth-1:0] prod_mag_vec_o
);
  always_comb begin
    for (int unsigned i = 0; i < Size; i++) begin
      prod_sign_vec_o[i] = op0_sign_vec_i[i] ^ op1_sign_vec_i[i];
      prod_mag_vec_o[i] = op0_mag_vec_i[i] * op1_mag_vec_i[i];
    end
  end
endmodule


module sign_mag_to_signed_int_vector #(
  parameter int unsigned DataWidth = 8,
  parameter int unsigned Size      = 64
)(
  input  logic        [Size-1:0]                sign_vec_i,
  input  logic        [Size-1:0][DataWidth-1:0] mag_vec_i,
  output logic signed [Size-1:0][DataWidth-1:0] int_vec_o
);
  always_comb begin
    for (int unsigned i = 0; i < Size; i++) begin
      int_vec_o[i] = sign_vec_i[i] ?
          ~mag_vec_i[i] + ~DataWidth'(1) : // Negative
          mag_vec_i[i];                    // Positive
    end
  end
endmodule

// -------------------- Vector --------------------

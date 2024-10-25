// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

// ++++++++++++++++++++ Scalar ++++++++++++++++++++

module minifloat_multiplier #(
  parameter  int unsigned ExpWidth        = 4,
  parameter  int unsigned ManWidth        = 3,
  localparam int unsigned FormatWidth     = 1 + ExpWidth + ManWidth, // +1 sign
  localparam int unsigned ManProductWidth = (ManWidth + 1) * 2, // +1 implicit
  localparam int unsigned ExpSumWidth     = ExpWidth + 1
)(
  input  logic [FormatWidth-1:0]     op0_i,
                                     op1_i,
  output logic                       sign_o,
  output logic [ManProductWidth-1:0] man_product_o,
  output logic [ExpSumWidth-1:0]     exp_sum_o
);
  // Separate fields
  logic op0_sign, op1_sign;
  logic [ExpWidth-1:0] op0_exponent, op1_exponent;
  logic [ManWidth-1:0] op0_mantissa, op1_mantissa;

  logic op0_normalized, op1_normalized;
  always_comb begin
    {op0_sign, op0_exponent, op0_mantissa} = op0_i;
    {op1_sign, op1_exponent, op1_mantissa} = op1_i;
    {op0_normalized, op1_normalized} = {|op0_exponent, |op1_exponent};

    sign_o = op0_sign ^ op1_sign;
    man_product_o = {op0_normalized, op0_mantissa} *
                    {op1_normalized, op1_mantissa};
    exp_sum_o = $unsigned({op0_exponent}) +
                $unsigned({op1_exponent}) -
                $unsigned(ExpWidth'(op0_normalized)) -
                $unsigned(ExpWidth'(op1_normalized));
  end
endmodule


module minifloat_unpack_signed #(
  parameter  int unsigned ExpWidth       = 4,
  parameter  int unsigned ManWidth       = 3,
  parameter  int unsigned Bias           = 2**(ExpWidth-1) - 1,
  parameter  int signed   Emin           = 1 - Bias,
  localparam int unsigned FormatWidth    = 1 + ExpWidth + ManWidth,
  localparam int unsigned ManFixedWidth  = ManWidth + 1,
  localparam int unsigned SignedExpWidth = ExpWidth + 2
)(
  input  logic        [FormatWidth-1:0]    op_i,
  output logic                             sign_o,
  output logic        [ManFixedWidth-1:0]  man_o,
  output logic signed [SignedExpWidth-1:0] exp_o
);
  logic normalized;
  logic [ExpWidth-1:0] exponent;
  logic [ManWidth-1:0] mantissa;

  always_comb begin
    {sign_o, exponent, mantissa} = op_i;
    normalized = |exponent;

    man_o = {normalized, mantissa};

    if (!normalized) begin // Denorm
      exp_o = mantissa == '0 ? '0 : SignedExpWidth'(Emin);
    end else begin
      exp_o = $signed(SignedExpWidth'({'0, exponent})) -
              $signed(SignedExpWidth'(Bias));
    end
  end
endmodule

// -------------------- Scalar --------------------

// ++++++++++++++++++++ Vector Multipliers ++++++++++++++++++++

module minifloat_vector_multiplier #(
  parameter  int unsigned ExpWidth        = 4,
  parameter  int unsigned ManWidth        = 3,
  parameter  int unsigned Size            = 64,
  localparam int unsigned FormatWidth     = 1 + ExpWidth + ManWidth, // +1 sign
  localparam int unsigned ManProductWidth = (ManWidth + 1) * 2,
  localparam int unsigned ExpSumWidth     = ExpWidth + 1
)(
  input  logic [Size-1:0][FormatWidth-1:0]     op0_vec_i,
                                               op1_vec_i,
  output logic [Size-1:0]                      sign_vec_o,
  output logic [Size-1:0][ManProductWidth-1:0] man_product_vec_o,
  output logic [Size-1:0][ExpSumWidth-1:0]     exp_sum_vec_o
);
  generate
    for (genvar i = 0; i < Size; i++) begin: gen_mul
      minifloat_multiplier #(
        .ExpWidth ( ExpWidth ),
        .ManWidth ( ManWidth )
      ) multiplier (
        .op0_i         ( op0_vec_i[i]         ),
        .op1_i         ( op1_vec_i[i]         ),
        .sign_o        ( sign_vec_o[i]        ),
        .man_product_o ( man_product_vec_o[i] ),
        .exp_sum_o     ( exp_sum_vec_o[i]     )
      );
    end
  endgenerate
endmodule


module minifloat_unpacked_signed_vector_multiplier #(
  parameter  int unsigned ManFixedWidth     = 4 + 3,
  parameter  int unsigned SignedExpWidth    = 3 + 1,
  parameter  int unsigned Size              = 4,
  localparam int unsigned ManFixedProdWidth = ManFixedWidth * 2
)(
  input  logic        [Size-1:0]                        op0_sign_vec_i,
                                                        op1_sign_vec_i,
  input  logic        [Size-1:0][ManFixedWidth-1:0]     op0_man_vec_i,
                                                        op1_man_vec_i,
  input  logic signed [Size-1:0][SignedExpWidth-1:0]    op0_exp_vec_i,
                                                        op1_exp_vec_i,
  output logic        [Size-1:0]                        sign_vec_o,
  output logic        [Size-1:0][ManFixedProdWidth-1:0] man_vec_o,
  output logic signed [Size-1:0][SignedExpWidth-1:0]    exp_vec_o
);
  always_comb begin
    for (int unsigned i = 0; i < Size; i++) begin
      sign_vec_o[i] = op0_sign_vec_i[i] ^ op1_sign_vec_i[i];
      man_vec_o[i] = ManFixedProdWidth'(op0_man_vec_i[i]) *
                     ManFixedProdWidth'(op1_man_vec_i[i]);
      exp_vec_o[i] = $signed(op0_exp_vec_i[i]) +
                     $signed(op1_exp_vec_i[i]);
    end
  end
endmodule

// -------------------- Vector Multipliers --------------------

// ++++++++++++++++++++ Vector Conversion ++++++++++++++++++++

module unpacked_minifloat_to_fixed_point_vector #(
  parameter int unsigned ExpWidth   = 5,
  parameter int unsigned ManWidth   = 8,
  parameter int unsigned Size       = 64,
  parameter int unsigned FixedWidth = 2 * ((1<<ExpWidth) + ManWidth-1)
)(
  input  logic [Size-1:0][ManWidth-1:0]   man_vec_i,
  input  logic [Size-1:0][ExpWidth-1:0]   exp_vec_i,
  output logic [Size-1:0][FixedWidth-1:0] fixed_vec_o
);
  always_comb begin
    for (int unsigned i = 0; i < Size; i++) begin
      fixed_vec_o[i] = man_vec_i[i] << exp_vec_i[i];
    end
  end
endmodule


module minifloat_unpack_signed_vector #(
  parameter  int unsigned ExpWidth       = 4,
  parameter  int unsigned ManWidth       = 3,
  parameter  int unsigned Size           = 3,
  parameter  int unsigned Bias           = 2**(ExpWidth-1) - 1,
  parameter  int signed   Emin           = 1 - Bias,
  localparam int unsigned FormatWidth    = 1 + ExpWidth + ManWidth,
  localparam int unsigned ManFixedWidth  = ManWidth + 1,
  localparam int unsigned SignedExpWidth = ExpWidth + 2
)(
  input  logic        [Size-1:0][FormatWidth-1:0]    op_vec_i,
  output logic        [Size-1:0]                     sign_vec_o,
  output logic        [Size-1:0][ManFixedWidth-1:0]  man_vec_o,
  output logic signed [Size-1:0][SignedExpWidth-1:0] exp_vec_o
);
  generate
    for (genvar i = 0; i < Size; i++) begin: gen_unpack
      minifloat_unpack_signed #(
        .ExpWidth ( ExpWidth ),
        .ManWidth ( ManWidth ),
        .Bias     ( Bias     ),
        .Emin     ( Emin     )
      ) unpack (
        .op_i   ( op_vec_i[i]   ),
        .sign_o ( sign_vec_o[i] ),
        .man_o  ( man_vec_o[i]  ),
        .exp_o  ( exp_vec_o[i]  )
      );
    end
  endgenerate
endmodule

// -------------------- Vector Conversion --------------------

// ++++++++++++++++++++ Vector Normalization ++++++++++++++++++++

module align_max_mantissa_vector #(
  parameter  int unsigned ManFixedWidth     = 6,
  parameter  int unsigned SignedExpWidth    = 5,
  parameter  int unsigned Size              = 4,
  parameter  int unsigned GuardBits         = 3,
  localparam int unsigned NormManFixedWidth = ManFixedWidth + GuardBits
)(
  input  logic        [Size-1:0][ManFixedWidth-1:0]     man_vec_i,
  input  logic signed [Size-1:0][SignedExpWidth-1:0]    exp_vec_i,
  output logic        [Size-1:0][NormManFixedWidth-1:0] man_vec_o,
  output logic signed [SignedExpWidth-1:0]              max_exp_o
);
  int_vector_max #(
    .DataWidth ( SignedExpWidth ),
    .Size      ( Size           )
  ) exponent_max (
    .op_vec_i ( exp_vec_i ),
    .max_o    ( max_exp_o )
  );

  always_comb begin
    for (int unsigned i = 0; i < Size; i++) begin
      man_vec_o[i] = NormManFixedWidth'({man_vec_i[i], GuardBits'(0)} >>
          (max_exp_o - exp_vec_i[i]));
    end
  end
endmodule

// -------------------- Vector Normalization --------------------

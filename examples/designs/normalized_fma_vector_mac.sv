// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

module mag_sum #(
  parameter  int unsigned DataWidth = 8
)(
  input  logic [DataWidth-1:0] positive_mag_i, negative_mag_i,
  output logic                 sign_o,
  output logic [DataWidth-1:0] sum_mag_o
);
  // assign sign_o = (negative_mag_i > positive_mag_i);
  // always_comb begin
  //   if (sign_o) // negative or 0
  //     sum_mag_o = negative_mag_i - positive_mag_i;
  //   else // positive
  //     sum_mag_o = positive_mag_i - negative_mag_i;
  // end
  always_comb begin
    if (positive_mag_i >  negative_mag_i) begin
      sign_o = 1'b0;
      sum_mag_o = positive_mag_i - negative_mag_i;
    end else begin
      sign_o = 1'b1;
      sum_mag_o = negative_mag_i - positive_mag_i;
    end
  end
endmodule

module minifloat_multiplier2 #(
  parameter  int unsigned SignedExponentWidth  = 4 + 2,
  parameter  int unsigned FullMantissaWidth    = 3 + 2,
  localparam int unsigned MantissaProductWidth = (FullMantissaWidth) * 2
)(
  input  logic                                   op0_sign_i, op1_sign_i,
  input  logic        [FullMantissaWidth-1:0]    op0_mantissa_i, op1_mantissa_i,
  input  logic signed [SignedExponentWidth-1:0]  op0_exponent_i, op1_exponent_i,
  output logic                                   sign_o,
  output logic        [MantissaProductWidth-1:0] mantissa_product_o,
  output logic signed [SignedExponentWidth-1:0]  exponent_sum_o
);
  always_comb begin
    sign_o = op0_sign_i ^ op1_sign_i;
    mantissa_product_o = MantissaProductWidth'({op0_mantissa_i}) *
                         MantissaProductWidth'({op1_mantissa_i});
    exponent_sum_o = $signed(op0_exponent_i) + $signed(op1_exponent_i);
  end
endmodule


module norm_normalize #(
  parameter int unsigned SignedExponentWidth  = 5,
  parameter int unsigned MantissaProductWidth = 3 + 2
)(
  input  logic                                   select,
  input  logic        [MantissaProductWidth-1:0] mantissa_i,
  input  logic signed [SignedExponentWidth-1:0]  exponent_i,
  output logic        [MantissaProductWidth-1:0] mantissa_o,
  output logic signed [SignedExponentWidth-1:0]  exponent_o
);
  always_comb begin
    if (!select) begin
      mantissa_o = '0;
      exponent_o = '0;
    end else begin
      if (mantissa_i[MantissaProductWidth-1] == 1'b1) begin
        // Need to right shift by 1, calc sticky bit
        mantissa_o = {1'b0, mantissa_i[MantissaProductWidth-1:2],
            |mantissa_i[1:0]};
        exponent_o = $signed(exponent_i) + SignedExponentWidth'(1);
      end else begin
        // Already normalized
        mantissa_o = mantissa_i;
        exponent_o = exponent_i;
      end
    end
  end
endmodule


module denorm_normalize_layer #(
  parameter int unsigned SignedExponentWidth  = 5,
  parameter int unsigned MantissaProductWidth = 3 + 2,
  parameter int signed   Emin                 = -6,
  parameter int signed   Emax                 = 8
)(
  input  logic        [MantissaProductWidth-1:0] mantissa_i,
  input  logic signed [SignedExponentWidth-1:0]  exponent_i,
  output logic        [MantissaProductWidth-1:0] mantissa_o,
  output logic signed [SignedExponentWidth-1:0]  exponent_o
);
  always_comb begin
    if ((mantissa_i == '0) && (exponent_i == '0)) begin // zero
      mantissa_o = mantissa_i;
      exponent_o = exponent_i;
    end else if ($signed(exponent_i) < $signed(Emin)) begin
      if ($signed(exponent_i) == $signed(Emin)-1) begin // right shift 1
        mantissa_o = mantissa_i >> 1;
        exponent_o = $signed(exponent_i) + SignedExponentWidth'(1);
      end else begin // too small, flush to 0
        mantissa_o = '0;
        exponent_o = '0;
      end
    end else if ($signed(exponent_i) == $signed(Emin)) begin // already denormed
      mantissa_o = mantissa_i;
      exponent_o = exponent_i;
    end else if ($signed(exponent_i) > $signed(Emax)) begin
      // do nothing and convert to inf later
      mantissa_o = mantissa_i;
      exponent_o = exponent_i;
    end else if (mantissa_i[MantissaProductWidth-2] == 1'b1) begin
      // normalized
      mantissa_o = mantissa_i;
      exponent_o = exponent_i;
    end else begin
      mantissa_o = mantissa_i << 1;
      exponent_o = $signed(exponent_i) - SignedExponentWidth'(1);
    end
  end
endmodule


module denorm_normalize #(
  parameter int unsigned SignedExponentWidth  = 5,
  parameter int unsigned MantissaProductWidth = 3 + 2,
  parameter int unsigned Size                 = 4,
  parameter int signed   Emin                 = -6,
  parameter int signed   Emax                 = 8
)(
  input  logic                                   select,
  input  logic        [MantissaProductWidth-1:0] mantissa_i,
  input  logic signed [SignedExponentWidth-1:0]  exponent_i,
  output logic        [MantissaProductWidth-1:0] mantissa_o,
  output logic signed [SignedExponentWidth-1:0]  exponent_o
);
  logic        [Size-1:0][MantissaProductWidth-1:0] last_mantissa;
  logic signed [Size-1:0][SignedExponentWidth-1:0]  last_exponent;

  generate
    for (genvar i = 0; i < Size; i++) begin
      if (i == 0) begin
        assign last_mantissa[i] = mantissa_i;
        assign last_exponent[i] = exponent_i;
      end else begin
        denorm_normalize_layer #(
          .SignedExponentWidth(SignedExponentWidth),
          .MantissaProductWidth(MantissaProductWidth),
          .Emin(Emin),
          .Emax(Emax)
        ) denorm (
          .mantissa_i(last_mantissa[i-1]),
          .exponent_i(last_exponent[i-1]),
          .mantissa_o(last_mantissa[i]),
          .exponent_o(last_exponent[i]),
        );
      end
    end
  endgenerate

  always_comb begin
    if (!select) begin
      mantissa_o = '0;
      exponent_o = '0;
    end else begin
      mantissa_o = last_mantissa[Size-1];
      exponent_o = last_exponent[Size-1];
    end
  end
endmodule


module minifloat_normalize #(
  parameter int unsigned SignedExponentWidth  = 5,
  parameter int unsigned MantissaProductWidth = 3 + 2,
  parameter int unsigned DenormSize           = 4,
  parameter int signed   Emin                 = -6,
  parameter int signed   Emax                 = 8
)(
  input  logic        [MantissaProductWidth-1:0] mantissa_i,
  input  logic signed [SignedExponentWidth-1:0]  exponent_i,
  output logic        [MantissaProductWidth-1:0] mantissa_o,
  output logic signed [SignedExponentWidth-1:0]  exponent_o
);

  logic norm_select;
  logic denorm_select;

  logic        [MantissaProductWidth-1:0] norm_mantissa, denorm_mantissa;
  logic signed [SignedExponentWidth-1:0]  norm_exponent, denorm_exponent;

  norm_normalize #(
    .SignedExponentWidth(SignedExponentWidth),
    .MantissaProductWidth(MantissaProductWidth),
  ) norm (
    .select(norm_select),
    .mantissa_i(mantissa_i),
    .exponent_i(exponent_i),
    .mantissa_o(norm_mantissa),
    .exponent_o(norm_exponent)
  );

  denorm_normalize #(
    .SignedExponentWidth(SignedExponentWidth),
    .MantissaProductWidth(MantissaProductWidth),
    .Size(DenormSize),
    .Emin(Emin),
    .Emax(Emax),
  ) denorm (
    .select(denorm_select),
    .mantissa_i(mantissa_i),
    .exponent_i(exponent_i),
    .mantissa_o(denorm_mantissa),
    .exponent_o(denorm_exponent)
  );
  assign denorm_select = !norm_select;
  always_comb begin
    if (((mantissa_i[MantissaProductWidth-1] == 1'b1) ||
         (mantissa_i[MantissaProductWidth-2] == 1'b1)) &&
        ($signed(exponent_i) >= $signed(Emin)) &&
        ($signed(exponent_i) <= $signed(Emax))) begin
      norm_select = 1'b1;
      mantissa_o = norm_mantissa;
      exponent_o = norm_exponent;
    end else begin
      // denorm
      norm_select = 1'b0;
      mantissa_o = denorm_mantissa;
      exponent_o = denorm_exponent;
    end
  end
endmodule


module export_minifloat #(
  parameter  int unsigned ExponentWidth        = 4,
  parameter  int unsigned MantissaWidth        = 3,
  parameter  int unsigned Bias = $pow(2, (ExponentWidth - 1)) - 1,
  parameter  int signed   Emin = 1 - Bias,
  parameter  int signed   Emax = $pow(2, ExponentWidth) - 1 - Bias,
  parameter  int unsigned MantissaProductWidth = 8,
  localparam int unsigned FormatWidth = 1 + ExponentWidth + MantissaWidth,
  localparam int unsigned SignedExponentWidth  = ExponentWidth + 2
)(
  input  logic                                   sign_i,
  input  logic        [MantissaProductWidth-1:0] mantissa_i,
  input  logic signed [SignedExponentWidth-1:0]  exponent_i,
  output logic        [FormatWidth-1:0]          format_o
);
  logic format_sign;
  logic [ExponentWidth-1:0] format_exponent;
  logic [MantissaWidth-1:0] format_mantissa;
  always_comb begin
    if (($signed(exponent_i) == '0) && mantissa_i == '0) begin
      format_sign     = 1'b0; // no negative 0
      format_exponent = '0;
      format_mantissa = '0;
    end else if (($signed(exponent_i) > $signed(Emax))) begin // infinity
      format_sign     = sign_i;
      format_exponent = ~0;
      format_mantissa = '0;
    end else begin
      // round? check sig bit to denorm (+ 1 exp)
      format_sign     = sign_i;
      format_exponent = $signed(exponent_i) + $signed(Bias);
      format_mantissa = {mantissa_i[MantissaProductWidth-3:0], {MantissaWidth-MantissaProductWidth{1'b0}}};
    end

    format_o = {format_sign, format_exponent, format_mantissa};
  end

endmodule


module int_pair_max2 #(
  parameter int unsigned DataWidth = 16
)(
  input  logic [DataWidth-1:0] op0_i, op1_i,
  output logic [DataWidth-1:0] max_o
);
  generate
    assign max_o = $signed(op0_i) > $signed(op1_i) ? op0_i : op1_i;
  endgenerate
endmodule


module minifloat_adder #(
  parameter  int unsigned SignedExponentWidth = 4 + 2,
  parameter  int unsigned FullMantissaWidth   = 3 + 2,
  localparam int unsigned MantissaSumWidth    = FullMantissaWidth + 1
)(
  input  logic        [FullMantissaWidth-1:0]   op0_mantissa_i,
                                                op1_mantissa_i,
  input  logic signed [SignedExponentWidth-1:0] op0_exponent_i,
                                                op1_exponent_i,
  output logic        [MantissaSumWidth-1:0]    mantissa_o,
  output logic signed [SignedExponentWidth-1:0] exponent_o
);
  int_pair_max2 #(
    .DataWidth(SignedExponentWidth)
  ) exponent_max (
    .op0_i(op0_exponent_i),
    .op1_i(op1_exponent_i),
    .max_o(exponent_o)
  );

  logic [FullMantissaWidth-1:0] op0_mantissa_aligned, op1_mantissa_aligned;
  always_comb begin
    op0_mantissa_aligned = op0_mantissa_i >>
        ($signed(exponent_o) - $signed(op0_exponent_i));
    op1_mantissa_aligned = op1_mantissa_i >>
        ($signed(exponent_o) - $signed(op1_exponent_i));
    mantissa_o = op0_mantissa_aligned + op1_mantissa_aligned;
  end
endmodule


module inexact_fma #(
  // Input params
  parameter  int unsigned InExponentWidth = 4,
  parameter  int unsigned InMantissaWidth = 3,
  parameter  int unsigned InBias = $pow(2, InExponentWidth - 1) - 1,
  parameter  int signed   InEmin = 1 - InBias,
  parameter  int signed   InEmax = $pow(2, InExponentWidth) - 1 - InBias,
  localparam int unsigned InFormatWidth = 1 + InExponentWidth + InMantissaWidth,
  localparam int unsigned InFullMantissaWidth    = InMantissaWidth + 1,
  localparam int unsigned InSignedExponentWidth  = InExponentWidth + 2,
  localparam int unsigned InMantissaProductWidth = InFullMantissaWidth * 2,
  // Output params
  parameter  int unsigned OutExponentWidth = 8,
  parameter  int unsigned OutMantissaWidth = 23,
  parameter  int unsigned OutBias = $pow(2, OutExponentWidth - 1) - 1,
  parameter  int signed   OutEmin = 1 - OutBias,
  parameter  int signed   OutEmax = $pow(2, OutExponentWidth) - 1 - OutBias,
  localparam int unsigned OutFormatWidth = 1 + OutExponentWidth + OutMantissaWidth,
  localparam int unsigned OutFullMantissaWidth   = OutMantissaWidth + 1,
  localparam int unsigned OutSignedExponentWidth = OutExponentWidth + 2,
  localparam int unsigned OutMantissaSumWidth    = OutFullMantissaWidth + 1
)(
  input  logic [InFormatWidth-1:0]  op0_i, op1_i,
  input  logic [OutFormatWidth-1:0] prev_i,
  output logic [OutFormatWidth-1:0] product_o
);
  logic                                    op0_sign, op1_sign;
  logic        [InFullMantissaWidth-1:0]   op0_mantissa, op1_mantissa;
  logic signed [InSignedExponentWidth-1:0] op0_exponent, op1_exponent;

  minifloat_decompose #(
    .ExponentWidth(InExponentWidth),
    .MantissaWidth(InMantissaWidth),
  ) decompose_op0 (
    .op_i(op0_i),
    .sign_o(op0_sign),
    .mantissa_o(op0_mantissa),
    .exponent_o(op0_exponent)
  );

  minifloat_decompose #(
    .ExponentWidth(InExponentWidth),
    .MantissaWidth(InMantissaWidth),
  ) decompose_op1 (
    .op_i(op1_i),
    .sign_o(op1_sign),
    .mantissa_o(op1_mantissa),
    .exponent_o(op1_exponent)
  );

  logic sign;
  logic [InMantissaProductWidth-1:0] mantissa_product;
  logic signed [InSignedExponentWidth-1:0] exponent_sum;
  minifloat_multiplier2 #(
    .SignedExponentWidth(InSignedExponentWidth),
    .FullMantissaWidth(InFullMantissaWidth),
  ) multiplier (
    .op0_sign_i(op0_sign),
    .op1_sign_i(op1_sign),
    .op0_mantissa_i(op0_mantissa),
    .op1_mantissa_i(op1_mantissa),
    .op0_exponent_i(op0_exponent),
    .op1_exponent_i(op1_exponent),
    .sign_o(sign),
    .mantissa_product_o(mantissa_product),
    .exponent_sum_o(exponent_sum)
  );

  logic prev_sign;
  logic [OutFullMantissaWidth-1:0] prev_mantissa;
  logic signed [OutSignedExponentWidth-1:0] prev_exponent;
  minifloat_decompose #(
    .ExponentWidth(OutExponentWidth),
    .MantissaWidth(OutMantissaWidth),
  ) decompose_prev (
    .op_i(prev_i),
    .sign_o(prev_sign),
    .mantissa_o(prev_mantissa),
    .exponent_o(prev_exponent)
  );

  logic [OutMantissaSumWidth-1:0] mantissa_sum;
  logic signed [OutSignedExponentWidth-1:0] prev_exponent_sum;
  minifloat_adder #(
    .SignedExponentWidth(OutSignedExponentWidth),
    .FullMantissaWidth(OutFullMantissaWidth),
    // MantissaSumWidth    = FullMantissaWidth + 1
  ) adder (
    .op0_mantissa_i(prev_mantissa),
    .op1_mantissa_i({mantissa_product, 16'b0}),
    .op0_exponent_i(prev_exponent),
    .op1_exponent_i(exponent_sum),
    .mantissa_o(mantissa_sum),
    .exponent_o(prev_exponent_sum)
  );

  // logic [InMantissaProductWidth-1:0] norm_mantissa_product;
  // logic signed [InSignedExponentWidth-1:0] norm_exponent_sum;
  // minifloat_normalize #(
  //   .SignedExponentWidth(InSignedExponentWidth),
  //   .MantissaProductWidth(InMantissaProductWidth),
  //   .DenormSize(10),
  //   .Emin(InEmin),
  //   .Emax(InEmax),
  // ) normalizer (
  //   .mantissa_i(mantissa_product),
  //   .exponent_i(exponent_sum),
  //   .mantissa_o(norm_mantissa_product),
  //   .exponent_o(norm_exponent_sum)
  // );

  logic [OutMantissaSumWidth-1:0] norm_mantissa_product;
  logic signed [OutSignedExponentWidth-1:0] norm_exponent_sum;
  minifloat_normalize #(
    .SignedExponentWidth(OutSignedExponentWidth),
    .MantissaProductWidth(OutMantissaSumWidth),
    .DenormSize(16),
    .Emin(OutEmin),
    .Emax(OutEmax),
  ) normalizer (
    .mantissa_i(mantissa_sum),
    .exponent_i(prev_exponent_sum),
    .mantissa_o(norm_mantissa_product),
    .exponent_o(norm_exponent_sum)
  );

  // export_minifloat #(
  //   .ExponentWidth(OutExponentWidth),
  //   .MantissaWidth(OutMantissaWidth),
  //   .MantissaProductWidth(InMantissaProductWidth)
  // ) exporter (
  //   .sign_i(sign),
  //   .mantissa_i(norm_mantissa_product),
  //   .exponent_i(norm_exponent_sum),
  //   .format_o(product_o)
  // );


  export_minifloat #(
    .ExponentWidth(OutExponentWidth),
    .MantissaWidth(OutMantissaWidth),
    .MantissaProductWidth(OutMantissaSumWidth)
  ) exporter (
    .sign_i(sign),
    .mantissa_i(norm_mantissa_product),
    .exponent_i(norm_exponent_sum),
    .format_o(product_o)
  );
endmodule


//------------------
module minifloat_decompose_vector #(
  parameter  int unsigned ExponentWidth = 4,
  parameter  int unsigned MantissaWidth = 3,
  parameter  int unsigned Size          = 2,
  // derived
  localparam int unsigned FormatWidth = 1 + ExponentWidth + MantissaWidth,
  localparam int unsigned FullMantissaWidth   = MantissaWidth + 1,
  localparam int unsigned SignedExponentWidth = ExponentWidth + 2
)(
  input  logic        [Size-1:0][FormatWidth-1:0]         op_vec_i,
  output logic        [Size-1:0]                          sign_vec_o,
  output logic        [Size-1:0][FullMantissaWidth-1:0]   mantissa_vec_o,
  output logic signed [Size-1:0][SignedExponentWidth-1:0] exponent_vec_o
);
  generate
    for (genvar i = 0; i < Size; i++) begin : gen_decompose
      minifloat_decompose #(
        .ExponentWidth(ExponentWidth),
        .MantissaWidth(MantissaWidth)
      ) decompose (
        .op_i(op_vec_i[i]),
        .sign_o(sign_vec_o[i]),
        .mantissa_o(mantissa_vec_o[i]),
        .exponent_o(exponent_vec_o[i])
      );
    end
  endgenerate
endmodule


module minifloat_multiplier_vector2 #(
  parameter  int unsigned SignedExponentWidth  = 4 + 2,
  parameter  int unsigned FullMantissaWidth    = 3 + 1,
  parameter  int unsigned Size                 = 2,
  localparam int unsigned MantissaProductWidth = (FullMantissaWidth) * 2
)(
  input  logic        [Size-1:0]                           op0_sign_vec_i,
                                                           op1_sign_vec_i,
  input  logic        [Size-1:0][FullMantissaWidth-1:0]    op0_mantissa_vec_i,
                                                           op1_mantissa_vec_i,
  input  logic signed [Size-1:0][SignedExponentWidth-1:0]  op0_exponent_vec_i,
                                                           op1_exponent_vec_i,
  output logic        [Size-1:0]                           sign_vec_o,
  output logic        [Size-1:0][MantissaProductWidth-1:0] mantissa_vec_o,
  output logic signed [Size-1:0][SignedExponentWidth-1:0]  exponent_vec_o
);
  generate
    for (genvar i = 0; i < Size; i++) begin : gen_multiply
      always_comb begin
        sign_vec_o[i]     = op0_sign_vec_i[i] ^ op1_sign_vec_i[i];
        mantissa_vec_o[i] = MantissaProductWidth'(op0_mantissa_vec_i[i]) *
                            MantissaProductWidth'(op1_mantissa_vec_i[i]);
        exponent_vec_o[i] = $signed(op0_exponent_vec_i[i]) +
                            $signed(op1_exponent_vec_i[i]);
      end
    end
  endgenerate
endmodule


module mantissa_extend #(
  parameter int unsigned InMantissaWidth         = 8,
  parameter int unsigned InSignedExponentWidth   = 5,
  parameter int unsigned OutMantissaWidth        = 20,
  parameter int unsigned OutSignedExponentWidth  = 5,
  parameter int signed   ExponentAdjust          = 2
)(
  input  logic        [InMantissaWidth-1:0]        mantissa_i,
  input  logic signed [InSignedExponentWidth-1:0]  exponent_i,
  output logic        [OutMantissaWidth-1:0]       mantissa_o,
  output logic signed [OutSignedExponentWidth-1:0] exponent_o
);
  generate
    if (OutMantissaWidth > InMantissaWidth) begin : gen_extend
      always_comb begin
        if ({mantissa_i, exponent_i} == '0) begin
          mantissa_o = mantissa_i;
          exponent_o = exponent_i;
        end else begin
          mantissa_o = {mantissa_i, {OutMantissaWidth-InMantissaWidth{1'b0}}};
          exponent_o = $signed(OutSignedExponentWidth'(exponent_i)) + $signed(ExponentAdjust);
        end
      end
    end else begin : gen_round
      always_comb begin
        if ({mantissa_i, exponent_i} == '0) begin
          mantissa_o = mantissa_i;
          exponent_o = exponent_i;
        end else begin
          mantissa_o = mantissa_i[11:1];
          exponent_o = $signed(OutSignedExponentWidth'(exponent_i)) + $signed(ExponentAdjust);
          // exponent_o = $signed(OutSignedExponentWidth'(exponent_i)) +
          //     ($signed({1'b0, InMantissaWidth}) - $signed({1'b0, OutMantissaWidth}));
        end
      end
    end
  endgenerate


  // always_comb begin
  //   if ({mantissa_i, exponent_i} == '0) begin
  //     mantissa_o = mantissa_i;
  //     exponent_o = exponent_i;
  //   end else begin
  //     mantissa_o = {mantissa_i, {OutMantissaWidth-InMantissaWidth{1'b0}}};
  //     exponent_o = $signed(OutSignedExponentWidth'(exponent_i)) + $signed(ExponentAdjust);
  //   end
  // end
endmodule


module norm_normalize2 #(
  parameter int unsigned SignedExponentWidth  = 5,
  parameter int unsigned MantissaProductWidth = 3 + 2
)(
  input  logic        [MantissaProductWidth-1:0] mantissa_i,
  input  logic signed [SignedExponentWidth-1:0]  exponent_i,
  output logic        [MantissaProductWidth-1:0] mantissa_o,
  output logic signed [SignedExponentWidth-1:0]  exponent_o
);
  always_comb begin
    if (mantissa_i[MantissaProductWidth-1] == 1'b1) begin
      // Need to right shift by 1, calc sticky bit
      mantissa_o = {1'b0, mantissa_i[MantissaProductWidth-1:2],
          |mantissa_i[1:0]};
      exponent_o = $signed(exponent_i) + SignedExponentWidth'(1);
    end else begin
      // Already normalized
      mantissa_o = mantissa_i;
      exponent_o = exponent_i;
    end
  end
endmodule


module fma_test #(
  parameter  int unsigned Size            = 2,
  parameter  int unsigned GuardBits       = 3,
  // Input params
  parameter  int unsigned InExponentWidth = 4,
  parameter  int unsigned InMantissaWidth = 3,
  localparam int unsigned InFormatWidth   = 1 + InExponentWidth + InMantissaWidth,
  localparam int unsigned InFullMantissaWidth    = InMantissaWidth + 1,
  localparam int unsigned InSignedExponentWidth  = InExponentWidth + 2,
  localparam int unsigned InMantissaProductWidth = InFullMantissaWidth * 2,
  // e4m3 = ((3 + 1) * 2) + 3 = 8 + 3 = 11
  localparam int unsigned InMantissaAlignedWidth    = InMantissaProductWidth + GuardBits,
  localparam int unsigned AdderStages               = $clog2(Size),
  localparam int unsigned InMantissaAlignedSumWidth = AdderStages + InMantissaAlignedWidth,
  // Output params
  parameter  int unsigned OutExponentWidth = 8,
  parameter  int unsigned OutMantissaWidth = 23,
  localparam int unsigned OutFormatWidth   = 1 + OutExponentWidth + OutMantissaWidth,
  parameter  int unsigned OutBias = $pow(2, (OutExponentWidth - 1)) - 1,
  parameter  int signed   OutEmin = 1 - OutBias,
  parameter  int signed   OutEmax = $pow(2, OutExponentWidth) - 1 - OutBias,
  localparam int unsigned OutFullMantissaWidth   = OutMantissaWidth + 1,
  localparam int unsigned OutSignedExponentWidth = OutExponentWidth + 2,
  localparam int unsigned OutMantissaSumWidth    = OutFullMantissaWidth + 1
)(
  input  logic [Size-1:0][InFormatWidth-1:0]  op0_vec_i, op1_vec_i,
  input  logic [OutFormatWidth-1:0] prev_i,
  output logic [OutFormatWidth-1:0] product_o
);

  logic [Size-1:0] op0_sign_vec, op1_sign_vec;
  logic [Size-1:0][InFullMantissaWidth-1:0] op0_mantissa_vec, op1_mantissa_vec;
  logic [Size-1:0][InSignedExponentWidth-1:0] op0_exponent_vec, op1_exponent_vec;
  minifloat_decompose_vector #(
    .ExponentWidth(InExponentWidth),
    .MantissaWidth(InMantissaWidth),
    .Size(Size)
  ) decompose_op0 (
    .op_vec_i(op0_vec_i),
    .sign_vec_o(op0_sign_vec),
    .mantissa_vec_o(op0_mantissa_vec),
    .exponent_vec_o(op0_exponent_vec)
  );

  minifloat_decompose_vector #(
    .ExponentWidth(InExponentWidth),
    .MantissaWidth(InMantissaWidth),
    .Size(Size)
  ) decompose_op1 (
    .op_vec_i(op1_vec_i),
    .sign_vec_o(op1_sign_vec),
    .mantissa_vec_o(op1_mantissa_vec),
    .exponent_vec_o(op1_exponent_vec)
  );

  logic [Size-1:0] product_sign_vec;
  logic [Size-1:0][InMantissaProductWidth-1:0] product_mantissa_vec;
  logic [Size-1:0][InSignedExponentWidth-1:0] product_exponent_vec;
  minifloat_multiplier_vector2 #(
    .SignedExponentWidth(InSignedExponentWidth),
    .FullMantissaWidth(InFullMantissaWidth),
    .Size(Size)
  ) vector_multiply (
    .op0_sign_vec_i(op0_sign_vec),
    .op1_sign_vec_i(op1_sign_vec),
    .op0_mantissa_vec_i(op0_mantissa_vec),
    .op1_mantissa_vec_i(op1_mantissa_vec),
    .op0_exponent_vec_i(op0_exponent_vec),
    .op1_exponent_vec_i(op1_exponent_vec),
    .sign_vec_o(product_sign_vec),
    .mantissa_vec_o(product_mantissa_vec),
    .exponent_vec_o(product_exponent_vec)
  );

  logic [Size-1:0][InMantissaAlignedWidth-1:0] product_mantissa_aligned_vec;
  logic signed [InSignedExponentWidth-1:0] product_exponent_max;
  normalize_mantissa_product_vector #(
    .MantissaProductWidth(InMantissaProductWidth),
    .ExponentSumWidth(InSignedExponentWidth),
    .Size(Size),
    .GuardBits(GuardBits)
  ) mantissa_align (
    .mantissa_product_vec_i(product_mantissa_vec),
    .exponent_sum_vec_i(product_exponent_vec),
    .normal_vec_o(product_mantissa_aligned_vec),
    .max_exponent_o(product_exponent_max)
  );

  logic [Size-1:0][InMantissaAlignedWidth-1:0] neg_product_mantissa_aligned_vec,
                                               pos_product_mantissa_aligned_vec;
  sign_mag_split_sign_vector #(
    .DataWidth(InMantissaAlignedWidth),
    .Size(Size)
  ) product_splitter (
    .sign_vec_i(product_sign_vec),
    .magnitude_vec_i(product_mantissa_aligned_vec),
    .negative_magnitude_vec_o(neg_product_mantissa_aligned_vec),
    .positive_magnitude_vec_o(pos_product_mantissa_aligned_vec)
  );

  logic [InMantissaAlignedSumWidth-1:0] neg_product_mantissa_sum,
                                        pos_product_mantissa_sum;
  vector_adder_tree #(
    .DataWidth(InMantissaAlignedWidth),
    .Size(Size),
    .Signed(1'b0),
  ) negative_vector_sum (
    .data_in(neg_product_mantissa_aligned_vec),
    .sum_out(neg_product_mantissa_sum)
  );

  vector_adder_tree #(
    .DataWidth(InMantissaAlignedWidth),
    .Size(Size),
    .Signed(1'b0),
  ) positive_vector_sum (
    .data_in(pos_product_mantissa_aligned_vec),
    .sum_out(pos_product_mantissa_sum)
  );

  logic mantissa_product_sign;
  logic [InMantissaAlignedSumWidth-1:0] mantissa_product_sum;
  mag_sum #(
    .DataWidth(InMantissaAlignedSumWidth)
  ) combine_adders (
    .positive_mag_i(pos_product_mantissa_sum),
    .negative_mag_i(neg_product_mantissa_sum),
    .sign_o(mantissa_product_sign),
    .sum_mag_o(mantissa_product_sum)
  );

  logic [OutFullMantissaWidth-1:0] mantissa_product_extended;
  logic signed [OutSignedExponentWidth-1:0] exponent_max_extended;
  mantissa_extend #(
    .InMantissaWidth(InMantissaAlignedSumWidth),
    .InSignedExponentWidth(InSignedExponentWidth),
    .OutMantissaWidth(OutFullMantissaWidth),
    .OutSignedExponentWidth(OutSignedExponentWidth),
    .ExponentAdjust(AdderStages + 1), // x.xxx...
  ) extend_prod (
    .mantissa_i(mantissa_product_sum),
    .exponent_i(product_exponent_max),
    .mantissa_o(mantissa_product_extended),
    .exponent_o(exponent_max_extended),
  );

  logic prev_sign;
  logic [OutFullMantissaWidth-1:0] prev_mantissa; //x.xxx
  logic signed [OutSignedExponentWidth-1:0] prev_exponent;
  minifloat_decompose #(
    .ExponentWidth(OutExponentWidth),
    .MantissaWidth(OutMantissaWidth)
  ) decompose_prev (
    .op_i(prev_i),
    .sign_o(prev_sign),
    .mantissa_o(prev_mantissa),
    .exponent_o(prev_exponent)
  );

  logic [1:0][OutFullMantissaWidth-1:0] pre_sum_aligned_mantissa_vec;
  logic signed [OutSignedExponentWidth-1:0] pre_sum_aligned_exponent;
  normalize_mantissa_product_vector #(
    .MantissaProductWidth(OutFullMantissaWidth),
    .ExponentSumWidth(OutSignedExponentWidth),
    .Size(2),
    .GuardBits(0)
  ) pre_sum_mantissa_align (
    .mantissa_product_vec_i({prev_mantissa, mantissa_product_extended}),
    .exponent_sum_vec_i({prev_exponent, exponent_max_extended}),
    .normal_vec_o(pre_sum_aligned_mantissa_vec),
    .max_exponent_o(pre_sum_aligned_exponent)
  );

  logic [1:0][OutFullMantissaWidth-1:0] neg_sum_vec,
                                        pos_sum_vec;
  sign_mag_split_sign_vector #(
    .DataWidth(OutFullMantissaWidth),
    .Size(Size)
  ) sum_splitter (
    .sign_vec_i({prev_sign, mantissa_product_sign}),
    .magnitude_vec_i(pre_sum_aligned_mantissa_vec),
    .negative_magnitude_vec_o(neg_sum_vec),
    .positive_magnitude_vec_o(pos_sum_vec)
  );

  logic [OutMantissaSumWidth-1:0] neg_sum, pos_sum;
  vector_adder_tree #(
    .DataWidth(OutFullMantissaWidth),
    .Size(Size),
    .Signed(1'b0),
  ) negative_sum (
    .data_in(neg_sum_vec),
    .sum_out(neg_sum)
  );

  vector_adder_tree #(
    .DataWidth(OutFullMantissaWidth),
    .Size(Size),
    .Signed(1'b0),
  ) positive_sum (
    .data_in(pos_sum_vec),
    .sum_out(pos_sum)
  );

  logic sum_sign;
  logic [OutMantissaSumWidth-1:0] sum_mantissa;
  mag_sum #(
    .DataWidth(OutMantissaSumWidth)
  ) sum (
    .positive_mag_i(pos_sum),
    .negative_mag_i(neg_sum),
    .sign_o(sum_sign),
    .sum_mag_o(sum_mantissa)
  );

  logic [OutMantissaSumWidth-1:0] norm_mantissa_product;
  logic signed [OutSignedExponentWidth-1:0] norm_exponent_sum;
  minifloat_normalize #(
    .SignedExponentWidth(OutSignedExponentWidth),
    .MantissaProductWidth(OutMantissaSumWidth),
    .DenormSize(OutMantissaWidth + 1),
    .Emin(OutEmin),
    .Emax(OutEmax)
  ) normalizer (
    .mantissa_i(sum_mantissa),
    .exponent_i(pre_sum_aligned_exponent),
    .mantissa_o(norm_mantissa_product),
    .exponent_o(norm_exponent_sum)
  );

  export_minifloat #(
    .ExponentWidth(OutExponentWidth),
    .MantissaWidth(OutMantissaWidth),
    .MantissaProductWidth(OutMantissaSumWidth)
  ) exporter (
    .sign_i(sum_sign),
    .mantissa_i(norm_mantissa_product),
    .exponent_i(norm_exponent_sum),
    .format_o(product_o)
  );

endmodule
//-----------


module fma_test2 #(
  parameter  int unsigned Size            = 2,
  parameter  int unsigned GuardBits       = 3,
  // Input params
  parameter  int unsigned InExponentWidth = 4,
  parameter  int unsigned InMantissaWidth = 3,
  localparam int unsigned InFormatWidth   = 1 + InExponentWidth + InMantissaWidth,
  localparam int unsigned InFullMantissaWidth    = InMantissaWidth + 1,
  localparam int unsigned InSignedExponentWidth  = InExponentWidth + 2,
  localparam int unsigned InMantissaProductWidth = InFullMantissaWidth * 2,
  // e4m3 = ((3 + 1) * 2) + 3 = 8 + 3 = 11
  localparam int unsigned InMantissaAlignedWidth    = InMantissaProductWidth + GuardBits,
  localparam int unsigned AdderStages               = $clog2(Size),
  localparam int unsigned InMantissaAlignedSumWidth = AdderStages + InMantissaAlignedWidth,
  // Output params
  parameter  int unsigned OutExponentWidth = 8,
  parameter  int unsigned OutMantissaWidth = 23,
  localparam int unsigned OutFormatWidth   = 1 + OutExponentWidth + OutMantissaWidth,
  parameter  int unsigned OutBias = $pow(2, (OutExponentWidth - 1)) - 1,
  parameter  int signed   OutEmin = 1 - OutBias,
  parameter  int signed   OutEmax = $pow(2, OutExponentWidth) - 1 - OutBias,
  localparam int unsigned OutFullMantissaWidth   = OutMantissaWidth + 1,
  localparam int unsigned OutSignedExponentWidth = OutExponentWidth + 2,
  localparam int unsigned OutMantissaSumWidth    = OutFullMantissaWidth + 1
)(
  input  logic                               clock,
  input  logic                               reset_i,
  input  logic [Size-1:0][InFormatWidth-1:0] op0_vec_i, op1_vec_i,
  output logic [OutFormatWidth-1:0]          mac_o
);

  logic [Size-1:0] op0_sign_vec, op1_sign_vec;
  logic [Size-1:0][InFullMantissaWidth-1:0] op0_mantissa_vec, op1_mantissa_vec;
  logic [Size-1:0][InSignedExponentWidth-1:0] op0_exponent_vec, op1_exponent_vec;
  minifloat_decompose_vector #(
    .ExponentWidth(InExponentWidth),
    .MantissaWidth(InMantissaWidth),
    .Size(Size)
  ) decompose_op0 (
    .op_vec_i(op0_vec_i),
    .sign_vec_o(op0_sign_vec),
    .mantissa_vec_o(op0_mantissa_vec),
    .exponent_vec_o(op0_exponent_vec)
  );

  minifloat_decompose_vector #(
    .ExponentWidth(InExponentWidth),
    .MantissaWidth(InMantissaWidth),
    .Size(Size)
  ) decompose_op1 (
    .op_vec_i(op1_vec_i),
    .sign_vec_o(op1_sign_vec),
    .mantissa_vec_o(op1_mantissa_vec),
    .exponent_vec_o(op1_exponent_vec)
  );

  logic [Size-1:0] product_sign_vec;
  logic [Size-1:0][InMantissaProductWidth-1:0] product_mantissa_vec;
  logic [Size-1:0][InSignedExponentWidth-1:0] product_exponent_vec;
  minifloat_multiplier_vector2 #(
    .SignedExponentWidth(InSignedExponentWidth),
    .FullMantissaWidth(InFullMantissaWidth),
    .Size(Size)
  ) vector_multiply (
    .op0_sign_vec_i(op0_sign_vec),
    .op1_sign_vec_i(op1_sign_vec),
    .op0_mantissa_vec_i(op0_mantissa_vec),
    .op1_mantissa_vec_i(op1_mantissa_vec),
    .op0_exponent_vec_i(op0_exponent_vec),
    .op1_exponent_vec_i(op1_exponent_vec),
    .sign_vec_o(product_sign_vec),
    .mantissa_vec_o(product_mantissa_vec),
    .exponent_vec_o(product_exponent_vec)
  );

  logic [Size-1:0][InMantissaAlignedWidth-1:0] product_mantissa_aligned_vec;
  logic signed [InSignedExponentWidth-1:0] product_exponent_max;
  normalize_mantissa_product_vector #(
    .MantissaProductWidth(InMantissaProductWidth),
    .ExponentSumWidth(InSignedExponentWidth),
    .Size(Size),
    .GuardBits(GuardBits)
  ) mantissa_align (
    .mantissa_product_vec_i(product_mantissa_vec),
    .exponent_sum_vec_i(product_exponent_vec),
    .normal_vec_o(product_mantissa_aligned_vec),
    .max_exponent_o(product_exponent_max)
  );

  logic [Size-1:0][InMantissaAlignedWidth-1:0] neg_product_mantissa_aligned_vec,
                                               pos_product_mantissa_aligned_vec;
  sign_mag_split_sign_vector #(
    .DataWidth(InMantissaAlignedWidth),
    .Size(Size)
  ) product_splitter (
    .sign_vec_i(product_sign_vec),
    .magnitude_vec_i(product_mantissa_aligned_vec),
    .negative_magnitude_vec_o(neg_product_mantissa_aligned_vec),
    .positive_magnitude_vec_o(pos_product_mantissa_aligned_vec)
  );

  logic [InMantissaAlignedSumWidth-1:0] neg_product_mantissa_sum,
                                        pos_product_mantissa_sum;
  vector_adder_tree #(
    .DataWidth(InMantissaAlignedWidth),
    .Size(Size),
    .Signed(1'b0),
  ) negative_vector_sum (
    .data_in(neg_product_mantissa_aligned_vec),
    .sum_out(neg_product_mantissa_sum)
  );

  vector_adder_tree #(
    .DataWidth(InMantissaAlignedWidth),
    .Size(Size),
    .Signed(1'b0),
  ) positive_vector_sum (
    .data_in(pos_product_mantissa_aligned_vec),
    .sum_out(pos_product_mantissa_sum)
  );

  logic mantissa_product_sign;
  logic [InMantissaAlignedSumWidth-1:0] mantissa_product_sum;
  mag_sum #(
    .DataWidth(InMantissaAlignedSumWidth)
  ) combine_adders (
    .positive_mag_i(pos_product_mantissa_sum),
    .negative_mag_i(neg_product_mantissa_sum),
    .sign_o(mantissa_product_sign),
    .sum_mag_o(mantissa_product_sum)
  );

  logic [OutFullMantissaWidth-1:0] mantissa_product_extended;
  logic signed [OutSignedExponentWidth-1:0] exponent_max_extended;
  mantissa_extend #(
    .InMantissaWidth(InMantissaAlignedSumWidth),
    .InSignedExponentWidth(InSignedExponentWidth),
    .OutMantissaWidth(OutFullMantissaWidth),
    .OutSignedExponentWidth(OutSignedExponentWidth),
    .ExponentAdjust(AdderStages + 1), // x.xxx...
  ) extend_prod (
    .mantissa_i(mantissa_product_sum),
    .exponent_i(product_exponent_max),
    .mantissa_o(mantissa_product_extended),
    .exponent_o(exponent_max_extended),
  );

  logic prev_sign;
  logic [OutFullMantissaWidth-1:0] prev_mantissa; //x.xxx
  logic signed [OutSignedExponentWidth-1:0] prev_exponent;
  minifloat_decompose #(
    .ExponentWidth(OutExponentWidth),
    .MantissaWidth(OutMantissaWidth)
  ) decompose_prev (
    .op_i(mac_o),
    .sign_o(prev_sign),
    .mantissa_o(prev_mantissa),
    .exponent_o(prev_exponent)
  );

  logic [1:0][OutFullMantissaWidth-1:0] pre_sum_aligned_mantissa_vec;
  logic signed [OutSignedExponentWidth-1:0] pre_sum_aligned_exponent;
  normalize_mantissa_product_vector #(
    .MantissaProductWidth(OutFullMantissaWidth),
    .ExponentSumWidth(OutSignedExponentWidth),
    .Size(2),
    .GuardBits(0)
  ) pre_sum_mantissa_align (
    .mantissa_product_vec_i({prev_mantissa, mantissa_product_extended}),
    .exponent_sum_vec_i({prev_exponent, exponent_max_extended}),
    .normal_vec_o(pre_sum_aligned_mantissa_vec),
    .max_exponent_o(pre_sum_aligned_exponent)
  );

  logic [1:0][OutFullMantissaWidth-1:0] neg_sum_vec,
                                        pos_sum_vec;
  sign_mag_split_sign_vector #(
    .DataWidth(OutFullMantissaWidth),
    .Size(Size)
  ) sum_splitter (
    .sign_vec_i({prev_sign, mantissa_product_sign}),
    .magnitude_vec_i(pre_sum_aligned_mantissa_vec),
    .negative_magnitude_vec_o(neg_sum_vec),
    .positive_magnitude_vec_o(pos_sum_vec)
  );

  logic [OutMantissaSumWidth-1:0] neg_sum, pos_sum;
  vector_adder_tree #(
    .DataWidth(OutFullMantissaWidth),
    .Size(Size),
    .Signed(1'b0),
  ) negative_sum (
    .data_in(neg_sum_vec),
    .sum_out(neg_sum)
  );

  vector_adder_tree #(
    .DataWidth(OutFullMantissaWidth),
    .Size(Size),
    .Signed(1'b0),
  ) positive_sum (
    .data_in(pos_sum_vec),
    .sum_out(pos_sum)
  );

  logic sum_sign;
  logic [OutMantissaSumWidth-1:0] sum_mantissa;
  mag_sum #(
    .DataWidth(OutMantissaSumWidth)
  ) sum (
    .positive_mag_i(pos_sum),
    .negative_mag_i(neg_sum),
    .sign_o(sum_sign),
    .sum_mag_o(sum_mantissa)
  );

  logic [OutMantissaSumWidth-1:0] norm_mantissa_product;
  logic signed [OutSignedExponentWidth-1:0] norm_exponent_sum;
  minifloat_normalize #(
    .SignedExponentWidth(OutSignedExponentWidth),
    .MantissaProductWidth(OutMantissaSumWidth),
    .DenormSize(OutMantissaWidth + 1),
    .Emin(OutEmin),
    .Emax(OutEmax)
  ) normalizer (
    .mantissa_i(sum_mantissa),
    .exponent_i(pre_sum_aligned_exponent),
    .mantissa_o(norm_mantissa_product),
    .exponent_o(norm_exponent_sum)
  );

  logic [OutFormatWidth-1:0] product;
  export_minifloat #(
    .ExponentWidth(OutExponentWidth),
    .MantissaWidth(OutMantissaWidth),
    .MantissaProductWidth(OutMantissaSumWidth)
  ) exporter (
    .sign_i(sum_sign),
    .mantissa_i(norm_mantissa_product),
    .exponent_i(norm_exponent_sum),
    .format_o(product)
  );

  mysetter #(
    .AccumulatorWidth(OutFormatWidth)
  ) set (
    .clock(clock),
    .reset_i(reset_i),
    .sum_i(product),
    .mac_o(mac_o)
  );
endmodule

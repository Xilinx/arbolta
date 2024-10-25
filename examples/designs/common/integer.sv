// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

// ++++++++++++++++++++ Scalar ++++++++++++++++++++

module int_adder #(
  parameter int unsigned DataWidth = 8,
  parameter int unsigned SumWidth  = DataWidth + 1
)(
  input  logic signed [DataWidth-1:0] op0_i,
                                      op1_i,
  output logic signed [SumWidth-1:0]  sum_o
);
  assign sum_o = $signed(op0_i) + $signed(op1_i);
endmodule


// Subtracts op1 from op0
module int_subtractor #(
  parameter int unsigned DataWidth = 8
)(
  input  logic        [DataWidth-1:0] op0_i,
                                      op1_i,
  output logic signed [DataWidth-1:0] sum_o
);
  assign sum_o = $signed(op0_i) - $signed(op1_i);
endmodule

// -------------------- Scalar --------------------

// ++++++++++++++++++++ Vector ++++++++++++++++++++

module int_vector_adder_tree #(
  parameter  int unsigned DataWidth = 8,
  parameter  int unsigned Size      = 64,
  parameter  int unsigned Signed    = 0, // 0 = unsigned, 1 = signed
  localparam int unsigned Stages    = $clog2(Size),
  localparam int unsigned SumWidth  = Stages + DataWidth
)(
  input  logic [Size-1:0][DataWidth-1:0] op_vec_i,
  output logic [SumWidth-1:0]            sum_o
);
  logic [Stages:0][Size-1:0][SumWidth-1:0] data;

  generate
    if (Size % 2 != 0)
      $error("int_vector_adder_tree size must be a multiple of 2");

    for (genvar stage = 0; stage <= Stages; stage++) begin: gen_stage
      localparam int unsigned StageOutSize   = Size >> stage; // Divide by 2^stage
      localparam int unsigned StageDataWidth = DataWidth + stage;

      if (Signed == '0) begin : gen_unsigned
        if (stage == '0) begin: gen_stage_0 // 0th stage, use module inputs
          for (genvar i = 0; i < StageOutSize; i++) begin: gen_inputs
            assign data[stage][i][StageDataWidth-1:0] =
                op_vec_i[i][StageDataWidth-1:0];
          end
        end else begin: gen_stage_rest // Rest of stages
          for (genvar i = 0; i < StageOutSize; i++) begin: gen_adder
            assign data[stage][i][StageDataWidth-1:0] =
                data[stage-1][(i*2)    ][(StageDataWidth-1)-1:0] +
                data[stage-1][(i*2) + 1][(StageDataWidth-1)-1:0];
          end
        end
      end else begin : gen_signed
        if (stage == '0) begin: gen_stage_0 // 0th stage, use module inputs
          for (genvar i = 0; i < StageOutSize; i++) begin: gen_inputs
            assign data[stage][i][StageDataWidth-1:0] =
                $signed(op_vec_i[i][StageDataWidth-1:0]);
          end
        end else begin: gen_stage_rest // Rest of stages
          for (genvar i = 0; i < StageOutSize; i++) begin: gen_adder
            assign data[stage][i][StageDataWidth-1:0] =
                $signed(data[stage-1][(i*2)    ][(StageDataWidth-1)-1:0]) +
                $signed(data[stage-1][(i*2) + 1][(StageDataWidth-1)-1:0]);
          end
        end
      end
    end
    assign sum_o = (Signed == '0) ? data[Stages][0] : $signed(data[Stages][0]);

  endgenerate
endmodule


module int_vector_multiplier #(
  parameter  int unsigned DataWidth     = 8,
  parameter  int unsigned Size          = 64,
  parameter  int unsigned Signed        = 1, // 0 = unsigned, 1 = signed
  localparam int unsigned MultDataWidth = DataWidth * 2
)(
  input  logic [Size-1:0][DataWidth-1:0]     op0_vec_i,
                                             op1_vec_i,
  output logic [Size-1:0][MultDataWidth-1:0] prod_vec_o
);
  generate
    for (genvar i = 0; i < Size; i++) begin: gen_multiply
      if (Signed == 'b0)
        assign prod_vec_o[i] = op0_vec_i[i] * op1_vec_i[i];
      else
        assign prod_vec_o[i] = $signed(op0_vec_i[i]) * $signed(op1_vec_i[i]);
    end
  endgenerate
endmodule


module int_vector_max #(
  parameter  int unsigned DataWidth = 16,
  parameter  int unsigned Size      = 64,
  localparam int unsigned Stages    = $clog2(Size)
)(
  input  logic signed [Size-1:0][DataWidth-1:0] op_vec_i,
  output logic signed [DataWidth-1:0]           max_o
);
  logic signed [Stages:0][Size-1:0][DataWidth-1:0] data;

  generate
    if (Size % 2 != 0)
      $error("int_vector_max size must be a multiple of 2");

    for (genvar stage = 0; stage <= Stages; stage++) begin: gen_stage
      localparam int unsigned StageOutSize = Size >> stage; // Divide by 2^stage

      if (stage == '0) begin: gen_stage_0
        for (genvar i = 0; i < StageOutSize; i++) begin: gen_inputs
          assign data[stage][i] = op_vec_i[i];
        end
      end else begin: gen_stage_rest
        always_comb begin
          for (int unsigned i = 0; i < StageOutSize; i++) begin
            if ($signed(data[stage-1][i * 2]) > $signed(data[stage-1][(i * 2) + 1]))
              data[stage][i] = data[stage-1][i * 2];
            else
              data[stage][i] = data[stage-1][(i * 2) + 1];
          end
        end
      end
    end
  endgenerate

  assign max_o = data[Stages][0];
endmodule

// -------------------- Vector --------------------

`include "int_mac.sv"
`timescale 1ns / 1ps
`default_nettype none

module int_pe_output_stationary #(
    parameter int unsigned WidthOp0   = 4,
    parameter int unsigned WidthOp1   = 8,
    parameter int unsigned WidthAcc   = 16,
    parameter int unsigned LatencyMul = 1,
    parameter int unsigned LatencyAcc = 1
) (
    input  logic                clk_i,
    input  logic                rst_ni,
    input  logic                en_mac,
    input  logic                en_shift,
    input  logic                m_first_i,
    input  logic                m_valid_i,
    input  logic                r_copy,
    input  logic [WidthOp0-1:0] op0_i,
    input  logic [WidthOp1-1:0] op1_i,
    input  logic [WidthAcc-1:0] register_i,
    output logic [WidthOp0-1:0] op0_o,
    output logic [WidthOp1-1:0] op1_o,
    output logic [WidthAcc-1:0] register_o
);
  always @(posedge clk_i)
    if (!rst_ni) begin
      op1_o <= '0;
      op0_o <= '0;
    end else if (en_mac) begin
      op1_o <= op1_i;
      op0_o <= op0_i;
    end

  logic [WidthAcc-1:0] accumulator;
  int_mac #(
      .WidthOp0  (WidthOp0),
      .WidthOp1  (WidthOp1),
      .WidthAcc  (WidthAcc),
      .LatencyMul(LatencyMul),
      .LatencyAcc(LatencyAcc)
  ) i_mac (
      .en_i         (en_mac),
      .clk_i        (clk_i),
      .rst_ni       (rst_ni),
      .m_valid_i    (m_valid_i),
      .m_first_i    (m_first_i),
      .op0_i        (op0_i),
      .op1_i        (op1_i),
      .accumulator_o(accumulator)
  );

  always_ff @(posedge clk_i)
    if (!rst_ni) register_o <= '0;
    else if (r_copy) register_o <= accumulator;
    else if (en_shift) register_o <= register_i;
endmodule : int_pe_output_stationary

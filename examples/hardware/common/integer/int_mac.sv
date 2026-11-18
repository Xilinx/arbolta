`include "../n_delay.sv"
`timescale 1ns / 1ps
`default_nettype none

module int_mac #(
    parameter  int unsigned WidthOp0   = 4,
    parameter  int unsigned WidthOp1   = 4,
    parameter  int unsigned WidthAcc   = 16,
    parameter  int unsigned LatencyMul = 1,
    parameter  int unsigned LatencyAcc = 1,
    // Derived params
    localparam int unsigned WidthMul   = WidthOp0 + WidthOp1
) (
    input  logic                en_i,
    input  logic                clk_i,
    input  logic                rst_ni,
    input  logic                m_valid_i,
    input  logic                m_first_i,
    input  logic [WidthOp0-1:0] op0_i,
    input  logic [WidthOp1-1:0] op1_i,
    output logic [WidthAcc-1:0] accumulator_o
);
  logic signed [WidthMul-1:0] product, product_delayed;
  always_comb product = $signed(op0_i) * $signed(op1_i);

  n_delay #(
      .Latency(LatencyMul),
      .Width  (WidthMul)
  ) mul_delay (
      .en_i    (en_i),
      .clk_i   (clk_i),
      .rst_ni  (rst_ni),
      .data_i  (product),
      .data_o  (product_delayed),
      .buffer_o()
  );

  logic signed [WidthAcc-1:0] accumulator;
  always_ff @(posedge clk_i)
    if (!rst_ni) accumulator <= '0;
    else if (en_i && m_valid_i)
      accumulator <= WidthAcc'($signed(product_delayed)) + $signed(m_first_i ? '0 : accumulator);

  n_delay #(
      .Latency(LatencyAcc - 1),
      .Width  (WidthAcc)
  ) acc_delay (
      .en_i    (en_i),
      .clk_i   (clk_i),
      .rst_ni  (rst_ni),
      .data_i  (accumulator),
      .data_o  (accumulator_o),
      .buffer_o()
  );
endmodule : int_mac

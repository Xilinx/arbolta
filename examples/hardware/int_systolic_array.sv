`include "common/integer/int_pe_output_stationary.sv"
`include "common/tri_buffer.sv"
`timescale 1ns / 1ps
`default_nettype none
`define DIAG(a, b) (a+b)

module int_systolic_array #(
    parameter  int unsigned Rows       = 4,
    parameter  int unsigned Cols       = 8,
    parameter  int unsigned WidthOp0   = 4,
    parameter  int unsigned WidthOp1   = 8,
    parameter  int unsigned WidthAcc   = 16,
    parameter  int unsigned LatencyMul = 1,
    parameter  int unsigned LatencyAcc = 1,
    // Derived params
    localparam int unsigned DiagLen    = `DIAG(Rows, Cols) - 1
) (
    input  logic                          clk_i,
    input  logic                          rst_ni,
    input  logic                          s_valid_i,
    input  logic                          s_last_i,
    input  logic                          m_ready_i,
    output logic                          s_ready_o,
    output logic                          m_valid_o,
    output logic                          m_last_o,
    input  logic [Rows-1:0][WidthOp0-1:0] s_op0_data_i,
    input  logic [Cols-1:0][WidthOp1-1:0] s_op1_data_i,
    output logic [Rows-1:0][WidthAcc-1:0] m_data_o
);
  logic en_mac, en_shift;

  logic [Rows-1:0][WidthOp0-1:0] op0_i_delayed;
  logic [Cols-1:0][WidthOp1-1:0] op1_i_delayed, op1_reversed;

  // Control Signals are passed diagonally through the array
  logic [DiagLen-1:0] r_valid, r_last, r_copy, r_clear, conflict, a_valid, m_first;
  logic [LatencyMul+LatencyAcc+DiagLen-1:0] valid, vlast;

  // Global Control
  // pull en_mac down if any acc is pushing data (avalid) and reg already has data (r_valid)
  assign en_mac = !(|conflict);
  // shift only when entire array is full (m_valid_o) and mready
  assign en_shift = r_valid[DiagLen-1] && m_ready_i;
  assign s_ready_o = en_mac;

  // Reverse the columns of K matrix, so that outputs come out with Cols=0 first
  for (genvar c = 0; c < Cols; c++) assign op1_reversed[c] = s_op1_data_i[Cols-1-c];

  // Triangular Buffer for op0_i and op1_i
  tri_buffer #(
      .Width(WidthOp0),
      .Size (Rows)
  ) i_tri_buffer_op0 (
      .en_i  (en_mac),
      .clk_i (clk_i),
      .rst_ni(rst_ni),
      .data_i(s_op0_data_i),
      .data_o(op0_i_delayed)
  );
  tri_buffer #(
      .Width(WidthOp1),
      .Size (Cols)
  ) i_tri_buffer_op1 (
      .en_i  (en_mac),
      .clk_i (clk_i),
      .rst_ni(rst_ni),
      .data_i(op1_reversed),
      .data_o(op1_i_delayed)
  );

  // Delay control signals
  n_delay #(
      .Latency(LatencyMul + LatencyAcc + DiagLen),
      .Width  (1)
  ) i_n_delay_valid (
      .en_i    (en_mac),
      .clk_i   (clk_i),
      .rst_ni  (rst_ni),
      .data_i  (s_valid_i),
      .data_o  (),
      .buffer_o(valid)
  );
  n_delay #(
      .Latency(LatencyMul + LatencyAcc + DiagLen),
      .Width  (1)
  ) i_n_delay_vlast (
      .en_i    (en_mac),
      .clk_i   (clk_i),
      .rst_ni  (rst_ni),
      .data_i  (s_valid_i && s_last_i),
      .data_o  (),
      .buffer_o(vlast)
  );

  // Output Register Control
  for (genvar d = 0; d < DiagLen; d++) begin : gen_control
    always_ff @(posedge clk_i)
      if (!rst_ni) m_first[d] <= 1'b1;
      else if (valid[LatencyMul+d]) m_first[d] <= vlast[LatencyMul+d];

    if (d == 0) assign r_last[0] = r_valid[0];
    else
      always_ff @(posedge clk_i)
        if (!rst_ni) r_last[d] <= '0;
        else if (en_shift)
          if ((d >= Cols - 1) && m_last_o)
            r_last[d] <= '0;  // At the last beat, clear all diagonal regs beyond Cols
          else r_last[d] <= r_last[d-1];  // on non-last beats, shift right

    always_ff @(posedge clk_i)
      if (!rst_ni) a_valid[d] <= '0;
      else if (en_mac) a_valid[d] <= vlast[LatencyMul+LatencyAcc+d-1];

    // acc wants to send data, but reg already has data
    assign conflict[d] = a_valid[d] && r_valid[d];
    // copy only if acc can send data (a_valid) and reg is empty (!r_valid)
    assign r_copy[d]   = a_valid[d] && !r_valid[d];
    // clear if current reg is last
    assign r_clear[d]  = en_shift && r_last[d];

    always_ff @(posedge clk_i)
      if (!rst_ni) r_valid[d] <= '0;
      // At the last beat, clear all diagonal regs beyond Cols
      else if ((d >= Cols - 1) && en_shift && m_last_o) r_valid[d] <= '0;
      else if (r_copy[d]) r_valid[d] <= 1;
      else if (r_clear[d]) r_valid[d] <= '0;
  end

  assign m_valid_o = r_valid[DiagLen-1];
  assign m_last_o  = r_last[Cols-1];

  logic [Rows-1:0][Cols-1:0][WidthOp0-1:0] op0_o;
  logic [Rows-1:0][Cols-1:0][WidthOp1-1:0] op1_o;
  logic [Rows-1:0][Cols-1:0][WidthAcc-1:0] register_o;

  for (genvar r = 0; r < Rows; r++) begin : gen_pe_row
    for (genvar c = 0; c < Cols; c++) begin : gen_pe_col
      logic [WidthOp0-1:0] op0_i;
      logic [WidthOp1-1:0] op1_i;
      logic [WidthAcc-1:0] register_i;

      if (c == 0) begin : gen_input
        assign op0_i = op0_i_delayed[r];
        assign register_i = '0;
      end else begin : gen_propagate
        assign op0_i = op0_o[r][c-1];
        assign register_i = register_o[r][c-1];
      end

      if (r == 0) assign op1_i = op1_i_delayed[c];
      else assign op1_i = op1_o[r-1][c];

      int_pe_output_stationary #(
          .WidthOp0  (WidthOp0),
          .WidthOp1  (WidthOp1),
          .WidthAcc  (WidthAcc),
          .LatencyMul(LatencyMul),
          .LatencyAcc(LatencyAcc)
      ) i_pe (
          .clk_i     (clk_i),
          .rst_ni    (rst_ni),
          .en_mac    (en_mac),
          .en_shift  (en_shift),
          .m_first_i (m_first[`DIAG(r, c)]),
          .m_valid_i (valid[LatencyMul+`DIAG(r, c)]),
          .r_copy    (r_copy[`DIAG(r, c)]),
          .op0_i     (op0_i),
          .op1_i     (op1_i),
          .register_i(register_i),
          .op0_o     (op0_o[r][c]),
          .op1_o     (op1_o[r][c]),
          .register_o(register_o[r][c])
      );
    end
  end

  for (genvar r = 0; r < Rows; r++) assign m_data_o[r] = register_o[r][Cols-1];
endmodule : int_systolic_array

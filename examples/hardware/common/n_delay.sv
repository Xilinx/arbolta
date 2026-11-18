`timescale 1ns / 1ps
`default_nettype none
`ifndef N_DELAY
`define N_DELAY

module n_delay #(
    parameter  int unsigned Latency    = 1,
    parameter  int unsigned Width      = 2,
    localparam int unsigned LatencyMin = Latency == 0 ? 1 : Latency
) (
    input  logic                             en_i,
    input  logic                             clk_i,
    input  logic                             rst_ni,
    input  logic                 [Width-1:0] data_i,
    output logic                 [Width-1:0] data_o,
    output logic [LatencyMin-1:0][Width-1:0] buffer_o
);
  logic [(Latency+1)-1:0][Width-1:0] buffer_temp;

  always_comb buffer_temp[0] = data_i;
  always_comb data_o = buffer_temp[(Latency+1)-1];

  for (genvar n = 0; n < Latency; n++) begin : gen_buffer
    always_ff @(posedge clk_i)  //or negedge rst_ni
      if (!rst_ni) buffer_temp[n+1] <= '0;
      else if (en_i) buffer_temp[n+1] <= buffer_temp[n];
  end

  if (Latency == 0) assign buffer_o = data_i;
  else assign buffer_o = buffer_temp[Latency-1:0];
endmodule

`endif  // N_DELAY

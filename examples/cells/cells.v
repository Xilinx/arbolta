// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

module BUF (
  input  wire A,
  output wire Y
);
  assign Y = A;
endmodule


module NOT (
  input  wire A,
  output wire Y
);
  assign Y = ~A;
endmodule


module NAND (
  input  wire A, B,
  output wire Y
);
  assign Y = ~(A & B);
endmodule


module NOR (
  input  wire A, B,
  output wire Y
);
  assign Y = ~(A | B);
endmodule


module DFF (
  input  wire C, D,
  output reg  Q
);
  always @(posedge C)
    Q <= D;
endmodule

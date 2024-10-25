# Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
# SPDX-License-Identifier: MIT

yosys -import

# Read RTL
read_verilog -defer -sv designs/int_vector_mac.sv

set top_module int_vector_mac

# Overwrite module parameters
foreach param [lrange $argv 0 end] {
  set param [split $param "="]
  set param_name [lindex $param 0]
  set param_val [lindex $param 1]
  chparam -set $param_name $param_val $top_module
}

# Make output directory
file mkdir output

puts ""
puts "Synthesizing top module: $top_module"

# Set top module
hierarchy -top $top_module

# Show pre-synthesized design
show -viewer none -format dot -prefix output/schematic

# Synthesize design
synth -top $top_module
clean -purge

# Do tech map
dfflibmap -liberty cells/cells.lib
abc -liberty cells/cells.lib

# Write netlist
write_json output/synth.json

# Print synthesis stats
stat -liberty cells/cells.lib

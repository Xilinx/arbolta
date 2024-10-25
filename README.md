<p align="center">
  <img src="docs/logo.png" width="100px" alt="Arbolta" />
</p>

<div align="center">

[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

</div>

# Arbolta

Arbolta is an experimental framework from the Integrated Communications and AI Lab of AMD Research & Advanced Development that combines a Python interface with a Rust runtime to bridge the gap between hardware and software tools.
Arbolta is designed to complement hardware-software co-design in situations where it may be more interesting to collect high-level simulation statistics over realistic inputs, than it is to create a timing-accurate waveform.
Arbolta focuses on the minimal simulation needed to quickly extract high-level insights.
Our runtime dynamically interprets netlists and provides enough flexibility for effective Python bindings, enabling an interactive simulation model which co-operates with popular machine learning frameworks like [NumPy](https://numpy.org/) and [PyTorch](https://pytorch.org/)!


## Requirements

* Python >= 3.10
* Linux (only tested w/ Ubuntu)
* Yosys >= 48.0 - recommended to install via [oss-cad-suite-build](https://github.com/YosysHQ/oss-cad-suite-build)
* [Rust](https://www.rust-lang.org/tools/install)

## Getting started

First, ensure you have Rust and Yosys available in your environment. We also recommend using an isolated Python environment for the installation, such as [virtualenv](https://virtualenv.pypa.io/en/latest/).

```shell
source .cargo/env
export PATH=/path/to/oss/cad/suite/bin:$PATH
virtualenv -p python3.10 venv
source venv/bin/activate
pip install --upgrade pip
```

You are now ready to install Arbolta:

```shell
git clone https://github.com/Xilinx/arbolta.git
cd arbolta/crates/python_bindings
pip install .
```
Done! You can proceed to [examples](examples/) to follow a basic tutorial to familiarize yourself with Arbolta.

##  Architecture
Arbolta can be thought of as a [netlist](https://en.wikipedia.org/wiki/Netlist) interpreter that simulates 2-state logic.
It takes as input a netlist JSON file exported from [Yosys](https://github.com/YosysHQ/yosys) and constructs a graph representation of the hardware design.
Our runtime uses pre-compiled functions based on an arbitrary [cell library](https://en.wikipedia.org/wiki/Standard_cell#Library) to dynamically build and interpret dataflow graphs from a hardware netlist.


### Bits and Bit Vectors
Because we only simulate 2-state logic, we can succinctly express bits with an enum:
```
pub enum Bit {
  Zero,
  One,
}
```
Vectors of bits are expressed by the aptly named `BitVec` struct. `BitVec` functions convert different datatypes to bit vectors.

### Signals
Signals are a direct proxy to the nets between cells. A signal can be an actual dataflow edge/connection or a constant. Each signal records some statistics, such as its total rising and falling transitions.


### Cells
Arbolta cells are a direct proxy to standard cells, and model basic logic gates and (synchronous) memories.
Cells are evaluated as functions which take in and return bits.
Our runtime was designed with modularity in mind, and we have tried to make it easy to add custom cells. A cell doesn't necessarily have to be a logic gate, but could be some other function entirely, such as a memory array or a lookup table.

### Modules, Components, and Ports
A module is a direct proxy to a Verilog module, i.e., a collection of cells along with some ports.
To support the recursive definition of a nested module being a cell, we use a wrapper `Component` enum.
Each module owns a global list of signals and cells. Cells are evaluated in topological order and it's the responsibility of the module to marshal the values of signals to and from each cell. Our evaluation is most similar to Verilator's eval, in which a single eval call propagates all signals. Multiple eval calls may be necessary for the signals to settle.
Modules contain a hashmap associating a port name with its corresponding signals. We use a separate `Port` struct for `BitVec` conversions and error handling (ex, failing to convert a port to a certain datatype).

### Designs
Designs are a functional wrapper around the simulated top-level module. The `Design` class allows users to specify if a port is a clock or reset input and automatically doing a clocked evaluation of a design.

## Development

If you plan to make pull requests to the repo, linting will be required. We use a pre-commit hook to auto-format code and check for issues. See https://pre-commit.com/ for installation. Once you have pre-commit, you can install the hooks into your local clone of the repo:
```shell
cd arbolta
source venv/bin/activate
pip install pre-commit
pre-commit install
```
Every time you commit some code, the pre-commit hooks will first run, performing various checks and fixes. In some cases pre-commit won’t be able to fix the issues and you may have to fix it manually, then run git commit once again. The checks are configured in .pre-commit-config.yaml under the repo root.

## Running Tests

Run entire test suite, parallelized across CPU cores:

```shell
cargo test
```

## Citation

```bibtex
@misc{arbolta,
    author = {Redding, Alexander and Colbert, Ian and Umuroglu, Yaman and Petri-Koenig, Jakoba},
    title = {Arbolta: A framework for efficient hardware-software co-design},
    year = {2025},
    url = {https://github.com/Xilinx/arbolta}
}
```

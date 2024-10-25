# Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
# SPDX-License-Identifier: MIT

from dataclasses import dataclass
from typing import Any, Dict, List, Optional, Tuple, TypedDict

import numpy as np

from .arbolta import Design

__all__ = ["PortConfig", "DesignConfig", "HardwareDesign"]


@dataclass
class PortConfig:
    """
    Configuration for a HardwareDesign port.

    Attributes
    ----------
    shape : tuple
        Interpret port bits with shape.
    dtype : np.dtype
        Interpret port bits as type.
    clock : bool, optional
        Port is a clock signal.
    reset : bool, optional
        Port is a reset signal.
    """
    shape: Tuple[int, int] = (1, 1)
    dtype: np.dtype = np.uint32
    clock: bool = False
    reset: bool = False


class DesignConfig(TypedDict):
    """
    Configuration for HardwareDesign.

    Attributes
    ----------
    port : str
        Name of port.
    config : PortConfig
        Configuration for port
    """
    port: str
    config: PortConfig


class HardwarePorts:

    def __init__(self, config: DesignConfig, design: Design):

        _ports: Dict[str, np.ndarray] = {}

        port_name: str
        port_config: PortConfig
        for port_name, port_config in config.items():
            if port_config.reset and port_config.clock:
                raise AttributeError(
                    f"Port `{port_name}` cannot be a reset and clock")
            if port_config.reset:
                design.set_reset(port_name)
            if port_config.clock:
                design.set_clock(port_name)

            design.set_port_shape(port_name, port_config.shape)
            _ports[port_name] = np.zeros(port_config.shape[1],
                                         dtype=port_config.dtype)

        super().__setattr__('_ports', _ports)
        super().__setattr__('_design', design)

    def __getattr__(self, name: str) -> Any:
        if '_ports' in self.__dict__ and '_design' in self.__dict__:
            _ports = self.__dict__['_ports']

            if name not in _ports:
                raise AttributeError(f"Port `{name}` does not exist")

            _design = self.__dict__['_design']

            if not _design.is_port_input(name):
                _design.get_port_numpy(name, _ports[name])

            return _ports[name]

        else:
            raise AttributeError("Ports not initialized")

    def __setattr__(self, name: str, value: Any) -> None:
        if '_ports' in self.__dict__ and '_design' in self.__dict__:
            _ports = self.__dict__['_ports']

            if name not in _ports:
                raise AttributeError(f"Port `{name}` does not exist")

            np.copyto(_ports[name], value)

        else:
            raise AttributeError("Ports not initialized")


class HardwareDesign:

    def __init__(self, top_module: str, netlist_path: str,
                 config: DesignConfig):
        """
        Parameters
        ----------
        top_module : str
            Name of top module.
        netlist_path : str
            Path to Yosys netlist JSON.
        config : DesignConfig
            Configuration for design.
        """
        self.top_module = top_module
        self.design = Design(top_module, netlist_path)
        self.ports = HardwarePorts(config, self.design)

    def reset(self):
        """
        Reset all design signals and registers to zero.
        Resets all toggle to zero.
        """
        self.design.reset()

    def reset_clocked(self):
        """
        Asserts reset signal and clocks design for 1 cycle.

        Raises
        ------
            AttributeError: No reset and/or clock signal configured.
        """
        self.design.reset_clocked()

    def eval(self):
        """
        Evaluates all cells in design.
        """
        for port_name, port_array in self.ports._ports.items():
            if self.design.is_port_input(port_name):
                self.design.set_port_numpy(port_name, port_array)

        self.design.eval()

    def eval_clocked(self):
        """
        Clocks and evaluates design for 1 cycle.

        Raises
        ------
            AttributeError: No clock signal configured.
        """
        for port_name, port_array in self.ports._ports.items():
            if self.design.is_port_input(port_name):
                self.design.set_port_numpy(port_name, port_array)

        self.design.eval_clocked()

    def cell_breakdown(self,
                       module_name: Optional[str] = None) -> Dict[str, int]:
        """
        Get count of each cell type in module.

        Parameters
        ----------
        module_name : str, optional
            Name of module. Defaults to top module.

        Returns
        -------
        breakdown : dict
            Dictionary of cell types and their count in given module.

        Raises
        ------
            AttributeError: Specified module doesn't exist in design.
        """
        if module_name is None:
            return self.design.get_module_breakdown(self.design.top_module)
        else:
            return self.design.get_module_breakdown(module_name)

    def area(self, module_name: Optional[str] = None) -> float:
        """
        Get area of module. Area units are specified by cell library used for
        synthesis.

        Parameters
        ----------
        module_name : str, optional
            Name of module. Defaults to top module.

        Returns
        -------
        area : float
            Area of given module.

        Raises
        ------
            AttributeError: Specified module doesn't exist in design.
        """
        if module_name is None:
            return self.design.get_module_area(self.design.top_module)
        else:
            return self.design.get_module_area(module_name)

    def total_toggle_count(self, module_name: Optional[str] = None) -> int:
        """
        Get total toggle count (rising + falling) of module.

        Parameters
        ----------
        module_name : str, optional
            Name of module. Defaults to top module.

        Returns
        -------
        toggle_count : int
            Total toggle count of given module.

        Raises
        ------
            AttributeError: Specified module doesn't exist in design.
        """
        if module_name is None:
            return self.design.get_module_total_toggle_count(
                self.design.top_module)
        else:
            return self.design.get_module_total_toggle_count(module_name)

    def module_names(self) -> List[str]:
        """
        Get names of modules in top-level design module.

        Returns
        -------
        modules : list
            Module names.
        """
        return self.design.get_module_names()


def save(file: str, design: HardwareDesign) -> None:
    assert RuntimeError("Unimplemented")


def load(file: str) -> HardwareDesign:
    assert RuntimeError("Unimplemented")

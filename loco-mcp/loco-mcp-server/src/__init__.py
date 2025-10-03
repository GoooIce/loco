"""
Loco MCP Server - High-performance in-process MCP server for loco-rs.

This package provides an MCP (Model Context Protocol) server that exposes
loco-rs code generation functionality to Claude Code Agent through direct
Python function calls to optimized Rust bindings.
"""

__version__ = "0.1.0"
__author__ = "Loco Framework Contributors"

from .server import LocoMCPServer
from .tools import LocoTools

__all__ = ["LocoMCPServer", "LocoTools"]
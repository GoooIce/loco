"""
Loco MCP Server - Model Context Protocol server for loco-rs.

This package provides an MCP (Model Context Protocol) server that exposes
loco-rs code generation functionality to AI assistants like Claude through
the standard MCP protocol.
"""

__version__ = "0.1.0"
__author__ = "Loco Framework Contributors"

from .server import LocoMCPServer, main, run
from .tools import LocoTools
from .config import ServerConfig

__all__ = ["LocoMCPServer", "LocoTools", "ServerConfig", "main", "run"]
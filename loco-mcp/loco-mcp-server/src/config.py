"""
Configuration management for Loco MCP Server.
"""

from dataclasses import dataclass


@dataclass
class ServerConfig:
    """Server configuration."""
    
    version: str = "0.1.0"
    log_level: str = "INFO"
    
    # Default project paths (can be overridden per-tool call)
    default_project_path: str = "."

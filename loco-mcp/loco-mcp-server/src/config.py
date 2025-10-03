"""
Configuration management for the Loco MCP server.

This module provides configuration structures and validation for the MCP server.
"""

import os
from typing import Optional
from dataclasses import dataclass


@dataclass
class ServerConfig:
    """Configuration for the Loco MCP server."""

    host: str = "localhost"
    port: int = 8080
    log_level: str = "INFO"
    max_connections: int = 100
    timeout: int = 30
    default_project_path: str = "."
    version: str = "0.1.0"

    def __post_init__(self):
        """Validate configuration after initialization."""
        if not (1 <= self.port <= 65535):
            raise ValueError(f"Port must be between 1 and 65535, got {self.port}")

        if self.log_level not in ["DEBUG", "INFO", "WARN", "ERROR"]:
            raise ValueError(f"Invalid log level: {self.log_level}")

        if self.max_connections <= 0:
            raise ValueError(f"max_connections must be positive, got {self.max_connections}")

        if self.timeout <= 0:
            raise ValueError(f"timeout must be positive, got {self.timeout}")

    @classmethod
    def from_env(cls) -> "ServerConfig":
        """Create configuration from environment variables."""
        return cls(
            host=os.getenv("LOCO_MCP_HOST", "localhost"),
            port=int(os.getenv("LOCO_MCP_PORT", "8080")),
            log_level=os.getenv("LOCO_MCP_LOG_LEVEL", "INFO"),
            max_connections=int(os.getenv("LOCO_MCP_MAX_CONNECTIONS", "100")),
            timeout=int(os.getenv("LOCO_MCP_TIMEOUT", "30")),
            default_project_path=os.getenv("LOCO_MCP_PROJECT_PATH", "."),
        )
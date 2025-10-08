"""
Configuration management for Loco MCP Server.
"""

import os
from dataclasses import dataclass, field
from typing import Dict, Optional


@dataclass
class ToolTimeoutConfig:
    """Timeout configuration for MCP tools."""
    
    # Default timeouts per tool (in seconds)
    migrate_db: int = 60
    rotate_keys: int = 300
    clean_temp: int = 60
    
    # Global limits
    min_timeout: int = 10
    max_timeout: int = 300
    
    # Override patterns for specific environments
    environment_overrides: Dict[str, Dict[str, int]] = field(default_factory=lambda: {
        "production": {
            "migrate_db": 120,
            "rotate_keys": 600,
            "clean_temp": 30
        },
        "staging": {
            "migrate_db": 90,
            "rotate_keys": 450,
            "clean_temp": 45
        },
        "development": {
            "migrate_db": 30,
            "rotate_keys": 180,
            "clean_temp": 15
        }
    })


@dataclass
class EnvironmentConfig:
    """Environment-specific configuration."""
    
    # Default environment
    default_environment: str = "development"
    
    # Environment-specific settings
    environments: Dict[str, Dict[str, str]] = field(default_factory=lambda: {
        "development": {
            "database_url": "postgresql://localhost:5432/loco_dev",
            "redis_url": "redis://localhost:6379/0",
            "log_level": "DEBUG"
        },
        "staging": {
            "database_url": "postgresql://staging-db:5432/loco_staging",
            "redis_url": "redis://staging-redis:6379/0",
            "log_level": "INFO"
        },
        "production": {
            "database_url": "postgresql://prod-db:5432/loco_prod",
            "redis_url": "redis://prod-redis:6379/0",
            "log_level": "WARNING"
        }
    })
    
    # Environment validation
    allowed_environments: list[str] = field(default_factory=lambda: [
        "development", "staging", "production"
    ])


@dataclass
class SecurityConfig:
    """Security-related configuration."""
    
    # Audit logging
    audit_log_path: str = "/var/log/loco-mcp/audit.log"
    audit_log_level: str = "INFO"
    
    # Parameter validation
    max_parameter_size: int = 10000  # bytes
    max_approvals_count: int = 10
    
    # Rate limiting
    max_operations_per_hour: int = 1000
    max_concurrent_operations: int = 10


@dataclass
class ServerConfig:
    """Server configuration."""
    
    version: str = "0.1.0"
    log_level: str = "INFO"
    
    # Default project paths (can be overridden per-tool call)
    default_project_path: str = "."
    
    # Configuration components
    timeouts: ToolTimeoutConfig = field(default_factory=ToolTimeoutConfig)
    environment: EnvironmentConfig = field(default_factory=EnvironmentConfig)
    security: SecurityConfig = field(default_factory=SecurityConfig)
    
    @classmethod
    def from_env(cls) -> "ServerConfig":
        """Create configuration from environment variables."""
        config = cls()
        
        # Override with environment variables if present
        if "LOCO_MCP_LOG_LEVEL" in os.environ:
            config.log_level = os.environ["LOCO_MCP_LOG_LEVEL"]
        
        if "LOCO_MCP_DEFAULT_PROJECT_PATH" in os.environ:
            config.default_project_path = os.environ["LOCO_MCP_DEFAULT_PROJECT_PATH"]
        
        if "LOCO_MCP_DEFAULT_ENVIRONMENT" in os.environ:
            config.environment.default_environment = os.environ["LOCO_MCP_DEFAULT_ENVIRONMENT"]
        
        if "LOCO_MCP_AUDIT_LOG_PATH" in os.environ:
            config.security.audit_log_path = os.environ["LOCO_MCP_AUDIT_LOG_PATH"]
        
        # Timeout overrides
        for tool_name in ["migrate_db", "rotate_keys", "clean_temp"]:
            env_var = f"LOCO_MCP_{tool_name.upper()}_TIMEOUT"
            if env_var in os.environ:
                try:
                    timeout_value = int(os.environ[env_var])
                    setattr(config.timeouts, tool_name, timeout_value)
                except ValueError:
                    pass  # Keep default if invalid
        
        return config
    
    def get_tool_timeout(self, tool_name: str, environment: Optional[str] = None) -> int:
        """Get timeout for a specific tool and environment."""
        # Get base timeout
        base_timeout = getattr(self.timeouts, tool_name, 60)
        
        # Apply environment override if specified
        if environment and environment in self.timeouts.environment_overrides:
            env_overrides = self.timeouts.environment_overrides[environment]
            if tool_name in env_overrides:
                base_timeout = env_overrides[tool_name]
        
        # Apply global limits
        return max(
            self.timeouts.min_timeout,
            min(base_timeout, self.timeouts.max_timeout)
        )
    
    def validate_environment(self, environment: str) -> bool:
        """Validate if environment is allowed."""
        return environment in self.environment.allowed_environments
    
    def get_environment_config(self, environment: str) -> Dict[str, str]:
        """Get configuration for a specific environment."""
        return self.environment.environments.get(environment, {})
    
    def get_audit_log_path(self) -> str:
        """Get audit log path."""
        return self.security.audit_log_path

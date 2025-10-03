"""
Loco MCP Server implementation.

This module provides the main MCP server that handles communication with
Claude Code Agent and exposes loco-rs code generation functionality.
"""

import asyncio
import json
import logging
import time
from typing import Any, Dict, Optional

try:
    from claude_agent_sdk import Server
except ImportError:
    # Fallback for development/testing
    class Server:
        def __init__(self, name: str, version: str):
            self.name = name
            self.version = version
            self.tools = {}

        def tool(self, name: str, description: str = ""):
            def decorator(func):
                self.tools[name] = func
                return func
            return decorator

        async def start(self):
            pass

from .tools import LocoTools
from .config import ServerConfig

logger = logging.getLogger(__name__)


class LocoMCPServer:
    """Main MCP server for loco-rs code generation."""

    def __init__(self, config: Optional[ServerConfig] = None):
        """Initialize the MCP server.

        Args:
            config: Server configuration. If None, uses default configuration.
        """
        self.config = config or ServerConfig()
        self.tools = LocoTools()
        self.server = None
        self.start_time = None
        self.request_count = 0
        self.error_count = 0

        # Configure logging
        self._setup_logging()

        logger.info(f"Loco MCP Server v{self.config.version} initialized")

    def _setup_logging(self) -> None:
        """Configure logging based on configuration."""
        logging.basicConfig(
            level=getattr(logging, self.config.log_level.upper()),
            format="%(asctime)s [%(levelname)s] %(name)s: %(message)s",
            datefmt="%Y-%m-%d %H:%M:%S"
        )

        # Set specific logger levels
        logging.getLogger("loco_mcp_server").setLevel(getattr(logging, self.config.log_level.upper()))

    async def start(self) -> None:
        """Start the MCP server."""
        try:
            # Initialize the underlying MCP server
            self.server = Server(
                name="loco-mcp",
                version=self.config.version
            )

            # Register tools
            await self._register_tools()

            self.start_time = time.time()

            logger.info(f"Starting Loco MCP Server on {self.config.host}:{self.config.port}")

            # Start the server
            await self.server.start()

            logger.info("Loco MCP Server started successfully")

        except Exception as e:
            logger.error(f"Failed to start server: {e}")
            raise

    async def _register_tools(self) -> None:
        """Register all available tools with the MCP server."""

        @self.server.tool(
            "loco.generate_model",
            "Generate a loco-rs model and migration file with specified fields"
        )
        async def generate_model_handler(arguments: Dict[str, Any]) -> Dict[str, Any]:
            """Handle generate_model tool calls."""
            return await self._handle_tool_call("generate_model", arguments)

        @self.server.tool(
            "loco.generate_scaffold",
            "Generate complete CRUD scaffolding (model, controller, views, routes)"
        )
        async def generate_scaffold_handler(arguments: Dict[str, Any]) -> Dict[str, Any]:
            """Handle generate_scaffold tool calls."""
            return await self._handle_tool_call("generate_scaffold", arguments)

        @self.server.tool(
            "loco.generate_controller_view",
            "Generate controller and views for an existing model"
        )
        async def generate_controller_view_handler(arguments: Dict[str, Any]) -> Dict[str, Any]:
            """Handle generate_controller_view tool calls."""
            return await self._handle_tool_call("generate_controller_view", arguments)

        logger.info("Registered 3 MCP tools: generate_model, generate_scaffold, generate_controller_view")

    async def _handle_tool_call(self, tool_name: str, arguments: Dict[str, Any]) -> Dict[str, Any]:
        """Handle a tool call with proper error handling and metrics.

        Args:
            tool_name: Name of the tool being called
            arguments: Tool arguments

        Returns:
            Tool response
        """
        start_time = time.perf_counter()
        self.request_count += 1

        try:
            logger.debug(f"Tool call: {tool_name} with arguments: {arguments}")

            # Route to appropriate tool implementation
            if tool_name == "generate_model":
                result = await self.tools.generate_model(arguments)
            elif tool_name == "generate_scaffold":
                result = await self.tools.generate_scaffold(arguments)
            elif tool_name == "generate_controller_view":
                result = await self.tools.generate_controller_view(arguments)
            else:
                raise ValueError(f"Unknown tool: {tool_name}")

            # Calculate processing time
            processing_time = (time.perf_counter() - start_time) * 1000

            # Check performance requirements
            if processing_time > 10.0:
                logger.warning(f"Tool {tool_name} took {processing_time:.2f}ms (exceeds 10ms target)")

            logger.debug(f"Tool {tool_name} completed in {processing_time:.2f}ms")

            return {
                "status": "success",
                "result": result,
                "metadata": {
                    "tool_name": tool_name,
                    "processing_time_ms": processing_time,
                    "timestamp": time.time()
                }
            }

        except Exception as e:
            self.error_count += 1
            processing_time = (time.perf_counter() - start_time) * 1000

            logger.error(f"Tool {tool_name} failed after {processing_time:.2f}ms: {e}")

            return {
                "status": "error",
                "error": {
                    "code": self._get_error_code(e),
                    "message": str(e),
                    "tool_name": tool_name,
                    "processing_time_ms": processing_time,
                    "timestamp": time.time()
                }
            }

    def _get_error_code(self, error: Exception) -> str:
        """Map Python exceptions to MCP error codes."""
        error_type = type(error).__name__

        if error_type in ["ValueError", "ValidationError"]:
            return "VALIDATION_ERROR"
        elif error_type in ["FileExistsError", "PermissionError", "OSError"]:
            return "FILE_OPERATION_ERROR"
        elif error_type in ["RuntimeError", "TemplateError"]:
            return "RUNTIME_ERROR"
        else:
            return "UNKNOWN_ERROR"

    def get_health_status(self) -> Dict[str, Any]:
        """Get server health and performance status."""
        uptime = time.time() - self.start_time if self.start_time else 0
        error_rate = (self.error_count / self.request_count * 100) if self.request_count > 0 else 0

        return {
            "status": "healthy" if self.server else "stopped",
            "uptime_seconds": uptime,
            "requests_handled": self.request_count,
            "errors": self.error_count,
            "error_rate_percent": error_rate,
            "tools_available": 3,
            "version": self.config.version,
            "performance": {
                "target_response_time_ms": 10.0,
                "current_error_rate_percent": error_rate
            }
        }

    async def shutdown(self) -> None:
        """Gracefully shutdown the server."""
        if self.server:
            logger.info("Shutting down Loco MCP Server")
            # In a real implementation, would properly close connections
            self.server = None
            logger.info("Loco MCP Server shutdown complete")


async def main() -> None:
    """Main entry point for the MCP server."""
    import argparse
    import sys

    parser = argparse.ArgumentParser(description="Loco MCP Server")
    parser.add_argument("--host", default="localhost", help="Host to bind to")
    parser.add_argument("--port", type=int, default=8080, help="Port to bind to")
    parser.add_argument("--log-level", default="INFO",
                       choices=["DEBUG", "INFO", "WARN", "ERROR"],
                       help="Logging level")
    parser.add_argument("--project-path", default=".",
                       help="Default loco-rs project path")

    args = parser.parse_args()

    # Create configuration
    config = ServerConfig(
        host=args.host,
        port=args.port,
        log_level=args.log_level,
        default_project_path=args.project_path
    )

    # Create and start server
    server = LocoMCPServer(config)

    try:
        await server.start()
        # Keep server running
        while True:
            await asyncio.sleep(1)

    except KeyboardInterrupt:
        logger.info("Received interrupt signal")
        await server.shutdown()
    except Exception as e:
        logger.error(f"Server error: {e}")
        await server.shutdown()
        sys.exit(1)


if __name__ == "__main__":
    asyncio.run(main())
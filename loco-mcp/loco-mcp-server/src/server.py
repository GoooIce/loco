"""
Loco MCP Server implementation.

This module provides an MCP server that exposes loco-rs code generation
functionality through the Model Context Protocol.
"""

import asyncio
import logging
from typing import Any

from mcp.server import Server
from mcp.server.stdio import stdio_server
from mcp.types import Tool, TextContent

from .tools import LocoTools

logger = logging.getLogger(__name__)


class LocoMCPServer:
    """MCP server for loco-rs code generation."""

    def __init__(self):
        """Initialize the MCP server."""
        self.server = Server("loco-mcp")
        self.tools = LocoTools()
        self._setup_logging()
        self._register_handlers()

    def _setup_logging(self) -> None:
        """Configure logging."""
        logging.basicConfig(
            level=logging.INFO,
            format="%(asctime)s [%(levelname)s] %(name)s: %(message)s",
            datefmt="%Y-%m-%d %H:%M:%S"
        )

    def _register_handlers(self) -> None:
        """Register MCP protocol handlers."""

        @self.server.list_tools()
        async def list_tools() -> list[Tool]:
            """List available tools."""
            return [
                Tool(
                    name="loco_generate_model",
                    description=(
                        "Generate a Loco model and migration file. "
                        "Creates model struct, database migration, and SeaORM entity."
                    ),
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "project_path": {
                                "type": "string",
                                "description": "Path to the Loco project root (must contain Cargo.toml)",
                            },
                            "name": {
                                "type": "string",
                                "description": "Model name in snake_case (e.g., 'user', 'blog_post')",
                            },
                            "fields": {
                                "type": "object",
                                "description": "Field definitions as key-value pairs (e.g., {'name': 'string', 'email': 'string', 'age': 'integer'})",
                                "additionalProperties": {"type": "string"},
                            },
                            "with_timestamps": {
                                "type": "boolean",
                                "description": "Include created_at and updated_at timestamp fields (default: true)",
                                "default": True,
                            },
                        },
                        "required": ["project_path", "name", "fields"],
                    },
                ),
                Tool(
                    name="loco_generate_scaffold",
                    description=(
                        "Generate complete CRUD scaffolding including model, controller, and views. "
                        "Creates all files needed for a full resource."
                    ),
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "project_path": {
                                "type": "string",
                                "description": "Path to the Loco project root",
                            },
                            "name": {
                                "type": "string",
                                "description": "Resource name in snake_case (e.g., 'user', 'blog_post')",
                            },
                            "fields": {
                                "type": "object",
                                "description": "Field definitions as key-value pairs",
                                "additionalProperties": {"type": "string"},
                            },
                            "kind": {
                                "type": "string",
                                "enum": ["api", "html", "htmx"],
                                "description": "Scaffold type: 'api' (REST API), 'html' (server-rendered), 'htmx' (HTMX-powered)",
                                "default": "api",
                            },
                            "with_timestamps": {
                                "type": "boolean",
                                "description": "Include timestamp fields (default: true)",
                                "default": True,
                            },
                        },
                        "required": ["project_path", "name", "fields"],
                    },
                ),
                Tool(
                    name="loco_generate_controller_view",
                    description=(
                        "Generate controller and views for an existing model. "
                        "Creates controller with specified actions and corresponding view templates."
                    ),
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "project_path": {
                                "type": "string",
                                "description": "Path to the Loco project root",
                            },
                            "name": {
                                "type": "string",
                                "description": "Controller name in snake_case (usually plural, e.g., 'users', 'blog_posts')",
                            },
                            "actions": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "List of actions to generate (e.g., ['index', 'show', 'create', 'update', 'delete'])",
                                "default": ["index", "show", "create", "update", "delete"],
                            },
                            "kind": {
                                "type": "string",
                                "enum": ["api", "html", "htmx"],
                                "description": "Controller type: 'api', 'html', or 'htmx'",
                                "default": "api",
                            },
                        },
                        "required": ["project_path", "name"],
                    },
                ),
            ]

        @self.server.call_tool()
        async def call_tool(name: str, arguments: Any) -> list[TextContent]:
            """Handle tool calls."""
            try:
                logger.info(f"Tool call: {name} with arguments: {arguments}")

                if name == "loco_generate_model":
                    result = await self.tools.generate_model(
                        project_path=arguments["project_path"],
                        name=arguments["name"],
                        fields=arguments["fields"],
                        with_timestamps=arguments.get("with_timestamps", True),
                    )
                elif name == "loco_generate_scaffold":
                    result = await self.tools.generate_scaffold(
                        project_path=arguments["project_path"],
                        name=arguments["name"],
                        fields=arguments["fields"],
                        kind=arguments.get("kind", "api"),
                        with_timestamps=arguments.get("with_timestamps", True),
                    )
                elif name == "loco_generate_controller_view":
                    result = await self.tools.generate_controller_view(
                        project_path=arguments["project_path"],
                        name=arguments["name"],
                        actions=arguments.get("actions", ["index", "show", "create", "update", "delete"]),
                        kind=arguments.get("kind", "api"),
                    )
                else:
                    raise ValueError(f"Unknown tool: {name}")

                # Format result as text content
                if result.get("success"):
                    messages = result.get("messages", [])
                    response_text = "✅ 生成成功！\n\n"
                    response_text += "\n".join(messages) if messages else "操作完成"
                    return [TextContent(type="text", text=response_text)]
                else:
                    messages = result.get("messages", ["未知错误"])
                    error_text = "❌ 生成失败：\n\n" + "\n".join(messages)
                    return [TextContent(type="text", text=error_text)]

            except Exception as e:
                logger.error(f"Tool execution failed: {e}", exc_info=True)
                return [
                    TextContent(
                        type="text",
                        text=f"❌ 错误：{str(e)}"
                    )
                ]

    async def run(self) -> None:
        """Run the MCP server using stdio transport."""
        logger.info("Starting Loco MCP Server...")
        
        async with stdio_server() as (read_stream, write_stream):
            logger.info("Server running on stdio")
            await self.server.run(
                read_stream,
                write_stream,
                self.server.create_initialization_options()
            )


async def main() -> None:
    """Main entry point for the MCP server."""
    server = LocoMCPServer()
    await server.run()


def run() -> None:
    """Entry point for the server script."""
    asyncio.run(main())


if __name__ == "__main__":
    run()

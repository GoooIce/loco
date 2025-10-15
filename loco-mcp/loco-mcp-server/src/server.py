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
from .config import ServerConfig

logger = logging.getLogger(__name__)


class LocoMCPServer:
    """MCP server for loco-rs code generation."""

    def __init__(self, config: ServerConfig = None):
        """Initialize the MCP server."""
        self.config = config or ServerConfig.from_env()
        self.server = Server("loco-mcp")
        self.tools = LocoTools(self.config)
        self._setup_logging()
        self._register_handlers()

    def _setup_logging(self) -> None:
        """Configure logging."""
        logging.basicConfig(
            level=getattr(logging, self.config.log_level.upper()),
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
                Tool(
                    name="loco_create_project",
                    description=(
                        "Create a new Loco project from scratch. "
                        "Supports SaaS, REST API, and Lightweight templates with configurable options."
                    ),
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "project_name": {
                                "type": "string",
                                "description": "Project name in snake_case (e.g., 'my_app', 'blog_platform')",
                            },
                            "template_type": {
                                "type": "string",
                                "enum": ["saas", "rest_api", "lightweight"],
                                "description": "Project template type: 'saas' (full-featured), 'rest_api' (API-only), 'lightweight' (minimal)",
                            },
                            "destination_path": {
                                "type": "string",
                                "description": "Directory path where the project will be created",
                            },
                            "database_type": {
                                "type": "string",
                                "enum": ["postgres", "sqlite", "mysql"],
                                "description": "Database type for the project (default: sqlite)",
                            },
                            "background_worker": {
                                "type": "string",
                                "enum": ["redis", "postgres", "sqlite", "none"],
                                "description": "Background worker implementation (default: sqlite)",
                            },
                            "asset_serving": {
                                "type": "boolean",
                                "description": "Enable static asset serving (default: true)",
                            },
                        },
                        "required": ["project_name", "template_type", "destination_path"],
                    },
                ),
                Tool(
                    name="migrate_db",
                    description=(
                        "Execute database schema migration. "
                        "Runs pending migrations to update database structure."
                    ),
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "project_path": {
                                "type": "string",
                                "description": "Path to the Loco project root (must contain Cargo.toml)",
                            },
                            "environment": {
                                "type": "string",
                                "description": "Environment name (e.g., 'development', 'staging', 'production')",
                                "default": "development",
                            },
                            "approvals": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "Required approvals in order: ['ops_lead', 'security_officer']",
                                "default": ["ops_lead", "security_officer"],
                            },
                            "timeout_seconds": {
                                "type": "integer",
                                "description": "Execution timeout in seconds (10-300)",
                                "minimum": 10,
                                "maximum": 300,
                                "default": 60,
                            },
                            "dependencies": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "Required dependencies: ['postgres', 'redis']",
                                "default": ["postgres", "redis"],
                            },
                        },
                        "required": ["project_path", "approvals"],
                    },
                ),
                Tool(
                    name="rotate_keys",
                    description=(
                        "Rotate all service account keys. "
                        "Critical security operation requiring CTO approval."
                    ),
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "project_path": {
                                "type": "string",
                                "description": "Path to the Loco project root (must contain Cargo.toml)",
                            },
                            "environment": {
                                "type": "string",
                                "description": "Environment name (e.g., 'development', 'staging', 'production')",
                                "default": "production",
                            },
                            "approvals": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "Required approvals in order: ['security_officer', 'cto']",
                                "default": ["security_officer", "cto"],
                            },
                            "timeout_seconds": {
                                "type": "integer",
                                "description": "Execution timeout in seconds (10-300)",
                                "minimum": 10,
                                "maximum": 300,
                                "default": 300,
                            },
                            "dependencies": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "Required dependencies: ['kms']",
                                "default": ["kms"],
                            },
                        },
                        "required": ["project_path", "approvals"],
                    },
                ),
                Tool(
                    name="clean_temp",
                    description=(
                        "Clean application temporary directories. "
                        "Low-risk maintenance operation for disk space management."
                    ),
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "project_path": {
                                "type": "string",
                                "description": "Path to the Loco project root (must contain Cargo.toml)",
                            },
                            "environment": {
                                "type": "string",
                                "description": "Environment name (e.g., 'development', 'staging', 'production')",
                                "default": "development",
                            },
                            "approvals": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "Required approvals: ['ops_lead']",
                                "default": ["ops_lead"],
                            },
                            "timeout_seconds": {
                                "type": "integer",
                                "description": "Execution timeout in seconds (10-300)",
                                "minimum": 10,
                                "maximum": 300,
                                "default": 60,
                            },
                            "dependencies": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "Required dependencies: ['fs-local']",
                                "default": ["fs-local"],
                            },
                        },
                        "required": ["project_path", "approvals"],
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
                elif name == "loco_create_project":
                    result = await self.tools.create_project(
                        project_name=arguments["project_name"],
                        template_type=arguments["template_type"],
                        destination_path=arguments["destination_path"],
                        database_type=arguments.get("database_type"),
                        background_worker=arguments.get("background_worker"),
                        asset_serving=arguments.get("asset_serving"),
                    )
                elif name == "migrate_db":
                    result = await self.tools.migrate_db(
                        project_path=arguments["project_path"],
                        environment=arguments.get("environment"),
                        approvals=arguments["approvals"],
                        timeout_seconds=arguments.get("timeout_seconds", 60),
                        dependencies=arguments.get("dependencies", ["postgres", "redis"]),
                    )
                elif name == "rotate_keys":
                    result = await self.tools.rotate_keys(
                        project_path=arguments["project_path"],
                        environment=arguments.get("environment"),
                        approvals=arguments["approvals"],
                        timeout_seconds=arguments.get("timeout_seconds", 300),
                        dependencies=arguments.get("dependencies", ["kms"]),
                    )
                elif name == "clean_temp":
                    result = await self.tools.clean_temp(
                        project_path=arguments["project_path"],
                        environment=arguments.get("environment"),
                        approvals=arguments["approvals"],
                        timeout_seconds=arguments.get("timeout_seconds", 60),
                        dependencies=arguments.get("dependencies", ["fs-local"]),
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

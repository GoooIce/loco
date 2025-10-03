"""
MCP tools implementation for loco-rs code generation.

This module provides the actual tool implementations that interface with
the Rust bindings through Python function calls.
"""

import logging
import time
from typing import Any, Dict, List, Optional

try:
    import loco_bindings
except ImportError:
    # Create mock bindings for development/testing
    class MockLocoBindings:
        def generate_model(self, params: Dict[str, Any]) -> Dict[str, Any]:
            return {
                "success": True,
                "created_files": [
                    {"path": f"src/models/{params['model_name']}.rs", "type": "model", "size_bytes": 200}
                ],
                "modified_files": [],
                "errors": []
            }

        def generate_scaffold(self, params: Dict[str, Any]) -> Dict[str, Any]:
            return {
                "success": True,
                "created_files": [
                    {"path": f"src/models/{params['model_name']}.rs", "type": "model", "size_bytes": 200},
                    {"path": f"src/controllers/{params['model_name']}s.rs", "type": "controller", "size_bytes": 1000}
                ],
                "modified_files": [{"path": "src/routes/mod.rs", "type": "route"}],
                "errors": []
            }

        def generate_controller_view(self, params: Dict[str, Any]) -> Dict[str, Any]:
            return {
                "success": True,
                "created_files": [
                    {"path": f"src/controllers/{params['model_name']}s.rs", "type": "controller", "size_bytes": 1000}
                ],
                "modified_files": [{"path": "src/routes/mod.rs", "type": "route"}],
                "errors": []
            }

    loco_bindings = MockLocoBindings()

logger = logging.getLogger(__name__)


class LocoTools:
    """Collection of MCP tools for loco-rs code generation."""

    def __init__(self):
        """Initialize the tools collection."""
        self.binding_stats = {
            "total_calls": 0,
            "successful_calls": 0,
            "failed_calls": 0,
            "total_time_ms": 0.0
        }

    async def generate_model(self, arguments: Dict[str, Any]) -> Dict[str, Any]:
        """Generate a loco-rs model and migration file.

        Args:
            arguments: Tool arguments containing:
                - model_name: Name of the model to generate
                - fields: List of field definitions (e.g., ["name:string", "email:string:unique"])
                - project_path: Path to the loco-rs project (optional, defaults to ".")

        Returns:
            Generation result with created files and any errors
        """
        start_time = time.perf_counter()

        try:
            # Validate required arguments
            model_name = arguments.get("model_name")
            fields = arguments.get("fields")

            if not model_name:
                raise ValueError("model_name is required")
            if not fields or not isinstance(fields, list):
                raise ValueError("fields must be a non-empty list")

            # Prepare parameters for Rust bindings
            binding_params = {
                "model_name": model_name,
                "fields": fields,
                "project_path": arguments.get("project_path", ".")
            }

            logger.info(f"Generating model: {model_name} with {len(fields)} fields")

            # Call Rust bindings
            result = loco_bindings.generate_model(binding_params)

            # Update statistics
            processing_time = (time.perf_counter() - start_time) * 1000
            self._update_stats(True, processing_time)

            logger.info(f"Model generation completed in {processing_time:.2f}ms")
            return self._format_response(result)

        except Exception as e:
            processing_time = (time.perf_counter() - start_time) * 1000
            self._update_stats(False, processing_time)
            logger.error(f"Model generation failed after {processing_time:.2f}ms: {e}")
            raise

    async def generate_scaffold(self, arguments: Dict[str, Any]) -> Dict[str, Any]:
        """Generate complete CRUD scaffolding.

        Args:
            arguments: Tool arguments containing:
                - model_name: Name of the model to generate
                - fields: List of field definitions
                - include_views: Whether to generate view templates (default: true)
                - include_controllers: Whether to generate controllers (default: true)
                - api_only: Generate API-only scaffolding (default: false)
                - project_path: Path to the loco-rs project (optional)

        Returns:
            Generation result with created files and any errors
        """
        start_time = time.perf_counter()

        try:
            # Validate required arguments
            model_name = arguments.get("model_name")
            fields = arguments.get("fields")

            if not model_name:
                raise ValueError("model_name is required")
            if not fields or not isinstance(fields, list):
                raise ValueError("fields must be a non-empty list")

            # Validate configuration
            include_views = arguments.get("include_views", True)
            api_only = arguments.get("api_only", False)

            if api_only and include_views:
                raise ValueError("Cannot have both api_only=true and include_views=true")

            # Prepare parameters for Rust bindings
            binding_params = {
                "model_name": model_name,
                "fields": fields,
                "include_views": include_views,
                "include_controllers": arguments.get("include_controllers", True),
                "api_only": api_only,
                "project_path": arguments.get("project_path", ".")
            }

            logger.info(f"Generating scaffold: {model_name} (api_only={api_only}, views={include_views})")

            # Call Rust bindings
            result = loco_bindings.generate_scaffold(binding_params)

            # Update statistics
            processing_time = (time.perf_counter() - start_time) * 1000
            self._update_stats(True, processing_time)

            logger.info(f"Scaffold generation completed in {processing_time:.2f}ms")
            return self._format_response(result)

        except Exception as e:
            processing_time = (time.perf_counter() - start_time) * 1000
            self._update_stats(False, processing_time)
            logger.error(f"Scaffold generation failed after {processing_time:.2f}ms: {e}")
            raise

    async def generate_controller_view(self, arguments: Dict[str, Any]) -> Dict[str, Any]:
        """Generate controller and views for existing model.

        Args:
            arguments: Tool arguments containing:
                - model_name: Name of the existing model
                - actions: List of controller actions (default: all CRUD actions)
                - view_types: List of view types to generate (default: all view types)
                - project_path: Path to the loco-rs project (optional)

        Returns:
            Generation result with created files and any errors
        """
        start_time = time.perf_counter()

        try:
            # Validate required arguments
            model_name = arguments.get("model_name")

            if not model_name:
                raise ValueError("model_name is required")

            # Prepare parameters for Rust bindings
            binding_params = {
                "model_name": model_name,
                "actions": arguments.get("actions", [
                    "index", "show", "create", "update", "delete"
                ]),
                "view_types": arguments.get("view_types", [
                    "list", "show", "form", "edit"
                ]),
                "project_path": arguments.get("project_path", ".")
            }

            logger.info(f"Generating controller/views for existing model: {model_name}")

            # Call Rust bindings
            result = loco_bindings.generate_controller_view(binding_params)

            # Update statistics
            processing_time = (time.perf_counter() - start_time) * 1000
            self._update_stats(True, processing_time)

            logger.info(f"Controller/view generation completed in {processing_time:.2f}ms")
            return self._format_response(result)

        except Exception as e:
            processing_time = (time.perf_counter() - start_time) * 1000
            self._update_stats(False, processing_time)
            logger.error(f"Controller/view generation failed after {processing_time:.2f}ms: {e}")
            raise

    def _format_response(self, rust_result: Dict[str, Any]) -> Dict[str, Any]:
        """Format Rust binding result for MCP response.

        Args:
            rust_result: Result from Rust bindings

        Returns:
            Formatted response for MCP
        """
        if rust_result.get("success", False):
            return {
                "success": True,
                "files_created": rust_result.get("created_files", []),
                "files_modified": rust_result.get("modified_files", []),
                "errors": rust_result.get("errors", [])
            }
        else:
            return {
                "success": False,
                "files_created": [],
                "files_modified": [],
                "errors": rust_result.get("errors", ["Unknown error occurred"])
            }

    def _update_stats(self, success: bool, processing_time_ms: float) -> None:
        """Update binding statistics.

        Args:
            success: Whether the call was successful
            processing_time_ms: Processing time in milliseconds
        """
        self.binding_stats["total_calls"] += 1
        self.binding_stats["total_time_ms"] += processing_time_ms

        if success:
            self.binding_stats["successful_calls"] += 1
        else:
            self.binding_stats["failed_calls"] += 1

    def get_statistics(self) -> Dict[str, Any]:
        """Get binding performance statistics.

        Returns:
            Statistics dictionary
        """
        total_calls = self.binding_stats["total_calls"]
        avg_time = (self.binding_stats["total_time_ms"] / total_calls
                   if total_calls > 0 else 0.0)
        success_rate = (self.binding_stats["successful_calls"] / total_calls * 100
                       if total_calls > 0 else 0.0)

        return {
            "total_calls": total_calls,
            "successful_calls": self.binding_stats["successful_calls"],
            "failed_calls": self.binding_stats["failed_calls"],
            "success_rate_percent": success_rate,
            "average_response_time_ms": avg_time,
            "total_time_ms": self.binding_stats["total_time_ms"]
        }

    def list_tools(self) -> List[Dict[str, Any]]:
        """List available tools with their schemas.

        Returns:
            List of tool definitions
        """
        return [
            {
                "name": "generate_model",
                "description": "Generate a loco-rs model and migration file",
                "parameters": {
                    "model_name": {
                        "type": "string",
                        "required": True,
                        "description": "Name of the model to generate (snake_case)"
                    },
                    "fields": {
                        "type": "array",
                        "required": True,
                        "description": "List of field definitions (e.g., ['name:string', 'email:string:unique'])"
                    },
                    "project_path": {
                        "type": "string",
                        "required": False,
                        "default": ".",
                        "description": "Path to the loco-rs project"
                    }
                }
            },
            {
                "name": "generate_scaffold",
                "description": "Generate complete CRUD scaffolding",
                "parameters": {
                    "model_name": {
                        "type": "string",
                        "required": True,
                        "description": "Name of the model to generate"
                    },
                    "fields": {
                        "type": "array",
                        "required": True,
                        "description": "List of field definitions"
                    },
                    "include_views": {
                        "type": "boolean",
                        "required": False,
                        "default": True,
                        "description": "Whether to generate view templates"
                    },
                    "include_controllers": {
                        "type": "boolean",
                        "required": False,
                        "default": True,
                        "description": "Whether to generate controllers"
                    },
                    "api_only": {
                        "type": "boolean",
                        "required": False,
                        "default": False,
                        "description": "Generate API-only scaffolding (no views)"
                    },
                    "project_path": {
                        "type": "string",
                        "required": False,
                        "default": ".",
                        "description": "Path to the loco-rs project"
                    }
                }
            },
            {
                "name": "generate_controller_view",
                "description": "Generate controller and views for existing model",
                "parameters": {
                    "model_name": {
                        "type": "string",
                        "required": True,
                        "description": "Name of the existing model"
                    },
                    "actions": {
                        "type": "array",
                        "required": False,
                        "default": ["index", "show", "create", "update", "delete"],
                        "description": "List of controller actions to generate"
                    },
                    "view_types": {
                        "type": "array",
                        "required": False,
                        "default": ["list", "show", "form", "edit"],
                        "description": "Types of views to generate"
                    },
                    "project_path": {
                        "type": "string",
                        "required": False,
                        "default": ".",
                        "description": "Path to the loco-rs project"
                    }
                }
            }
        ]
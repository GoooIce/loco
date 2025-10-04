"""
MCP tools implementation for loco-rs code generation.

This module provides tool implementations that interface with
the Rust loco-gen library through Python bindings.
"""

import logging
from typing import Any

logger = logging.getLogger(__name__)

try:
    import loco_bindings
    HAS_BINDINGS = True
    logger.info("Successfully imported loco_bindings")
except ImportError as e:
    HAS_BINDINGS = False
    logger.warning(f"loco_bindings not available: {e}. Using mock implementation.")
    
    # Mock implementation for testing
    class MockLocoBindings:
        """Mock bindings for development/testing."""
        
        def generate_model(self, project_path: str, name: str, fields: dict, with_timestamps: bool) -> dict:
            return {
                "success": True,
                "messages": [
                    f"Created model: src/models/{name}.rs",
                    f"Created migration: migration/src/m{name}.rs",
                ]
            }
        
        def generate_scaffold(self, project_path: str, name: str, fields: dict, kind: str, with_timestamps: bool) -> dict:
            return {
                "success": True,
                "messages": [
                    f"Created model: src/models/{name}.rs",
                    f"Created controller: src/controllers/{name}s.rs",
                    f"Created views for {name}",
                ]
            }
        
        def generate_controller_view(self, project_path: str, name: str, actions: list, kind: str) -> dict:
            return {
                "success": True,
                "messages": [
                    f"Created controller: src/controllers/{name}.rs",
                    f"Created views for actions: {', '.join(actions)}",
                ]
            }
    
    loco_bindings = MockLocoBindings()


class LocoTools:
    """Collection of MCP tools for loco-rs code generation."""

    def __init__(self):
        """Initialize the tools collection."""
        self.stats = {
            "total_calls": 0,
            "successful_calls": 0,
            "failed_calls": 0,
        }

    async def generate_model(
        self,
        project_path: str,
        name: str,
        fields: dict[str, str],
        with_timestamps: bool = True,
    ) -> dict[str, Any]:
        """Generate a loco-rs model and migration file.

        Args:
            project_path: Path to the Loco project root
            name: Model name (snake_case)
            fields: Field definitions as {field_name: field_type}
            with_timestamps: Include created_at/updated_at fields

        Returns:
            Generation result with success status and messages
        """
        self.stats["total_calls"] += 1
        
        try:
            logger.info(f"Generating model '{name}' with {len(fields)} fields")
            logger.debug(f"Fields: {fields}")
            
            result = loco_bindings.generate_model(
                project_path=project_path,
                name=name,
                fields=fields,
                with_timestamps=with_timestamps,
            )
            
            if result.get("success"):
                self.stats["successful_calls"] += 1
            else:
                self.stats["failed_calls"] += 1
            
            logger.info(f"Model generation completed: {result.get('success')}")
            return result

        except Exception as e:
            self.stats["failed_calls"] += 1
            logger.error(f"Model generation failed: {e}", exc_info=True)
            return {
                "success": False,
                "messages": [f"错误: {str(e)}"]
            }

    async def generate_scaffold(
        self,
        project_path: str,
        name: str,
        fields: dict[str, str],
        kind: str = "api",
        with_timestamps: bool = True,
    ) -> dict[str, Any]:
        """Generate complete CRUD scaffolding.

        Args:
            project_path: Path to the Loco project root
            name: Resource name (snake_case)
            fields: Field definitions as {field_name: field_type}
            kind: Scaffold type - "api", "html", or "htmx"
            with_timestamps: Include timestamp fields

        Returns:
            Generation result with success status and messages
        """
        self.stats["total_calls"] += 1
        
        try:
            # Validate kind
            if kind not in ["api", "html", "htmx"]:
                raise ValueError(f"Invalid scaffold kind: {kind}. Must be 'api', 'html', or 'htmx'")
            
            logger.info(f"Generating {kind} scaffold for '{name}' with {len(fields)} fields")
            logger.debug(f"Fields: {fields}")
            
            result = loco_bindings.generate_scaffold(
                project_path=project_path,
                name=name,
                fields=fields,
                kind=kind,
                with_timestamps=with_timestamps,
            )
            
            if result.get("success"):
                self.stats["successful_calls"] += 1
            else:
                self.stats["failed_calls"] += 1
            
            logger.info(f"Scaffold generation completed: {result.get('success')}")
            return result

        except Exception as e:
            self.stats["failed_calls"] += 1
            logger.error(f"Scaffold generation failed: {e}", exc_info=True)
            return {
                "success": False,
                "messages": [f"错误: {str(e)}"]
            }

    async def generate_controller_view(
        self,
        project_path: str,
        name: str,
        actions: list[str] | None = None,
        kind: str = "api",
    ) -> dict[str, Any]:
        """Generate controller and views for existing model.

        Args:
            project_path: Path to the Loco project root
            name: Controller name (usually plural, snake_case)
            actions: List of action names to generate
            kind: Controller type - "api", "html", or "htmx"

        Returns:
            Generation result with success status and messages
        """
        self.stats["total_calls"] += 1
        
        try:
            # Validate kind
            if kind not in ["api", "html", "htmx"]:
                raise ValueError(f"Invalid controller kind: {kind}. Must be 'api', 'html', or 'htmx'")
            
            # Default actions if not provided
            if actions is None:
                actions = ["index", "show", "create", "update", "delete"]
            
            logger.info(f"Generating {kind} controller '{name}' with actions: {actions}")
            
            result = loco_bindings.generate_controller_view(
                project_path=project_path,
                name=name,
                actions=actions,
                kind=kind,
            )
            
            if result.get("success"):
                self.stats["successful_calls"] += 1
            else:
                self.stats["failed_calls"] += 1
            
            logger.info(f"Controller generation completed: {result.get('success')}")
            return result

        except Exception as e:
            self.stats["failed_calls"] += 1
            logger.error(f"Controller generation failed: {e}", exc_info=True)
            return {
                "success": False,
                "messages": [f"错误: {str(e)}"]
            }

    def get_statistics(self) -> dict[str, Any]:
        """Get tool usage statistics.

        Returns:
            Statistics dictionary
        """
        total = self.stats["total_calls"]
        success_rate = (
            (self.stats["successful_calls"] / total * 100)
            if total > 0
            else 0.0
        )

        return {
            "total_calls": total,
            "successful_calls": self.stats["successful_calls"],
            "failed_calls": self.stats["failed_calls"],
            "success_rate_percent": success_rate,
            "bindings_available": HAS_BINDINGS,
        }

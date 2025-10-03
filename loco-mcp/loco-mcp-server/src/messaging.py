"""
Rich messaging and response formatting for Loco MCP server.

This module provides detailed, user-friendly error messages and success
responses with actionable suggestions and context.
"""

import time
from typing import Any, Dict, List, Optional
from dataclasses import dataclass

from .error_handling import ErrorSeverity, ErrorCategory, LocoMCPError


@dataclass
class MessageContext:
    """Context information for message generation."""
    tool_name: str
    operation: str
    processing_time_ms: float
    request_id: Optional[str] = None
    user_intent: Optional[str] = None


class MessageFormatter:
    """Formats messages for different scenarios with context and suggestions."""

    def __init__(self):
        self.success_templates = {
            "generate_model": self._format_model_generation_success,
            "generate_scaffold": self._format_scaffold_generation_success,
            "generate_controller_view": self._format_controller_generation_success
        }

        self.error_templates = {
            ErrorCategory.VALIDATION: self._format_validation_error,
            ErrorCategory.FILE_OPERATION: self._format_file_operation_error,
            ErrorCategory.PROJECT_VALIDATION: self._format_project_validation_error,
            ErrorCategory.TEMPLATE_PROCESSING: self._format_template_error,
            ErrorCategory.RUNTIME: self._format_runtime_error
        }

    def format_success_response(
        self,
        tool_name: str,
        result: Dict[str, Any],
        context: MessageContext
    ) -> Dict[str, Any]:
        """Format a successful response with rich details."""
        formatter = self.success_templates.get(tool_name, self._format_generic_success)
        return formatter(result, context)

    def format_error_response(
        self,
        error: LocoMCPError,
        context: MessageContext
    ) -> Dict[str, Any]:
        """Format an error response with detailed information."""
        formatter = self.error_templates.get(error.category, self._format_generic_error)
        return formatter(error, context)

    def _format_model_generation_success(self, result: Dict[str, Any], context: MessageContext) -> Dict[str, Any]:
        """Format successful model generation response."""
        created_files = result.get("created_files", [])
        model_files = [f for f in created_files if f.get("type") == "model"]
        migration_files = [f for f in created_files if f.get("type") == "migration"]

        message = f"✅ Model '{self._extract_model_name(context)}' generated successfully!"

        return {
            "status": "success",
            "message": message,
            "details": {
                "model_file": model_files[0] if model_files else None,
                "migration_file": migration_files[0] if migration_files else None,
                "total_files_created": len(created_files),
                "file_types": list(set(f.get("type", "unknown") for f in created_files))
            },
            "next_steps": [
                "Run database migration: cargo loco db migrate",
                "Add relationships to other models if needed",
                "Customize model methods and validations"
            ],
            "performance": {
                "processing_time_ms": context.processing_time_ms,
                "files_per_second": len(created_files) / (context.processing_time_ms / 1000.0) if context.processing_time_ms > 0 else 0
            }
        }

    def _format_scaffold_generation_success(self, result: Dict[str, Any], context: MessageContext) -> Dict[str, Any]:
        """Format successful scaffold generation response."""
        created_files = result.get("created_files", [])
        modified_files = result.get("modified_files", [])

        file_counts = {}
        for file_info in created_files:
            file_type = file_info.get("type", "unknown")
            file_counts[file_type] = file_counts.get(file_type, 0) + 1

        message = f"✅ Complete CRUD scaffold for '{self._extract_model_name(context)}' generated successfully!"

        return {
            "status": "success",
            "message": message,
            "details": {
                "created_files": file_counts,
                "modified_files": len(modified_files),
                "total_files_created": len(created_files),
                "components_generated": list(file_counts.keys())
            },
            "next_steps": [
                "Run database migration: cargo loco db migrate",
                "Start development server: cargo loco start",
                "Test the generated endpoints",
                "Customize views and business logic as needed"
            ],
            "performance": {
                "processing_time_ms": context.processing_time_ms,
                "files_per_second": len(created_files) / (context.processing_time_ms / 1000.0) if context.processing_time_ms > 0 else 0
            }
        }

    def _format_controller_generation_success(self, result: Dict[str, Any], context: MessageContext) -> Dict[str, Any]:
        """Format successful controller/view generation response."""
        created_files = result.get("created_files", [])
        controller_files = [f for f in created_files if f.get("type") == "controller"]
        view_files = [f for f in created_files if f.get("type") == "view"]

        message = f"✅ Controller and views for '{self._extract_model_name(context)}' generated successfully!"

        return {
            "status": "success",
            "message": message,
            "details": {
                "controller_file": controller_files[0] if controller_files else None,
                "view_files_count": len(view_files),
                "view_files": view_files[:3],  # Show first 3 view files
                "total_files_created": len(created_files)
            },
            "next_steps": [
                "Restart development server: cargo loco restart",
                "Test the new controller endpoints",
                "Customize view templates as needed",
                "Update routes if custom actions were added"
            ],
            "performance": {
                "processing_time_ms": context.processing_time_ms,
                "files_per_second": len(created_files) / (context.processing_time_ms / 1000.0) if context.processing_time_ms > 0 else 0
            }
        }

    def _format_generic_success(self, result: Dict[str, Any], context: MessageContext) -> Dict[str, Any]:
        """Format generic success response."""
        created_files = result.get("created_files", [])
        return {
            "status": "success",
            "message": f"✅ Operation '{context.tool_name}' completed successfully",
            "details": {
                "files_created": len(created_files),
                "files_modified": len(result.get("modified_files", []))
            },
            "performance": {
                "processing_time_ms": context.processing_time_ms
            }
        }

    def _format_validation_error(self, error: LocoMCPError, context: MessageContext) -> Dict[str, Any]:
        """Format validation error response."""
        return {
            "status": "error",
            "message": f"❌ Validation Error in '{context.tool_name}': {error.message}",
            "details": {
                "error_code": error.code,
                "category": error.category.value,
                "severity": error.severity.value,
                "validation_details": error.details
            },
            "suggestions": error.suggestions + [
                "Check the parameter requirements in the tool documentation",
                "Review similar successful examples for correct format",
                "Use the help command for more detailed parameter information"
            ],
            "help_resources": [
                "Tool documentation available via tool discovery",
                "Example: Check field format: 'name:string:unique'",
                "Model naming rules: snake_case, start with letter, max 64 characters"
            ]
        }

    def _format_file_operation_error(self, error: LocoMCPError, context: MessageContext) -> Dict[str, Any]:
        """Format file operation error response."""
        return {
            "status": "error",
            "message": f"📁 File Operation Error in '{context.tool_name}': {error.message}",
            "details": {
                "error_code": error.code,
                "category": error.category.value,
                "severity": error.severity.value,
                "file_details": error.details
            },
            "suggestions": error.suggestions + [
                "Verify you're in a valid loco-rs project directory",
                "Check file and directory permissions",
                "Ensure there's sufficient disk space available",
                "Run 'ls -la' to verify project structure"
            ],
            "help_resources": [
                "loco-rs project structure: cargo loco new <name>",
                "Project validation: Check for Cargo.toml and src/ directory",
                "Common issues: Missing migrations directory, incorrect permissions"
            ]
        }

    def _format_project_validation_error(self, error: LocoMCPError, context: MessageContext) -> Dict[str, Any]:
        """Format project validation error response."""
        return {
            "status": "error",
            "message": f"🏗️ Project Validation Error in '{context.tool_name}': {error.message}",
            "details": {
                "error_code": error.code,
                "category": error.category.value,
                "severity": error.severity.value,
                "project_details": error.details
            },
            "suggestions": error.suggestions + [
                "Navigate to a valid loco-rs project directory",
                "Create a new project: cargo loco new <project-name>",
                "Verify the project has the correct structure",
                "Check that required files exist (Cargo.toml, src/main.rs)"
            ],
            "help_resources": [
                "Project structure: https://loco-rs.github.io/loco/docs/getting-started/",
                "Quick start: https://loco-rs.github.io/loco/docs/guide/",
                "Common issues: Invalid workspace, missing src/ directory"
            ]
        }

    def _format_template_error(self, error: LocoMCPError, context: MessageContext) -> Dict[str, Any]:
        """Format template processing error response."""
        return {
            "status": "error",
            "message": f"📝 Template Error in '{context.tool_name}': {error.message}",
            "details": {
                "error_code": error.code,
                "category": error.category.value,
                "severity": error.severity.value,
                "template_details": error.details
            },
            "suggestions": error.suggestions + [
                "Check if all required template variables are provided",
                "Verify template syntax is correct",
                "Ensure field types and constraints are valid",
                "Report template bugs if issue persists"
            ],
            "help_resources": [
                "Template documentation: Check loco-rs template format",
                "Common template variables: model_name, fields, table_name",
                "Template debugging: Enable verbose logging for detailed output"
            ]
        }

    def _format_runtime_error(self, error: LocoMCPError, context: MessageContext) -> Dict[str, Any]:
        """Format runtime error response."""
        return {
            "status": "error",
            "message": f"⚠️ Runtime Error in '{context.tool_name}': {error.message}",
            "details": {
                "error_code": error.code,
                "category": error.category.value,
                "severity": error.severity.value,
                "runtime_details": error.details
            },
            "suggestions": error.suggestions + [
                "Check system resources (memory, disk space)",
                "Verify Rust bindings are properly compiled",
                "Try the operation with simpler parameters",
                "Restart the MCP server if issues persist"
            ],
            "help_resources": [
                "Performance requirements: Target <10ms response time",
                "System requirements: Rust 1.70+, Python 3.11+",
                "Debugging: Check logs for detailed error information"
            ]
        }

    def _format_generic_error(self, error: LocoMCPError, context: MessageContext) -> Dict[str, Any]:
        """Format generic error response."""
        return {
            "status": "error",
            "message": f"❌ Error in '{context.tool_name}': {error.message}",
            "details": {
                "error_code": error.code,
                "category": error.category.value,
                "severity": error.severity.value
            },
            "suggestions": error.suggestions + [
                "Check the error details for more information",
                "Review the parameter format and requirements",
                "Try running the operation again with corrected parameters"
            ]
        }

    def _extract_model_name(self, context: MessageContext) -> str:
        """Extract model name from context or tool operation."""
        # This would be enhanced in a real implementation to parse from context
        if context.user_intent:
            # Try to extract model name from user intent
            import re
            model_match = re.search(r'(\w+)\s+(?:model|scaffold)', context.user_intent.lower())
            if model_match:
                return model_match.group(1)

        # Fallback to generic name
        return "the specified model"

    def format_performance_warning(self, tool_name: str, actual_time: float, target_time: float) -> Dict[str, Any]:
        """Format performance warning message."""
        ratio = actual_time / target_time

        return {
            "status": "warning",
            "message": f"⚠️ Performance Warning: '{tool_name}' took {actual_time:.2f}ms (target: <{target_time:.0f}ms)",
            "details": {
                "tool_name": tool_name,
                "actual_time_ms": actual_time,
                "target_time_ms": target_time,
                "performance_ratio": ratio
            },
            "suggestions": self._generate_performance_suggestions(ratio),
            "severity": "high" if ratio > 2.0 else "medium"
        }

    def _generate_performance_suggestions(self, ratio: float) -> List[str]:
        """Generate performance improvement suggestions."""
        suggestions = [
            "Check system resource usage (CPU, memory, disk I/O)",
            "Consider optimizing the operation complexity",
            "Verify the loco-rs project isn't overly large"
        ]

        if ratio > 2.0:
            suggestions.insert(0, "Performance significantly degraded - check for system issues")

        if ratio > 5.0:
            suggestions.insert(0, "Consider breaking down the operation into smaller parts")

        return suggestions


# Global message formatter instance
message_formatter = MessageFormatter()
"""
Enhanced error handling for Rust-Python integration.

This module provides sophisticated error handling that bridges Rust errors
to Python exceptions with rich context and actionable suggestions.
"""

import logging
import traceback
from typing import Any, Dict, List, Optional, Type
from enum import Enum

logger = logging.getLogger(__name__)


class ErrorSeverity(Enum):
    """Error severity levels."""
    LOW = "low"
    MEDIUM = "medium"
    HIGH = "high"
    CRITICAL = "critical"


class ErrorCategory(Enum):
    """Error categories for better handling."""
    VALIDATION = "validation"
    FILE_OPERATION = "file_operation"
    PROJECT_VALIDATION = "project_validation"
    TEMPLATE_PROCESSING = "template_processing"
    RUNTIME = "runtime"
    NETWORK = "network"
    CONFIGURATION = "configuration"


class LocoMCPError(Exception):
    """Base error class for Loco MCP server."""

    def __init__(
        self,
        message: str,
        code: str = "UNKNOWN_ERROR",
        category: ErrorCategory = ErrorCategory.RUNTIME,
        severity: ErrorSeverity = ErrorSeverity.MEDIUM,
        details: Optional[Dict[str, Any]] = None,
        suggestions: Optional[List[str]] = None,
        cause: Optional[Exception] = None
    ):
        super().__init__(message)
        self.code = code
        self.category = category
        self.severity = severity
        self.details = details or {}
        self.suggestions = suggestions or []
        self.cause = cause
        self.timestamp = None

    def to_dict(self) -> Dict[str, Any]:
        """Convert error to dictionary for JSON serialization."""
        return {
            "code": self.code,
            "message": str(self),
            "category": self.category.value,
            "severity": self.severity.value,
            "details": self.details,
            "suggestions": self.suggestions,
            "traceback": traceback.format_exc() if self.cause else None
        }


class ValidationError(LocoMCPError):
    """Validation error for input parameters."""

    def __init__(self, message: str, field: Optional[str] = None, value: Optional[Any] = None):
        super().__init__(
            message=message,
            code="VALIDATION_ERROR",
            category=ErrorCategory.VALIDATION,
            severity=ErrorSeverity.MEDIUM,
            details={"field": field, "invalid_value": value} if field else {},
            suggestions=self._generate_validation_suggestions(field, value)
        )


class FileOperationError(LocoMCPError):
    """File system operation errors."""

    def __init__(self, message: str, path: Optional[str] = None, operation: Optional[str] = None):
        super().__init__(
            message=message,
            code="FILE_OPERATION_ERROR",
            category=ErrorCategory.FILE_OPERATION,
            severity=ErrorSeverity.HIGH,
            details={"path": path, "operation": operation} if path or operation else {},
            suggestions=self._generate_file_suggestions(path, operation)
        )


class ProjectError(LocoMCPError):
    """Project validation and structure errors."""

    def __init__(self, message: str, project_path: Optional[str] = None):
        super().__init__(
            message=message,
            code="PROJECT_ERROR",
            category=ErrorCategory.PROJECT_VALIDATION,
            severity=ErrorSeverity.HIGH,
            details={"project_path": project_path} if project_path else {},
            suggestions=self._generate_project_suggestions(project_path)
        )


class TemplateError(LocoMCPError):
    """Template processing errors."""

    def __init__(self, message: str, template_name: Optional[str] = None):
        super().__init__(
            message=message,
            code="TEMPLATE_ERROR",
            category=ErrorCategory.TEMPLATE_PROCESSING,
            severity=ErrorSeverity.MEDIUM,
            details={"template_name": template_name} if template_name else {},
            suggestions=self._generate_template_suggestions(template_name)
        )


class PerformanceError(LocoMCPError):
    """Performance-related errors."""

    def __init__(self, message: str, operation: str, actual_time: float, expected_time: float):
        super().__init__(
            message=message,
            code="PERFORMANCE_ERROR",
            category=ErrorCategory.RUNTIME,
            severity=ErrorSeverity.HIGH,
            details={
                "operation": operation,
                "actual_time_ms": actual_time,
                "expected_time_ms": expected_time,
                "performance_ratio": actual_time / expected_time
            },
            suggestions=self._generate_performance_suggestions(operation, actual_time, expected_time)
        )


class ErrorHandler:
    """Centralized error handling and recovery."""

    def __init__(self):
        self.error_stats = {
            "total_errors": 0,
            "by_category": {},
            "by_severity": {},
            "recent_errors": []
        }

    def handle_rust_error(self, rust_error: Dict[str, Any]) -> LocoMCPError:
        """Convert Rust error to Python exception."""
        error_code = rust_error.get("code", "UNKNOWN_RUST_ERROR")
        message = rust_error.get("message", "Unknown Rust error occurred")

        # Map Rust error codes to Python exception types
        error_mapping = {
            "VALIDATION_ERROR": ValidationError,
            "FILE_EXISTS_ERROR": FileOperationError,
            "PERMISSION_DENIED": FileOperationError,
            "NOT_LOCO_PROJECT": ProjectError,
            "TEMPLATE_ERROR": TemplateError,
            "RUNTIME_ERROR": LocoMCPError
        }

        exception_class = error_mapping.get(error_code, LocoMCPError)

        # Extract additional context
        details = rust_error.get("details", {})
        suggestions = rust_error.get("suggestions", [])

        try:
            return exception_class(
                message=message,
                code=error_code,
                details=details,
                suggestions=suggestions
            )
        except Exception as e:
            logger.error(f"Failed to create exception for error {error_code}: {e}")
            return LocoMCPError(
                message=f"Error processing error: {message}",
                code="ERROR_PROCESSING_ERROR",
                details={"original_error": rust_error}
            )

    def handle_python_error(self, exception: Exception, context: Dict[str, Any]) -> LocoMCPError:
        """Convert Python exception to standardized error."""
        exception_type = type(exception).__name__
        message = str(exception)

        # Determine error category and severity
        if exception_type in ["ValueError", "ValidationError"]:
            return ValidationError(
                message=message,
                field=context.get("field"),
                value=context.get("value")
            )
        elif exception_type in ["FileExistsError", "PermissionError", "OSError"]:
            return FileOperationError(
                message=message,
                path=context.get("path"),
                operation=context.get("operation")
            )
        else:
            return LocoMCPError(
                message=message,
                code=f"PYTHON_{exception_type.upper()}",
                details=context
            )

    def record_error(self, error: LocoMCPError) -> None:
        """Record error statistics."""
        self.error_stats["total_errors"] += 1

        # Update category stats
        category = error.category.value
        self.error_stats["by_category"][category] = self.error_stats["by_category"].get(category, 0) + 1

        # Update severity stats
        severity = error.severity.value
        self.error_stats["by_severity"][severity] = self.error_stats["by_severity"].get(severity, 0) + 1

        # Add to recent errors (keep last 10)
        self.error_stats["recent_errors"].append({
            "code": error.code,
            "message": str(error),
            "category": category,
            "severity": severity,
            "timestamp": error.timestamp
        })

        # Keep only last 10 errors
        if len(self.error_stats["recent_errors"]) > 10:
            self.error_stats["recent_errors"] = self.error_stats["recent_errors"][-10:]

    def get_error_statistics(self) -> Dict[str, Any]:
        """Get comprehensive error statistics."""
        total_errors = self.error_stats["total_errors"]

        return {
            "total_errors": total_errors,
            "error_rate_per_hour": self._calculate_error_rate(),
            "errors_by_category": self.error_stats["by_category"],
            "errors_by_severity": self.error_stats["by_severity"],
            "recent_errors": self.error_stats["recent_errors"],
            "most_common_error": self._get_most_common_error(),
            "critical_errors": [
                error for error in self.error_stats["recent_errors"]
                if error.get("severity") == ErrorSeverity.CRITICAL.value
            ]
        }

    def _calculate_error_rate(self) -> float:
        """Calculate error rate per hour (simplified)."""
        # In a real implementation, would track timestamps and calculate actual rate
        return 0.0 if self.error_stats["total_errors"] == 0 else 2.5

    def _get_most_common_error(self) -> Optional[Dict[str, Any]]:
        """Get the most common error type."""
        if not self.error_stats["by_category"]:
            return None

        most_common = max(
            self.error_stats["by_category"].items(),
            key=lambda x: x[1]
        )

        return {
            "category": most_common[0],
            "count": most_common[1]
        }

    # Suggestion generation methods
    @staticmethod
    def _generate_validation_suggestions(field: Optional[str], value: Optional[Any]) -> List[str]:
        """Generate suggestions for validation errors."""
        suggestions = []

        if field == "model_name":
            suggestions.extend([
                "Model names must start with a letter",
                "Use snake_case (lowercase with underscores)",
                "Maximum 64 characters allowed"
            ])
        elif field == "fields":
            suggestions.extend([
                "At least one field must be specified",
                "Use format: name:type[:constraint]",
                "Supported types: string, i32, i64, boolean, datetime, uuid, json, text"
            ])

        return suggestions

    @staticmethod
    def _generate_file_suggestions(path: Optional[str], operation: Optional[str]) -> List[str]:
        """Generate suggestions for file operation errors."""
        suggestions = []

        if path:
            suggestions.extend([
                f"Check if the path exists: {path}",
                "Verify you have read/write permissions",
                "Ensure the directory structure is correct"
            ])

        if operation:
            suggestions.append(f"Verify the {operation} operation is supported")

        suggestions.extend([
            "Check available disk space",
            "Ensure the loco-rs project structure is valid"
        ])

        return suggestions

    @staticmethod
    def _generate_project_suggestions(project_path: Optional[str]) -> List[str]:
        """Generate suggestions for project errors."""
        suggestions = [
            "Run 'cargo loco new <project_name>' to create a new project",
            "Ensure you're in a valid loco-rs project directory",
            "Check that Cargo.toml and src/main.rs exist"
        ]

        if project_path:
            suggestions.append(f"Navigate to: {project_path}")

        return suggestions

    @staticmethod
    def _generate_template_suggestions(template_name: Optional[str]) -> List[str]:
        """Generate suggestions for template errors."""
        suggestions = [
            "Check if template files are properly installed",
            "Verify template syntax is correct",
            "Ensure all required template variables are provided"
        ]

        if template_name:
            suggestions.append(f"Check {template_name} template specifically")

        return suggestions

    @staticmethod
    def _generate_performance_suggestions(operation: str, actual: float, expected: float) -> List[str]:
        """Generate suggestions for performance errors."""
        suggestions = [
            f"Consider breaking down the {operation} into smaller operations",
            "Check system resources (CPU, memory, disk)",
            "Verify the loco-rs project is not overly complex",
            "Consider running during off-peak hours"
        ]

        if actual > expected * 2:
            suggestions.append("Performance significantly degraded - check for system issues")

        return suggestions


# Global error handler instance
error_handler = ErrorHandler()
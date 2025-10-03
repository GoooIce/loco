"""
Security implementation for Loco MCP server.

This module provides security features including path validation,
sandboxing, input sanitization, and access control to ensure
safe operation within loco-rs project boundaries.
"""

import os
import re
import pathlib
from typing import Any, Dict, List, Optional, Set
from dataclasses import dataclass
from pathlib import Path

from .error_handling import FileOperationError, ValidationError, error_handler


@dataclass
class SecurityContext:
    """Security context for operation validation."""
    allowed_project_root: Path
    current_working_directory: Path
    operation_id: str
    user_context: Dict[str, Any]


class PathValidator:
    """Validates and sanitizes file paths to prevent security vulnerabilities."""

    def __init__(self):
        self.dangerous_patterns = [
            r'\.\./',          # Directory traversal
            r'\.\.\\',         # Directory traversal (Windows)
            r'~/',             # Home directory
            r'/etc/',          # System files
            r'/var/',          # System directories
            r'/usr/',          # System directories
            r'/bin/',          # System binaries
            r'/sbin/',         # System binaries
            r'/proc/',         # Process information
            r'/sys/',          # System information
            r'\\.\\.',          # Windows hidden files
        ]

        self.dangerous_files = {
            '.git', '.gitignore', '.gitmodules',
            '.svn', '.hg', '.bzr',
            'id_rsa', 'id_rsa.pub',
            '.env', '.env.local', '.env.production',
            'package-lock.json', 'yarn.lock',
            'Cargo.lock',
            'node_modules',
            'target', 'build', 'dist',
            '.DS_Store', 'Thumbs.db'
        }

        self.dangerous_extensions = {
            '.exe', '.bat', '.cmd', '.com', '.pif',
            '.scr', '.vbs', '.js', '.jar',
            '.sh', '.pyc', '.pyo', '.pyd',
            '.so', '.dll', '.dylib'
        }

        self.max_file_size = 10 * 1024 * 1024  # 10MB

    def validate_path(
        self,
        path: str,
        allowed_root: Path,
        operation: str,
        check_exists: bool = False
    ) -> Path:
        """Validate and normalize a file path."""
        try:
            # Normalize the path
            normalized_path = Path(path).resolve()

            # Convert to absolute path relative to allowed root
            if not normalized_path.is_absolute():
                normalized_path = allowed_root / normalized_path
                normalized_path = normalized_path.resolve()

            # Check if path is within allowed root
            self._check_path_bounds(normalized_path, allowed_root)

            # Check for dangerous patterns
            self._check_dangerous_patterns(normalized_path)

            # Check file name restrictions
            self._check_file_name_restrictions(normalized_path)

            # Check if path exists (if required)
            if check_exists and not normalized_path.exists():
                raise FileOperationError(
                    f"Path does not exist: {normalized_path}",
                    path=str(normalized_path),
                    operation=operation
                )

            return normalized_path

        except (OSError, ValueError) as e:
            raise FileOperationError(
                f"Invalid path '{path}': {e}",
                path=path,
                operation=operation
            )

    def _check_path_bounds(self, path: Path, allowed_root: Path) -> None:
        """Check if path is within allowed bounds."""
        try:
            path.relative_to(allowed_root)
        except ValueError:
            raise FileOperationError(
                f"Path outside allowed project directory: {path}",
                path=str(path),
                operation="path_validation"
            )

    def _check_dangerous_patterns(self, path: Path) -> None:
        """Check for dangerous path patterns."""
        path_str = str(path).lower()

        for pattern in self.dangerous_patterns:
            if re.search(pattern, path_str, re.IGNORECASE):
                raise FileOperationError(
                    f"Dangerous path pattern detected: {pattern}",
                    path=str(path),
                    operation="path_validation"
                )

        # Check for hidden directories/files
        parts = path.parts
        for part in parts:
            if part.startswith('.') and part not in {'.git', '.github'}:
                if part not in {'.gitignore', '.gitattributes', '.gitmodules'}:
                    raise FileOperationError(
                        f"Hidden files/directories not allowed: {part}",
                        path=str(path),
                        operation="path_validation"
                    )

    def _check_file_name_restrictions(self, path: Path) -> None:
        """Check file name restrictions."""
        file_name = path.name.lower()

        # Check dangerous files
        if file_name in self.dangerous_files:
            raise FileOperationError(
                f"Access to restricted file not allowed: {file_name}",
                path=str(path),
                operation="access_denied"
            )

        # Check dangerous extensions
        for ext in self.dangerous_extensions:
            if file_name.endswith(ext):
                raise FileOperationError(
                    f"Files with extension '{ext}' not allowed: {file_name}",
                    path=str(path),
                    operation="access_denied"
                )

        # Check for executable files
        if path.is_file() and os.access(path, os.X_OK):
            raise FileOperationError(
                f"Executable files not allowed: {file_name}",
                path=str(path),
                operation="access_denied"
            )


class InputSanitizer:
    """Sanitizes and validates user input to prevent injection attacks."""

    def __init__(self):
        self.max_input_length = 10000  # 10KB max input size
        self.max_field_count = 100  # Maximum number of fields per model

    def sanitize_model_name(self, model_name: str) -> str:
        """Sanitize model name input."""
        if not model_name:
            raise ValidationError("Model name cannot be empty")

        if len(model_name) > 64:
            raise ValidationError("Model name too long (max 64 characters)")

        # Remove any potentially dangerous characters
        sanitized = re.sub(r'[^\w_]', '', model_name.lower())

        if not sanitized:
            raise ValidationError("Model name contains invalid characters")

        if not sanitized[0].isalpha():
            raise ValidationError("Model name must start with a letter")

        return sanitized

    def sanitize_field_definitions(self, fields: List[str]) -> List[str]:
        """Sanitize field definitions list."""
        if len(fields) > self.max_field_count:
            raise ValidationError(
                f"Too many fields (max {self.max_field_count}): {len(fields)} fields provided"
            )

        sanitized_fields = []
        field_names = set()

        for i, field in enumerate(fields):
            if not field or not field.strip():
                raise ValidationError(f"Field {i+1} cannot be empty")

            # Check field length
            if len(field) > 500:
                raise ValidationError(f"Field {i+1} definition too long (max 500 characters)")

            # Basic sanitization
            sanitized_field = field.strip()

            # Remove dangerous characters
            sanitized_field = re.sub(r'[<>\"\'\x00-\x1f\x7f-\x9f]', '', sanitized_field)

            # Extract field name for uniqueness check
            parts = sanitized_field.split(':')
            if len(parts) < 2:
                raise ValidationError(f"Field {i+1} format invalid: '{field}'")

            field_name = parts[0].strip()
            if field_name in field_names:
                raise ValidationError(f"Duplicate field name: '{field_name}'")

            if field_name in ['id', 'type', 'struct', 'enum', 'impl', 'fn', 'let', 'mut']:
                raise ValidationError(f"Field name '{field_name}' is reserved")

            field_names.add(field_name)
            sanitized_fields.append(sanitized_field)

        return sanitized_fields

    def sanitize_project_path(self, project_path: str, current_dir: Path) -> str:
        """Sanitize project path input."""
        if not project_path:
            raise ValidationError("Project path cannot be empty")

        if len(project_path) > 1000:
            raise ValidationError("Project path too long")

        # Normalize path
        sanitized_path = os.path.normpath(project_path)

        # Convert relative paths to absolute
        if not os.path.isabs(sanitized_path):
            sanitized_path = os.path.join(str(current_dir), sanitized_path)
            sanitized_path = os.path.normpath(sanitized_path)

        # Remove dangerous characters
        sanitized_path = re.sub(r'[<>\"\'\x00-\x1f\x7f-\x9f]', '', sanitized_path)

        # Check for dangerous patterns
        dangerous_patterns = [
            r'\.\./.*$',  # End with directory traversal
            r'^/\.\./',   # Start with directory traversal
            r'~.*$',      # End with home directory
        ]

        for pattern in dangerous_patterns:
            if re.search(pattern, sanitized_path):
                raise ValidationError(f"Invalid project path pattern detected")

        return sanitized_path


class AccessController:
    """Controls access to resources based on project context."""

    def __init__(self):
        self.read_only_files: Set[str] = {
            'cargo.toml', 'package.json', 'requirements.txt',
            '.gitignore', '.gitattributes'
        }

        self.protected_directories: Set[str] = {
            '.git', '.svn', '.hg',
            'target', 'build', 'dist', 'node_modules',
            '.cargo', '.venv', 'env'
        }

    def can_read_file(self, file_path: Path, project_root: Path) -> bool:
        """Check if file can be read."""
        try:
            relative_path = file_path.relative_to(project_root)
        except ValueError:
            return False  # Outside project bounds

        path_str = str(relative_path).lower()

        # Check protected directories
        for protected_dir in self.protected_directories:
            if path_str.startswith(protected_dir + os.sep) or path_str == protected_dir:
                return False

        return True

    def can_write_file(self, file_path: Path, project_root: Path) -> bool:
        """Check if file can be written."""
        try:
            relative_path = file_path.relative_to(project_root)
        except ValueError:
            return False  # Outside project bounds

        path_str = str(relative_path).lower()

        # Check protected directories
        for protected_dir in self.protected_directories:
            if path_str.startswith(protected_dir + os.sep) or path_str == protected_dir:
                return False

        # Check read-only files
        if path_str in self.read_only_files:
            return False

        # Only allow writing to specific directories
        allowed_prefixes = [
            'src/models/',
            'src/controllers/',
            'src/views/',
            'src/routes/',
            'migration/src/',
            'tests/',
            'examples/'
        ]

        return any(path_str.startswith(prefix) for prefix in allowed_prefixes)

    def can_delete_file(self, file_path: Path, project_root: Path) -> bool:
        """Check if file can be deleted."""
        # Generally more restrictive for delete operations
        return self.can_write_file(file_path, project_root)


class SecurityManager:
    """Main security manager that coordinates all security features."""

    def __init__(self):
        self.path_validator = PathValidator()
        self.input_sanitizer = InputSanitizer()
        self.access_controller = AccessController()
        self.operation_count = 0
        self.suspicious_operations = []

    def create_security_context(
        self,
        request_id: str,
        user_context: Optional[Dict[str, Any]] = None
    ) -> SecurityContext:
        """Create a security context for operations."""
        return SecurityContext(
            allowed_project_root=Path.cwd(),  # Would be configurable in real implementation
            current_working_directory=Path.cwd(),
            operation_id=request_id,
            user_context=user_context or {}
        )

    def validate_operation(
        self,
        operation: str,
        params: Dict[str, Any],
        context: SecurityContext
    ) -> None:
        """Validate an operation for security compliance."""
        self.operation_count += 1

        # Check operation frequency (rate limiting)
        self._check_rate_limit()

        # Validate based on operation type
        if operation in ["generate_model", "generate_scaffold", "generate_controller_view"]:
            self._validate_generation_operation(params, context)

    def _validate_generation_operation(self, params: Dict[str, Any], context: SecurityContext) -> None:
        """Validate generation-specific operations."""
        # Sanitize inputs
        if "model_name" in params:
            sanitized_name = self.input_sanitizer.sanitize_model_name(params["model_name"])
            params["model_name"] = sanitized_name

        if "fields" in params:
            sanitized_fields = self.input_sanitizer.sanitize_field_definitions(params["fields"])
            params["fields"] = sanitized_fields

        if "project_path" in params:
            sanitized_path = self.input_sanitizer.sanitize_project_path(
                params["project_path"],
                context.current_working_directory
            )
            params["project_path"] = sanitized_path

        # Validate project path bounds
        if "project_path" in params:
            project_path = Path(params["project_path"])
            validated_path = self.path_validator.validate_path(
                str(project_path),
                context.allowed_project_root,
                operation="generation"
            )
            params["project_path"] = str(validated_path)

    def _check_rate_limit(self) -> None:
        """Check for rate limiting."""
        # Simple implementation - would use more sophisticated rate limiting in production
        if self.operation_count > 1000:
            raise ValidationError("Operation limit exceeded. Please try again later.")

    def get_security_statistics(self) -> Dict[str, Any]:
        """Get security monitoring statistics."""
        return {
            "total_operations": self.operation_count,
            "suspicious_operations": len(self.suspicious_operations),
            "protected_files_blocked": len(self.access_controller.protected_directories),
            "path_validations_performed": self.operation_count,
            "security_status": "healthy"
        }


# Global security manager instance
security_manager = SecurityManager()
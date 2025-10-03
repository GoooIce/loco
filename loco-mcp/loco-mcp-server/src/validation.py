"""
Comprehensive input validation for MCP tool parameters.

This module provides robust validation for all tool parameters with
detailed error reporting and suggestions for fixing common issues.
"""

import re
import os
from typing import Any, Dict, List, Optional, Union
from dataclasses import dataclass

from .error_handling import ValidationError, error_handler


@dataclass
class ValidationRule:
    """Represents a validation rule for a parameter."""
    name: str
    required: bool = True
    validator: callable = None
    description: str = ""
    error_message: str = ""


class ParameterValidator:
    """Comprehensive parameter validation for MCP tools."""

    def __init__(self):
        self.field_type_pattern = re.compile(r'^[a-z][a-z0-9_]*:(string|i32|i64|f32|f64|boolean|datetime|uuid|json|text)(:unique|:primary_key|:nullable|:optional|:default:[^:]+)*$')
        self.model_name_pattern = re.compile(r'^[a-z][a-z0-9_]{1,63}$')
        self.reserved_keywords = {
            'id', 'type', 'struct', 'enum', 'impl', 'fn', 'let', 'mut',
            'if', 'else', 'for', 'while', 'loop', 'match', 'break', 'continue',
            'return', 'async', 'await', 'mod', 'use', 'pub', 'trait'
        }

    def validate_generate_model_params(self, params: Dict[str, Any]) -> None:
        """Validate parameters for generate_model tool."""
        rules = [
            ValidationRule(
                name="model_name",
                required=True,
                validator=self._validate_model_name,
                description="Model name in snake_case (e.g., 'user_profile')"
            ),
            ValidationRule(
                name="fields",
                required=True,
                validator=self._validate_field_list,
                description="List of field definitions (e.g., ['name:string', 'email:string:unique'])"
            ),
            ValidationRule(
                name="project_path",
                required=False,
                validator=self._validate_project_path,
                description="Path to loco-rs project directory"
            )
        ]

        self._validate_parameters(params, rules, "generate_model")

    def validate_generate_scaffold_params(self, params: Dict[str, Any]) -> None:
        """Validate parameters for generate_scaffold tool."""
        rules = [
            ValidationRule(
                name="model_name",
                required=True,
                validator=self._validate_model_name,
                description="Model name in snake_case"
            ),
            ValidationRule(
                name="fields",
                required=True,
                validator=self._validate_field_list,
                description="List of field definitions"
            ),
            ValidationRule(
                name="include_views",
                required=False,
                validator=self._validate_boolean,
                description="Whether to generate view templates"
            ),
            ValidationRule(
                name="include_controllers",
                required=False,
                validator=self._validate_boolean,
                description="Whether to generate controllers"
            ),
            ValidationRule(
                name="api_only",
                required=False,
                validator=self._validate_boolean,
                description="Generate API-only scaffolding (no views)"
            ),
            ValidationRule(
                name="project_path",
                required=False,
                validator=self._validate_project_path,
                description="Path to loco-rs project directory"
            )
        ]

        self._validate_parameters(params, rules, "generate_scaffold")

        # Additional cross-parameter validation
        self._validate_scaffold_configuration(params)

    def validate_generate_controller_view_params(self, params: Dict[str, Any]) -> None:
        """Validate parameters for generate_controller_view tool."""
        rules = [
            ValidationRule(
                name="model_name",
                required=True,
                validator=self._validate_model_name,
                description="Name of existing model"
            ),
            ValidationRule(
                name="actions",
                required=False,
                validator=self._validate_controller_actions,
                description="List of controller actions to generate"
            ),
            ValidationRule(
                name="view_types",
                required=False,
                validator=self._validate_view_types,
                description="Types of views to generate"
            ),
            ValidationRule(
                name="project_path",
                required=False,
                validator=self._validate_project_path,
                description="Path to loco-rs project directory"
            )
        ]

        self._validate_parameters(params, rules, "generate_controller_view")

    def _validate_parameters(self, params: Dict[str, Any], rules: List[ValidationRule], tool_name: str) -> None:
        """Validate parameters against a list of rules."""
        # Check for unexpected parameters
        expected_params = {rule.name for rule in rules}
        provided_params = set(params.keys())
        unexpected_params = provided_params - expected_params

        if unexpected_params:
            raise ValidationError(
                f"Unexpected parameter(s) for {tool_name}: {', '.join(unexpected_params)}",
                details={"unexpected_parameters": list(unexpected_params), "expected_parameters": list(expected_params)},
                suggestions=[f"Remove unexpected parameter: {param}" for param in unexpected_params]
            )

        # Validate each required parameter
        for rule in rules:
            value = params.get(rule.name)

            if rule.required and value is None:
                raise ValidationError(
                    f"Required parameter '{rule.name}' is missing for {tool_name}",
                    details={"missing_parameter": rule.name},
                    suggestions=[f"Add parameter '{rule.name}': {rule.description}"]
                )

            if value is not None:
                try:
                    rule.validator(value)
                except Exception as e:
                    raise ValidationError(
                        f"Invalid value for parameter '{rule.name}': {e}",
                        details={"parameter": rule.name, "invalid_value": value},
                        suggestions=[f"Fix parameter '{rule.name}': {rule.description}"]
                    ) from e

    def _validate_model_name(self, model_name: Any) -> None:
        """Validate model name."""
        if not isinstance(model_name, str):
            raise ValidationError("Model name must be a string")

        if not model_name:
            raise ValidationError("Model name cannot be empty")

        if not self.model_name_pattern.match(model_name):
            raise ValidationError(
                f"Invalid model name: '{model_name}'. Must be snake_case, start with letter, max 64 characters",
                details={"invalid_name": model_name}
            )

        if model_name in self.reserved_keywords:
            raise ValidationError(
                f"Model name '{model_name}' is a reserved keyword",
                details={"reserved_keyword": model_name},
                suggestions=[f"Use alternative name: {model_name}_model", f"Use alternative name: {model_name}_entity"]
            )

    def _validate_field_list(self, fields: Any) -> None:
        """Validate list of field definitions."""
        if not isinstance(fields, list):
            raise ValidationError("Fields must be a list")

        if not fields:
            raise ValidationError("At least one field must be specified")

        field_names = set()
        for i, field in enumerate(fields):
            if not isinstance(field, str):
                raise ValidationError(f"Field definition at index {i} must be a string")

            if not field.strip():
                raise ValidationError(f"Field definition at index {i} cannot be empty")

            # Validate field format
            if not self.field_type_pattern.match(field.strip()):
                raise ValidationError(
                    f"Invalid field format: '{field}'. Expected format: 'name:type[:constraint]*'",
                    details={
                        "invalid_field": field,
                        "index": i,
                        "expected_format": "name:type[:constraint]",
                        "examples": [
                            "name:string",
                            "email:string:unique",
                            "price:i32",
                            "published_at:datetime:nullable"
                        ]
                    },
                    suggestions=[
                        "Check field format",
                        "Supported types: string, i32, i64, f32, f64, boolean, datetime, uuid, json, text",
                        "Supported constraints: unique, primary_key, nullable, optional, default:<value>"
                    ]
                )

            # Extract field name and check for duplicates
            field_name = field.split(':')[0].strip()
            if field_name in field_names:
                raise ValidationError(
                    f"Duplicate field name: '{field_name}'. Each field name must be unique",
                    details={"duplicate_field": field_name},
                    suggestions=[f"Remove or rename duplicate field: {field_name}"]
                )
            field_names.add(field_name)

            # Validate field name constraints
            if field_name == 'id':
                raise ValidationError(
                    "Field name 'id' is reserved for primary key",
                    details={"reserved_field": "id"},
                    suggestions=["Use a different field name", "Let the system handle the 'id' field automatically"]
                )

    def _validate_boolean(self, value: Any) -> None:
        """Validate boolean parameter."""
        if not isinstance(value, bool):
            raise ValidationError("Parameter must be a boolean (true or false)")

    def _validate_controller_actions(self, actions: Any) -> None:
        """Validate controller actions list."""
        if not isinstance(actions, list):
            raise ValidationError("Actions must be a list")

        valid_actions = {"index", "show", "create", "update", "delete", "edit", "new"}

        for action in actions:
            if not isinstance(action, str):
                raise ValidationError("Each action must be a string")

            if action not in valid_actions:
                raise ValidationError(
                    f"Invalid action: '{action}'",
                    details={
                        "invalid_action": action,
                        "valid_actions": sorted(valid_actions)
                    },
                    suggestions=[f"Use one of: {', '.join(sorted(valid_actions))}"]
                )

    def _validate_view_types(self, view_types: Any) -> None:
        """Validate view types list."""
        if not isinstance(view_types, list):
            raise ValidationError("View types must be a list")

        valid_view_types = {"list", "show", "form", "edit", "new"}

        for view_type in view_types:
            if not isinstance(view_type, str):
                raise ValidationError("Each view type must be a string")

            if view_type not in valid_view_types:
                raise ValidationError(
                    f"Invalid view type: '{view_type}'",
                    details={
                        "invalid_view_type": view_type,
                        "valid_view_types": sorted(valid_view_types)
                    },
                    suggestions=[f"Use one of: {', '.join(sorted(valid_view_types))}"]
                )

    def _validate_project_path(self, project_path: Any) -> None:
        """Validate project path."""
        if not isinstance(project_path, str):
            raise ValidationError("Project path must be a string")

        if not project_path.strip():
            raise ValidationError("Project path cannot be empty")

        # Normalize path
        normalized_path = os.path.normpath(project_path)

        # Check for basic path validity
        if normalized_path.startswith(".."):
            raise ValidationError("Project path cannot reference parent directories")

        # Check for suspicious path patterns
        if "~" in normalized_path and not normalized_path.startswith("~"):
            raise ValidationError("Invalid tilde usage in project path")

    def _validate_scaffold_configuration(self, params: Dict[str, Any]) -> None:
        """Validate cross-parameter constraints for scaffold generation."""
        include_views = params.get("include_views", True)
        include_controllers = params.get("include_controllers", True)
        api_only = params.get("api_only", False)

        if api_only and include_views:
            raise ValidationError(
                "Cannot have both api_only=true and include_views=true",
                details={
                    "api_only": api_only,
                    "include_views": include_views
                },
                suggestions=[
                    "Set api_only=false for full scaffolding",
                    "Set include_views=false for API-only scaffolding"
                ]
            )

        if not include_controllers:
            raise ValidationError(
                "include_controllers=false is not supported yet",
                details={"include_controllers": include_controllers},
                suggestions=["Set include_controllers=true (default)"]
            )

    def validate_field_types(self, field_types: List[str]) -> None:
        """Validate that field types are supported."""
        supported_types = {"string", "i32", "i64", "f32", "f64", "boolean", "datetime", "uuid", "json", "text"}

        for field_type in field_types:
            if field_type not in supported_types:
                raise ValidationError(
                    f"Unsupported field type: '{field_type}'",
                    details={
                        "unsupported_type": field_type,
                        "supported_types": sorted(supported_types)
                    },
                    suggestions=[f"Use one of: {', '.join(sorted(supported_types))}"]
                )

    def validate_field_constraints(self, field_name: str, constraints: List[str]) -> None:
        """Validate field constraints."""
        supported_constraints = {"unique", "primary_key", "nullable", "optional", "default"}

        for constraint in constraints:
            if constraint == "default":
                continue  # Default values have their own validation

            if constraint not in supported_constraints:
                raise ValidationError(
                    f"Unsupported constraint: '{constraint}' for field '{field_name}'",
                    details={
                        "field": field_name,
                        "unsupported_constraint": constraint,
                        "supported_constraints": sorted(supported_constraints)
                    },
                    suggestions=[f"Use one of: {', '.join(sorted(supported_constraints))}"]
                )

        # Check for conflicting constraints
        if "nullable" in constraints and "optional" in constraints:
            raise ValidationError(
                f"Cannot specify both 'nullable' and 'optional' constraints for field '{field_name}'",
                details={"field": field_name, "conflicting_constraints": ["nullable", "optional"]},
                suggestions=["Use either 'nullable' or 'optional' (they are equivalent)"]
            )


# Global validator instance
validator = ParameterValidator()
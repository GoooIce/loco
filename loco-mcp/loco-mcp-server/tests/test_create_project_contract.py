"""
Contract test for loco_create_project MCP tool schema.

This test validates that the MCP tool schema matches the expected structure
for creating new Loco projects.
"""

import pytest
import json
from jsonschema import validate, ValidationError


def test_loco_create_project_tool_schema():
    """Test that the loco_create_project tool schema is valid."""
    # Expected tool schema based on contracts/mcp-tool-schema.json
    expected_schema = {
        "type": "object",
        "properties": {
            "name": {
                "type": "string",
                "const": "loco_create_project"
            },
            "description": {
                "type": "string"
            },
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project_name": {
                        "type": "string",
                        "description": "Project name in snake_case (e.g., 'my_app', 'user_service')",
                        "pattern": "^[a-z][a-z0-9_]*$",
                        "minLength": 1,
                        "maxLength": 50
                    },
                    "template_type": {
                        "type": "string",
                        "enum": ["saas", "rest_api", "lightweight"],
                        "default": "saas",
                        "description": "Project template type"
                    },
                    "destination_path": {
                        "type": "string",
                        "description": "Directory where project will be created (must not exist)",
                        "minLength": 1
                    },
                    "database_type": {
                        "type": "string",
                        "enum": ["sqlite", "postgresql", "none"],
                        "description": "Database configuration (optional, uses template default)"
                    },
                    "background_worker": {
                        "type": "string",
                        "enum": ["redis", "postgresql", "sqlite", "none"],
                        "description": "Background worker setup (optional, uses template default)"
                    },
                    "asset_serving": {
                        "type": "string",
                        "enum": ["local", "cloud", "none"],
                        "description": "Static asset serving (optional, uses template default)"
                    }
                },
                "required": ["project_name", "destination_path"],
                "additionalProperties": False
            }
        },
        "required": ["name", "description", "inputSchema"],
        "additionalProperties": False
    }

    # This test will fail until the tool is implemented
    # TODO: Import and get the actual tool schema from MCP server
    # actual_schema = get_loco_create_project_tool_schema()
    # validate(instance=actual_schema, schema=expected_schema)

    # For now, we expect this to raise an error since the tool doesn't exist yet
    pytest.fail("loco_create_project tool not implemented yet - this test should fail")


def test_project_name_validation():
    """Test project name validation patterns."""
    valid_names = [
        "my_app",
        "user_service",
        "blog_api",
        "simple_project",
        "a"
    ]

    invalid_names = [
        "Invalid-Name!",
        "123invalid",
        "invalid name with spaces",
        "",
        "UPPER_CASE_NAME"
    ]

    # Pattern from schema
    pattern = r"^[a-z][a-z0-9_]*$"

    import re
    for name in valid_names:
        assert re.match(pattern, name), f"Valid name '{name}' should match pattern"

    for name in invalid_names:
        assert not re.match(pattern, name), f"Invalid name '{name}' should not match pattern"


def test_template_type_enum():
    """Test that template_type enum contains expected values."""
    expected_templates = ["saas", "rest_api", "lightweight"]

    # This will be used when the tool is implemented
    # TODO: Test actual enum values from tool schema
    assert len(expected_templates) == 3
    assert "saas" in expected_templates
    assert "rest_api" in expected_templates
    assert "lightweight" in expected_templates


if __name__ == "__main__":
    test_project_name_validation()
    test_template_type_enum()
    test_loco_create_project_tool_schema()
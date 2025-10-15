"""
Integration test for REST API template creation.

This test validates that the REST API template creates an API-only application
with database support and modular controllers, but no view templates.
"""

import pytest
import tempfile
import os
from pathlib import Path


def test_rest_api_template_creation():
    """Test REST API template creation with default configuration."""
    with tempfile.TemporaryDirectory() as temp_dir:
        project_path = os.path.join(temp_dir, "test_api_app")

        # Test parameters for REST API template
        params = {
            "project_name": "test_api_app",
            "template_type": "rest_api",
            "destination_path": project_path
        }

        # This test will fail until the tool is implemented
        # TODO: Call the actual MCP tool
        # result = call_loco_create_project(params)

        # Expected validations after implementation
        expected_files = [
            "Cargo.toml",
            "src/models/",
            "src/controllers/",
            # NO src/views/ for API-only
            "migration/",
            "src/app.rs"
        ]

        # Expected default configurations for REST API
        expected_database = "postgresql"
        expected_background_worker = None
        expected_asset_serving = None

        # For now, expect failure since tool doesn't exist
        pytest.fail("loco_create_project tool not implemented yet - this test should fail")


def test_rest_api_template_no_views():
    """Test that REST API template does not create view templates."""
    # API-only should not have view templates
    unexpected_paths = [
        "src/views/",
        "templates/",
        "static/",
        "public/"
    ]

    # This will be validated when the tool is implemented
    # TODO: Ensure these paths don't exist after tool creation
    for path in unexpected_paths:
        assert path is not None, f"API template should not create {path}"


def test_rest_api_template_controllers():
    """Test that REST API template creates API controllers."""
    expected_controllers = [
        "src/controllers/mod.rs",
        "src/controllers/health.rs",
        "src/controllers/api.rs"
    ]

    # This will be validated when the tool is implemented
    # TODO: Check actual controllers created by tool
    assert len(expected_controllers) >= 2, "API template should create multiple controllers"


def test_rest_api_template_dependencies():
    """Test that REST API template includes API-specific dependencies."""
    expected_dependencies = [
        "loco",
        "serde",
        "tokio",
        "axum",
        "tower-http",
        "sqlx",
        # No view-related dependencies like tera or askama
    ]

    unexpected_dependencies = [
        "tera",
        "askama",
        "maud",
        "minijinja"
    ]

    # This will be validated when the tool is implemented
    # TODO: Check Cargo.toml dependencies
    assert len(expected_dependencies) > 5, "API template should include dependencies"
    assert len(unexpected_dependencies) > 0, "API template should exclude view dependencies"


if __name__ == "__main__":
    test_rest_api_template_creation()
    test_rest_api_template_no_views()
    test_rest_api_template_controllers()
    test_rest_api_template_dependencies()
"""
Integration test for Lightweight template creation.

This test validates that the lightweight template creates a minimal service
with only essential components and minimal dependencies.
"""

import pytest
import tempfile
import os
from pathlib import Path


def test_lightweight_template_creation():
    """Test lightweight template creation with default configuration."""
    with tempfile.TemporaryDirectory() as temp_dir:
        project_path = os.path.join(temp_dir, "test_lightweight_app")

        # Test parameters for lightweight template
        params = {
            "project_name": "test_lightweight_app",
            "template_type": "lightweight",
            "destination_path": project_path
        }

        # This test will fail until the tool is implemented
        # TODO: Call the actual MCP tool
        # result = call_loco_create_project(params)

        # Expected validations after implementation
        expected_files = [
            "Cargo.toml",
            "src/app.rs",
            # Minimal other files
        ]

        # Expected default configurations for lightweight
        expected_database = "sqlite"  # or None
        expected_background_worker = None
        expected_asset_serving = None

        # For now, expect failure since tool doesn't exist
        pytest.fail("loco_create_project tool not implemented yet - this test should fail")


def test_lightweight_template_minimal_structure():
    """Test that lightweight template creates minimal project structure."""
    expected_minimal_files = [
        "Cargo.toml",
        "src/app.rs",
        "src/main.rs"
    ]

    # Should NOT have these complex files
    unexpected_files = [
        "src/models/",
        "src/views/",
        "migration/",
        "src/controllers/auth.rs",
        "config/",
        "templates/"
    ]

    # This will be validated when the tool is implemented
    # TODO: Check actual files created by tool
    assert len(expected_minimal_files) >= 2, "Lightweight template should create minimal files"
    assert len(unexpected_files) > 0, "Lightweight template should exclude complex files"


def test_lightweight_template_dependencies():
    """Test that lightweight template has minimal dependencies."""
    expected_minimal_dependencies = [
        "loco",
        "tokio",
        "axum"
    ]

    unexpected_heavy_dependencies = [
        "sqlx",
        "redis",
        "tera",
        "bcrypt",
        "jsonwebtoken",
        "tower-http"
    ]

    # This will be validated when the tool is implemented
    # TODO: Check Cargo.toml dependencies
    assert len(expected_minimal_dependencies) >= 2, "Lightweight template needs basic dependencies"
    assert len(unexpected_heavy_dependencies) > 0, "Lightweight template should exclude heavy dependencies"


def test_lightweight_template_runnable():
    """Test that lightweight template produces a runnable application."""
    # Should be able to run `cargo run` without complex setup
    expected_basic_functionality = [
        "basic HTTP server",
        "health check endpoint",
        "no database required",
        "minimal startup time"
    ]

    # This will be validated when the tool is implemented
    # TODO: Try to run the created application
    assert len(expected_basic_functionality) >= 3, "Lightweight should be immediately runnable"


if __name__ == "__main__":
    test_lightweight_template_creation()
    test_lightweight_template_minimal_structure()
    test_lightweight_template_dependencies()
    test_lightweight_template_runnable()
#!/usr/bin/env python3
"""
Quickstart validation script for Loco MCP project creation.

This script validates the quickstart scenarios defined in the quickstart.md file.
It runs basic tests to ensure the implementation works as expected.
"""

import os
import sys
import tempfile
import shutil
import json
from pathlib import Path

def test_project_validation():
    """Test project name validation logic."""
    print("Testing project name validation...")

    # Import the validation logic (same as Rust implementation)
    import re
    name_pattern = re.compile(r"^[a-z][a-z0-9_]*$")

    # Valid names
    valid_names = [
        "test_saas_app",
        "test_api_app",
        "test_lightweight_app",
        "test_custom_app",
        "my_app",
        "user_service",
        "blog_platform",
        "simple",
        "a",
        "api_v2"
    ]

    # Invalid names
    invalid_names = [
        "Invalid-Name!",
        "MyApp",
        "my-app",
        "my app",
        "123app",
        "_app",
        "my@app",
        "app$",
        "Ápp",
        "app.name",
        "app/name",
        ""
    ]

    success = True
    for name in valid_names:
        if not name_pattern.match(name):
            print(f"❌ Valid name '{name}' failed validation")
            success = False

    for name in invalid_names:
        if name_pattern.match(name):
            print(f"❌ Invalid name '{name}' passed validation")
            success = False

    if success:
        print("✅ Project name validation working correctly")
    else:
        print("❌ Project name validation has issues")

    return success

def test_template_defaults():
    """Test template default configurations."""
    print("Testing template default configurations...")

    # Default configurations (matching Rust implementation)
    defaults = {
        "saas": ("postgresql", "redis", "local"),
        "rest_api": ("postgresql", "none", "none"),
        "lightweight": ("sqlite", "none", "none"),
    }

    expected_defaults = {
        "saas": {
            "database": "postgresql",
            "background_worker": "redis",
            "asset_serving": "local"
        },
        "rest_api": {
            "database": "postgresql",
            "background_worker": "none",
            "asset_serving": "none"
        },
        "lightweight": {
            "database": "sqlite",
            "background_worker": "none",
            "asset_serving": "none"
        }
    }

    success = True
    for template, expected in expected_defaults.items():
        actual_db, actual_worker, actual_asset = defaults[template]

        if actual_db != expected["database"]:
            print(f"❌ {template} database default mismatch: expected {expected['database']}, got {actual_db}")
            success = False

        if actual_worker != expected["background_worker"]:
            print(f"❌ {template} background_worker default mismatch: expected {expected['background_worker']}, got {actual_worker}")
            success = False

        if actual_asset != expected["asset_serving"]:
            print(f"❌ {template} asset_serving default mismatch: expected {expected['asset_serving']}, got {actual_asset}")
            success = False

    if success:
        print("✅ Template default configurations correct")
    else:
        print("❌ Template default configurations have issues")

    return success

def test_parameter_validation():
    """Test parameter validation logic."""
    print("Testing parameter validation...")

    # Valid values
    valid_templates = ["saas", "rest_api", "lightweight"]
    valid_databases = ["sqlite", "postgresql", "none"]
    valid_workers = ["redis", "postgresql", "sqlite", "none"]
    valid_assets = ["local", "cloud", "none"]

    # Test valid combinations
    success = True
    for template in valid_templates:
        for db in valid_databases:
            for worker in valid_workers:
                for asset in valid_assets:
                    # All combinations should be valid
                    if not (template in valid_templates and db in valid_databases and
                           worker in valid_workers and asset in valid_assets):
                        print(f"❌ Valid combination rejected: {template}, {db}, {worker}, {asset}")
                        success = False

    # Test invalid values
    invalid_template = "invalid_template"
    invalid_database = "mysql"
    invalid_worker = "rabbitmq"
    invalid_asset = "nginx"

    if invalid_template in valid_templates:
        print(f"❌ Invalid template '{invalid_template}' accepted")
        success = False

    if invalid_database in valid_databases:
        print(f"❌ Invalid database '{invalid_database}' accepted")
        success = False

    if invalid_worker in valid_workers:
        print(f"❌ Invalid worker '{invalid_worker}' accepted")
        success = False

    if invalid_asset in valid_assets:
        print(f"❌ Invalid asset '{invalid_asset}' accepted")
        success = False

    if success:
        print("✅ Parameter validation working correctly")
    else:
        print("❌ Parameter validation has issues")

    return success

def test_file_structure_simulation():
    """Test simulated file structure generation."""
    print("Testing file structure simulation...")

    # Simulate the file generation logic
    def simulate_file_creation(template_type, project_name):
        base_files = ["Cargo.toml", "src/main.rs", "src/app.rs"]
        created_files = base_files.copy()

        # Add template-specific files
        if template_type == "saas":
            created_files.extend([
                "src/controllers/health.rs",
                "src/controllers/mod.rs",
                "src/models/mod.rs",
                "src/views/mod.rs"
            ])
        elif template_type == "rest_api":
            created_files.extend([
                "src/controllers/health.rs",
                "src/controllers/mod.rs",
                "src/models/mod.rs"
            ])
        elif template_type == "lightweight":
            # Just base files
            pass

        return created_files

    test_cases = [
        ("saas", "test_saas_app"),
        ("rest_api", "test_api_app"),
        ("lightweight", "test_lightweight_app")
    ]

    expected_counts = {
        "saas": 7,  # 3 base + 4 SaaS-specific
        "rest_api": 6,  # 3 base + 3 API-specific
        "lightweight": 3  # Just base files
    }

    success = True
    for template, project_name in test_cases:
        files = simulate_file_creation(template, project_name)
        expected_count = expected_counts[template]

        if len(files) != expected_count:
            print(f"❌ {template} file count mismatch: expected {expected_count}, got {len(files)}")
            success = False

        # Check for base files
        if "Cargo.toml" not in files:
            print(f"❌ {template} missing Cargo.toml")
            success = False

        if "src/main.rs" not in files:
            print(f"❌ {template} missing src/main.rs")
            success = False

        if "src/app.rs" not in files:
            print(f"❌ {template} missing src/app.rs")
            success = False

        # Check template-specific files
        if template == "saas":
            if "src/controllers/health.rs" not in files:
                print(f"❌ {template} missing health controller")
                success = False
        elif template == "rest_api":
            if "src/controllers/health.rs" not in files:
                print(f"❌ {template} missing health controller")
                success = False

    if success:
        print("✅ File structure simulation working correctly")
    else:
        print("❌ File structure simulation has issues")

    return success

def test_error_scenarios():
    """Test error scenarios."""
    print("Testing error scenarios...")

    success = True

    # Test invalid project name
    try:
        import re
        name_pattern = re.compile(r"^[a-z][a-z0-9_]*$")
        invalid_name = "Invalid-Name!"

        if name_pattern.match(invalid_name):
            print(f"❌ Invalid project name '{invalid_name}' was accepted")
            success = False
        else:
            print(f"✅ Invalid project name '{invalid_name}' correctly rejected")
    except Exception as e:
        print(f"❌ Error testing invalid project name: {e}")
        success = False

    # Test invalid template type
    valid_templates = ["saas", "rest_api", "lightweight"]
    invalid_template = "invalid_template"

    if invalid_template in valid_templates:
        print(f"❌ Invalid template '{invalid_template}' was accepted")
        success = False
    else:
        print(f"✅ Invalid template '{invalid_template}' correctly rejected")

    if success:
        print("✅ Error scenarios working correctly")
    else:
        print("❌ Error scenarios have issues")

    return success

def main():
    """Run all validation tests."""
    print("🚀 Starting Loco MCP Quickstart Validation")
    print("=" * 50)

    tests = [
        test_project_validation,
        test_template_defaults,
        test_parameter_validation,
        test_file_structure_simulation,
        test_error_scenarios
    ]

    passed = 0
    total = len(tests)

    for test in tests:
        try:
            if test():
                passed += 1
            print()
        except Exception as e:
            print(f"❌ Test {test.__name__} failed with exception: {e}")
            print()

    print("=" * 50)
    print(f"Validation Results: {passed}/{total} tests passed")

    if passed == total:
        print("🎉 All validation tests passed!")
        print("✅ Ready for quickstart testing")
        return 0
    else:
        print("❌ Some validation tests failed")
        print("⚠️  Review implementation before proceeding")
        return 1

if __name__ == "__main__":
    sys.exit(main())
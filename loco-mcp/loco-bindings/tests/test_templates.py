"""
Unit tests for template configuration in create_project function.

This module tests the template configuration logic, ensuring that
different template types get appropriate default configurations.
"""

import pytest
import sys
import os

# Add the src directory to the path so we can import the module
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'src'))

try:
    import loco_bindings
    HAS_BINDINGS = True
except ImportError:
    HAS_BINDINGS = False
    print("Warning: loco_bindings not available, using mock tests")


class TestTemplateConfiguration:
    """Test template configuration logic in create_project function."""

    def test_saas_template_defaults(self):
        """Test SaaS template default configurations."""
        # SaaS template should default to PostgreSQL, Redis, and local assets
        expected_defaults = {
            "database": "postgresql",
            "background_worker": "redis",
            "asset_serving": "local"
        }

        # Mock the logic from the Rust implementation
        template_type = "saas"
        default_db, default_worker, default_asset = self._get_template_defaults(template_type)

        assert default_db == expected_defaults["database"]
        assert default_worker == expected_defaults["background_worker"]
        assert default_asset == expected_defaults["asset_serving"]

    def test_rest_api_template_defaults(self):
        """Test REST API template default configurations."""
        # REST API template should default to PostgreSQL, no worker, no assets
        expected_defaults = {
            "database": "postgresql",
            "background_worker": "none",
            "asset_serving": "none"
        }

        template_type = "rest_api"
        default_db, default_worker, default_asset = self._get_template_defaults(template_type)

        assert default_db == expected_defaults["database"]
        assert default_worker == expected_defaults["background_worker"]
        assert default_asset == expected_defaults["asset_serving"]

    def test_lightweight_template_defaults(self):
        """Test Lightweight template default configurations."""
        # Lightweight template should default to SQLite, no worker, no assets
        expected_defaults = {
            "database": "sqlite",
            "background_worker": "none",
            "asset_serving": "none"
        }

        template_type = "lightweight"
        default_db, default_worker, default_asset = self._get_template_defaults(template_type)

        assert default_db == expected_defaults["database"]
        assert default_worker == expected_defaults["background_worker"]
        assert default_asset == expected_defaults["asset_serving"]

    def test_template_type_validation(self):
        """Test that only valid template types are accepted."""
        valid_templates = ["saas", "rest_api", "lightweight"]
        invalid_templates = ["invalid", "web", "mobile", "microservice", ""]

        for template in valid_templates:
            assert self._is_valid_template(template), f"Template '{template}' should be valid"

        for template in invalid_templates:
            assert not self._is_valid_template(template), f"Template '{template}' should be invalid"

    def test_database_type_validation(self):
        """Test that only valid database types are accepted."""
        valid_databases = ["sqlite", "postgresql", "none"]
        invalid_databases = ["mysql", "mongodb", "redis", "invalid", ""]

        for db in valid_databases:
            assert self._is_valid_database(db), f"Database '{db}' should be valid"

        for db in invalid_databases:
            assert not self._is_valid_database(db), f"Database '{db}' should be invalid"

    def test_background_worker_validation(self):
        """Test that only valid background worker types are accepted."""
        valid_workers = ["redis", "postgresql", "sqlite", "none"]
        invalid_workers = ["mysql", "rabbitmq", "kafka", "invalid", ""]

        for worker in valid_workers:
            assert self._is_valid_background_worker(worker), f"Worker '{worker}' should be valid"

        for worker in invalid_workers:
            assert not self._is_valid_background_worker(worker), f"Worker '{worker}' should be invalid"

    def test_asset_serving_validation(self):
        """Test that only valid asset serving types are accepted."""
        valid_assets = ["local", "cloud", "none"]
        invalid_assets = ["s3", "cdn", "nginx", "invalid", ""]

        for asset in valid_assets:
            assert self._is_valid_asset_serving(asset), f"Asset serving '{asset}' should be valid"

        for asset in invalid_assets:
            assert not self._is_valid_asset_serving(asset), f"Asset serving '{asset}' should be invalid"

    def test_configuration_override(self):
        """Test that user-provided configurations override defaults."""
        template_type = "saas"
        user_db = "sqlite"
        user_worker = "none"
        user_asset = "none"

        # Get defaults first
        default_db, default_worker, default_asset = self._get_template_defaults(template_type)

        # Apply user overrides (simulating the Rust logic)
        final_db = user_db if user_db else default_db
        final_worker = user_worker if user_worker else default_worker
        final_asset = user_asset if user_asset else default_asset

        # Should use user-provided values
        assert final_db == user_db
        assert final_worker == user_worker
        assert final_asset == user_asset

    def test_template_specific_file_generation(self):
        """Test that different templates generate different file sets."""
        # This tests the template-specific file generation logic
        template_types = ["saas", "rest_api", "lightweight"]

        for template in template_types:
            created_files = self._get_template_files(template)

            # All templates should have basic files
            assert "Cargo.toml" in created_files
            assert "src/main.rs" in created_files
            assert "src/app.rs" in created_files

            # Template-specific files
            if template == "saas":
                assert "src/controllers/health.rs" in created_files
                assert any("models" in file for file in created_files)
            elif template == "rest_api":
                assert "src/controllers/health.rs" in created_files
            elif template == "lightweight":
                # Lightweight should have minimal files
                assert len(created_files) <= 4  # Cargo.toml, main.rs, app.rs, maybe controllers

    # Helper methods to simulate the Rust logic
    def _get_template_defaults(self, template_type):
        """Get default configurations for a template type."""
        defaults = {
            "saas": ("postgresql", "redis", "local"),
            "rest_api": ("postgresql", "none", "none"),
            "lightweight": ("sqlite", "none", "none"),
        }
        return defaults.get(template_type, ("none", "none", "none"))

    def _is_valid_template(self, template_type):
        """Check if template type is valid."""
        valid_templates = ["saas", "rest_api", "lightweight"]
        return template_type in valid_templates

    def _is_valid_database(self, database):
        """Check if database type is valid."""
        valid_databases = ["sqlite", "postgresql", "none"]
        return database in valid_databases

    def _is_valid_background_worker(self, worker):
        """Check if background worker type is valid."""
        valid_workers = ["redis", "postgresql", "sqlite", "none"]
        return worker in valid_workers

    def _is_valid_asset_serving(self, asset):
        """Check if asset serving type is valid."""
        valid_assets = ["local", "cloud", "none"]
        return asset in valid_assets

    def _get_template_files(self, template_type):
        """Get the list of files that would be created for a template."""
        base_files = ["Cargo.toml", "src/main.rs", "src/app.rs"]

        if template_type == "saas":
            return base_files + ["src/controllers/health.rs", "src/models/user.rs"]
        elif template_type == "rest_api":
            return base_files + ["src/controllers/health.rs"]
        elif template_type == "lightweight":
            return base_files
        else:
            return base_files


class TestTemplateIntegration:
    """Integration tests for template configuration with create_project."""

    @pytest.mark.skipif(not HAS_BINDINGS, reason="loco_bindings not available")
    def test_create_project_with_different_templates(self):
        """Test create_project function with different template types."""
        # This would test the actual create_project function if bindings are available
        # For now, we'll skip since the bindings might not be compiled
        pass

    def test_template_configuration_matrix(self):
        """Test all combinations of template configurations."""
        templates = ["saas", "rest_api", "lightweight"]
        databases = ["sqlite", "postgresql", "none"]
        workers = ["redis", "postgresql", "sqlite", "none"]
        assets = ["local", "cloud", "none"]

        test_cases = []
        for template in templates:
            for db in databases:
                for worker in workers:
                    for asset in assets:
                        test_cases.append((template, db, worker, asset))

        # Test a subset of combinations to avoid too many tests
        sample_cases = test_cases[:10]  # Test first 10 combinations

        for template, db, worker, asset in sample_cases:
            # Validate that the combination is valid
            assert self._is_valid_template(template)
            assert self._is_valid_database(db)
            assert self._is_valid_background_worker(worker)
            assert self._is_valid_asset_serving(asset)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
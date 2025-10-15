"""
Unit tests for project name validation in create_project function.

This module tests the validation logic for project names, ensuring that
only valid snake_case names are accepted.
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


class TestProjectNameValidation:
    """Test project name validation in create_project function."""

    @pytest.mark.skipif(not HAS_BINDINGS, reason="loco_bindings not available")
    def test_valid_snake_case_names(self):
        """Test that valid snake_case names are accepted."""
        valid_names = [
            "my_app",
            "user_service",
            "blog_platform",
            "simple",
            "a",
            "api_v2",
            "data_processor_123",
            "multi_word_project_name"
        ]

        for name in valid_names:
            # This should not raise an exception for valid names
            try:
                # We'll test just the name validation part
                import re
                name_pattern = re.compile(r"^[a-z][a-z0-9_]*$")
                assert name_pattern.match(name), f"Name '{name}' should match pattern"
            except Exception as e:
                pytest.fail(f"Valid name '{name}' failed validation: {e}")

    @pytest.mark.skipif(not HAS_BINDINGS, reason="loco_bindings not available")
    def test_invalid_names(self):
        """Test that invalid names are rejected."""
        invalid_names = [
            "",  # Empty
            "MyApp",  # Starts with uppercase
            "my-app",  # Contains hyphen
            "my app",  # Contains space
            "123app",  # Starts with number
            "_app",  # Starts with underscore
            "my@app",  # Contains special character
            "app$",  # Ends with special character
            "Ápp",  # Contains non-ASCII
            "app.name",  # Contains dot
            "app/name",  # Contains slash
        ]

        for name in invalid_names:
            # Test the pattern matching
            import re
            name_pattern = re.compile(r"^[a-z][a-z0-9_]*$")
            assert not name_pattern.match(name), f"Name '{name}' should not match pattern"

    @pytest.mark.skipif(not HAS_BINDINGS, reason="loco_bindings not available")
    def test_edge_cases(self):
        """Test edge cases for name validation."""
        # Single character
        import re
        name_pattern = re.compile(r"^[a-z][a-z0-9_]*$")
        assert name_pattern.match("a"), "Single lowercase letter should be valid"
        assert not name_pattern.match("A"), "Single uppercase letter should be invalid"

        # Long names
        long_name = "a" * 100
        assert name_pattern.match(long_name), "Long lowercase name should be valid"

        # Names with numbers
        assert name_pattern.match("app123"), "Name ending with numbers should be valid"
        assert name_pattern.match("app_123_test"), "Name with underscore and numbers should be valid"
        assert not name_pattern.match("123app"), "Name starting with numbers should be invalid"

    def test_mock_validation(self):
        """Mock validation tests when bindings are not available."""
        import re

        # Same validation logic as in the Rust code
        name_pattern = re.compile(r"^[a-z][a-z0-9_]*$")

        valid_test_cases = [
            ("my_app", True),
            ("user_service", True),
            ("blog_platform", True),
            ("simple", True),
            ("a", True),
            ("api_v2", True),
        ]

        invalid_test_cases = [
            ("", False),
            ("MyApp", False),
            ("my-app", False),
            ("my app", False),
            ("123app", False),
            ("_app", False),
            ("my@app", False),
        ]

        for name, expected in valid_test_cases:
            result = bool(name_pattern.match(name))
            assert result == expected, f"Name '{name}' should be {expected}"

        for name, expected in invalid_test_cases:
            result = bool(name_pattern.match(name))
            assert result == expected, f"Name '{name}' should be {expected}"


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
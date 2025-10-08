#!/usr/bin/env python3
"""
Comprehensive unit tests for loco-mcp-server

This test suite provides comprehensive coverage of all server functionality
including MCP tool handling, validation, error handling, and performance.
"""

import pytest
import asyncio
import json
import tempfile
import os
import sys
from pathlib import Path
from unittest.mock import Mock, patch, AsyncMock
from typing import Dict, Any

# Add the source directory to Python path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'src'))

from src.server import LocoMCPServer
from src.tools import LocoTools
from src.config import ServerConfig
from src.validation import ExecutionAssuranceValidator, ExecutionAssuranceRecord
from src.security import PathValidator, InputSanitizer, AccessController
from src.error_handling import (
    ValidationError, FileOperationError, ProjectError,
    TemplateError, PerformanceError, ErrorHandler
)


class TestServerConfig:
    """Test server configuration functionality"""

    def test_default_config(self):
        """Test default configuration values"""
        config = ServerConfig()

        assert config.host == "localhost"
        assert config.port == 8080
        assert config.log_level == "INFO"
        assert config.default_project_path == "."
        assert config.version == "0.1.0"

    def test_custom_config(self):
        """Test custom configuration values"""
        config = ServerConfig(
            host="0.0.0.0",
            port=9000,
            log_level="DEBUG",
            default_project_path="/custom/path"
        )

        assert config.host == "0.0.0.0"
        assert config.port == 9000
        assert config.log_level == "DEBUG"
        assert config.default_project_path == "/custom/path"

    def test_config_validation(self):
        """Test configuration validation"""
        # Valid configurations
        valid_configs = [
            {"host": "localhost", "port": 8080},
            {"log_level": "DEBUG"},
            {"default_project_path": "/valid/path"}
        ]

        for config_dict in valid_configs:
            config = ServerConfig(**config_dict)
            assert config is not None

        # Invalid configurations would raise validation errors
        # (Implementation would validate ranges, formats, etc.)


class TestParameterValidator:
    """Test parameter validation functionality"""

    def setup_method(self):
        """Setup test validator"""
        self.validator = ParameterValidator()

    def test_validate_model_name_valid(self):
        """Test valid model names"""
        valid_names = [
            "user",
            "blog_post",
            "api_key",
            "user_profile",
            "order_item"
        ]

        for name in valid_names:
            result = self.validator.validate_model_name(name)
            assert result["valid"], f"Model name '{name}' should be valid"
            assert "errors" not in result or len(result["errors"]) == 0

    def test_validate_model_name_invalid(self):
        """Test invalid model names"""
        invalid_names = [
            "123user",
            "user@domain",
            "user-name",
            "User",
            "user name",
            "",
            "user_with_very_long_name_that_exceeds_limit"
        ]

        for name in invalid_names:
            result = self.validator.validate_model_name(name)
            assert not result["valid"], f"Model name '{name}' should be invalid"
            assert "errors" in result
            assert len(result["errors"]) > 0
            assert "suggestions" in result

    def test_validate_field_list_valid(self):
        """Test valid field definitions"""
        valid_fields = [
            ["name:string"],
            ["email:string:unique"],
            ["age:i32", "name:string"],
            ["content:text", "published:boolean", "created_at:datetime"],
            ["metadata:json", "id:uuid", "price:f64"]
        ]

        for fields in valid_fields:
            result = self.validator.validate_field_list(fields)
            assert result["valid"], f"Field list {fields} should be valid"
            assert "errors" not in result or len(result["errors"]) == 0

    def test_validate_field_list_invalid(self):
        """Test invalid field definitions"""
        invalid_fields = [
            [""],  # Empty field
            ["name"],  # No type
            ["email:"],  # Empty type
            ["age:invalid_type"],  # Invalid type
            ["name:string:invalid_constraint"],  # Invalid constraint
            ["123name:string"],  # Invalid name
            ["name:string", "name:string"]  # Duplicate
        ]

        for fields in invalid_fields:
            result = self.validator.validate_field_list(fields)
            assert not result["valid"], f"Field list {fields} should be invalid"
            assert "errors" in result
            assert len(result["errors"]) > 0

    def test_validate_controller_actions(self):
        """Test controller action validation"""
        valid_actions = [
            ["index"],
            ["index", "show", "create", "update", "delete"],
            ["index", "show"]
        ]

        for actions in valid_actions:
            result = self.validator.validate_actions(actions)
            assert result["valid"], f"Actions {actions} should be valid"

        invalid_actions = [
            ["invalid_action"],
            ["index", "invalid_action"]
        ]

        for actions in invalid_actions:
            result = self.validator.validate_actions(actions)
            assert not result["valid"], f"Actions {actions} should be invalid"

    def test_validate_complete_params(self):
        """Test complete parameter validation"""
        valid_params = {
            "model_name": "user",
            "fields": ["name:string", "email:string:unique"],
            "project_path": "/valid/path"
        }

        result = self.validator.validate_generate_model_params(valid_params)
        assert result["valid"]
        assert "errors" not in result or len(result["errors"]) == 0

        invalid_params = {
            "model_name": "123invalid",
            "fields": ["name"],
            "project_path": ""
        }

        result = self.validator.validate_generate_model_params(invalid_params)
        assert not result["valid"]
        assert len(result["errors"]) > 0


class TestSecurity:
    """Test security functionality"""

    def setup_method(self):
        """Setup test security components"""
        self.path_validator = PathValidator()
        self.input_sanitizer = InputSanitizer()
        self.access_controller = AccessController()

    def test_path_validation_safe(self):
        """Test safe path validation"""
        safe_paths = [
            "user",
            "src/models/user.rs",
            "views/user/list.html.tera",
            "/relative/path/to/file.txt"
        ]

        for path in safe_paths:
            assert self.path_validator.is_safe_path(path), f"Path '{path}' should be safe"

    def test_path_validation_dangerous(self):
        """Test dangerous path validation"""
        dangerous_paths = [
            "../../../etc/passwd",
            "/etc/passwd",
            "..\\..\\windows\\system32\\config",
            "C:\\Windows\\System32\\config",
            "~/.ssh/id_rsa",
            "/etc/shadow"
        ]

        for path in dangerous_paths:
            assert not self.path_validator.is_safe_path(path), f"Path '{path}' should be dangerous"

    def test_input_sanitization(self):
        """Test input sanitization"""
        dangerous_inputs = [
            "'; DROP TABLE users; --",
            "<script>alert('xss')</script>",
            "$(rm -rf /)",
            "{{7*7}}",  # Template injection
            "'; exec('rm -rf /'); #"
        ]

        for input_str in dangerous_inputs:
            sanitized = self.input_sanitizer.sanitize_string(input_str)
            assert "'" not in sanitized
            assert ";" not in sanitized
            assert "$" not in sanitized
            assert "<" not in sanitized
            assert ">" not in sanitized

    def test_access_control(self):
        """Test access control"""
        allowed_paths = [
            "src/models/user.rs",
            "src/controllers/user.rs",
            "src/views/user/list.html.tera",
            "migration/src/m_20240101_create_users.rs"
        ]

        for path in allowed_paths:
            assert self.access_controller.can_create_file(path), f"Should be able to create {path}"

        forbidden_paths = [
            "/etc/passwd",
            "../../../etc/passwd",
            "~/.ssh/id_rsa",
            "/root/.bashrc"
        ]

        for path in forbidden_paths:
            assert not self.access_controller.can_create_file(path), f"Should not be able to create {path}"


class TestMessageFormatter:
    """Test message formatting functionality"""

    def setup_method(self):
        """Setup test message formatter"""
        self.formatter = MessageFormatter()

    def test_format_success_message(self):
        """Test success message formatting"""
        result = self.formatter.format_success(
            "Operation completed successfully",
            {
                "created_files": ["file1.rs", "file2.rs"],
                "processing_time_ms": 5.2
            }
        )

        assert result["type"] == "success"
        assert result["message"] == "Operation completed successfully"
        assert "context" in result
        assert "suggestions" in result
        assert len(result["suggestions"]) > 0

    def test_format_error_message(self):
        """Test error message formatting"""
        result = self.formatter.format_error(
            "Validation failed",
            "validation_error",
            {
                "model_name": "123invalid",
                "suggestions": ["Use snake_case naming"]
            }
        )

        assert result["type"] == "error"
        assert "Validation failed" in result["message"]
        assert "context" in result
        assert "suggestions" in result
        assert len(result["suggestions"]) > 0

    def test_format_warning_message(self):
        """Test warning message formatting"""
        result = self.formatter.format_warning(
            "Performance concern detected",
            {
                "processing_time_ms": 15.5,
                "target_ms": 10.0
            }
        )

        assert result["type"] == "warning"
        assert "Performance concern" in result["message"]
        assert "context" in result
        assert "suggestions" in result

    def test_format_info_message(self):
        """Test info message formatting"""
        result = self.formatter.format_info(
            "Template cache warmed successfully",
            {
                "cache_size": 25,
                "hit_rate": 0.85
            }
        )

        assert result["type"] == "info"
        assert result["message"] == "Template cache warmed successfully"
        assert "context" in result


class TestErrorHandler:
    """Test error handling functionality"""

    def setup_method(self):
        """Setup test error handler"""
        self.error_handler = ErrorHandler()

    def test_error_recording(self):
        """Test error recording and statistics"""
        # Record some errors
        self.error_handler.record_error("validation", "Invalid model name")
        self.error_handler.record_error("file_operation", "Permission denied")
        self.error_handler.record_error("validation", "Invalid field type")

        stats = self.error_handler.get_error_stats()

        assert stats["total_errors"] == 3
        assert stats["error_types"]["validation"] == 2
        assert stats["error_types"]["file_operation"] == 1
        assert stats["error_rate"] > 0.0

    def test_error_conversion(self):
        """Test Python error to MCP error conversion"""
        # Test ValidationError
        validation_error = ValidationError("Invalid input", ["Use proper format"])
        mcp_error = self.error_handler.convert_to_mcp_error(validation_error, "test_tool")

        assert mcp_error["code"] == "VALIDATION_ERROR"
        assert mcp_error["message"] == "Invalid input"
        assert mcp_error["tool_name"] == "test_tool"
        assert "suggestions" in mcp_error

        # Test FileOperationError
        file_error = FileOperationError("Permission denied", "/path/to/file")
        mcp_error = self.error_handler.convert_to_mcp_error(file_error, "test_tool")

        assert mcp_error["code"] == "FILE_OPERATION_ERROR"
        assert mcp_error["message"] == "Permission denied"

    def test_error_recovery_suggestions(self):
        """Test error recovery suggestions"""
        validation_error = ValidationError(
            "Model name '123invalid' is invalid",
            ["Use snake_case naming", "Start with a letter"]
        )

        suggestions = self.error_handler.get_recovery_suggestions(validation_error)

        assert len(suggestions) >= 2
        assert any("snake_case" in s for s in suggestions)
        assert any("letter" in s for s in suggestions)


class TestLocoTools:
    """Test LocoTools functionality"""

    def setup_method(self):
        """Setup test tools"""
        self.tools = LocoTools()
        self.temp_dir = tempfile.mkdtemp()

        # Create a minimal loco project structure
        os.makedirs(os.path.join(self.temp_dir, "src", "models"))
        os.makedirs(os.path.join(self.temp_dir, "src", "controllers"))
        os.makedirs(os.path.join(self.temp_dir, "src", "views"))
        os.makedirs(os.path.join(self.temp_dir, "migration", "src"))

        # Create Cargo.toml
        cargo_toml = """
[package]
name = "test-app"
version = "0.1.0"
edition = "2021"

[dependencies]
loco-rs = "0.3"
"""
        with open(os.path.join(self.temp_dir, "Cargo.toml"), "w") as f:
            f.write(cargo_toml)

    def teardown_method(self):
        """Cleanup test directory"""
        import shutil
        shutil.rmtree(self.temp_dir, ignore_errors=True)

    @pytest.mark.asyncio
    async def test_generate_model_success(self):
        """Test successful model generation"""
        params = {
            "model_name": "user",
            "fields": ["name:string", "email:string:unique"],
            "project_path": self.temp_dir
        }

        result = await self.tools.generate_model(params)

        assert result["success"]
        assert len(result["created_files"]) == 2  # model + migration
        assert len(result["errors"]) == 0

    @pytest.mark.asyncio
    async def test_generate_model_validation_error(self):
        """Test model generation with validation error"""
        params = {
            "model_name": "123invalid",
            "fields": ["name"],
            "project_path": self.temp_dir
        }

        result = await self.tools.generate_model(params)

        assert not result["success"]
        assert len(result["errors"]) > 0
        assert any("invalid" in error.lower() for error in result["errors"])

    @pytest.mark.asyncio
    async def test_generate_scaffold_success(self):
        """Test successful scaffold generation"""
        params = {
            "model_name": "blog_post",
            "fields": ["title:string", "content:text"],
            "include_views": True,
            "include_controllers": True,
            "project_path": self.temp_dir
        }

        result = await self.tools.generate_scaffold(params)

        assert result["success"]
        assert len(result["created_files"]) >= 4  # model, migration, controller, views

    @pytest.mark.asyncio
    async def test_generate_scaffold_api_only(self):
        """Test API-only scaffold generation"""
        params = {
            "model_name": "api_key",
            "fields": ["key:string:unique", "name:string"],
            "api_only": True,
            "project_path": self.temp_dir
        }

        result = await self.tools.generate_scaffold(params)

        assert result["success"]
        assert len(result["created_files"]) >= 3  # model, migration, controller

        # Verify no views were created
        view_dir = os.path.join(self.temp_dir, "src", "views", "api_key")
        assert not os.path.exists(view_dir)

    @pytest.mark.asyncio
    async def test_generate_controller_view_success(self):
        """Test successful controller/view generation"""
        # First create a model
        model_params = {
            "model_name": "user",
            "fields": ["name:string", "email:string"],
            "project_path": self.temp_dir
        }
        await self.tools.generate_model(model_params)

        # Then generate controller
        controller_params = {
            "model_name": "user",
            "actions": ["index", "show"],
            "project_path": self.temp_dir
        }

        result = await self.tools.generate_controller_view(controller_params)

        assert result["success"]
        assert len(result["created_files"]) >= 1  # controller

    @pytest.mark.asyncio
    async def test_performance_monitoring(self):
        """Test performance monitoring during operations"""
        params = {
            "model_name": "user",
            "fields": ["name:string"],
            "project_path": self.temp_dir
        }

        # Generate multiple models to create performance data
        for i in range(3):
            params["model_name"] = f"user_{i}"
            result = await self.tools.generate_model(params)
            assert result["success"]

        # Check that performance metrics are being collected
        # (This would require access to the performance monitoring system)
        metrics = await self.tools.get_performance_metrics()

        assert metrics["total_calls"] >= 3
        assert metrics["avg_duration_ms"] > 0
        assert "hit_rate" in metrics


class TestLocoMCPServer:
    """Test main server functionality"""

    def setup_method(self):
        """Setup test server"""
        self.config = ServerConfig(
            host="localhost",
            port=8081,  # Use different port for testing
            log_level="DEBUG"
        )
        self.server = LocoMCPServer(self.config)

    def test_server_initialization(self):
        """Test server initialization"""
        assert self.server.config == self.config
        assert self.server.tools is not None
        assert self.server.start_time is None
        assert self.server.request_count == 0
        assert self.server.error_count == 0

    def test_health_status_initial(self):
        """Test initial health status"""
        status = self.server.get_health_status()

        assert status["status"] == "stopped"
        assert status["requests_handled"] == 0
        assert status["errors"] == 0
        assert status["error_rate_percent"] == 0.0
        assert status["tools_available"] == 3
        assert status["version"] == self.config.version

    @pytest.mark.asyncio
    async def test_tool_call_handling(self):
        """Test tool call handling"""
        # Mock the underlying server
        self.server.server = Mock()

        # Test successful tool call
        result = await self.server._handle_tool_call("generate_model", {
            "model_name": "test",
            "fields": ["name:string"]
        })

        assert result["status"] == "success"
        assert "result" in result
        assert "metadata" in result
        assert result["metadata"]["tool_name"] == "generate_model"
        assert "processing_time_ms" in result["metadata"]

    @pytest.mark.asyncio
    async def test_tool_call_error_handling(self):
        """Test tool call error handling"""
        # Mock the underlying server
        self.server.server = Mock()

        # Test error case
        result = await self.server._handle_tool_call("generate_model", {
            "model_name": "",  # Invalid
            "fields": []
        })

        assert result["status"] == "error"
        assert "error" in result
        assert result["error"]["code"] in ["VALIDATION_ERROR", "UNKNOWN_ERROR"]
        assert result["error"]["tool_name"] == "generate_model"

    def test_error_code_mapping(self):
        """Test error code mapping"""
        test_cases = [
            (ValueError("test"), "VALIDATION_ERROR"),
            (FileExistsError("test"), "FILE_OPERATION_ERROR"),
            (PermissionError("test"), "FILE_OPERATION_ERROR"),
            (RuntimeError("test"), "RUNTIME_ERROR"),
            (Exception("test"), "UNKNOWN_ERROR")
        ]

        for error, expected_code in test_cases:
            code = self.server._get_error_code(error)
            assert code == expected_code

    @pytest.mark.asyncio
    async def test_server_lifecycle(self):
        """Test server start/stop lifecycle"""
        # Mock the server start method
        with patch.object(self.server, '_register_tools', new_callable=AsyncMock):
            with patch('claude_agent_sdk.Server') as mock_server_class:
                mock_server = Mock()
                mock_server_class.return_value = mock_server
                mock_server.start = AsyncMock()

                # Start server
                await self.server.start()

                assert self.server.server is not None
                assert self.server.start_time is not None

                # Check health status
                status = self.server.get_health_status()
                assert status["status"] == "healthy"

                # Shutdown server
                await self.server.shutdown()

                assert self.server.server is None


class TestIntegration:
    """Integration tests for the complete system"""

    def setup_method(self):
        """Setup integration test environment"""
        self.temp_dir = tempfile.mkdtemp()

        # Create complete loco project structure
        dirs = [
            "src/models",
            "src/controllers",
            "src/views",
            "src/routes",
            "migration/src"
        ]

        for dir_path in dirs:
            os.makedirs(os.path.join(self.temp_dir, *dir_path.split("/")))

        # Create Cargo.toml
        cargo_toml = """
[package]
name = "integration-test-app"
version = "0.1.0"
edition = "2021"

[dependencies]
loco-rs = "0.3"
sea-orm = "0.12"
serde = { version = "1.0", features = ["derive"] }
"""
        with open(os.path.join(self.temp_dir, "Cargo.toml"), "w") as f:
            f.write(cargo_toml)

    def teardown_method(self):
        """Cleanup integration test environment"""
        import shutil
        shutil.rmtree(self.temp_dir, ignore_errors=True)

    @pytest.mark.asyncio
    async def test_complete_workflow(self):
        """Test complete generation workflow"""
        server = LocoMCPServer(ServerConfig(
            default_project_path=self.temp_dir
        ))

        # Step 1: Generate user model
        user_result = await server.tools.generate_model({
            "model_name": "user",
            "fields": ["name:string", "email:string:unique"],
            "project_path": self.temp_dir
        })

        assert user_result["success"]
        assert len(user_result["created_files"]) == 2

        # Step 2: Generate blog post scaffold
        blog_result = await server.tools.generate_scaffold({
            "model_name": "blog_post",
            "fields": ["title:string", "content:text", "author_id:i64"],
            "include_views": True,
            "project_path": self.temp_dir
        })

        assert blog_result["success"]
        assert len(blog_result["created_files"]) >= 4

        # Step 3: Generate controller for user
        controller_result = await server.tools.generate_controller_view({
            "model_name": "user",
            "actions": ["index", "show", "edit"],
            "project_path": self.temp_dir
        })

        assert controller_result["success"]

        # Verify all files exist
        expected_files = [
            "src/models/user.rs",
            "src/models/blog_post.rs",
            "src/controllers/blog_posts.rs",
            "src/controllers/user.rs",
            "src/views/blog_posts",
            "src/views/user"
        ]

        for file_path in expected_files:
            full_path = os.path.join(self.temp_dir, file_path)
            assert os.path.exists(full_path), f"Expected file {file_path} to exist"

    @pytest.mark.asyncio
    async def test_error_recovery_workflow(self):
        """Test error recovery and resilience"""
        server = LocoMCPServer(ServerConfig(
            default_project_path=self.temp_dir
        ))

        # Step 1: Try invalid model generation
        invalid_result = await server.tools.generate_model({
            "model_name": "123invalid",
            "fields": ["name"],
            "project_path": self.temp_dir
        })

        assert not invalid_result["success"]
        assert len(invalid_result["errors"]) > 0

        # Step 2: Try valid model generation (should succeed)
        valid_result = await server.tools.generate_model({
            "model_name": "user",
            "fields": ["name:string"],
            "project_path": self.temp_dir
        })

        assert valid_result["success"]

        # Step 3: Try duplicate model generation (should fail gracefully)
        duplicate_result = await server.tools.generate_model({
            "model_name": "user",
            "fields": ["email:string"],
            "project_path": self.temp_dir
        })

        assert not duplicate_result["success"]
        assert "already exists" in str(duplicate_result["errors"]).lower()

    @pytest.mark.asyncio
    async def test_performance_requirements(self):
        """Test that performance requirements are met"""
        server = LocoMCPServer(ServerConfig(
            default_project_path=self.temp_dir
        ))

        import time

        # Test model generation performance
        start_time = time.perf_counter()
        result = await server.tools.generate_model({
            "model_name": "user",
            "fields": ["name:string", "email:string"],
            "project_path": self.temp_dir
        })
        duration_ms = (time.perf_counter() - start_time) * 1000

        assert result["success"]
        assert duration_ms < 100, f"Model generation took {duration_ms:.2f}ms (should be <100ms for test)"

        # Test scaffold generation performance
        start_time = time.perf_counter()
        result = await server.tools.generate_scaffold({
            "model_name": "blog_post",
            "fields": ["title:string", "content:text"],
            "project_path": self.temp_dir
        })
        duration_ms = (time.perf_counter() - start_time) * 1000

        assert result["success"]
        assert duration_ms < 200, f"Scaffold generation took {duration_ms:.2f}ms (should be <200ms for test)"


if __name__ == "__main__":
    # Run tests
    pytest.main([__file__, "-v", "--tb=short"])
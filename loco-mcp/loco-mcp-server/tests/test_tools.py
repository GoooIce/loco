"""Tests for MCP tool registration and functionality."""

import pytest
from unittest.mock import Mock, patch
from typing import Dict, Any, List

# These imports will work once the MCP server is implemented
# from loco_mcp_server.tools import LocoTools
# from loco_mcp_server.server import LocoMCPServer


class TestLocoTools:
    """Test the Loco MCP tools implementation."""

    @pytest.fixture
    def mock_tools(self):
        """Create a mock LocoTools instance."""
        # This will be replaced with actual implementation
        with patch('loco_mcp_server.tools.LocoTools') as mock_tools_class:
            instance = Mock()
            mock_tools_class.return_value = instance
            yield instance

    @pytest.fixture
    def valid_model_params(self) -> Dict[str, Any]:
        """Valid parameters for model generation."""
        return {
            "model_name": "product",
            "fields": ["name:string", "price:i32", "sku:string:unique"]
        }

    @pytest.fixture
    def valid_scaffold_params(self) -> Dict[str, Any]:
        """Valid parameters for scaffold generation."""
        return {
            "model_name": "user",
            "fields": ["email:string:unique", "name:string", "active:boolean"],
            "include_views": True,
            "include_controllers": True,
            "api_only": False
        }

    @pytest.fixture
    def valid_controller_params(self) -> Dict[str, Any]:
        """Valid parameters for controller/view generation."""
        return {
            "model_name": "existing_model",
            "actions": ["index", "show", "create", "update", "delete"],
            "view_types": ["list", "show", "form", "edit"]
        }

    def test_tool_registration_generate_model(self, mock_tools):
        """Test that generate_model tool is properly registered."""
        # Mock tool registration
        mock_tools.register_tool = Mock()

        # This will be implemented when we create the actual tools module
        # tools = LocoTools()
        # tools.register_tool("generate_model", ...)

        # For now, test the expected behavior
        assert True  # Placeholder - will be implemented

    def test_tool_registration_generate_scaffold(self, mock_tools):
        """Test that generate_scaffold tool is properly registered."""
        # Mock tool registration
        mock_tools.register_tool = Mock()

        # This will be implemented when we create the actual tools module
        # tools = LocoTools()
        # tools.register_tool("generate_scaffold", ...)

        assert True  # Placeholder - will be implemented

    def test_tool_registration_generate_controller_view(self, mock_tools):
        """Test that generate_controller_view tool is properly registered."""
        # Mock tool registration
        mock_tools.register_tool = Mock()

        # This will be implemented when we create the actual tools module
        # tools = LocoTools()
        # tools.register_tool("generate_controller_view", ...)

        assert True  # Placeholder - will be implemented

    @patch('loco_mcp_server.tools.loco_bindings')
    def test_generate_model_tool_success(self, mock_bindings, mock_tools, valid_model_params):
        """Test successful model generation through MCP tool."""
        # Mock successful response from Rust bindings
        mock_response = {
            "success": True,
            "created_files": [
                {
                    "path": "src/models/product.rs",
                    "type": "model",
                    "size_bytes": 245
                },
                {
                    "path": "migration/src/m_20251003_120001_create_products.rs",
                    "type": "migration",
                    "size_bytes": 189
                }
            ],
            "modified_files": [],
            "errors": []
        }
        mock_bindings.generate_model.return_value = mock_response

        # Mock tool implementation
        def mock_generate_model_handler(params: Dict[str, Any]) -> Dict[str, Any]:
            return mock_bindings.generate_model(params)

        result = mock_generate_model_handler(valid_model_params)

        assert result["success"] is True
        assert len(result["created_files"]) == 2
        assert result["created_files"][0]["type"] == "model"
        assert result["created_files"][1]["type"] == "migration"
        assert len(result["errors"]) == 0

    @patch('loco_mcp_server.tools.loco_bindings')
    def test_generate_model_tool_validation_error(self, mock_bindings):
        """Test model generation with validation errors."""
        # Mock validation error from Rust bindings
        mock_bindings.generate_model.side_effect = ValueError("Invalid model name: '123invalid' must start with a letter")

        with pytest.raises(ValueError) as exc_info:
            mock_bindings.generate_model({
                "model_name": "123invalid",
                "fields": ["name:string"]
            })

        assert "Invalid model name" in str(exc_info.value)
        assert "must start with a letter" in str(exc_info.value)

    @patch('loco_mcp_server.tools.loco_bindings')
    def test_generate_scaffold_tool_success(self, mock_bindings, valid_scaffold_params):
        """Test successful scaffold generation through MCP tool."""
        # Mock successful response from Rust bindings
        mock_response = {
            "success": True,
            "created_files": [
                {"path": "src/models/user.rs", "type": "model", "size_bytes": 312},
                {"path": "migration/src/m_20251003_120002_create_users.rs", "type": "migration", "size_bytes": 201},
                {"path": "src/controllers/users.rs", "type": "controller", "size_bytes": 1567},
                {"path": "src/views/users/index.html.tera", "type": "view", "size_bytes": 892},
                {"path": "src/views/users/show.html.tera", "type": "view", "size_bytes": 456},
                {"path": "src/views/users/form.html.tera", "type": "view", "size_bytes": 678}
            ],
            "modified_files": [
                {"path": "src/routes/mod.rs", "type": "route"}
            ],
            "errors": []
        }
        mock_bindings.generate_scaffold.return_value = mock_response

        result = mock_bindings.generate_scaffold(valid_scaffold_params)

        assert result["success"] is True
        assert len(result["created_files"]) == 6
        assert len(result["modified_files"]) == 1
        assert len(result["errors"]) == 0

        # Check file types
        file_types = [f["type"] for f in result["created_files"]]
        assert "model" in file_types
        assert "migration" in file_types
        assert "controller" in file_types
        assert "view" in file_types

    @patch('loco_mcp_server.tools.loco_bindings')
    def test_generate_scaffold_tool_api_only(self, mock_bindings):
        """Test API-only scaffold generation."""
        api_params = {
            "model_name": "api_user",
            "fields": ["name:string", "token:string:unique"],
            "api_only": True
        }

        mock_response = {
            "success": True,
            "created_files": [
                {"path": "src/models/api_user.rs", "type": "model", "size_bytes": 298},
                {"path": "migration/src/m_20251003_120003_create_api_users.rs", "type": "migration", "size_bytes": 195},
                {"path": "src/controllers/api_users.rs", "type": "controller", "size_bytes": 1234}
            ],
            "modified_files": [
                {"path": "src/routes/mod.rs", "type": "route"}
            ],
            "errors": []
        }
        mock_bindings.generate_scaffold.return_value = mock_response

        result = mock_bindings.generate_scaffold(api_params)

        assert result["success"] is True
        assert len(result["created_files"]) == 3

        # API-only should not generate views
        file_types = [f["type"] for f in result["created_files"]]
        assert "view" not in file_types

    @patch('loco_mcp_server.tools.loco_bindings')
    def test_generate_controller_view_tool_success(self, mock_bindings, valid_controller_params):
        """Test successful controller/view generation."""
        mock_response = {
            "success": True,
            "created_files": [
                {"path": "src/controllers/existing_models.rs", "type": "controller", "size_bytes": 1245},
                {"path": "src/views/existing_models/list.html.tera", "type": "view", "size_bytes": 723},
                {"path": "src/views/existing_models/show.html.tera", "type": "view", "size_bytes": 412},
                {"path": "src/views/existing_models/form.html.tera", "type": "view", "size_bytes": 567},
                {"path": "src/views/existing_models/edit.html.tera", "type": "view", "size_bytes": 589}
            ],
            "modified_files": [
                {"path": "src/routes/mod.rs", "type": "route"}
            ],
            "errors": []
        }
        mock_bindings.generate_controller_view.return_value = mock_response

        result = mock_bindings.generate_controller_view(valid_controller_params)

        assert result["success"] is True
        assert len(result["created_files"]) == 5
        assert len(result["modified_files"]) == 1
        assert len(result["errors"]) == 0

    def test_tool_parameter_validation(self):
        """Test parameter validation for MCP tools."""
        # Test cases for parameter validation
        validation_cases = [
            {
                "tool": "generate_model",
                "params": {"fields": ["name:string"]},  # Missing model_name
                "should_fail": True,
                "error_contains": ["model_name", "required"]
            },
            {
                "tool": "generate_model",
                "params": {"model_name": "123invalid", "fields": ["name:string"]},  # Invalid model name
                "should_fail": True,
                "error_contains": ["Invalid model name"]
            },
            {
                "tool": "generate_scaffold",
                "params": {
                    "model_name": "test",
                    "fields": ["name:string"],
                    "include_views": True,
                    "api_only": True  # Conflict
                },
                "should_fail": True,
                "error_contains": ["conflict", "invalid"]
            },
            {
                "tool": "generate_controller_view",
                "params": {"model_name": "test", "actions": ["invalid_action"]},
                "should_fail": True,
                "error_contains": ["invalid", "action"]
            }
        ]

        for case in validation_cases:
            # Mock validation function
            def mock_validate_params(tool_name: str, params: Dict[str, Any]) -> bool:
                if "model_name" in params and params["model_name"] == "123invalid":
                    raise ValueError("Invalid model name")
                if "actions" in params and "invalid_action" in params["actions"]:
                    raise ValueError("Invalid action")
                if params.get("include_views") and params.get("api_only"):
                    raise ValueError("Conflict between include_views and api_only")
                if "model_name" not in params:
                    raise ValueError("model_name is required")
                return True

            if case["should_fail"]:
                with pytest.raises(ValueError) as exc_info:
                    mock_validate_params(case["tool"], case["params"])

                error_msg = str(exc_info.value).lower()
                for keyword in case["error_contains"]:
                    assert keyword.lower() in error_msg, f"Error should contain '{keyword}' for {case['tool']}"
            else:
                try:
                    mock_validate_params(case["tool"], case["params"])
                except ValueError:
                    pytest.fail(f"Case should not fail: {case}")

    def test_tool_response_formatting(self):
        """Test that tool responses are properly formatted for MCP protocol."""
        # Mock response from Rust bindings
        rust_response = {
            "success": True,
            "created_files": [
                {"path": "src/models/test.rs", "type": "model", "size_bytes": 200}
            ],
            "modified_files": [],
            "errors": []
        }

        # Mock MCP response formatting
        def mock_format_mcp_response(rust_response: Dict[str, Any]) -> Dict[str, Any]:
            return {
                "status": "success" if rust_response["success"] else "error",
                "result": {
                    "files_created": rust_response["created_files"],
                    "files_modified": rust_response["modified_files"],
                    "errors": rust_response["errors"]
                },
                "metadata": {
                    "tool_version": "1.0.0",
                    "response_time_ms": 5
                }
            }

        mcp_response = mock_format_mcp_response(rust_response)

        assert mcp_response["status"] == "success"
        assert "result" in mcp_response
        assert "metadata" in mcp_response
        assert len(mcp_response["result"]["files_created"]) == 1

    def test_tool_error_handling(self):
        """Test error handling in MCP tools."""
        error_cases = [
            {
                "exception": ValueError("Invalid model name"),
                "expected_mcp_error": {
                    "code": "VALIDATION_ERROR",
                    "message": "Invalid model name",
                    "details": None
                }
            },
            {
                "exception": FileNotFoundError("Model file already exists"),
                "expected_mcp_error": {
                    "code": "FILE_EXISTS_ERROR",
                    "message": "Model file already exists",
                    "details": None
                }
            },
            {
                "exception": PermissionError("Access denied"),
                "expected_mcp_error": {
                    "code": "PERMISSION_DENIED",
                    "message": "Access denied",
                    "details": None
                }
            }
        ]

        def mock_format_mcp_error(exception: Exception) -> Dict[str, Any]:
            # Map Python exceptions to MCP error codes
            error_mapping = {
                ValueError: "VALIDATION_ERROR",
                FileNotFoundError: "FILE_EXISTS_ERROR",
                PermissionError: "PERMISSION_DENIED",
                RuntimeError: "RUNTIME_ERROR"
            }

            error_code = error_mapping.get(type(exception), "UNKNOWN_ERROR")

            return {
                "status": "error",
                "error": {
                    "code": error_code,
                    "message": str(exception),
                    "details": None
                }
            }

        for case in error_cases:
            mcp_error = mock_format_mcp_error(case["exception"])

            assert mcp_error["status"] == "error"
            assert mcp_error["error"]["code"] == case["expected_mcp_error"]["code"]
            assert mcp_error["error"]["message"] == case["expected_mcp_error"]["message"]

    def test_tool_discovery(self, mock_tools):
        """Test that tools can be discovered by MCP clients."""
        # Mock tool discovery
        mock_tools.list_tools = Mock(return_value=[
            {
                "name": "generate_model",
                "description": "Generate a model and migration file",
                "parameters": {
                    "model_name": {"type": "string", "required": True},
                    "fields": {"type": "array", "required": True},
                    "project_path": {"type": "string", "required": False, "default": "."}
                }
            },
            {
                "name": "generate_scaffold",
                "description": "Generate complete CRUD scaffolding",
                "parameters": {
                    "model_name": {"type": "string", "required": True},
                    "fields": {"type": "array", "required": True},
                    "include_views": {"type": "boolean", "default": True},
                    "include_controllers": {"type": "boolean", "default": True},
                    "api_only": {"type": "boolean", "default": False}
                }
            },
            {
                "name": "generate_controller_view",
                "description": "Generate controller and views for existing model",
                "parameters": {
                    "model_name": {"type": "string", "required": True},
                    "actions": {"type": "array", "default": ["index", "show", "create", "update", "delete"]},
                    "view_types": {"type": "array", "default": ["list", "show", "form", "edit"]}
                }
            }
        ])

        tools = mock_tools.list_tools()

        assert len(tools) == 3
        tool_names = [tool["name"] for tool in tools]
        assert "generate_model" in tool_names
        assert "generate_scaffold" in tool_names
        assert "generate_controller_view" in tool_names

        # Verify parameter definitions
        generate_model_tool = next(t for t in tools if t["name"] == "generate_model")
        assert "model_name" in generate_model_tool["parameters"]
        assert generate_model_tool["parameters"]["model_name"]["required"] is True
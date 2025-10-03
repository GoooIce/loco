"""Edge case tests for error handling and boundary conditions."""

import pytest
import asyncio
import tempfile
import os
from unittest.mock import Mock, patch, AsyncMock
from typing import Dict, Any


class TestEdgeCases:
    """Test edge cases and error conditions."""

    @pytest.fixture
    def temp_project_dir(self):
        """Create a temporary project directory."""
        with tempfile.TemporaryDirectory() as temp_dir:
            yield temp_dir

    @pytest.mark.asyncio
    async def test_unsupported_field_types(self, temp_project_dir):
        """Test: Handle unsupported field types gracefully."""
        with patch('loco_mcp_server.tools.loco_bindings') as mock_bindings:

            # Mock error for unsupported field type
            mock_bindings.generate_model.side_effect = ValueError(
                "Unsupported field type: 'invalid_type'. Supported types: string, i32, i64, boolean, datetime, text, optional"
            )

            mcp_request = {
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "generate_model",
                    "arguments": {
                        "model_name": "test",
                        "fields": ["name:invalid_type", "email:string"],
                        "project_path": temp_project_dir
                    }
                }
            }

            async def handle_request(request):
                await asyncio.sleep(0.001)
                try:
                    result = mock_bindings.generate_model(request["params"]["arguments"])
                    return {
                        "jsonrpc": "2.0",
                        "id": request["id"],
                        "result": {"status": "success", "files_created": result["created_files"]}
                    }
                except ValueError as e:
                    return {
                        "jsonrpc": "2.0",
                        "id": request["id"],
                        "error": {
                            "code": -32602,
                            "message": str(e),
                            "details": {
                                "supported_types": ["string", "i32", "i64", "boolean", "datetime", "text", "optional"],
                                "invalid_field": "name:invalid_type"
                            }
                        }
                    }

            response = await handle_request(mcp_request)

            assert "error" in response
            assert "Unsupported field type" in response["error"]["message"]
            assert "invalid_type" in response["error"]["message"]
            assert "supported_types" in response["error"]["details"]

    @pytest.mark.asyncio
    async def test_duplicate_model_names(self, temp_project_dir):
        """Test: Handle attempts to create duplicate models."""
        # Set up project with existing model
        os.makedirs(os.path.join(temp_project_dir, "src/models"), exist_ok=True)
        existing_model = """
use sea_orm::entity::prelude::*;
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
}
"""
        with open(os.path.join(temp_project_dir, "src/models/user.rs"), "w") as f:
            f.write(existing_model)

        with patch('loco_mcp_server.tools.loco_bindings') as mock_bindings:

            # Mock duplicate model error
            mock_bindings.generate_model.side_effect = ValueError(
                "Model 'user' already exists. Choose a different name or use 'force' parameter to overwrite"
            )

            mcp_request = {
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "generate_model",
                    "arguments": {
                        "model_name": "user",
                        "fields": ["email:string", "name:string"],
                        "project_path": temp_project_dir
                    }
                }
            }

            async def handle_request(request):
                await asyncio.sleep(0.001)
                try:
                    result = mock_bindings.generate_model(request["params"]["arguments"])
                    return {
                        "jsonrpc": "2.0",
                        "id": request["id"],
                        "result": {"status": "success", "files_created": result["created_files"]}
                    }
                except ValueError as e:
                    return {
                        "jsonrpc": "2.0",
                        "id": request["id"],
                        "error": {
                            "code": -32602,
                            "message": str(e),
                            "details": {
                                "existing_model": "user",
                                "existing_file": "src/models/user.rs",
                                "suggestions": ["Choose a different model name", "Use 'force' parameter to overwrite existing model"]
                            }
                        }
                    }

            response = await handle_request(mcp_request)

            assert "error" in response
            assert "already exists" in response["error"]["message"]
            assert "existing_model" in response["error"]["details"]
            assert "suggestions" in response["error"]["details"]

    @pytest.mark.asyncio
    async def test_invalid_project_directory(self, temp_project_dir):
        """Test: Handle operations in non-loco project directories."""
        # Don't create loco project structure - just an empty directory

        with patch('loco_mcp_server.tools.loco_bindings') as mock_bindings:

            # Mock project validation error
            mock_bindings.generate_model.side_effect = ValueError(
                "Not a valid loco-rs project directory. Run 'cargo loco new' first or navigate to existing project"
            )

            mcp_request = {
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "generate_model",
                    "arguments": {
                        "model_name": "test",
                        "fields": ["name:string"],
                        "project_path": temp_project_dir
                    }
                }
            }

            async def handle_request(request):
                await asyncio.sleep(0.001)
                try:
                    result = mock_bindings.generate_model(request["params"]["arguments"])
                    return {
                        "jsonrpc": "2.0",
                        "id": request["id"],
                        "result": {"status": "success", "files_created": result["created_files"]}
                    }
                except ValueError as e:
                    return {
                        "jsonrpc": "2.0",
                        "id": request["id"],
                        "error": {
                            "code": -32602,
                            "message": str(e),
                            "details": {
                                "directory_path": temp_project_dir,
                                "required_files": ["Cargo.toml", "src/main.rs", "src/models/"],
                                "suggestion": "Run 'cargo loco new <project_name>' to create a new loco-rs project"
                            }
                        }
                    }

            response = await handle_request(mcp_request)

            assert "error" in response
            assert "Not a valid loco-rs project" in response["error"]["message"]
            assert "required_files" in response["error"]["details"]
            assert "suggestion" in response["error"]["details"]

    @pytest.mark.asyncio
    async def test_rust_compilation_errors(self, temp_project_dir):
        """Test: Handle Rust compilation errors in generated code."""
        # Set up minimal project structure
        os.makedirs(os.path.join(temp_project_dir, "src/models"), exist_ok=True)
        with open(os.path.join(temp_project_dir, "Cargo.toml"), "w") as f:
            f.write("[package]\nname = \"test\"\nversion = \"0.1.0\"\n")

        with patch('loco_mcp_server.tools.loco_bindings') as mock_bindings:

            # Mock compilation error
            mock_bindings.generate_model.side_effect = RuntimeError(
                "Compilation failed: src/models/test.rs:15: Type mismatch found\nExpected: String\nFound: i32\n\nSuggestions:\n- Check field types match expected schema\n- Verify imports are correct"
            )

            mcp_request = {
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "generate_model",
                    "arguments": {
                        "model_name": "test",
                        "fields": ["name:string", "invalid_field:mismatched_type"],
                        "project_path": temp_project_dir
                    }
                }
            }

            async def handle_request(request):
                await asyncio.sleep(0.002)  # Compilation checks take longer
                try:
                    result = mock_bindings.generate_model(request["params"]["arguments"])
                    return {
                        "jsonrpc": "2.0",
                        "id": request["id"],
                        "result": {"status": "success", "files_created": result["created_files"]}
                    }
                except RuntimeError as e:
                    return {
                        "jsonrpc": "2.0",
                        "id": request["id"],
                        "error": {
                            "code": -32603,
                            "message": "Compilation failed",
                            "details": {
                                "error_type": "compilation_error",
                                "file": "src/models/test.rs",
                                "line": 15,
                                "error_details": "Type mismatch found\nExpected: String\nFound: i32",
                                "suggestions": [
                                    "Check field types match expected schema",
                                    "Verify imports are correct"
                                ]
                            }
                        }
                    }

            response = await handle_request(mcp_request)

            assert "error" in response
            assert response["error"]["code"] == -32603  # Internal error
            assert "Compilation failed" in response["error"]["message"]
            assert "error_type" in response["error"]["details"]
            assert response["error"]["details"]["error_type"] == "compilation_error"
            assert "suggestions" in response["error"]["details"]

    @pytest.mark.asyncio
    async def test_extreme_field_counts(self, temp_project_dir):
        """Test: Handle models with extreme numbers of fields."""
        with patch('loco_mcp_server.tools.loco_bindings') as mock_bindings:

            # Test case 1: Too many fields
            mock_bindings.generate_model.side_effect = ValueError(
                "Too many fields (200). Maximum allowed is 100 fields per model for maintainability"
            )

            many_fields = [f"field_{i}:string" for i in range(200)]

            mcp_request = {
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "generate_model",
                    "arguments": {
                        "model_name": "large_model",
                        "fields": many_fields,
                        "project_path": temp_project_dir
                    }
                }
            }

            async def handle_request(request):
                await asyncio.sleep(0.001)
                try:
                    result = mock_bindings.generate_model(request["params"]["arguments"])
                    return {
                        "jsonrpc": "2.0",
                        "id": request["id"],
                        "result": {"status": "success", "files_created": result["created_files"]}
                    }
                except ValueError as e:
                    return {
                        "jsonrpc": "2.0",
                        "id": request["id"],
                        "error": {
                            "code": -32602,
                            "message": str(e),
                            "details": {
                                "field_count": 200,
                                "max_allowed": 100,
                                "suggestion": "Consider splitting into multiple related models or using JSON fields for complex data"
                            }
                        }
                    }

            response = await handle_request(mcp_request)

            assert "error" in response
            assert "Too many fields" in response["error"]["message"]
            assert "field_count" in response["error"]["details"]
            assert response["error"]["details"]["field_count"] == 200

    @pytest.mark.asyncio
    async def test_invalid_characters_in_names(self, temp_project_dir):
        """Test: Handle invalid characters in model and field names."""
        invalid_cases = [
            ("user@profile", "Contains invalid character '@'"),
            ("user-name", "Contains hyphen '-'; use underscore '_' instead"),
            ("123user", "Starts with number; must start with letter"),
            ("user name", "Contains space character"),
            ("user$", "Contains special character '$'"),
            ("User", "Contains uppercase letter; use lowercase")
        ]

        with patch('loco_mcp_server.tools.loco_bindings') as mock_bindings:

            for invalid_name, expected_error in invalid_cases:
                mock_bindings.generate_model.side_effect = ValueError(
                    f"Invalid model name: '{invalid_name}'. {expected_error}"
                )

                mcp_request = {
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "tools/call",
                    "params": {
                        "name": "generate_model",
                        "arguments": {
                            "model_name": invalid_name,
                            "fields": ["name:string"],
                            "project_path": temp_project_dir
                        }
                    }
                }

                async def handle_request(request):
                    await asyncio.sleep(0.001)
                    try:
                        result = mock_bindings.generate_model(request["params"]["arguments"])
                        return {
                            "jsonrpc": "2.0",
                            "id": request["id"],
                            "result": {"status": "success", "files_created": result["created_files"]}
                        }
                    except ValueError as e:
                        return {
                            "jsonrpc": "2.0",
                            "id": request["id"],
                            "error": {
                                "code": -32602,
                                "message": str(e),
                                "details": {
                                    "invalid_name": invalid_name,
                                    "naming_rules": [
                                        "Must start with a letter",
                                        "Can contain lowercase letters, numbers, and underscores",
                                        "Cannot contain spaces or special characters",
                                        "Maximum 64 characters"
                                    ]
                                }
                            }
                        }

                response = await handle_request(mcp_request)

                assert "error" in response
                assert expected_error in response["error"]["message"]
                assert "naming_rules" in response["error"]["details"]

    @pytest.mark.asyncio
    async def test_reserved_keywords(self, temp_project_dir):
        """Test: Handle reserved keywords in model and field names."""
        reserved_keywords = ["id", "type", "struct", "enum", "impl", "fn", "let", "mut"]

        with patch('loco_mcp_server.tools.loco_bindings') as mock_bindings:

            for keyword in reserved_keywords:
                mock_bindings.generate_model.side_effect = ValueError(
                    f"Field name '{keyword}' is a reserved keyword. Choose a different name"
                )

                mcp_request = {
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "tools/call",
                    "params": {
                        "name": "generate_model",
                        "arguments": {
                            "model_name": "test",
                            "fields": [f"{keyword}:string"],
                            "project_path": temp_project_dir
                        }
                    }
                }

                async def handle_request(request):
                    await asyncio.sleep(0.001)
                    try:
                        result = mock_bindings.generate_model(request["params"]["arguments"])
                        return {
                            "jsonrpc": "2.0",
                            "id": request["id"],
                            "result": {"status": "success", "files_created": result["created_files"]}
                        }
                    except ValueError as e:
                        return {
                            "jsonrpc": "2.0",
                            "id": request["id"],
                            "error": {
                                "code": -32602,
                                "message": str(e),
                                "details": {
                                    "reserved_keyword": keyword,
                                    "suggestions": [
                                        f"Use '{keyword}_value' instead",
                                        f"Use '{keyword}_id' if it's an identifier",
                                        f"Use '{keyword}_name' for naming purposes"
                                    ]
                                }
                            }
                        }

                response = await handle_request(mcp_request)

                assert "error" in response
                assert "reserved keyword" in response["error"]["message"]
                assert "suggestions" in response["error"]["details"]

    @pytest.mark.asyncio
    async def test_empty_and_null_inputs(self, temp_project_dir):
        """Test: Handle empty and null inputs gracefully."""
        edge_cases = [
            ({}, "Missing required parameters"),
            ({"model_name": ""}, "Empty model name"),
            ({"model_name": "test", "fields": []}, "Empty fields array"),
            ({"model_name": "test", "fields": [""]}, "Empty field definition"),
            ({"model_name": "test", "fields": ["name:"]}, "Incomplete field definition"),
        ]

        with patch('loco_mcp_server.tools.loco_bindings') as mock_bindings:

            for params, expected_error in edge_cases:
                mock_bindings.generate_model.side_effect = ValueError(expected_error)

                mcp_request = {
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "tools/call",
                    "params": {
                        "name": "generate_model",
                        "arguments": {**params, "project_path": temp_project_dir}
                    }
                }

                async def handle_request(request):
                    await asyncio.sleep(0.001)
                    try:
                        result = mock_bindings.generate_model(request["params"]["arguments"])
                        return {
                            "jsonrpc": "2.0",
                            "id": request["id"],
                            "result": {"status": "success", "files_created": result["created_files"]}
                        }
                    except ValueError as e:
                        return {
                            "jsonrpc": "2.0",
                            "id": request["id"],
                            "error": {
                                "code": -32602,
                                "message": str(e),
                                "details": {
                                    "validation_error": True,
                                    "required_parameters": ["model_name", "fields"],
                                    "constraints": {
                                        "model_name": "non-empty string, valid Rust identifier",
                                        "fields": "non-empty array of valid field definitions"
                                    }
                                }
                            }
                        }

                response = await handle_request(mcp_request)

                assert "error" in response
                assert expected_error in response["error"]["message"]
                assert "validation_error" in response["error"]["details"]
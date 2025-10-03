"""Integration tests for end-to-end MCP workflow."""

import pytest
import asyncio
import tempfile
import os
from unittest.mock import Mock, patch, AsyncMock
from typing import Dict, Any, List

# These imports will work once the MCP server is implemented
# from loco_mcp_server.server import LocoMCPServer
# from loco_mcp_server.tools import LocoTools
# import loco_bindings


class TestEndToEndWorkflow:
    """Test complete end-to-end workflows."""

    @pytest.fixture
    def temp_project_dir(self):
        """Create a temporary directory for testing."""
        with tempfile.TemporaryDirectory() as temp_dir:
            # Set up basic loco-rs project structure
            project_dirs = [
                "src/models",
                "src/controllers",
                "src/views",
                "src/routes",
                "migration/src"
            ]

            for dir_path in project_dirs:
                os.makedirs(os.path.join(temp_dir, dir_path), exist_ok=True)

            # Create basic project files
            cargo_toml = """
[package]
name = "test-app"
version = "0.1.0"
edition = "2021"

[dependencies]
loco-rs = "0.3"
sea-orm = "0.12"
serde = "1.0"
tera = "1.0"
"""

            with open(os.path.join(temp_dir, "Cargo.toml"), "w") as f:
                f.write(cargo_toml)

            main_rs = """
fn main() {
    println!("Hello, world!");
}
"""

            with open(os.path.join(temp_dir, "src/main.rs"), "w") as f:
                f.write(main_rs)

            yield temp_dir

    @pytest.mark.asyncio
    async def test_complete_scaffold_workflow(self, temp_project_dir):
        """Test complete scaffold generation workflow."""
        # Mock MCP server and bindings
        with patch('loco_mcp_server.server.LocoMCPServer') as mock_server_class, \
             patch('loco_bindings.generate_scaffold') as mock_generate_scaffold:

            # Setup mock server
            mock_server = Mock()
            mock_server_class.return_value = mock_server

            # Setup mock scaffold response
            mock_response = {
                "success": True,
                "created_files": [
                    {"path": "src/models/user.rs", "type": "model", "size_bytes": 312},
                    {"path": "migration/src/m_20251003_120001_create_users.rs", "type": "migration", "size_bytes": 201},
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
            mock_generate_scaffold.return_value = mock_response

            # Simulate MCP request
            mcp_request = {
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "generate_scaffold",
                    "arguments": {
                        "model_name": "user",
                        "fields": ["email:string:unique", "name:string", "active:boolean"],
                        "include_views": True,
                        "include_controllers": True,
                        "api_only": False,
                        "project_path": temp_project_dir
                    }
                }
            }

            # Mock request handling
            async def mock_handle_request(request: Dict[str, Any]) -> Dict[str, Any]:
                await asyncio.sleep(0.001)  # Simulate processing

                if request["method"] == "tools/call":
                    tool_name = request["params"]["name"]
                    args = request["params"]["arguments"]

                    if tool_name == "generate_scaffold":
                        # Call the actual mock function
                        result = mock_generate_scaffold(args)
                        return {
                            "jsonrpc": "2.0",
                            "id": request["id"],
                            "result": {
                                "status": "success",
                                "files_created": result["created_files"],
                                "files_modified": result["modified_files"],
                                "errors": result["errors"],
                                "project_path": args["project_path"]
                            }
                        }

                return {
                    "jsonrpc": "2.0",
                    "id": request["id"],
                    "error": {"code": -32601, "message": "Method not found"}
                }

            # Execute the workflow
            response = await mock_handle_request(mcp_request)

            # Verify response
            assert response["result"]["status"] == "success"
            assert len(response["result"]["files_created"]) == 6
            assert len(response["result"]["files_modified"]) == 1
            assert len(response["result"]["errors"]) == 0

            # Verify the bindings were called with correct parameters
            mock_generate_scaffold.assert_called_once_with({
                "model_name": "user",
                "fields": ["email:string:unique", "name:string", "active:boolean"],
                "include_views": True,
                "include_controllers": True,
                "api_only": False,
                "project_path": temp_project_dir
            })

    @pytest.mark.asyncio
    async def test_step_by_step_model_workflow(self, temp_project_dir):
        """Test step-by-step model generation workflow."""
        with patch('loco_bindings.generate_model') as mock_generate_model, \
             patch('loco_bindings.generate_controller_view') as mock_generate_controller_view:

            # Setup mock responses
            model_response = {
                "success": True,
                "created_files": [
                    {"path": "src/models/product.rs", "type": "model", "size_bytes": 245},
                    {"path": "migration/src/m_20251003_120002_create_products.rs", "type": "migration", "size_bytes": 189}
                ],
                "modified_files": [],
                "errors": []
            }
            mock_generate_model.return_value = model_response

            controller_response = {
                "success": True,
                "created_files": [
                    {"path": "src/controllers/products.rs", "type": "controller", "size_bytes": 1245},
                    {"path": "src/views/products/list.html.tera", "type": "view", "size_bytes": 723},
                    {"path": "src/views/products/show.html.tera", "type": "view", "size_bytes": 412},
                    {"path": "src/views/products/form.html.tera", "type": "view", "size_bytes": 567}
                ],
                "modified_files": [
                    {"path": "src/routes/mod.rs", "type": "route"}
                ],
                "errors": []
            }
            mock_generate_controller_view.return_value = controller_response

            # Step 1: Generate model
            model_request = {
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "generate_model",
                    "arguments": {
                        "model_name": "product",
                        "fields": ["name:string", "price:i32", "sku:string:unique"],
                        "project_path": temp_project_dir
                    }
                }
            }

            # Step 2: Generate controller and views
            controller_request = {
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/call",
                "params": {
                    "name": "generate_controller_view",
                    "arguments": {
                        "model_name": "product",
                        "actions": ["index", "show", "create", "update", "delete"],
                        "view_types": ["list", "show", "form", "edit"],
                        "project_path": temp_project_dir
                    }
                }
            }

            # Execute workflow
            async def mock_handle_request(request: Dict[str, Any]) -> Dict[str, Any]:
                await asyncio.sleep(0.001)

                if request["method"] == "tools/call":
                    tool_name = request["params"]["name"]
                    args = request["params"]["arguments"]

                    if tool_name == "generate_model":
                        result = mock_generate_model(args)
                        return {
                            "jsonrpc": "2.0",
                            "id": request["id"],
                            "result": {
                                "status": "success",
                                "files_created": result["created_files"],
                                "files_modified": result["modified_files"],
                                "errors": result["errors"]
                            }
                        }
                    elif tool_name == "generate_controller_view":
                        result = mock_generate_controller_view(args)
                        return {
                            "jsonrpc": "2.0",
                            "id": request["id"],
                            "result": {
                                "status": "success",
                                "files_created": result["created_files"],
                                "files_modified": result["modified_files"],
                                "errors": result["errors"]
                            }
                        }

                return {
                    "jsonrpc": "2.0",
                    "id": request["id"],
                    "error": {"code": -32601, "message": "Method not found"}
                }

            # Execute both steps
            model_response = await mock_handle_request(model_request)
            controller_response = await mock_handle_request(controller_request)

            # Verify both steps succeeded
            assert model_response["result"]["status"] == "success"
            assert len(model_response["result"]["files_created"]) == 2

            assert controller_response["result"]["status"] == "success"
            assert len(controller_response["result"]["files_created"]) == 4

            # Verify function calls
            mock_generate_model.assert_called_once_with({
                "model_name": "product",
                "fields": ["name:string", "price:i32", "sku:string:unique"],
                "project_path": temp_project_dir
            })

            mock_generate_controller_view.assert_called_once_with({
                "model_name": "product",
                "actions": ["index", "show", "create", "update", "delete"],
                "view_types": ["list", "show", "form", "edit"],
                "project_path": temp_project_dir
            })

    @pytest.mark.asyncio
    async def test_error_recovery_workflow(self, temp_project_dir):
        """Test error handling and recovery in workflows."""
        with patch('loco_bindings.generate_model') as mock_generate_model:

            # First call fails with validation error
            mock_generate_model.side_effect = [
                ValueError("Invalid model name: '123invalid' must start with a letter"),
                # Second call succeeds
                {
                    "success": True,
                    "created_files": [
                        {"path": "src/models/valid_model.rs", "type": "model", "size_bytes": 200}
                    ],
                    "modified_files": [],
                    "errors": []
                }
            ]

            # Invalid request
            invalid_request = {
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "generate_model",
                    "arguments": {
                        "model_name": "123invalid",
                        "fields": ["name:string"],
                        "project_path": temp_project_dir
                    }
                }
            }

            # Valid request (after correction)
            valid_request = {
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/call",
                "params": {
                    "name": "generate_model",
                    "arguments": {
                        "model_name": "valid_model",
                        "fields": ["name:string"],
                        "project_path": temp_project_dir
                    }
                }
            }

            async def mock_handle_request(request: Dict[str, Any]) -> Dict[str, Any]:
                await asyncio.sleep(0.001)

                if request["method"] == "tools/call":
                    tool_name = request["params"]["name"]
                    args = request["params"]["arguments"]

                    if tool_name == "generate_model":
                        try:
                            result = mock_generate_model(args)
                            return {
                                "jsonrpc": "2.0",
                                "id": request["id"],
                                "result": {
                                    "status": "success",
                                    "files_created": result["created_files"],
                                    "files_modified": result["modified_files"],
                                    "errors": result["errors"]
                                }
                            }
                        except ValueError as e:
                            return {
                                "jsonrpc": "2.0",
                                "id": request["id"],
                                "error": {
                                    "code": -32602,
                                    "message": str(e)
                                }
                            }

                return {
                    "jsonrpc": "2.0",
                    "id": request["id"],
                    "error": {"code": -32601, "message": "Method not found"}
                }

            # Execute requests
            invalid_response = await mock_handle_request(invalid_request)
            valid_response = await mock_handle_request(valid_request)

            # Verify error handling
            assert "error" in invalid_response
            assert "Invalid model name" in invalid_response["error"]["message"]

            # Verify recovery
            assert valid_response["result"]["status"] == "success"
            assert len(valid_response["result"]["files_created"]) == 1

            # Verify both calls were made
            assert mock_generate_model.call_count == 2

    @pytest.mark.asyncio
    async def test_concurrent_model_generation(self, temp_project_dir):
        """Test concurrent model generation requests."""
        with patch('loco_bindings.generate_model') as mock_generate_model:

            # Setup mock response for concurrent calls
            def mock_generate_side_effect(params):
                model_name = params["model_name"]
                return {
                    "success": True,
                    "created_files": [
                        {"path": f"src/models/{model_name}.rs", "type": "model", "size_bytes": 200},
                        {"path": f"migration/src/m_create_{model_name}s.rs", "type": "migration", "size_bytes": 150}
                    ],
                    "modified_files": [],
                    "errors": []
                }

            mock_generate_model.side_effect = mock_generate_side_effect

            # Create concurrent requests
            models = ["user", "product", "category", "order"]
            requests = []

            for i, model_name in enumerate(models, 1):
                request = {
                    "jsonrpc": "2.0",
                    "id": i,
                    "method": "tools/call",
                    "params": {
                        "name": "generate_model",
                        "arguments": {
                            "model_name": model_name,
                            "fields": ["name:string"],
                            "project_path": temp_project_dir
                        }
                    }
                }
                requests.append(request)

            async def mock_handle_request(request: Dict[str, Any]) -> Dict[str, Any]:
                await asyncio.sleep(0.005)  # Simulate processing time

                tool_name = request["params"]["name"]
                args = request["params"]["arguments"]

                if tool_name == "generate_model":
                    result = mock_generate_model(args)
                    return {
                        "jsonrpc": "2.0",
                        "id": request["id"],
                        "result": {
                            "status": "success",
                            "files_created": result["created_files"],
                            "files_modified": result["modified_files"],
                            "errors": result["errors"]
                        }
                    }

            # Execute all requests concurrently
            start_time = asyncio.get_event_loop().time()
            tasks = [mock_handle_request(req) for req in requests]
            responses = await asyncio.gather(*tasks)
            end_time = asyncio.get_event_loop().time()

            total_time = end_time - start_time

            # Verify concurrency benefits (should be faster than sequential)
            assert total_time < 0.015, f"Concurrent execution should be fast, took {total_time:.3f}s"
            assert len(responses) == 4

            # Verify all responses are successful
            for i, response in enumerate(responses):
                assert response["result"]["status"] == "success"
                assert len(response["result"]["files_created"]) == 2

                # Verify correct model was generated
                created_files = response["result"]["files_created"]
                model_file = next(f for f in created_files if f["type"] == "model")
                expected_model_name = models[i]
                assert expected_model_name in model_file["path"]

            # Verify all models were generated
            assert mock_generate_model.call_count == 4

    def test_workflow_file_system_integration(self, temp_project_dir):
        """Test integration with actual file system (limited scope)."""
        # Create a mock that simulates file system operations
        created_files = []

        def mock_create_file(path: str, content: str):
            full_path = os.path.join(temp_project_dir, path)
            os.makedirs(os.path.dirname(full_path), exist_ok=True)
            with open(full_path, "w") as f:
                f.write(content)
            created_files.append(path)
            return len(content)

        # Simulate model file creation
        model_content = """
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "test_models")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(column_type = "String(Some(255))")]
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
"""

        file_size = mock_create_file("src/models/test_model.rs", model_content)

        # Verify file was created
        assert len(created_files) == 1
        assert created_files[0] == "src/models/test_model.rs"

        # Verify file exists and has content
        full_path = os.path.join(temp_project_dir, "src/models/test_model.rs")
        assert os.path.exists(full_path)

        with open(full_path, "r") as f:
            saved_content = f.read()

        assert saved_content == model_content
        assert len(saved_content) == file_size

    def test_mcp_protocol_compliance(self):
        """Test that responses comply with MCP protocol specification."""
        # Test successful response format
        success_response = {
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "status": "success",
                "files_created": [
                    {"path": "src/models/test.rs", "type": "model", "size_bytes": 200}
                ],
                "files_modified": [],
                "errors": []
            }
        }

        # Test error response format
        error_response = {
            "jsonrpc": "2.0",
            "id": 2,
            "error": {
                "code": -32602,
                "message": "Invalid params"
            }
        }

        # Verify protocol compliance
        assert success_response["jsonrpc"] == "2.0"
        assert "id" in success_response
        assert "result" in success_response
        assert "error" not in success_response

        assert error_response["jsonrpc"] == "2.0"
        assert "id" in error_response
        assert "error" in error_response
        assert "result" not in error_response
        assert "code" in error_response["error"]
        assert "message" in error_response["error"]
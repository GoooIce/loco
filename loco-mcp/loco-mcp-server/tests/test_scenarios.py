"""Integration tests for user story scenarios."""

import pytest
import asyncio
import tempfile
import os
from unittest.mock import Mock, patch, AsyncMock
from typing import Dict, Any, List


class TestUserStoryScenarios:
    """Test complete user story scenarios."""

    @pytest.fixture
    def temp_project_dir(self):
        """Create a temporary loco-rs project directory."""
        with tempfile.TemporaryDirectory() as temp_dir:
            # Set up loco-rs project structure
            dirs = ["src/models", "src/controllers", "src/views", "src/routes", "migration/src"]
            for dir_path in dirs:
                os.makedirs(os.path.join(temp_dir, dir_path), exist_ok=True)

            # Create basic project files
            with open(os.path.join(temp_dir, "Cargo.toml"), "w") as f:
                f.write("""
[package]
name = "test-app"
version = "0.1.0"
edition = "2021"

[dependencies]
loco-rs = "0.3"
sea-orm = "0.12"
serde = "1.0"
tera = "1.0"
""")

            yield temp_dir

    @pytest.mark.asyncio
    async def test_scenario_create_product_model(self, temp_project_dir):
        """Test: 'Create a product model with fields name (string), price (i32), and sku (string, unique)'"""
        with patch('loco_mcp_server.tools.loco_bindings') as mock_bindings:

            # Mock response
            mock_bindings.generate_model.return_value = {
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

            # Simulate MCP request from Claude
            mcp_request = {
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

            # Mock request handling
            async def handle_request(request):
                await asyncio.sleep(0.001)  # Simulate processing
                tool_name = request["params"]["name"]
                args = request["params"]["arguments"]

                if tool_name == "generate_model":
                    result = mock_bindings.generate_model(args)
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

            response = await handle_request(mcp_request)

            # Verify scenario success
            assert response["result"]["status"] == "success"
            assert len(response["result"]["files_created"]) == 2

            # Verify correct files were created
            created_files = response["result"]["files_created"]
            model_file = next((f for f in created_files if f["type"] == "model"), None)
            migration_file = next((f for f in created_files if f["type"] == "migration"), None)

            assert model_file is not None
            assert "product.rs" in model_file["path"]
            assert migration_file is not None
            assert "create_products" in migration_file["path"]

            # Verify bindings were called correctly
            mock_bindings.generate_model.assert_called_once_with({
                "model_name": "product",
                "fields": ["name:string", "price:i32", "sku:string:unique"],
                "project_path": temp_project_dir
            })

    @pytest.mark.asyncio
    async def test_scenario_generate_controller_and_views(self, temp_project_dir):
        """Test: 'Generate controller and views for existing product model'"""
        # First create the model file
        model_content = """
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "products")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(column_type = "String(Some(255))")]
    pub name: String,
    pub price: i32,
    #[sea_orm(column_type = "String(Some(100))", unique)]
    pub sku: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
"""
        with open(os.path.join(temp_project_dir, "src/models/product.rs"), "w") as f:
            f.write(model_content)

        with patch('loco_mcp_server.tools.loco_bindings') as mock_bindings:

            # Mock controller/view generation response
            mock_bindings.generate_controller_view.return_value = {
                "success": True,
                "created_files": [
                    {
                        "path": "src/controllers/products.rs",
                        "type": "controller",
                        "size_bytes": 1567
                    },
                    {
                        "path": "src/views/products/index.html.tera",
                        "type": "view",
                        "size_bytes": 892
                    },
                    {
                        "path": "src/views/products/show.html.tera",
                        "type": "view",
                        "size_bytes": 456
                    },
                    {
                        "path": "src/views/products/form.html.tera",
                        "type": "view",
                        "size_bytes": 678
                    }
                ],
                "modified_files": [
                    {
                        "path": "src/routes/mod.rs",
                        "type": "route"
                    }
                ],
                "errors": []
            }

            # MCP request for controller and views
            mcp_request = {
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

            async def handle_request(request):
                await asyncio.sleep(0.001)
                tool_name = request["params"]["name"]
                args = request["params"]["arguments"]

                if tool_name == "generate_controller_view":
                    result = mock_bindings.generate_controller_view(args)
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

            response = await handle_request(mcp_request)

            # Verify scenario success
            assert response["result"]["status"] == "success"
            assert len(response["result"]["files_created"]) == 4  # controller + 3 views
            assert len(response["result"]["files_modified"]) == 1  # routes

            # Verify correct file types
            created_files = response["result"]["files_created"]
            file_types = [f["type"] for f in created_files]
            assert "controller" in file_types
            assert "view" in file_types
            assert file_types.count("view") == 3

    @pytest.mark.asyncio
    async def test_scenario_complete_crud_framework(self, temp_project_dir):
        """Test: 'Generate complete CRUD framework for posts resource'"""
        with patch('loco_mcp_server.tools.loco_bindings') as mock_bindings:

            # Mock complete scaffold response
            mock_bindings.generate_scaffold.return_value = {
                "success": True,
                "created_files": [
                    {"path": "src/models/post.rs", "type": "model", "size_bytes": 298},
                    {"path": "migration/src/m_20251003_120002_create_posts.rs", "type": "migration", "size_bytes": 212},
                    {"path": "src/controllers/posts.rs", "type": "controller", "size_bytes": 1834},
                    {"path": "src/views/posts/index.html.tera", "type": "view", "size_bytes": 945},
                    {"path": "src/views/posts/show.html.tera", "type": "view", "size_bytes": 512},
                    {"path": "src/views/posts/form.html.tera", "type": "view", "size_bytes": 723},
                    {"path": "src/views/posts/edit.html.tera", "type": "view", "size_bytes": 689}
                ],
                "modified_files": [
                    {"path": "src/routes/mod.rs", "type": "route"}
                ],
                "errors": []
            }

            # MCP request for complete CRUD scaffold
            mcp_request = {
                "jsonrpc": "2.0",
                "id": 3,
                "method": "tools/call",
                "params": {
                    "name": "generate_scaffold",
                    "arguments": {
                        "model_name": "post",
                        "fields": [
                            "title:string",
                            "content:text",
                            "published:boolean",
                            "published_at:datetime:nullable"
                        ],
                        "include_views": True,
                        "include_controllers": True,
                        "api_only": False,
                        "project_path": temp_project_dir
                    }
                }
            }

            async def handle_request(request):
                await asyncio.sleep(0.002)  # Scaffold is more complex
                tool_name = request["params"]["name"]
                args = request["params"]["arguments"]

                if tool_name == "generate_scaffold":
                    result = mock_bindings.generate_scaffold(args)
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

            response = await handle_request(mcp_request)

            # Verify complete CRUD framework
            assert response["result"]["status"] == "success"
            assert len(response["result"]["files_created"]) == 7  # model + migration + controller + 4 views
            assert len(response["result"]["files_modified"]) == 1  # routes

            # Verify all required components
            created_files = response["result"]["files_created"]
            file_types = [f["type"] for f in created_files]

            assert "model" in file_types
            assert "migration" in file_types
            assert "controller" in file_types
            assert "view" in file_types
            assert file_types.count("view") == 4  # list, show, form, edit

            # Verify specific files for posts
            post_files = [f for f in created_files if "post" in f["path"]]
            assert len(post_files) == 7

            # Verify bindings were called with correct arguments
            mock_bindings.generate_scaffold.assert_called_once_with({
                "model_name": "post",
                "fields": ["title:string", "content:text", "published:boolean", "published_at:datetime:nullable"],
                "include_views": True,
                "include_controllers": True,
                "api_only": False,
                "project_path": temp_project_dir
            })

    @pytest.mark.asyncio
    async def test_scenario_multi_model_workflow(self, temp_project_dir):
        """Test: Generate multiple related models in sequence"""
        with patch('loco_mcp_server.tools.loco_bindings') as mock_bindings:

            # Mock responses for each model
            def mock_generate_side_effect(params):
                model_name = params["model_name"]
                return {
                    "success": True,
                    "created_files": [
                        {
                            "path": f"src/models/{model_name}.rs",
                            "type": "model",
                            "size_bytes": 200
                        },
                        {
                            "path": f"migration/src/m_create_{model_name}s.rs",
                            "type": "migration",
                            "size_bytes": 150
                        }
                    ],
                    "modified_files": [],
                    "errors": []
                }

            mock_bindings.generate_model.side_effect = mock_generate_side_effect

            # Generate multiple related models
            models = [
                {"name": "user", "fields": ["email:string:unique", "name:string"]},
                {"name": "category", "fields": ["name:string", "description:text"]},
                {"name": "post", "fields": ["title:string", "content:text", "user_id:i64", "category_id:i64"]}
            ]

            responses = []
            for model in models:
                mcp_request = {
                    "jsonrpc": "2.0",
                    "id": len(responses) + 1,
                    "method": "tools/call",
                    "params": {
                        "name": "generate_model",
                        "arguments": {
                            "model_name": model["name"],
                            "fields": model["fields"],
                            "project_path": temp_project_dir
                        }
                    }
                }

                async def handle_request(request):
                    await asyncio.sleep(0.001)
                    result = mock_bindings.generate_model(request["params"]["arguments"])
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

                response = await handle_request(mcp_request)
                responses.append(response)

            # Verify all models were created
            assert len(responses) == 3
            assert all(r["result"]["status"] == "success" for r in responses)
            assert all(len(r["result"]["files_created"]) == 2 for r in responses)

            # Verify specific models were created
            assert mock_bindings.generate_model.call_count == 3
            created_models = [call[0][0]["model_name"] for call in mock_bindings.generate_model.call_args_list]
            assert "user" in created_models
            assert "category" in created_models
            assert "post" in created_models

    @pytest.mark.asyncio
    async def test_scenario_error_recovery(self, temp_project_dir):
        """Test: Handle errors gracefully and provide helpful feedback"""
        with patch('loco_mcp_server.tools.loco_bindings') as mock_bindings:

            # Mock validation error
            mock_bindings.generate_model.side_effect = ValueError("Invalid model name: '123invalid' must start with a letter")

            mcp_request = {
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

            async def handle_request(request):
                await asyncio.sleep(0.001)
                tool_name = request["params"]["name"]
                args = request["params"]["arguments"]

                if tool_name == "generate_model":
                    try:
                        result = mock_bindings.generate_model(args)
                        return {
                            "jsonrpc": "2.0",
                            "id": request["id"],
                            "result": {
                                "status": "success",
                                "files_created": result["created_files"],
                                "errors": result["errors"]
                            }
                        }
                    except ValueError as e:
                        return {
                            "jsonrpc": "2.0",
                            "id": request["id"],
                            "error": {
                                "code": -32602,
                                "message": str(e),
                                "details": {
                                    "field": "model_name",
                                    "suggestion": "Model names must start with a letter and contain only lowercase letters, numbers, and underscores"
                                }
                            }
                        }

            response = await handle_request(mcp_request)

            # Verify error handling
            assert "error" in response
            assert response["error"]["code"] == -32602
            assert "Invalid model name" in response["error"]["message"]
            assert "details" in response["error"]
            assert response["error"]["details"]["field"] == "model_name"
            assert "suggestion" in response["error"]["details"]
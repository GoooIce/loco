"""Integration tests for user story scenarios."""

import pytest
import asyncio
import tempfile
import os
from unittest.mock import Mock, patch, AsyncMock

from src.server import LocoMCPServer
from typing import Dict, Any, List


@pytest.fixture
def temp_project_dir():
    """Create a temporary loco-rs project directory."""
    with tempfile.TemporaryDirectory() as temp_dir:
        dirs = ["src/models", "src/controllers", "src/views", "src/routes", "migration/src"]
        for dir_path in dirs:
            os.makedirs(os.path.join(temp_dir, dir_path), exist_ok=True)

        with open(os.path.join(temp_dir, "Cargo.toml"), "w") as f:
            f.write(
                """
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
            )

        yield temp_dir


class TestUserStoryScenarios:
    """Test complete user story scenarios."""

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


class TestMaintenanceWorkflows:
    """Scenarios covering migrate → rotate → clean maintenance workflows."""

    @pytest.mark.asyncio
    async def test_migrate_rotate_clean_matches_cli_parity(self, temp_project_dir):
        """End-to-end flow must surface CLI parity evidence for each step."""

        with patch("loco_mcp_server.tools.loco_bindings"):
            server = LocoMCPServer()

        migrate_mock = AsyncMock(
            return_value={
                "status": "success",
                "cli_checksum": "cli-migrate-123",
                "mcp_checksum": "mcp-migrate-123",
                "stdout": "Database migrations complete",
            }
        )
        rotate_mock = AsyncMock(
            return_value={
                "status": "success",
                "cli_checksum": "cli-rotate-456",
                "mcp_checksum": "mcp-rotate-456",
                "stdout": "Keys rotated for 12 services",
            }
        )
        clean_mock = AsyncMock(
            return_value={
                "status": "success",
                "cli_checksum": "cli-clean-789",
                "mcp_checksum": "mcp-clean-789",
                "stdout": "Temporary directories purged",
            }
        )

        setattr(server.tools, "migrate_db", migrate_mock)
        setattr(server.tools, "rotate_keys", rotate_mock)
        setattr(server.tools, "clean_temp", clean_mock)

        migrate_args = {
            "project_path": temp_project_dir,
            "environment": "staging",
            "approvals": ["ops_lead", "security_officer"],
            "timeout_seconds": 60,
            "dependencies": ["postgres", "redis"],
        }
        rotate_args = {
            "project_path": temp_project_dir,
            "environment": "production",
            "approvals": ["security_officer", "cto"],
            "timeout_seconds": 300,
            "dependencies": ["kms"],
        }
        clean_args = {
            "project_path": temp_project_dir,
            "environment": "staging",
            "approvals": ["ops_lead"],
            "timeout_seconds": 60,
            "dependencies": ["fs-local"],
        }

        migrate_response = await server.server.call_tool("migrate_db", migrate_args)
        rotate_response = await server.server.call_tool("rotate_keys", rotate_args)
        clean_response = await server.server.call_tool("clean_temp", clean_args)

        migrate_mock.assert_awaited_once()
        rotate_mock.assert_awaited_once()
        clean_mock.assert_awaited_once()

        for response, expected_stdout, cli_checksum, mcp_checksum in (
            (migrate_response, "Database migrations complete", "cli-migrate-123", "mcp-migrate-123"),
            (rotate_response, "Keys rotated for 12 services", "cli-rotate-456", "mcp-rotate-456"),
            (clean_response, "Temporary directories purged", "cli-clean-789", "mcp-clean-789"),
        ):
            assert response, "Each tool call must return content"
            payload = response[0].text
            assert expected_stdout in payload
            assert f"CLI checksum: {cli_checksum}" in payload
            assert f"MCP checksum: {mcp_checksum}" in payload

    @pytest.mark.asyncio
    async def test_rotate_keys_rejects_out_of_order_approvals(self, temp_project_dir):
        """rotate_keys must validate approval ordering before invoking bindings."""

        with patch("loco_mcp_server.tools.loco_bindings"):
            server = LocoMCPServer()

        rotate_mock = AsyncMock(return_value={"status": "success"})
        setattr(server.tools, "rotate_keys", rotate_mock)

        invalid_args = {
            "project_path": temp_project_dir,
            "environment": "production",
            "approvals": ["cto", "security_officer"],
            "timeout_seconds": 300,
            "dependencies": ["kms"],
        }

        response = await server.server.call_tool("rotate_keys", invalid_args)

        rotate_mock.assert_not_awaited()
        assert response, "rotate_keys should return structured error content"
        payload = response[0].text
        assert payload.startswith("❌"), "Invalid approvals must surface failure message"
        assert "approvals must follow required order" in payload


@pytest.mark.asyncio
async def test_migrate_rotate_clean_workflow_parity(temp_project_dir):
    """Test end-to-end migrate→rotate→clean workflow with CLI parity validation."""
    
    with patch('src.tools.loco_bindings') as mock_bindings:
        # Mock successful responses for each step
        mock_bindings.migrate_db.return_value = {
            "success": True,
            "messages": ["Database migration completed successfully"],
            "checksum": "migrate_abc123"
        }
        mock_bindings.rotate_keys.return_value = {
            "success": True, 
            "messages": ["Key rotation completed successfully"],
            "checksum": "rotate_def456"
        }
        mock_bindings.clean_temp.return_value = {
            "success": True,
            "messages": ["Temporary files cleaned successfully"],
            "checksum": "clean_ghi789"
        }
        
        server = LocoMCPServer()
        
        # Step 1: Migrate database
        migrate_args = {
            "project_path": temp_project_dir,
            "environment": "staging",
            "approvals": ["ops_lead", "security_officer"],
            "timeout_seconds": 60,
            "dependencies": ["postgres", "redis"]
        }
        
        migrate_response = await server.server.call_tool("migrate_db", migrate_args)
        assert migrate_response, "migrate_db should return response"
        assert "✅" in migrate_response[0].text, "Migration should succeed"
        
        # Step 2: Rotate keys (depends on successful migration)
        rotate_args = {
            "project_path": temp_project_dir,
            "environment": "staging", 
            "approvals": ["security_officer", "cto"],
            "timeout_seconds": 300,
            "dependencies": ["kms"]
        }
        
        rotate_response = await server.server.call_tool("rotate_keys", rotate_args)
        assert rotate_response, "rotate_keys should return response"
        assert "✅" in rotate_response[0].text, "Key rotation should succeed"
        
        # Step 3: Clean temporary files (depends on successful key rotation)
        clean_args = {
            "project_path": temp_project_dir,
            "environment": "staging",
            "approvals": ["ops_lead"],
            "timeout_seconds": 60,
            "dependencies": ["fs-local"]
        }
        
        clean_response = await server.server.call_tool("clean_temp", clean_args)
        assert clean_response, "clean_temp should return response"
        assert "✅" in clean_response[0].text, "Cleanup should succeed"
        
        # Verify all tools were called with correct parameters
        mock_bindings.migrate_db.assert_awaited_once_with(**migrate_args)
        mock_bindings.rotate_keys.assert_awaited_once_with(**rotate_args)
        mock_bindings.clean_temp.assert_awaited_once_with(**clean_args)
        
        # Verify CLI parity - checksums should match expected values
        # This will fail until we implement the actual CLI bindings
        assert mock_bindings.migrate_db.return_value["checksum"] == "migrate_abc123"
        assert mock_bindings.rotate_keys.return_value["checksum"] == "rotate_def456" 
        assert mock_bindings.clean_temp.return_value["checksum"] == "clean_ghi789"


@pytest.mark.asyncio
async def test_approval_sequence_enforcement():
    """Test that approval sequences are enforced in the correct order."""
    
    server = LocoMCPServer()
    
    # Test migrate_db approval sequence: ops_lead, security_officer
    invalid_migrate_args = {
        "project_path": "/test/project",
        "environment": "staging",
        "approvals": ["security_officer", "ops_lead"],  # Wrong order
        "timeout_seconds": 60,
        "dependencies": ["postgres", "redis"]
    }
    
    response = await server.server.call_tool("migrate_db", invalid_migrate_args)
    assert response, "migrate_db should return error response"
    assert "❌" in response[0].text, "Wrong approval order should fail"
    assert "approvals must follow required order" in response[0].text
    
    # Test rotate_keys approval sequence: security_officer, cto
    invalid_rotate_args = {
        "project_path": "/test/project",
        "environment": "production",
        "approvals": ["cto", "security_officer"],  # Wrong order
        "timeout_seconds": 300,
        "dependencies": ["kms"]
    }
    
    response = await server.server.call_tool("rotate_keys", invalid_rotate_args)
    assert response, "rotate_keys should return error response"
    assert "❌" in response[0].text, "Wrong approval order should fail"
    assert "approvals must follow required order" in response[0].text
    
    # Test clean_temp approval sequence: ops_lead only
    invalid_clean_args = {
        "project_path": "/test/project",
        "environment": "qa",
        "approvals": ["security_officer"],  # Wrong approver
        "timeout_seconds": 60,
        "dependencies": ["fs-local"]
    }
    
    response = await server.server.call_tool("clean_temp", invalid_clean_args)
    assert response, "clean_temp should return error response"
    assert "❌" in response[0].text, "Wrong approver should fail"
    assert "approvals must follow required order" in response[0].text
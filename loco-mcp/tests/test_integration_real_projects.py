#!/usr/bin/env python3
"""
Integration tests with actual loco-rs projects

This test suite validates loco-mcp-server functionality against real loco-rs projects
to ensure compatibility and proper integration with the loco-rs ecosystem.
"""

import asyncio
import subprocess
import tempfile
import os
import sys
import pytest
from pathlib import Path
from typing import Dict, Any, Optional

# Add the source directories to Python path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'loco-mcp-server', 'src'))

from server import LocoMCPServer
from config import ServerConfig


class RealLocoProjectTest:
    """Test with real loco-rs projects"""

    def setup_method(self):
        """Setup test environment"""
        self.temp_dir = tempfile.mkdtemp(prefix="loco_real_integration_")
        self.project_path = None

    def teardown_method(self):
        """Cleanup test environment"""
        import shutil
        shutil.rmtree(self.temp_dir, ignore_errors=True)

    async def create_real_loco_project(self, project_name: str, starter: str = "saas") -> str:
        """Create a real loco-rs project using the loco CLI"""
        project_path = os.path.join(self.temp_dir, project_name)

        # Check if loco CLI is available
        try:
            result = subprocess.run(
                ["loco", "--version"],
                capture_output=True,
                text=True,
                timeout=30
            )
            if result.returncode != 0:
                pytest.skip("loco CLI not available - skipping real project tests")
                return None
        except (subprocess.TimeoutExpired, FileNotFoundError):
            pytest.skip("loco CLI not available - skipping real project tests")
            return None

        # Create the project
        try:
            result = subprocess.run(
                ["loco", "new", project_name, "--starter", starter, "--no-deps"],
                cwd=self.temp_dir,
                capture_output=True,
                text=True,
                timeout=120  # Allow more time for project creation
            )

            if result.returncode != 0:
                pytest.fail(f"Failed to create loco project: {result.stderr}")
                return None

            self.project_path = project_path
            return project_path

        except subprocess.TimeoutExpired:
            pytest.fail("loco new command timed out")
            return None

    def verify_loco_project_structure(self, project_path: str) -> bool:
        """Verify that the created project has proper loco-rs structure"""
        required_paths = [
            "src",
            "src/models",
            "src/controllers",
            "src/views",
            "src/routes",
            "migration",
            "Cargo.toml"
        ]

        for path in required_paths:
            full_path = os.path.join(project_path, path)
            if not os.path.exists(full_path):
                print(f"Missing required path: {full_path}")
                return False

        # Check Cargo.toml content
        cargo_toml_path = os.path.join(project_path, "Cargo.toml")
        with open(cargo_toml_path, 'r') as f:
            cargo_content = f.read()
            if "loco-rs" not in cargo_content:
                print("Cargo.toml does not contain loco-rs dependency")
                return False

        return True

    async def test_real_project_model_generation(self):
        """Test model generation in a real loco-rs project"""
        project_path = await self.create_real_loco_project("test_model_generation")
        if not project_path:
            return

        assert self.verify_loco_project_structure(project_path), "Invalid loco project structure"

        server = LocoMCPServer(ServerConfig(default_project_path=project_path))

        # Generate a user model
        result = await server.tools.generate_model({
            "model_name": "user",
            "fields": [
                "name:string",
                "email:string:unique",
                "password_hash:string",
                "created_at:datetime"
            ],
            "project_path": project_path
        })

        # Verify results
        assert result["success"], f"Model generation failed: {result.get('errors', [])}"
        assert len(result["created_files"]) == 2, "Should create model and migration files"

        # Verify files exist and have correct content
        model_file = os.path.join(project_path, "src/models/user.rs")
        assert os.path.exists(model_file), "Model file should exist"

        with open(model_file, 'r') as f:
            model_content = f.read()
            assert "pub struct Model" in model_content, "Model struct should exist"
            assert "pub name: String" in model_content, "Name field should exist"
            assert "pub email: String" in model_content, "Email field should exist"

        # Verify migration file
        migration_files = list(Path(project_path).glob("migration/src/m_*_create_users.rs"))
        assert len(migration_files) == 1, "Should create exactly one migration file"

        migration_file = migration_files[0]
        with open(migration_file, 'r') as f:
            migration_content = f.read()
            assert "create_table" in migration_content, "Migration should create table"
            assert "users" in migration_content, "Migration should reference users table"

        # Test that the generated code compiles
        self.verify_code_compiles(project_path)

    async def test_real_project_scaffold_generation(self):
        """Test scaffold generation in a real loco-rs project"""
        project_path = await self.create_real_loco_project("test_scaffold_generation")
        if not project_path:
            return

        assert self.verify_loco_project_structure(project_path), "Invalid loco project structure"

        server = LocoMCPServer(ServerConfig(default_project_path=project_path))

        # Generate a complete scaffold
        result = await server.tools.generate_scaffold({
            "model_name": "blog_post",
            "fields": [
                "title:string",
                "content:text",
                "published:boolean",
                "author_id:i64",
                "created_at:datetime"
            ],
            "include_views": True,
            "include_controllers": True,
            "project_path": project_path
        })

        # Verify results
        assert result["success"], f"Scaffold generation failed: {result.get('errors', [])}"
        assert len(result["created_files"]) >= 4, "Should create model, migration, controller, and views"

        # Verify model file
        model_file = os.path.join(project_path, "src/models/blog_post.rs")
        assert os.path.exists(model_file), "Model file should exist"

        # Verify controller file
        controller_file = os.path.join(project_path, "src/controllers/blog_posts.rs")
        assert os.path.exists(controller_file), "Controller file should exist"

        with open(controller_file, 'r') as f:
            controller_content = f.read()
            assert "impl ControllerActions" in controller_content, "Controller should implement ControllerActions"
            assert "async fn index" in controller_content, "Should have index action"
            assert "async fn show" in controller_content, "Should have show action"

        # Verify view files
        view_dir = os.path.join(project_path, "src/views/blog_posts")
        assert os.path.exists(view_dir), "View directory should exist"

        view_files = os.listdir(view_dir)
        expected_views = ["list.html.tera", "show.html.tera", "form.html.tera"]
        for view in expected_views:
            assert any(view in f for f in view_files), f"Should have {view} view"

        # Test compilation
        self.verify_code_compiles(project_path)

    async def test_real_project_api_only_scaffold(self):
        """Test API-only scaffold generation in a real loco-rs project"""
        project_path = await self.create_real_loco_project("test_api_scaffold")
        if not project_path:
            return

        assert self.verify_loco_project_structure(project_path), "Invalid loco project structure"

        server = LocoMCPServer(ServerConfig(default_project_path=project_path))

        # Generate API-only scaffold
        result = await server.tools.generate_scaffold({
            "model_name": "api_key",
            "fields": [
                "key:string:unique",
                "name:string",
                "permissions:json",
                "expires_at:datetime",
                "is_active:boolean"
            ],
            "api_only": True,
            "include_controllers": True,
            "project_path": project_path
        })

        # Verify results
        assert result["success"], f"API scaffold generation failed: {result.get('errors', [])}"
        assert len(result["created_files"]) >= 3, "Should create model, migration, and controller"

        # Verify controller exists
        controller_file = os.path.join(project_path, "src/controllers/api_keys.rs")
        assert os.path.exists(controller_file), "Controller file should exist"

        with open(controller_file, 'r') as f:
            controller_content = f.read()
            assert "format::json" in controller_content, "API controller should return JSON"
            assert "render!" not in controller_content, "API controller should not use templates"

        # Verify no view directory was created
        view_dir = os.path.join(project_path, "src/views/api_key")
        assert not os.path.exists(view_dir), "API-only scaffold should not create views"

        # Test compilation
        self.verify_code_compiles(project_path)

    async def test_real_project_controller_generation(self):
        """Test controller generation for existing model in real project"""
        project_path = await self.create_real_loco_project("test_controller_generation")
        if not project_path:
            return

        assert self.verify_loco_project_structure(project_path), "Invalid loco project structure"

        server = LocoMCPServer(ServerConfig(default_project_path=project_path))

        # First create a model
        model_result = await server.tools.generate_model({
            "model_name": "product",
            "fields": [
                "name:string",
                "description:text",
                "price:f64",
                "in_stock:boolean"
            ],
            "project_path": project_path
        })

        assert model_result["success"], "Model creation should succeed"

        # Then generate controller for the existing model
        controller_result = await server.tools.generate_controller_view({
            "model_name": "product",
            "actions": ["index", "show", "edit"],
            "view_types": ["list", "show", "form"],
            "project_path": project_path
        })

        # Verify results
        assert controller_result["success"], f"Controller generation failed: {controller_result.get('errors', [])}"
        assert len(controller_result["created_files"]) >= 1, "Should create controller and/or views"

        # Verify controller file exists
        controller_file = os.path.join(project_path, "src/controllers/product.rs")
        assert os.path.exists(controller_file), "Controller file should exist"

        # Verify views were created
        view_dir = os.path.join(project_path, "src/views/product")
        assert os.path.exists(view_dir), "View directory should exist"

        # Test compilation
        self.verify_code_compiles(project_path)

    async def test_real_project_complex_workflow(self):
        """Test complex workflow with multiple related models in real project"""
        project_path = await self.create_real_loco_project("test_complex_workflow")
        if not project_path:
            return

        assert self.verify_loco_project_structure(project_path), "Invalid loco project structure"

        server = LocoMCPServer(ServerConfig(default_project_path=project_path))

        # Step 1: Create user model
        user_result = await server.tools.generate_model({
            "model_name": "user",
            "fields": [
                "username:string:unique",
                "email:string:unique",
                "password_hash:string",
                "is_admin:boolean"
            ],
            "project_path": project_path
        })

        assert user_result["success"], "User model creation should succeed"

        # Step 2: Create category model
        category_result = await server.tools.generate_model({
            "model_name": "category",
            "fields": [
                "name:string",
                "description:text",
                "parent_id:i64:nullable"
            ],
            "project_path": project_path
        })

        assert category_result["success"], "Category model creation should succeed"

        # Step 3: Create blog post scaffold with foreign key
        blog_result = await server.tools.generate_scaffold({
            "model_name": "blog_post",
            "fields": [
                "title:string",
                "content:text",
                "published:boolean",
                "author_id:i64",  # Foreign key to user
                "category_id:i64"  # Foreign key to category
            ],
            "include_views": True,
            "include_controllers": True,
            "project_path": project_path
        })

        assert blog_result["success"], "Blog post scaffold creation should succeed"

        # Step 4: Create comment model with relationships
        comment_result = await server.tools.generate_model({
            "model_name": "comment",
            "fields": [
                "content:text",
                "author_id:i64",
                "blog_post_id:i64",
                "is_approved:boolean"
            ],
            "project_path": project_path
        })

        assert comment_result["success"], "Comment model creation should succeed"

        # Step 5: Generate controller for comments
        comment_controller_result = await server.tools.generate_controller_view({
            "model_name": "comment",
            "project_path": project_path
        })

        assert comment_controller_result["success"], "Comment controller creation should succeed"

        # Verify all files exist
        expected_files = [
            "src/models/user.rs",
            "src/models/category.rs",
            "src/models/blog_post.rs",
            "src/models/comment.rs",
            "src/controllers/blog_posts.rs",
            "src/controllers/comment.rs",
            "src/views/blog_posts",
            "src/views/comment"
        ]

        for file_path in expected_files:
            full_path = os.path.join(project_path, file_path)
            assert os.path.exists(full_path), f"Expected file {file_path} should exist"

        # Verify migration files exist
        migration_dir = Path(project_path) / "migration" / "src"
        migration_files = list(migration_dir.glob("m_*_create_*.rs"))
        assert len(migration_files) == 4, "Should have 4 migration files"

        # Test compilation
        self.verify_code_compiles(project_path)

    def verify_code_compiles(self, project_path: str):
        """Verify that the generated code compiles without errors"""
        try:
            # Run cargo check to verify compilation
            result = subprocess.run(
                ["cargo", "check"],
                cwd=project_path,
                capture_output=True,
                text=True,
                timeout=60  # Allow 60 seconds for compilation check
            )

            if result.returncode != 0:
                # Print compilation errors for debugging
                print("Compilation errors:")
                print(result.stderr)
                pytest.fail(f"Generated code does not compile: {result.stderr}")

        except subprocess.TimeoutExpired:
            pytest.fail("Cargo check timed out")

    async def test_real_project_error_handling(self):
        """Test error handling with real loco-rs projects"""
        project_path = await self.create_real_loco_project("test_error_handling")
        if not project_path:
            return

        assert self.verify_loco_project_structure(project_path), "Invalid loco project structure"

        server = LocoMCPServer(ServerConfig(default_project_path=project_path))

        # Test 1: Invalid model name
        invalid_name_result = await server.tools.generate_model({
            "model_name": "123invalid",
            "fields": ["name:string"],
            "project_path": project_path
        })

        assert not invalid_name_result["success"], "Should fail with invalid model name"
        assert len(invalid_name_result["errors"]) > 0, "Should provide error messages"

        # Test 2: Invalid field types
        invalid_field_result = await server.tools.generate_model({
            "model_name": "test",
            "fields": ["name:invalid_type"],
            "project_path": project_path
        })

        assert not invalid_field_result["success"], "Should fail with invalid field type"
        assert len(invalid_field_result["errors"]) > 0, "Should provide error messages"

        # Test 3: Duplicate model creation
        # First create a valid model
        valid_result = await server.tools.generate_model({
            "model_name": "duplicate_test",
            "fields": ["name:string"],
            "project_path": project_path
        })

        assert valid_result["success"], "First model creation should succeed"

        # Try to create the same model again
        duplicate_result = await server.tools.generate_model({
            "model_name": "duplicate_test",
            "fields": ["email:string"],
            "project_path": project_path
        })

        assert not duplicate_result["success"], "Should fail when creating duplicate model"
        assert any("already exists" in error.lower() for error in duplicate_result["errors"]), \
            "Should mention that model already exists"

        # Test 4: Controller for non-existent model
        controller_result = await server.tools.generate_controller_view({
            "model_name": "nonexistent_model",
            "project_path": project_path
        })

        assert not controller_result["success"], "Should fail when creating controller for non-existent model"
        assert any("not found" in error.lower() for error in controller_result["errors"]), \
            "Should mention that model was not found"

    async def test_real_project_performance(self):
        """Test performance with real loco-rs projects"""
        project_path = await self.create_real_loco_project("test_performance")
        if not project_path:
            return

        assert self.verify_loco_project_structure(project_path), "Invalid loco project structure"

        server = LocoMCPServer(ServerConfig(default_project_path=project_path))

        import time

        # Test model generation performance
        start_time = time.perf_counter()
        model_result = await server.tools.generate_model({
            "model_name": "performance_test",
            "fields": [
                "name:string",
                "description:text",
                "count:i32",
                "price:f64",
                "is_active:boolean",
                "created_at:datetime"
            ],
            "project_path": project_path
        })
        model_duration = (time.perf_counter() - start_time) * 1000

        assert model_result["success"], "Model generation should succeed"
        assert model_duration < 100, f"Model generation took {model_duration:.2f}ms (should be <100ms)"

        # Test scaffold generation performance
        start_time = time.perf_counter()
        scaffold_result = await server.tools.generate_scaffold({
            "model_name": "performance_scaffold",
            "fields": ["title:string", "content:text", "published:boolean"],
            "project_path": project_path
        })
        scaffold_duration = (time.perf_counter() - start_time) * 1000

        assert scaffold_result["success"], "Scaffold generation should succeed"
        assert scaffold_duration < 200, f"Scaffold generation took {scaffold_duration:.2f}ms (should be <200ms)"

        print(f"Performance with real project:")
        print(f"  Model generation: {model_duration:.2f}ms")
        print(f"  Scaffold generation: {scaffold_duration:.2f}ms")


class TestLocoVersionCompatibility:
    """Test compatibility with different loco-rs versions"""

    async def test_different_starters(self):
        """Test different loco-rs starter templates"""
        starters_to_test = ["saas", "api"]

        for starter in starters_to_test:
            print(f"\nTesting with {starter} starter...")

            temp_dir = tempfile.mkdtemp(prefix=f"loco_{starter}_test_")
            project_path = os.path.join(temp_dir, f"test_{starter}_project")

            try:
                # Create project with specific starter
                result = subprocess.run(
                    ["loco", "new", project_path, "--starter", starter, "--no-deps"],
                    cwd=temp_dir,
                    capture_output=True,
                    text=True,
                    timeout=120
                )

                if result.returncode != 0:
                    print(f"Could not create {starter} project: {result.stderr}")
                    continue

                # Test basic functionality
                server = LocoMCPServer(ServerConfig(default_project_path=project_path))

                test_result = await server.tools.generate_model({
                    "model_name": "test_model",
                    "fields": ["name:string", "value:i32"],
                    "project_path": project_path
                })

                assert test_result["success"], f"Model generation should work with {starter} starter"

                model_file = os.path.join(project_path, "src/models/test_model.rs")
                assert os.path.exists(model_file), f"Model file should exist in {starter} project"

                print(f"✅ {starter} starter works correctly")

            except subprocess.TimeoutExpired:
                print(f"❌ {starter} starter creation timed out")
            except Exception as e:
                print(f"❌ {starter} starter test failed: {e}")
            finally:
                import shutil
                shutil.rmtree(temp_dir, ignore_errors=True)


if __name__ == "__main__":
    # Run integration tests
    pytest.main([__file__, "-v", "-s", "--tb=short"])
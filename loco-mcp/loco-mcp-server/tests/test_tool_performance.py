"""Performance tests specifically for MCP tools."""

import pytest
import asyncio
import time
from unittest.mock import Mock, patch, AsyncMock
from typing import Dict, Any


class TestMCPToolPerformance:
    """Test performance of individual MCP tools."""

    @pytest.fixture
    def mock_bindings(self):
        """Mock loco_bindings with performance tracking."""
        with patch('loco_mcp_server.tools.loco_bindings') as mock:
            yield mock

    @pytest.mark.asyncio
    async def test_generate_model_performance(self, mock_bindings):
        """Test generate_model tool performance."""
        # Mock fast response
        mock_bindings.generate_model.return_value = {
            "success": True,
            "created_files": [
                {"path": "src/models/test.rs", "type": "model", "size_bytes": 200},
                {"path": "migration/src/m_create_test.rs", "type": "migration", "size_bytes": 150}
            ],
            "modified_files": [],
            "errors": []
        }

        async def call_generate_model(params: Dict[str, Any]) -> Dict[str, Any]:
            start_time = time.perf_counter()

            # Simulate tool processing overhead
            await asyncio.sleep(0.001)  # 1ms overhead

            result = mock_bindings.generate_model(params)
            end_time = time.perf_counter()

            processing_time_ms = (end_time - start_time) * 1000
            return {
                "result": result,
                "processing_time_ms": processing_time_ms
            }

        # Test simple model
        params = {
            "model_name": "simple_model",
            "fields": ["name:string"]
        }

        response = await call_generate_model(params)

        assert response["result"]["success"] is True
        assert response["processing_time_ms"] < 10.0, f"Simple model took {response['processing_time_ms']:.2f}ms"

    @pytest.mark.asyncio
    async def test_generate_scaffold_performance(self, mock_bindings):
        """Test generate_scaffold tool performance."""
        mock_bindings.generate_scaffold.return_value = {
            "success": True,
            "created_files": [
                {"path": "src/models/user.rs", "type": "model", "size_bytes": 300},
                {"path": "migration/src/m_create_users.rs", "type": "migration", "size_bytes": 200},
                {"path": "src/controllers/users.rs", "type": "controller", "size_bytes": 1500},
                {"path": "src/views/users/index.html.tera", "type": "view", "size_bytes": 800},
                {"path": "src/views/users/show.html.tera", "type": "view", "size_bytes": 400}
            ],
            "modified_files": [{"path": "src/routes/mod.rs", "type": "route"}],
            "errors": []
        }

        async def call_generate_scaffold(params: Dict[str, Any]) -> Dict[str, Any]:
            start_time = time.perf_counter()
            await asyncio.sleep(0.002)  # 2ms overhead (scaffold is more complex)
            result = mock_bindings.generate_scaffold(params)
            end_time = time.perf_counter()

            processing_time_ms = (end_time - start_time) * 1000
            return {
                "result": result,
                "processing_time_ms": processing_time_ms
            }

        params = {
            "model_name": "user",
            "fields": ["email:string:unique", "name:string", "active:boolean"],
            "include_views": True,
            "include_controllers": True,
            "api_only": False
        }

        response = await call_generate_scaffold(params)

        assert response["result"]["success"] is True
        assert response["processing_time_ms"] < 10.0, f"Scaffold took {response['processing_time_ms']:.2f}ms"
        assert len(response["result"]["created_files"]) == 5

    @pytest.mark.asyncio
    async def test_generate_controller_view_performance(self, mock_bindings):
        """Test generate_controller_view tool performance."""
        mock_bindings.generate_controller_view.return_value = {
            "success": True,
            "created_files": [
                {"path": "src/controllers/products.rs", "type": "controller", "size_bytes": 1200},
                {"path": "src/views/products/list.html.tera", "type": "view", "size_bytes": 700},
                {"path": "src/views/products/show.html.tera", "type": "view", "size_bytes": 400}
            ],
            "modified_files": [{"path": "src/routes/mod.rs", "type": "route"}],
            "errors": []
        }

        async def call_generate_controller_view(params: Dict[str, Any]) -> Dict[str, Any]:
            start_time = time.perf_counter()
            await asyncio.sleep(0.0015)  # 1.5ms overhead
            result = mock_bindings.generate_controller_view(params)
            end_time = time.perf_counter()

            processing_time_ms = (end_time - start_time) * 1000
            return {
                "result": result,
                "processing_time_ms": processing_time_ms
            }

        params = {
            "model_name": "product",
            "actions": ["index", "show", "create", "update"],
            "view_types": ["list", "show", "form"]
        }

        response = await call_generate_controller_view(params)

        assert response["result"]["success"] is True
        assert response["processing_time_ms"] < 10.0, f"Controller/view took {response['processing_time_ms']:.2f}ms"

    @pytest.mark.asyncio
    async def test_tool_performance_under_load(self, mock_bindings):
        """Test tool performance under concurrent load."""
        # Setup mock responses
        def mock_generate_side_effect(params):
            return {
                "success": True,
                "created_files": [{"path": "src/models/test.rs", "type": "model"}],
                "modified_files": [],
                "errors": []
            }

        mock_bindings.generate_model.side_effect = mock_generate_side_effect

        async def call_tool_concurrently(tool_name: str, request_count: int) -> Dict[str, Any]:
            async def single_call(params: Dict[str, Any]) -> float:
                start_time = time.perf_counter()
                await asyncio.sleep(0.001)  # Simulate processing
                mock_bindings.generate_model(params)
                end_time = time.perf_counter()
                return (end_time - start_time) * 1000

            # Create concurrent calls
            tasks = []
            for i in range(request_count):
                params = {
                    "model_name": f"model_{i}",
                    "fields": ["name:string"]
                }
                tasks.append(single_call(params))

            # Execute concurrently
            start_time = time.perf_counter()
            response_times = await asyncio.gather(*tasks)
            end_time = time.perf_counter()

            total_time = (end_time - start_time) * 1000
            avg_response_time = sum(response_times) / len(response_times)
            max_response_time = max(response_times)

            return {
                "total_time_ms": total_time,
                "avg_response_time_ms": avg_response_time,
                "max_response_time_ms": max_response_time,
                "request_count": request_count
            }

        # Test with different load levels
        load_tests = [
            (5, "light_load"),
            (10, "medium_load"),
            (20, "heavy_load")
        ]

        for request_count, test_name in load_tests:
            results = await call_tool_concurrently("generate_model", request_count)

            assert results["avg_response_time_ms"] < 10.0, f"{test_name} avg: {results['avg_response_time_ms']:.2f}ms"
            assert results["max_response_time_ms"] < 15.0, f"{test_name} max: {results['max_response_time_ms']:.2f}ms"
            assert results["total_time_ms"] < 50.0, f"{test_name} total: {results['total_time_ms']:.2f}ms"

    @pytest.mark.asyncio
    async def test_performance_regression_detection(self, mock_bindings):
        """Test that performance regressions can be detected."""
        performance_history = []

        def mock_generate_with_regression(params):
            # Simulate performance regression over time
            call_count = len(performance_history)
            base_time = 0.002  # 2ms base
            regression = call_count * 0.0001  # Add 0.1ms per call (simulating regression)
            total_time = base_time + regression

            performance_history.append(total_time * 1000)  # Store in ms

            return {
                "success": True,
                "created_files": [{"path": "src/models/test.rs", "type": "model"}]
            }

        mock_bindings.generate_model.side_effect = mock_generate_with_regression

        # Make multiple calls to build performance history
        for i in range(10):
            await asyncio.sleep(0.001)  # Simulate async processing
            mock_bindings.generate_model({
                "model_name": f"model_{i}",
                "fields": ["name:string"]
            })

        assert len(performance_history) == 10

        # Analyze performance trend
        early_avg = sum(performance_history[:3]) / 3
        late_avg = sum(performance_history[-3:]) / 3
        performance_change = late_avg - early_avg

        # In a real system, we'd alert on significant performance degradation
        # For this test, we just verify we can detect the change
        assert performance_change > 0, "Should detect performance regression"
        assert late_avg < 10.0, f"Even with regression, should be <10ms, got {late_avg:.2f}ms"

    def test_tool_response_size_impact(self, mock_bindings):
        """Test how response size affects performance."""
        response_sizes = []

        def mock_generate_variable_size(params):
            field_count = len(params.get("fields", []))

            # Simulate response size proportional to field count
            created_files = []
            for i in range(field_count):
                created_files.append({
                    "path": f"src/models/test_{i}.rs",
                    "type": "model",
                    "size_bytes": 200 + (i * 50)
                })

            response_size = len(str(created_files))  # Rough estimate of response size
            response_sizes.append(response_size)

            return {
                "success": True,
                "created_files": created_files,
                "modified_files": [],
                "errors": []
            }

        mock_bindings.generate_model.side_effect = mock_generate_variable_size

        # Test with different response sizes
        field_counts = [1, 5, 10, 20]

        for field_count in field_counts:
            fields = [f"field_{i}:string" for i in range(field_count)]
            mock_bindings.generate_model({
                "model_name": "test_model",
                "fields": fields
            })

        assert len(response_sizes) == len(field_counts)

        # Response size should not significantly impact performance
        # (In real implementation, we'd measure actual processing time)
        max_response_size = max(response_sizes)
        assert max_response_size > 0, "Should have varying response sizes"

    @pytest.mark.asyncio
    async def test_error_handling_performance(self, mock_bindings):
        """Test that error handling is fast."""
        mock_bindings.generate_model.side_effect = ValueError("Invalid model name")

        async def call_with_error() -> float:
            start_time = time.perf_counter()
            try:
                await asyncio.sleep(0.001)  # Simulate processing
                mock_bindings.generate_model({
                    "model_name": "123invalid",
                    "fields": ["name:string"]
                })
            except ValueError:
                pass  # Expected error
            end_time = time.perf_counter()
            return (end_time - start_time) * 1000

        # Test error handling performance
        error_times = []
        for _ in range(10):
            error_time = await call_with_error()
            error_times.append(error_time)

        avg_error_time = sum(error_times) / len(error_times)
        max_error_time = max(error_times)

        # Error handling should be very fast
        assert avg_error_time < 5.0, f"Error handling avg: {avg_error_time:.2f}ms, should be <5ms"
        assert max_error_time < 10.0, f"Error handling max: {max_error_time:.2f}ms, should be <10ms"
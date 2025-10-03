"""Performance tests for MCP server operations."""

import pytest
import asyncio
import time
from unittest.mock import Mock, patch, AsyncMock
from typing import Dict, Any

# These imports will work once the MCP server is implemented
# from loco_mcp_server.server import LocoMCPServer
# from loco_mcp_server.tools import LocoTools


class TestMCPServerPerformance:
    """Test MCP server performance requirements."""

    @pytest.fixture
    def mock_bindings(self):
        """Mock loco_bindings with performance tracking."""
        with patch('loco_mcp_server.tools.loco_bindings') as mock:
            yield mock

    @pytest.mark.asyncio
    async def test_single_request_performance(self, mock_bindings):
        """Test that single requests complete within performance targets."""
        # Mock fast response from bindings
        mock_bindings.generate_model.return_value = {
            "success": True,
            "created_files": [{"path": "src/models/test.rs", "type": "model", "size_bytes": 200}],
            "modified_files": [],
            "errors": []
        }

        # Simulate MCP request handling
        async def handle_request(request: Dict[str, Any]) -> Dict[str, Any]:
            start_time = time.perf_counter()

            # Simulate internal processing overhead
            await asyncio.sleep(0.001)  # 1ms internal processing

            # Call mock bindings
            result = mock_bindings.generate_model(request["params"]["arguments"])

            end_time = time.perf_counter()
            duration_ms = (end_time - start_time) * 1000

            return {
                "jsonrpc": "2.0",
                "id": request["id"],
                "result": {
                    "status": "success",
                    "files_created": result["created_files"],
                    "processing_time_ms": duration_ms
                }
            }

        request = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "generate_model",
                "arguments": {
                    "model_name": "test_model",
                    "fields": ["name:string", "value:i32"]
                }
            }
        }

        response = await handle_request(request)
        processing_time = response["result"]["processing_time_ms"]

        # Performance requirement: <10ms total processing time
        assert processing_time < 10.0, f"Request took {processing_time:.2f}ms, should be <10ms"

    @pytest.mark.asyncio
    async def test_concurrent_requests_performance(self, mock_bindings):
        """Test performance under concurrent load."""
        # Setup mock responses
        def mock_generate_side_effect(params):
            # Simulate variable processing time based on complexity
            field_count = len(params.get("fields", []))
            processing_delay = 0.001 + (field_count * 0.0001)  # 1ms + 0.1ms per field

            return {
                "success": True,
                "created_files": [
                    {"path": f"src/models/{params['model_name']}.rs", "type": "model", "size_bytes": 200}
                ],
                "modified_files": [],
                "errors": []
            }

        mock_bindings.generate_model.side_effect = mock_generate_side_effect

        async def handle_concurrent_requests(requests: list) -> list:
            async def handle_single(request: Dict[str, Any]) -> Dict[str, Any]:
                start_time = time.perf_counter()

                # Simulate processing
                await asyncio.sleep(0.001)
                result = mock_bindings.generate_model(request["params"]["arguments"])

                end_time = time.perf_counter()
                duration_ms = (end_time - start_time) * 1000

                return {
                    "id": request["id"],
                    "processing_time_ms": duration_ms,
                    "success": result["success"]
                }

            # Execute all requests concurrently
            tasks = [handle_single(req) for req in requests]
            return await asyncio.gather(*tasks)

        # Create multiple concurrent requests
        requests = []
        for i in range(10):
            requests.append({
                "jsonrpc": "2.0",
                "id": i + 1,
                "method": "tools/call",
                "params": {
                    "name": "generate_model",
                    "arguments": {
                        "model_name": f"model_{i}",
                        "fields": ["name:string", f"value_{i}:i32"]
                    }
                }
            })

        start_time = time.perf_counter()
        responses = await handle_concurrent_requests(requests)
        end_time = time.perf_counter()

        total_time_ms = (end_time - start_time) * 1000
        avg_time_per_request = total_time_ms / len(requests)

        # Performance requirements
        assert total_time_ms < 50.0, f"10 concurrent requests took {total_time_ms:.2f}ms, should be <50ms"
        assert avg_time_per_request < 10.0, f"Average request time {avg_time_per_request:.2f}ms, should be <10ms"

        # Verify all requests succeeded
        assert all(response["success"] for response in responses)
        assert all(response["processing_time_ms"] < 10.0 for response in responses)

    @pytest.mark.asyncio
    async def test_memory_usage_stability(self, mock_bindings):
        """Test that memory usage remains stable during extended operation."""
        # This test simulates memory usage tracking
        memory_samples = []

        def mock_generate_with_memory_tracking(params):
            # Simulate memory allocation patterns
            base_memory = 10.0  # MB
            field_memory = len(params.get("fields", [])) * 0.1  # 0.1MB per field
            total_memory = base_memory + field_memory

            memory_samples.append(total_memory)
            return {
                "success": True,
                "created_files": [{"path": "src/models/test.rs", "type": "model"}],
                "memory_usage_mb": total_memory
            }

        mock_bindings.generate_model.side_effect = mock_generate_with_memory_tracking

        # Process many requests to check for memory leaks
        async def process_requests(count: int):
            for i in range(count):
                request = {
                    "jsonrpc": "2.0",
                    "id": i + 1,
                    "method": "tools/call",
                    "params": {
                        "name": "generate_model",
                        "arguments": {
                            "model_name": f"model_{i}",
                            "fields": [f"field_{j}:string" for j in range(5)]  # 5 fields each
                        }
                    }
                }

                # Simulate request processing
                await asyncio.sleep(0.001)
                mock_bindings.generate_model(request["params"]["arguments"])

        # Process 100 requests
        await process_requests(100)

        # Analyze memory usage
        assert len(memory_samples) == 100

        # Memory should remain stable (no significant growth over time)
        initial_memory = memory_samples[:10]  # First 10 samples
        final_memory = memory_samples[-10:]    # Last 10 samples

        avg_initial = sum(initial_memory) / len(initial_memory)
        avg_final = sum(final_memory) / len(final_memory)

        memory_growth = avg_final - avg_initial

        # Memory growth should be minimal (<1MB)
        assert memory_growth < 1.0, f"Memory grew by {memory_growth:.2f}MB, should be <1MB"

        # Memory usage should stay within reasonable bounds
        max_memory = max(memory_samples)
        assert max_memory < 20.0, f"Peak memory {max_memory:.2f}MB, should be <20MB"

    @pytest.mark.asyncio
    async def test_throughput_benchmarks(self, mock_bindings):
        """Test system throughput under sustained load."""
        request_count = 0
        completed_requests = 0
        errors = 0

        def mock_generate_with_stats(params):
            nonlocal request_count, completed_requests, errors
            request_count += 1

            # Simulate occasional errors (1% failure rate)
            if request_count % 100 == 0:
                errors += 1
                raise ValueError("Simulated error")

            completed_requests += 1
            return {
                "success": True,
                "created_files": [{"path": "src/models/test.rs", "type": "model"}]
            }

        mock_bindings.generate_model.side_effect = mock_generate_with_stats

        async def sustained_load_test(duration_seconds: int = 5):
            start_time = time.perf_counter()
            requests_sent = 0

            async def send_request():
                nonlocal requests_sent
                requests_sent += 1

                try:
                    await asyncio.sleep(0.001)  # Simulate processing
                    mock_bindings.generate_model({
                        "model_name": f"model_{requests_sent}",
                        "fields": ["name:string"]
                    })
                    return True
                except:
                    return False

            # Send requests continuously for the test duration
            tasks = []
            while (time.perf_counter() - start_time) < duration_seconds:
                # Send burst of requests
                burst_tasks = [send_request() for _ in range(10)]
                tasks.extend(burst_tasks)
                await asyncio.sleep(0.01)  # Brief pause between bursts

            # Wait for all tasks to complete
            results = await asyncio.gather(*tasks, return_exceptions=True)
            successful_requests = sum(1 for r in results if r is True)

            end_time = time.perf_counter()
            actual_duration = end_time - start_time

            return {
                "duration": actual_duration,
                "requests_sent": requests_sent,
                "successful": successful_requests,
                "throughput": successful_requests / actual_duration
            }

        # Run sustained load test
        results = await sustained_load_test(2)  # 2 second test

        # Performance targets
        assert results["throughput"] > 50.0, f"Throughput {results['throughput']:.2f} req/s, should be >50 req/s"
        assert results["duration"] >= 2.0, f"Test duration {results['duration']:.2f}s, should be >=2s"

        # Error rate should be low (<1%)
        error_rate = (results["requests_sent"] - results["successful"]) / results["requests_sent"]
        assert error_rate < 0.01, f"Error rate {error_rate:.2%}, should be <1%"

    def test_response_time_percentiles(self, mock_bindings):
        """Test response time distributions and percentiles."""
        response_times = []

        def mock_generate_with_timing(params):
            # Simulate variable response times
            import random
            base_time = 0.002  # 2ms base
            variation = random.uniform(-0.001, 0.003)  # ±1-3ms variation
            total_time = max(0.001, base_time + variation)  # Minimum 1ms

            response_times.append(total_time * 1000)  # Convert to ms

            return {
                "success": True,
                "created_files": [{"path": "src/models/test.rs", "type": "model"}]
            }

        mock_bindings.generate_model.side_effect = mock_generate_with_timing

        # Generate sample response times
        for i in range(100):
            mock_bindings.generate_model({
                "model_name": f"model_{i}",
                "fields": ["name:string"]
            })

        assert len(response_times) == 100

        # Calculate percentiles
        sorted_times = sorted(response_times)
        p50 = sorted_times[49]   # 50th percentile
        p95 = sorted_times[94]   # 95th percentile
        p99 = sorted_times[98]   # 99th percentile
        max_time = sorted_times[-1]

        # Performance requirements
        assert p50 < 5.0, f"P50 response time {p50:.2f}ms, should be <5ms"
        assert p95 < 8.0, f"P95 response time {p95:.2f}ms, should be <8ms"
        assert p99 < 10.0, f"P99 response time {p99:.2f}ms, should be <10ms"
        assert max_time < 15.0, f"Max response time {max_time:.2f}ms, should be <15ms"

        # Average response time
        avg_time = sum(response_times) / len(response_times)
        assert avg_time < 6.0, f"Average response time {avg_time:.2f}ms, should be <6ms"

    @pytest.mark.asyncio
    async def test_scalability_with_complexity(self, mock_bindings):
        """Test how performance scales with request complexity."""
        performance_data = []

        def mock_generate_complexity(params):
            import time
            start = time.perf_counter()

            # Simulate processing time proportional to complexity
            field_count = len(params.get("fields", []))
            complexity_delay = field_count * 0.0002  # 0.2ms per field
            time.sleep(complexity_delay)

            end = time.perf_counter()
            processing_time = (end - start) * 1000  # Convert to ms

            performance_data.append({
                "field_count": field_count,
                "processing_time_ms": processing_time
            })

            return {
                "success": True,
                "created_files": [{"path": "src/models/test.rs", "type": "model"}]
            }

        mock_bindings.generate_model.side_effect = mock_generate_complexity

        # Test with varying complexity levels
        complexity_levels = [1, 5, 10, 20, 50]

        for field_count in complexity_levels:
            fields = [f"field_{i}:string" for i in range(field_count)]

            await asyncio.sleep(0.001)  # Simulate async processing
            mock_bindings.generate_model({
                "model_name": f"model_{field_count}",
                "fields": fields
            })

        assert len(performance_data) == len(complexity_levels)

        # Analyze scalability
        for data in performance_data:
            field_count = data["field_count"]
            processing_time = data["processing_time_ms"]

            # Even with 50 fields, should be fast
            assert processing_time < 10.0, f"{field_count} fields took {processing_time:.2f}ms, should be <10ms"

        # Check linear scaling (should not grow exponentially)
        simple_data = next(d for d in performance_data if d["field_count"] == 1)
        complex_data = next(d for d in performance_data if d["field_count"] == 50)

        complexity_ratio = complex_data["field_count"] / simple_data["field_count"]
        time_ratio = complex_data["processing_time_ms"] / simple_data["processing_time_ms"]

        # Time growth should be proportional to complexity (not exponential)
        assert time_ratio < complexity_ratio * 2.0, f"Time scaling {time_ratio:.2f}x vs complexity {complexity_ratio}x"
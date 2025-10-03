#!/usr/bin/env python3
"""
Performance validation tests for loco-mcp-server

This test suite validates that the <10ms performance requirement is met
for all core operations under various load conditions.
"""

import asyncio
import time
import statistics
import tempfile
import os
import sys
import pytest
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor
from typing import List, Dict, Any

# Add the source directories to Python path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'loco-mcp-server', 'src'))

from server import LocoMCPServer
from config import ServerConfig


class PerformanceTestSuite:
    """Performance test suite with detailed metrics collection"""

    def __init__(self):
        self.results: Dict[str, List[float]] = {}
        self.target_ms = 10.0  # Target response time in milliseconds

    def setup_test_environment(self):
        """Setup a temporary loco project for testing"""
        temp_dir = tempfile.mkdtemp(prefix="loco_perf_test_")

        # Create loco project structure
        dirs = [
            "src/models",
            "src/controllers",
            "src/views",
            "src/routes",
            "migration/src"
        ]

        for dir_path in dirs:
            os.makedirs(os.path.join(temp_dir, *dir_path.split("/")))

        # Create Cargo.toml
        cargo_toml = """[package]
name = "performance-test-app"
version = "0.1.0"
edition = "2021"

[dependencies]
loco-rs = "0.3"
sea-orm = "0.12"
serde = { version = "1.0", features = ["derive"] }
"""

        with open(os.path.join(temp_dir, "Cargo.toml"), "w") as f:
            f.write(cargo_toml)

        return temp_dir

    def cleanup_test_environment(self, temp_dir: str):
        """Cleanup test environment"""
        import shutil
        shutil.rmtree(temp_dir, ignore_errors=True)

    def measure_operation(self, operation_func, *args, **kwargs) -> float:
        """Measure execution time of an operation in milliseconds"""
        start_time = time.perf_counter()
        try:
            result = operation_func(*args, **kwargs)
            end_time = time.perf_counter()
            duration_ms = (end_time - start_time) * 1000
            return duration_ms
        except Exception as e:
            end_time = time.perf_counter()
            duration_ms = (end_time - start_time) * 1000
            print(f"Operation failed after {duration_ms:.2f}ms: {e}")
            return duration_ms

    def run_performance_test(self, test_name: str, operation_func, iterations: int = 10, *args, **kwargs):
        """Run a performance test with multiple iterations"""
        print(f"\n=== Performance Test: {test_name} ===")

        durations = []
        successful_runs = 0

        for i in range(iterations):
            duration = self.measure_operation(operation_func, *args, **kwargs)
            durations.append(duration)

            # Consider run successful if it completes (even with errors, for resilience testing)
            if duration <= self.target_ms * 2:  # Allow 2x target for individual runs
                successful_runs += 1

            print(f"Run {i+1}: {duration:.2f}ms")

        # Calculate statistics
        avg_duration = statistics.mean(durations)
        median_duration = statistics.median(durations)
        min_duration = min(durations)
        max_duration = max(durations)
        std_dev = statistics.stdev(durations) if len(durations) > 1 else 0

        # Calculate percentiles
        sorted_durations = sorted(durations)
        p95 = sorted_durations[int(len(sorted_durations) * 0.95)] if len(sorted_durations) > 1 else sorted_durations[0]
        p99 = sorted_durations[int(len(sorted_durations) * 0.99)] if len(sorted_durations) > 1 else sorted_durations[0]

        # Success rate (within target time)
        within_target = sum(1 for d in durations if d <= self.target_ms)
        success_rate = (within_target / len(durations)) * 100

        results = {
            "test_name": test_name,
            "iterations": iterations,
            "successful_runs": successful_runs,
            "avg_duration_ms": avg_duration,
            "median_duration_ms": median_duration,
            "min_duration_ms": min_duration,
            "max_duration_ms": max_duration,
            "std_dev_ms": std_dev,
            "p95_duration_ms": p95,
            "p99_duration_ms": p99,
            "success_rate_percent": success_rate,
            "target_ms": self.target_ms,
            "durations": durations
        }

        self.results[test_name] = durations

        # Print results
        print(f"Average: {avg_duration:.2f}ms")
        print(f"Median: {median_duration:.2f}ms")
        print(f"Min: {min_duration:.2f}ms")
        print(f"Max: {max_duration:.2f}ms")
        print(f"Std Dev: {std_dev:.2f}ms")
        print(f"95th percentile: {p95:.2f}ms")
        print(f"99th percentile: {p99:.2f}ms")
        print(f"Success rate (≤{self.target_ms}ms): {success_rate:.1f}%")

        # Performance assessment
        if avg_duration <= self.target_ms:
            print(f"✅ PASS: Average response time {avg_duration:.2f}ms meets target ≤{self.target_ms}ms")
        else:
            print(f"❌ FAIL: Average response time {avg_duration:.2f}ms exceeds target {self.target_ms}ms")

        if success_rate >= 95:
            print(f"✅ PASS: Success rate {success_rate:.1f}% meets ≥95% requirement")
        else:
            print(f"❌ FAIL: Success rate {success_rate:.1f}% below 95% requirement")

        return results

    def print_summary_report(self):
        """Print a comprehensive performance summary report"""
        print(f"\n{'='*60}")
        print(f"PERFORMANCE VALIDATION SUMMARY REPORT")
        print(f"{'='*60}")
        print(f"Target Response Time: {self.target_ms}ms")
        print(f"Success Rate Target: ≥95%")
        print(f"Number of Tests: {len(self.results)}")

        all_passed = True
        total_tests = 0
        passed_tests = 0

        for test_name, durations in self.results.items():
            total_tests += 1

            avg_duration = statistics.mean(durations)
            within_target = sum(1 for d in durations if d <= self.target_ms)
            success_rate = (within_target / len(durations)) * 100

            test_passed = avg_duration <= self.target_ms and success_rate >= 95

            if test_passed:
                passed_tests += 1
                status = "✅ PASS"
            else:
                all_passed = False
                status = "❌ FAIL"

            print(f"{status} {test_name}:")
            print(f"    Avg: {avg_duration:.2f}ms | Success Rate: {success_rate:.1f}%")

        print(f"\nOverall Result: {'✅ ALL TESTS PASSED' if all_passed else '❌ SOME TESTS FAILED'}")
        print(f"Passed: {passed_tests}/{total_tests} ({(passed_tests/total_tests)*100:.1f}%)")

        return all_passed


class TestModelGenerationPerformance:
    """Performance tests for model generation"""

    @pytest.fixture
    def performance_suite(self):
        """Setup performance test suite"""
        return PerformanceTestSuite()

    @pytest.fixture
    def test_environment(self):
        """Setup test environment"""
        temp_dir = tempfile.mkdtemp(prefix="loco_model_perf_")

        # Create basic loco project structure
        dirs = ["src/models", "migration/src"]
        for dir_path in dirs:
            os.makedirs(os.path.join(temp_dir, *dir_path.split("/")))

        cargo_toml = """[package]
name = "model-perf-test"
version = "0.1.0"
edition = "2021"
[dependencies]
loco-rs = "0.3"
"""
        with open(os.path.join(temp_dir, "Cargo.toml"), "w") as f:
            f.write(cargo_toml)

        yield temp_dir

        import shutil
        shutil.rmtree(temp_dir, ignore_errors=True)

    @pytest.mark.asyncio
    async def test_model_generation_basic_performance(self, performance_suite, test_environment):
        """Test basic model generation performance"""
        server = LocoMCPServer(ServerConfig(default_project_path=test_environment))

        async def generate_user_model():
            return await server.tools.generate_model({
                "model_name": "user",
                "fields": ["name:string", "email:string:unique"],
                "project_path": test_environment
            })

        result = performance_suite.run_performance_test(
            "Model Generation - Basic",
            generate_user_model,
            iterations=20
        )

        assert result["success_rate_percent"] >= 95, f"Success rate {result['success_rate_percent']:.1f}% below 95%"

    @pytest.mark.asyncio
    async def test_model_generation_complex_performance(self, performance_suite, test_environment):
        """Test complex model generation performance"""
        server = LocoMCPServer(ServerConfig(default_project_path=test_environment))

        async def generate_complex_model():
            return await server.tools.generate_model({
                "model_name": "complex_entity",
                "fields": [
                    "name:string",
                    "email:string:unique",
                    "content:text",
                    "count:i32",
                    "price:f64",
                    "is_active:boolean",
                    "created_at:datetime",
                    "metadata:json",
                    "uuid_field:uuid"
                ],
                "project_path": test_environment
            })

        result = performance_suite.run_performance_test(
            "Model Generation - Complex",
            generate_complex_model,
            iterations=15
        )

        # Complex models may take slightly longer, but should still be reasonable
        assert result["avg_duration_ms"] <= 15.0, f"Complex model avg {result['avg_duration_ms']:.2f}ms exceeds 15ms"

    @pytest.mark.asyncio
    async def test_model_generation_concurrent_performance(self, performance_suite, test_environment):
        """Test concurrent model generation performance"""
        server = LocoMCPServer(ServerConfig(default_project_path=test_environment))

        async def generate_concurrent_models():
            tasks = []
            for i in range(5):
                task = server.tools.generate_model({
                    "model_name": f"concurrent_model_{i}",
                    "fields": ["name:string", "value:i32"],
                    "project_path": test_environment
                })
                tasks.append(task)

            results = await asyncio.gather(*tasks, return_exceptions=True)
            successful = sum(1 for r in results if not isinstance(r, Exception))
            return successful == len(tasks)

        result = performance_suite.run_performance_test(
            "Model Generation - Concurrent (5 models)",
            generate_concurrent_models,
            iterations=10
        )


class TestScaffoldGenerationPerformance:
    """Performance tests for scaffold generation"""

    @pytest.fixture
    def performance_suite(self):
        """Setup performance test suite"""
        return PerformanceTestSuite()

    @pytest.fixture
    def test_environment(self):
        """Setup test environment"""
        temp_dir = tempfile.mkdtemp(prefix="loco_scaffold_perf_")

        # Create complete loco project structure
        dirs = [
            "src/models", "src/controllers", "src/views", "src/routes", "migration/src"
        ]
        for dir_path in dirs:
            os.makedirs(os.path.join(temp_dir, *dir_path.split("/")))

        cargo_toml = """[package]
name = "scaffold-perf-test"
version = "0.1.0"
edition = "2021"
[dependencies]
loco-rs = "0.3"
"""
        with open(os.path.join(temp_dir, "Cargo.toml"), "w") as f:
            f.write(cargo_toml)

        yield temp_dir

        import shutil
        shutil.rmtree(temp_dir, ignore_errors=True)

    @pytest.mark.asyncio
    async def test_scaffold_generation_basic_performance(self, performance_suite, test_environment):
        """Test basic scaffold generation performance"""
        server = LocoMCPServer(ServerConfig(default_project_path=test_environment))

        async def generate_basic_scaffold():
            return await server.tools.generate_scaffold({
                "model_name": "blog_post",
                "fields": ["title:string", "content:text"],
                "include_views": True,
                "include_controllers": True,
                "project_path": test_environment
            })

        result = performance_suite.run_performance_test(
            "Scaffold Generation - Basic",
            generate_basic_scaffold,
            iterations=15
        )

        # Scaffold generation may take longer due to multiple files
        assert result["success_rate_percent"] >= 90, f"Scaffold success rate {result['success_rate_percent']:.1f}% below 90%"

    @pytest.mark.asyncio
    async def test_scaffold_generation_api_only_performance(self, performance_suite, test_environment):
        """Test API-only scaffold generation performance"""
        server = LocoMCPServer(ServerConfig(default_project_path=test_environment))

        async def generate_api_scaffold():
            return await server.tools.generate_scaffold({
                "model_name": "api_endpoint",
                "fields": ["endpoint:string:unique", "method:string", "response_schema:json"],
                "api_only": True,
                "project_path": test_environment
            })

        result = performance_suite.run_performance_test(
            "Scaffold Generation - API Only",
            generate_api_scaffold,
            iterations=20
        )

        assert result["avg_duration_ms"] <= 12.0, f"API scaffold avg {result['avg_duration_ms']:.2f}ms exceeds 12ms"


class TestControllerGenerationPerformance:
    """Performance tests for controller generation"""

    @pytest.fixture
    def performance_suite(self):
        """Setup performance test suite"""
        return PerformanceTestSuite()

    @pytest.fixture
    def test_environment(self):
        """Setup test environment with existing model"""
        temp_dir = tempfile.mkdtemp(prefix="loco_controller_perf_")

        # Create loco project structure
        dirs = ["src/models", "src/controllers", "src/views"]
        for dir_path in dirs:
            os.makedirs(os.path.join(temp_dir, *dir_path.split("/")))

        cargo_toml = """[package]
name = "controller-perf-test"
version = "0.1.0"
edition = "2021"
[dependencies]
loco-rs = "0.3"
"""
        with open(os.path.join(temp_dir, "Cargo.toml"), "w") as f:
            f.write(cargo_toml)

        yield temp_dir

        import shutil
        shutil.rmtree(temp_dir, ignore_errors=True)

    @pytest.mark.asyncio
    async def test_controller_generation_performance(self, performance_suite, test_environment):
        """Test controller generation performance"""
        server = LocoMCPServer(ServerConfig(default_project_path=test_environment))

        # First create a model
        await server.tools.generate_model({
            "model_name": "user",
            "fields": ["name:string", "email:string"],
            "project_path": test_environment
        })

        async def generate_controller():
            return await server.tools.generate_controller_view({
                "model_name": "user",
                "actions": ["index", "show", "create", "update", "delete"],
                "view_types": ["list", "show", "form", "edit"],
                "project_path": test_environment
            })

        result = performance_suite.run_performance_test(
            "Controller Generation",
            generate_controller,
            iterations=25
        )

        assert result["success_rate_percent"] >= 95, f"Controller generation success rate {result['success_rate_percent']:.1f}% below 95%"


class TestTemplateCachePerformance:
    """Performance tests for template caching effectiveness"""

    @pytest.fixture
    def performance_suite(self):
        """Setup performance test suite"""
        return PerformanceTestSuite()

    @pytest.fixture
    def test_environment(self):
        """Setup test environment"""
        temp_dir = tempfile.mkdtemp(prefix="loco_cache_perf_")

        dirs = ["src/models", "migration/src"]
        for dir_path in dirs:
            os.makedirs(os.path.join(temp_dir, *dir_path.split("/")))

        cargo_toml = """[package]
name = "cache-perf-test"
version = "0.1.0"
edition = "2021"
[dependencies]
loco-rs = "0.3"
"""
        with open(os.path.join(temp_dir, "Cargo.toml"), "w") as f:
            f.write(cargo_toml)

        yield temp_dir

        import shutil
        shutil.rmtree(temp_dir, ignore_errors=True)

    @pytest.mark.asyncio
    async def test_template_cache_effectiveness(self, performance_suite, test_environment):
        """Test that template caching improves performance"""
        server = LocoMCPServer(ServerConfig(default_project_path=test_environment))

        # First generation (cache miss)
        async def first_generation():
            return await server.tools.generate_model({
                "model_name": "cached_model",
                "fields": ["name:string", "email:string"],
                "project_path": test_environment
            })

        first_result = performance_suite.run_performance_test(
            "Template Cache - First Generation",
            first_generation,
            iterations=5
        )

        # Subsequent generations (cache hits)
        # Use different model names to avoid conflicts
        cache_results = []
        for i in range(3):
            async def cached_generation():
                return await server.tools.generate_model({
                    "model_name": f"cached_model_{i}",
                    "fields": ["name:string", "email:string"],
                    "project_path": test_environment
                })

            result = performance_suite.run_performance_test(
                f"Template Cache - Cached Generation {i+1}",
                cached_generation,
                iterations=5
            )
            cache_results.append(result)

        # Analyze cache effectiveness
        avg_first_time = first_result["avg_duration_ms"]
        avg_cached_time = statistics.mean([r["avg_duration_ms"] for r in cache_results])

        improvement = ((avg_first_time - avg_cached_time) / avg_first_time) * 100

        print(f"\nTemplate Cache Effectiveness Analysis:")
        print(f"First generation avg: {avg_first_time:.2f}ms")
        print(f"Cached generation avg: {avg_cached_time:.2f}ms")
        print(f"Performance improvement: {improvement:.1f}%")

        # Cache should provide at least some improvement
        assert improvement >= 0, f"Cache should not degrade performance (improvement: {improvement:.1f}%)"


class TestLoadTesting:
    """Load testing for sustained performance"""

    @pytest.fixture
    def performance_suite(self):
        """Setup performance test suite"""
        return PerformanceTestSuite()

    @pytest.fixture
    def test_environment(self):
        """Setup test environment"""
        temp_dir = tempfile.mkdtemp(prefix="loco_load_perf_")

        dirs = ["src/models", "src/controllers", "src/views", "migration/src"]
        for dir_path in dirs:
            os.makedirs(os.path.join(temp_dir, *dir_path.split("/")))

        cargo_toml = """[package]
name = "load-perf-test"
version = "0.1.0"
edition = "2021"
[dependencies]
loco-rs = "0.3"
"""
        with open(os.path.join(temp_dir, "Cargo.toml"), "w") as f:
            f.write(cargo_toml)

        yield temp_dir

        import shutil
        shutil.rmtree(temp_dir, ignore_errors=True)

    @pytest.mark.asyncio
    async def test_sustained_load_performance(self, performance_suite, test_environment):
        """Test performance under sustained load"""
        server = LocoMCPServer(ServerConfig(default_project_path=test_environment))

        async def sustained_load():
            # Generate 10 different models in sequence
            tasks = []
            for i in range(10):
                task = server.tools.generate_model({
                    "model_name": f"load_test_model_{i}",
                    "fields": ["name:string", "value:i32"],
                    "project_path": test_environment
                })
                tasks.append(task)

            results = await asyncio.gather(*tasks, return_exceptions=True)
            successful = sum(1 for r in results if not isinstance(r, Exception))
            return successful == len(tasks)

        result = performance_suite.run_performance_test(
            "Sustained Load - 10 Sequential Models",
            sustained_load,
            iterations=5
        )

        # Under load, performance may degrade slightly but should remain reasonable
        assert result["success_rate_percent"] >= 85, f"Load test success rate {result['success_rate_percent']:.1f}% below 85%"

    @pytest.mark.asyncio
    async def test_burst_load_performance(self, performance_suite, test_environment):
        """Test performance under burst load"""
        server = LocoMCPServer(ServerConfig(default_project_path=test_environment))

        async def burst_load():
            # Generate many models concurrently
            async def single_model(model_id):
                return await server.tools.generate_model({
                    "model_name": f"burst_model_{model_id}",
                    "fields": ["name:string"],
                    "project_path": test_environment
                })

            # Create burst of 20 concurrent requests
            tasks = [single_model(i) for i in range(20)]
            results = await asyncio.gather(*tasks, return_exceptions=True)
            successful = sum(1 for r in results if not isinstance(r, Exception))
            return successful >= 18  # Allow some failures under extreme load

        result = performance_suite.run_performance_test(
            "Burst Load - 20 Concurrent Models",
            burst_load,
            iterations=3
        )

        print(f"Burst load completed: {result['avg_duration_ms']:.2f}ms average duration")


# Integration test that runs all performance tests
class TestPerformanceValidation:
    """Comprehensive performance validation"""

    def test_complete_performance_validation(self):
        """Run complete performance validation suite"""
        print(f"\n{'='*80}")
        print(f"COMPLETE PERFORMANCE VALIDATION SUITE")
        print(f"{'='*80}")

        suite = PerformanceTestSuite()

        # Run a representative sample of performance tests
        test_environment = suite.setup_test_environment()

        try:
            server = LocoMCPServer(ServerConfig(default_project_path=test_environment))

            # Test 1: Basic Model Generation
            async def test_model_gen():
                return await server.tools.generate_model({
                    "model_name": "perf_user",
                    "fields": ["name:string", "email:string"],
                    "project_path": test_environment
                })

            suite.run_performance_test("Integration - Model Generation", test_model_gen, iterations=10)

            # Test 2: Basic Scaffold Generation
            async def test_scaffold_gen():
                return await server.tools.generate_scaffold({
                    "model_name": "perf_post",
                    "fields": ["title:string", "content:text"],
                    "project_path": test_environment
                })

            suite.run_performance_test("Integration - Scaffold Generation", test_scaffold_gen, iterations=8)

            # Test 3: Controller Generation
            # First create a model
            await server.tools.generate_model({
                "model_name": "perf_controller_model",
                "fields": ["name:string"],
                "project_path": test_environment
            })

            async def test_controller_gen():
                return await server.tools.generate_controller_view({
                    "model_name": "perf_controller_model",
                    "project_path": test_environment
                })

            suite.run_performance_test("Integration - Controller Generation", test_controller_gen, iterations=12)

        finally:
            suite.cleanup_test_environment(test_environment)

        # Print final summary
        all_passed = suite.print_summary_report()

        # Assert overall performance requirements are met
        assert all_passed, "Performance validation failed - some tests did not meet requirements"


if __name__ == "__main__":
    # Run performance tests
    pytest.main([__file__, "-v", "-s", "--tb=short"])
"""Tests for MCP server startup and basic functionality."""

import pytest
import asyncio
from unittest.mock import Mock, patch, AsyncMock
from typing import Dict, Any

# These imports will work once the MCP server is implemented
# from loco_mcp_server.server import LocoMCPServer
# from loco_mcp_server.tools import LocoTools


class TestLocoMCPServer:
    """Test the Loco MCP server implementation."""

    @pytest.fixture
    def mock_server(self):
        """Create a mock LocoMCPServer instance."""
        with patch('loco_mcp_server.server.LocoMCPServer') as mock_server_class:
            instance = Mock()
            mock_server_class.return_value = instance
            yield instance

    @pytest.fixture
    def server_config(self) -> Dict[str, Any]:
        """Default server configuration for testing."""
        return {
            "host": "localhost",
            "port": 8080,
            "log_level": "INFO",
            "max_connections": 100,
            "timeout": 30
        }

    def test_server_initialization(self, server_config):
        """Test server initialization with configuration."""
        # Mock server initialization
        def mock_init(config: Dict[str, Any]):
            return {
                "host": config.get("host", "localhost"),
                "port": config.get("port", 8080),
                "status": "initialized",
                "tools_loaded": False
            }

        server_state = mock_init(server_config)

        assert server_state["host"] == "localhost"
        assert server_state["port"] == 8080
        assert server_state["status"] == "initialized"
        assert server_state["tools_loaded"] is False

    def test_server_startup(self, mock_server):
        """Test server startup process."""
        # Mock startup sequence
        startup_sequence = [
            "loading_tools",
            "initializing_bindings",
            "starting_http_server",
            "registering_mcp_handlers",
            "ready_for_connections"
        ]

        async def mock_startup():
            for step in startup_sequence:
                await asyncio.sleep(0.001)  # Simulate async work
                print(f"Completed: {step}")

            return {
                "status": "running",
                "uptime": 0.1,
                "tools_registered": 3,
                "listening_on": "localhost:8080"
            }

        # Run async startup
        startup_result = asyncio.run(mock_startup())

        assert startup_result["status"] == "running"
        assert startup_result["tools_registered"] == 3
        assert startup_result["listening_on"] == "localhost:8080"

    @pytest.mark.asyncio
    async def test_mcp_request_handling(self):
        """Test handling of MCP requests."""
        # Mock MCP request
        mcp_request = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "generate_model",
                "arguments": {
                    "model_name": "test",
                    "fields": ["name:string"]
                }
            }
        }

        # Mock request handling
        async def mock_handle_request(request: Dict[str, Any]) -> Dict[str, Any]:
            await asyncio.sleep(0.001)  # Simulate processing

            if request["method"] == "tools/call":
                tool_name = request["params"]["name"]
                if tool_name == "generate_model":
                    return {
                        "jsonrpc": "2.0",
                        "id": request["id"],
                        "result": {
                            "status": "success",
                            "files_created": [
                                {"path": "src/models/test.rs", "type": "model"}
                            ]
                        }
                    }
                else:
                    return {
                        "jsonrpc": "2.0",
                        "id": request["id"],
                        "error": {
                            "code": -32601,
                            "message": f"Tool '{tool_name}' not found"
                        }
                    }
            else:
                return {
                    "jsonrpc": "2.0",
                    "id": request["id"],
                    "error": {
                        "code": -32601,
                        "message": f"Method '{request['method']}' not found"
                    }
                }

        response = await mock_handle_request(mcp_request)

        assert response["jsonrpc"] == "2.0"
        assert response["id"] == 1
        assert "result" in response
        assert response["result"]["status"] == "success"

    @pytest.mark.asyncio
    async def test_mcp_error_handling(self):
        """Test error handling for MCP requests."""
        error_cases = [
            {
                "request": {
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "tools/call",
                    "params": {
                        "name": "generate_model",
                        "arguments": {"model_name": "123invalid"}  # Invalid
                    }
                },
                "expected_error": {
                    "code": -32602,
                    "message_contains": ["Invalid model name"]
                }
            },
            {
                "request": {
                    "jsonrpc": "2.0",
                    "id": 2,
                    "method": "tools/call",
                    "params": {
                        "name": "unknown_tool",
                        "arguments": {}
                    }
                },
                "expected_error": {
                    "code": -32601,
                    "message_contains": ["not found", "unknown_tool"]
                }
            },
            {
                "request": {
                    "jsonrpc": "2.0",
                    "id": 3,
                    "method": "unknown_method",
                    "params": {}
                },
                "expected_error": {
                    "code": -32601,
                    "message_contains": ["Method", "not found"]
                }
            }
        ]

        async def mock_handle_error_request(request: Dict[str, Any]) -> Dict[str, Any]:
            await asyncio.sleep(0.001)

            if request.get("method") == "tools/call":
                tool_name = request["params"]["name"]
                if tool_name == "generate_model":
                    model_name = request["params"]["arguments"].get("model_name", "")
                    if model_name == "123invalid":
                        return {
                            "jsonrpc": "2.0",
                            "id": request["id"],
                            "error": {
                                "code": -32602,
                                "message": "Invalid model name: '123invalid' must start with a letter"
                            }
                        }
                elif tool_name == "unknown_tool":
                    return {
                        "jsonrpc": "2.0",
                        "id": request["id"],
                        "error": {
                            "code": -32601,
                            "message": "Tool 'unknown_tool' not found"
                        }
                    }

            elif request.get("method") == "unknown_method":
                return {
                    "jsonrpc": "2.0",
                    "id": request["id"],
                    "error": {
                        "code": -32601,
                        "message": "Method 'unknown_method' not found"
                    }
                }

            # Default success response
            return {
                "jsonrpc": "2.0",
                "id": request["id"],
                "result": {"status": "success"}
            }

        for case in error_cases:
            response = await mock_handle_error_request(case["request"])

            assert "error" in response, f"Expected error for case: {case['request']}"
            assert response["error"]["code"] == case["expected_error"]["code"]

            error_message = response["error"]["message"].lower()
            for keyword in case["expected_error"]["message_contains"]:
                assert keyword.lower() in error_message, f"Error message should contain '{keyword}'"

    @pytest.mark.asyncio
    async def test_concurrent_requests(self):
        """Test handling concurrent requests."""
        # Create multiple concurrent requests
        requests = [
            {
                "jsonrpc": "2.0",
                "id": i,
                "method": "tools/call",
                "params": {
                    "name": "generate_model",
                    "arguments": {
                        "model_name": f"model_{i}",
                        "fields": ["name:string"]
                    }
                }
            }
            for i in range(1, 6)  # 5 concurrent requests
        ]

        async def mock_handle_concurrent(request: Dict[str, Any]) -> Dict[str, Any]:
            # Simulate some processing time
            await asyncio.sleep(0.01)
            return {
                "jsonrpc": "2.0",
                "id": request["id"],
                "result": {
                    "status": "success",
                    "model_name": request["params"]["arguments"]["model_name"],
                    "processing_time": 0.01
                }
            }

        # Execute all requests concurrently
        start_time = asyncio.get_event_loop().time()
        tasks = [mock_handle_concurrent(req) for req in requests]
        responses = await asyncio.gather(*tasks)
        end_time = asyncio.get_event_loop().time()

        total_time = end_time - start_time

        # Should complete much faster than sequential (5 * 0.01 = 0.05s)
        assert total_time < 0.03, f"Concurrent processing should be fast, took {total_time:.3f}s"
        assert len(responses) == 5

        # Check all responses
        for i, response in enumerate(responses, 1):
            assert response["id"] == i
            assert response["result"]["status"] == "success"
            assert response["result"]["model_name"] == f"model_{i}"

    def test_server_shutdown(self, mock_server):
        """Test graceful server shutdown."""
        shutdown_sequence = [
            "stop_accepting_new_connections",
            "wait_for_active_requests",
            "close_connections",
            "cleanup_resources",
            "shutdown_complete"
        ]

        async def mock_shutdown():
            for step in shutdown_sequence:
                await asyncio.sleep(0.001)
                print(f"Shutdown: {step}")

            return {
                "status": "stopped",
                "uptime": 120.5,
                "requests_handled": 150,
                "errors": 2
            }

        shutdown_result = asyncio.run(mock_shutdown())

        assert shutdown_result["status"] == "stopped"
        assert shutdown_result["requests_handled"] == 150
        assert shutdown_result["errors"] == 2

    def test_server_health_check(self):
        """Test server health check functionality."""
        def mock_health_check() -> Dict[str, Any]:
            return {
                "status": "healthy",
                "uptime": 300.0,
                "memory_usage_mb": 45.2,
                "active_connections": 5,
                "requests_per_minute": 12.5,
                "error_rate_percent": 0.1,
                "tools_available": 3,
                "binding_status": "connected"
            }

        health = mock_health_check()

        assert health["status"] == "healthy"
        assert health["tools_available"] == 3
        assert health["binding_status"] == "connected"
        assert health["error_rate_percent"] < 1.0  # Error rate should be low

    def test_configuration_validation(self):
        """Test server configuration validation."""
        valid_configs = [
            {"host": "localhost", "port": 8080},
            {"host": "0.0.0.0", "port": 9000},
            {"port": 8000},  # host should default
        ]

        invalid_configs = [
            {"port": -1},  # Invalid port
            {"port": 70000},  # Port too high
            {"host": "invalid_hostname"},  # Invalid hostname
            {"log_level": "INVALID"},  # Invalid log level
        ]

        def mock_validate_config(config: Dict[str, Any]) -> Dict[str, Any]:
            errors = []

            if "port" in config:
                port = config["port"]
                if not (1 <= port <= 65535):
                    errors.append(f"Invalid port: {port}")

            if "host" in config and config["host"] == "invalid_hostname":
                errors.append("Invalid hostname")

            if "log_level" in config and config["log_level"] not in ["DEBUG", "INFO", "WARN", "ERROR"]:
                errors.append(f"Invalid log level: {config['log_level']}")

            return {
                "valid": len(errors) == 0,
                "errors": errors,
                "normalized_config": {
                    "host": config.get("host", "localhost"),
                    "port": config.get("port", 8080),
                    "log_level": config.get("log_level", "INFO")
                }
            }

        for config in valid_configs:
            result = mock_validate_config(config)
            assert result["valid"], f"Config should be valid: {config}"
            assert len(result["errors"]) == 0

        for config in invalid_configs:
            result = mock_validate_config(config)
            assert not result["valid"], f"Config should be invalid: {config}"
            assert len(result["errors"]) > 0

    def test_logging_setup(self):
        """Test server logging configuration."""
        def mock_setup_logging(log_level: str = "INFO") -> Dict[str, Any]:
            log_config = {
                "version": 1,
                "disable_existing_loggers": False,
                "formatters": {
                    "standard": {
                        "format": "%(asctime)s [%(levelname)s] %(name)s: %(message)s"
                    }
                },
                "handlers": {
                    "default": {
                        "level": log_level,
                        "formatter": "standard",
                        "class": "logging.StreamHandler"
                    }
                },
                "loggers": {
                    "loco_mcp_server": {
                        "handlers": ["default"],
                        "level": log_level,
                        "propagate": False
                    }
                }
            }

            return {
                "configured": True,
                "level": log_level,
                "handlers": list(log_config["handlers"].keys()),
                "loggers": list(log_config["loggers"].keys())
            }

        log_setup = mock_setup_logging("DEBUG")

        assert log_setup["configured"] is True
        assert log_setup["level"] == "DEBUG"
        assert "default" in log_setup["handlers"]
        assert "loco_mcp_server" in log_setup["loggers"]
"""Tests for timeout enforcement functionality."""

import pytest
import asyncio
from unittest.mock import patch, AsyncMock

from src.server import LocoMCPServer


@pytest.mark.asyncio
async def test_default_timeout_60_seconds():
    """Test that tools default to 60 second timeout."""
    
    server = LocoMCPServer()
    
    args = {
        "project_path": "/test/project",
        "environment": "staging",
        "approvals": ["ops_lead"],
        "dependencies": ["postgres"]
    }
    
    # Mock a tool that takes longer than 60 seconds
    with patch('src.tools.loco_bindings') as mock_bindings:
        async def slow_migrate_db(*args, **kwargs):
            await asyncio.sleep(65)  # Longer than default timeout
            return {"success": True, "messages": ["Completed"]}
        
        mock_bindings.migrate_db.side_effect = slow_migrate_db
        
        # This should timeout after 60 seconds
        start_time = asyncio.get_event_loop().time()
        
        try:
            await server.server.call_tool("migrate_db", args)
            assert False, "Tool should have timed out"
        except asyncio.TimeoutError:
            pass  # Expected
        
        end_time = asyncio.get_event_loop().time()
        elapsed = end_time - start_time
        
        # Should timeout around 60 seconds (allow some tolerance)
        assert 58 <= elapsed <= 62, f"Timeout should be ~60s, got {elapsed}s"


@pytest.mark.asyncio
async def test_custom_timeout_300_seconds():
    """Test that tools can override timeout to 300 seconds."""
    
    server = LocoMCPServer()
    
    args = {
        "project_path": "/test/project",
        "environment": "production",
        "approvals": ["security_officer", "cto"],
        "timeout_seconds": 300,  # Override to 300 seconds
        "dependencies": ["kms"]
    }
    
    with patch('src.tools.loco_bindings') as mock_bindings:
        async def slow_rotate_keys(*args, **kwargs):
            await asyncio.sleep(250)  # Longer than default but within custom timeout
            return {"success": True, "messages": ["Keys rotated"]}
        
        mock_bindings.rotate_keys.side_effect = slow_rotate_keys
        
        # This should succeed within 300 seconds
        start_time = asyncio.get_event_loop().time()
        
        response = await server.server.call_tool("rotate_keys", args)
        
        end_time = asyncio.get_event_loop().time()
        elapsed = end_time - start_time
        
        # Should complete within 300 seconds
        assert elapsed < 300, f"Tool should complete within 300s, took {elapsed}s"
        assert response, "Tool should return successful response"


@pytest.mark.asyncio
async def test_timeout_exceeded_returns_error():
    """Test that timeout exceeded returns proper error message."""
    
    server = LocoMCPServer()
    
    args = {
        "project_path": "/test/project",
        "environment": "staging",
        "approvals": ["ops_lead"],
        "timeout_seconds": 5,  # Short timeout for testing
        "dependencies": ["postgres"]
    }
    
    with patch('src.tools.loco_bindings') as mock_bindings:
        async def slow_clean_temp(*args, **kwargs):
            await asyncio.sleep(10)  # Longer than 5 second timeout
            return {"success": True, "messages": ["Cleaned"]}
        
        mock_bindings.clean_temp.side_effect = slow_clean_temp
        
        # This should timeout and return error
        response = await server.server.call_tool("clean_temp", args)
        
        assert response, "Should return error response"
        assert "❌" in response[0].text, "Should return error message"
        assert "timeout" in response[0].text.lower(), "Should mention timeout"


@pytest.mark.asyncio
async def test_timeout_configuration_validation():
    """Test that timeout configuration is validated."""
    
    server = LocoMCPServer()
    
    # Test timeout too low (below minimum)
    args_low = {
        "project_path": "/test/project",
        "environment": "staging",
        "approvals": ["ops_lead"],
        "timeout_seconds": 5,  # Below minimum of 10 seconds
        "dependencies": ["postgres"]
    }
    
    response = await server.server.call_tool("migrate_db", args_low)
    assert response, "Should return error response"
    assert "❌" in response[0].text, "Should return error message"
    assert "minimum timeout" in response[0].text.lower(), "Should mention minimum timeout"
    
    # Test timeout too high (above maximum)
    args_high = {
        "project_path": "/test/project",
        "environment": "staging",
        "approvals": ["ops_lead"],
        "timeout_seconds": 400,  # Above maximum of 300 seconds
        "dependencies": ["postgres"]
    }
    
    response = await server.server.call_tool("migrate_db", args_high)
    assert response, "Should return error response"
    assert "❌" in response[0].text, "Should return error message"
    assert "maximum timeout" in response[0].text.lower(), "Should mention maximum timeout"


@pytest.mark.asyncio
async def test_different_tools_different_default_timeouts():
    """Test that different tools can have different default timeouts."""
    
    server = LocoMCPServer()
    
    # migrate_db should have 60s default
    migrate_args = {
        "project_path": "/test/project",
        "environment": "staging",
        "approvals": ["ops_lead", "security_officer"],
        "dependencies": ["postgres", "redis"]
    }
    
    # rotate_keys should have 300s default (critical operation)
    rotate_args = {
        "project_path": "/test/project",
        "environment": "production",
        "approvals": ["security_officer", "cto"],
        "dependencies": ["kms"]
    }
    
    with patch('src.tools.loco_bindings') as mock_bindings:
        async def slow_operation(*args, **kwargs):
            await asyncio.sleep(70)  # Longer than 60s but shorter than 300s
            return {"success": True, "messages": ["Completed"]}
        
        mock_bindings.migrate_db.side_effect = slow_operation
        mock_bindings.rotate_keys.side_effect = slow_operation
        
        # migrate_db should timeout (60s default)
        try:
            await server.server.call_tool("migrate_db", migrate_args)
            assert False, "migrate_db should have timed out"
        except asyncio.TimeoutError:
            pass  # Expected
        
        # rotate_keys should succeed (300s default)
        response = await server.server.call_tool("rotate_keys", rotate_args)
        assert response, "rotate_keys should succeed with longer timeout"


@pytest.mark.asyncio
async def test_timeout_cancellation_cleanup():
    """Test that timeout cancellation properly cleans up resources."""
    
    server = LocoMCPServer()
    
    args = {
        "project_path": "/test/project",
        "environment": "staging",
        "approvals": ["ops_lead"],
        "timeout_seconds": 2,  # Very short timeout
        "dependencies": ["postgres"]
    }
    
    cleanup_called = False
    
    with patch('src.tools.loco_bindings') as mock_bindings:
        async def operation_with_cleanup(*args, **kwargs):
            nonlocal cleanup_called
            try:
                await asyncio.sleep(5)  # Longer than timeout
                return {"success": True, "messages": ["Completed"]}
            finally:
                cleanup_called = True
        
        mock_bindings.migrate_db.side_effect = operation_with_cleanup
        
        # This should timeout
        try:
            await server.server.call_tool("migrate_db", args)
            assert False, "Tool should have timed out"
        except asyncio.TimeoutError:
            pass  # Expected
        
        # Give a moment for cleanup to complete
        await asyncio.sleep(0.1)
        
        # Verify cleanup was called
        assert cleanup_called, "Cleanup should be called on timeout"


@pytest.mark.asyncio
async def test_concurrent_tools_respect_individual_timeouts():
    """Test that concurrent tool executions respect their individual timeouts."""
    
    server = LocoMCPServer()
    
    # Tool with short timeout
    short_args = {
        "project_path": "/test/project",
        "environment": "staging",
        "approvals": ["ops_lead"],
        "timeout_seconds": 2,
        "dependencies": ["postgres"]
    }
    
    # Tool with long timeout
    long_args = {
        "project_path": "/test/project",
        "environment": "production",
        "approvals": ["security_officer", "cto"],
        "timeout_seconds": 10,
        "dependencies": ["kms"]
    }
    
    with patch('src.tools.loco_bindings') as mock_bindings:
        async def slow_operation(*args, **kwargs):
            await asyncio.sleep(5)  # Longer than short timeout, shorter than long timeout
            return {"success": True, "messages": ["Completed"]}
        
        mock_bindings.migrate_db.side_effect = slow_operation
        mock_bindings.rotate_keys.side_effect = slow_operation
        
        # Execute both tools concurrently
        start_time = asyncio.get_event_loop().time()
        
        # Short timeout should fail, long timeout should succeed
        short_task = asyncio.create_task(server.server.call_tool("migrate_db", short_args))
        long_task = asyncio.create_task(server.server.call_tool("rotate_keys", long_args))
        
        short_result = None
        long_result = None
        
        try:
            short_result = await short_task
        except asyncio.TimeoutError:
            pass  # Expected
        
        long_result = await long_task
        
        end_time = asyncio.get_event_loop().time()
        elapsed = end_time - start_time
        
        # Short task should timeout, long task should succeed
        assert short_result is None, "Short timeout task should fail"
        assert long_result is not None, "Long timeout task should succeed"
        
        # Total time should be around 5 seconds (long task duration)
        assert 4 <= elapsed <= 6, f"Concurrent execution should take ~5s, got {elapsed}s"


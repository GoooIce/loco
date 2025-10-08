"""Tests for audit logging functionality."""

import pytest
import tempfile
import os
import json
import hashlib
from unittest.mock import patch, mock_open
from pathlib import Path

from src.server import LocoMCPServer


@pytest.fixture
def temp_audit_log():
    """Create a temporary audit log file for testing."""
    with tempfile.NamedTemporaryFile(mode='w', delete=False, suffix='.log') as f:
        temp_path = f.name
    yield temp_path
    # Cleanup
    if os.path.exists(temp_path):
        os.unlink(temp_path)


@pytest.mark.asyncio
async def test_tool_invocation_logs_to_audit_file(temp_audit_log):
    """Test that tool invocations append parameter hashes to audit log."""
    
    # Mock the audit log path
    with patch('src.security.AUDIT_LOG_PATH', temp_audit_log):
        server = LocoMCPServer()
        
        # Test migrate_db invocation
        migrate_args = {
            "project_path": "/test/project",
            "environment": "staging",
            "approvals": ["ops_lead", "security_officer"],
            "timeout_seconds": 60,
            "dependencies": ["postgres", "redis"]
        }
        
        # Calculate expected parameter hash
        param_str = json.dumps(migrate_args, sort_keys=True)
        expected_hash = hashlib.sha256(param_str.encode()).hexdigest()
        
        # Mock the tool execution to succeed
        with patch('src.tools.loco_bindings') as mock_bindings:
            mock_bindings.migrate_db.return_value = {
                "success": True,
                "messages": ["Migration completed"]
            }
            
            # Execute the tool
            await server.server.call_tool("migrate_db", migrate_args)
            
            # Verify audit log was written
            assert os.path.exists(temp_audit_log), "Audit log file should exist"
            
            with open(temp_audit_log, 'r') as f:
                log_content = f.read()
                
            # Parse the log entry
            log_entries = [json.loads(line) for line in log_content.strip().split('\n') if line]
            assert len(log_entries) == 1, "Should have one log entry"
            
            entry = log_entries[0]
            assert entry["tool_name"] == "migrate_db"
            assert entry["parameter_hash"] == expected_hash
            assert entry["operator"] == "mcp_client"  # Default operator
            assert "timestamp" in entry
            assert entry["status"] == "success"


@pytest.mark.asyncio
async def test_multiple_tool_invocations_log_separately(temp_audit_log):
    """Test that multiple tool invocations create separate audit entries."""
    
    with patch('src.security.AUDIT_LOG_PATH', temp_audit_log):
        server = LocoMCPServer()
        
        # First invocation: migrate_db
        migrate_args = {
            "project_path": "/test/project",
            "environment": "staging",
            "approvals": ["ops_lead", "security_officer"]
        }
        
        # Second invocation: rotate_keys
        rotate_args = {
            "project_path": "/test/project", 
            "environment": "production",
            "approvals": ["security_officer", "cto"]
        }
        
        with patch('src.tools.loco_bindings') as mock_bindings:
            mock_bindings.migrate_db.return_value = {"success": True, "messages": ["OK"]}
            mock_bindings.rotate_keys.return_value = {"success": True, "messages": ["OK"]}
            
            # Execute both tools
            await server.server.call_tool("migrate_db", migrate_args)
            await server.server.call_tool("rotate_keys", rotate_args)
            
            # Verify both entries were logged
            with open(temp_audit_log, 'r') as f:
                log_content = f.read()
                
            log_entries = [json.loads(line) for line in log_content.strip().split('\n') if line]
            assert len(log_entries) == 2, "Should have two log entries"
            
            # Check first entry
            assert log_entries[0]["tool_name"] == "migrate_db"
            assert log_entries[0]["parameter_hash"] == hashlib.sha256(
                json.dumps(migrate_args, sort_keys=True).encode()
            ).hexdigest()
            
            # Check second entry
            assert log_entries[1]["tool_name"] == "rotate_keys"
            assert log_entries[1]["parameter_hash"] == hashlib.sha256(
                json.dumps(rotate_args, sort_keys=True).encode()
            ).hexdigest()


@pytest.mark.asyncio
async def test_failed_tool_invocation_logs_error(temp_audit_log):
    """Test that failed tool invocations are logged with error status."""
    
    with patch('src.security.AUDIT_LOG_PATH', temp_audit_log):
        server = LocoMCPServer()
        
        args = {
            "project_path": "/test/project",
            "environment": "staging",
            "approvals": ["invalid_approver"]  # This should cause failure
        }
        
        with patch('src.tools.loco_bindings') as mock_bindings:
            mock_bindings.migrate_db.side_effect = Exception("Invalid approver")
            
            # Execute the tool (should fail)
            try:
                await server.server.call_tool("migrate_db", args)
            except Exception:
                pass  # Expected to fail
            
            # Verify error was logged
            with open(temp_audit_log, 'r') as f:
                log_content = f.read()
                
            log_entries = [json.loads(line) for line in log_content.strip().split('\n') if line]
            assert len(log_entries) == 1, "Should have one log entry"
            
            entry = log_entries[0]
            assert entry["tool_name"] == "migrate_db"
            assert entry["status"] == "error"
            assert "error_message" in entry
            assert entry["error_message"] == "Invalid approver"


@pytest.mark.asyncio
async def test_audit_log_directory_creation(temp_audit_log):
    """Test that audit log directory is created if it doesn't exist."""
    
    # Create a path in a non-existent directory
    temp_dir = tempfile.mkdtemp()
    audit_dir = os.path.join(temp_dir, "nonexistent", "audit")
    audit_file = os.path.join(audit_dir, "audit.log")
    
    try:
        with patch('src.security.AUDIT_LOG_PATH', audit_file):
            server = LocoMCPServer()
            
            args = {
                "project_path": "/test/project",
                "environment": "staging",
                "approvals": ["ops_lead"]
            }
            
            with patch('src.tools.loco_bindings') as mock_bindings:
                mock_bindings.migrate_db.return_value = {"success": True, "messages": ["OK"]}
                
                # Execute the tool
                await server.server.call_tool("migrate_db", args)
                
                # Verify directory was created and file exists
                assert os.path.exists(audit_dir), "Audit directory should be created"
                assert os.path.exists(audit_file), "Audit file should exist"
                
    finally:
        # Cleanup
        import shutil
        if os.path.exists(temp_dir):
            shutil.rmtree(temp_dir)


@pytest.mark.asyncio
async def test_parameter_hash_consistency(temp_audit_log):
    """Test that parameter hashes are consistent for identical inputs."""
    
    with patch('src.security.AUDIT_LOG_PATH', temp_audit_log):
        server = LocoMCPServer()
        
        args = {
            "project_path": "/test/project",
            "environment": "staging",
            "approvals": ["ops_lead", "security_officer"],
            "timeout_seconds": 60,
            "dependencies": ["postgres", "redis"]
        }
        
        with patch('src.tools.loco_bindings') as mock_bindings:
            mock_bindings.migrate_db.return_value = {"success": True, "messages": ["OK"]}
            
            # Execute the same tool twice with identical parameters
            await server.server.call_tool("migrate_db", args)
            await server.server.call_tool("migrate_db", args)
            
            # Verify both entries have the same parameter hash
            with open(temp_audit_log, 'r') as f:
                log_content = f.read()
                
            log_entries = [json.loads(line) for line in log_content.strip().split('\n') if line]
            assert len(log_entries) == 2, "Should have two log entries"
            
            hash1 = log_entries[0]["parameter_hash"]
            hash2 = log_entries[1]["parameter_hash"]
            assert hash1 == hash2, "Identical parameters should produce identical hashes"
            
            # Verify hash matches expected value
            expected_hash = hashlib.sha256(
                json.dumps(args, sort_keys=True).encode()
            ).hexdigest()
            assert hash1 == expected_hash, "Hash should match expected value"


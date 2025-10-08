"""
Execution Assurance Record persistence utilities.

This module provides utilities for recording and validating execution assurance
records to ensure MCP tools match CLI behaviors and remain compliant.
"""

import json
import hashlib
import logging
from datetime import datetime
from pathlib import Path
from typing import Dict, Any, Optional, List
from dataclasses import dataclass, asdict
from enum import Enum

logger = logging.getLogger(__name__)


class ValidationStatus(Enum):
    """Validation status enumeration."""
    PASS = "pass"
    FAIL = "fail"
    WAIVED = "waived"


@dataclass
class ExecutionAssuranceRecord:
    """Execution Assurance Record data structure."""
    cli_id: str
    verification_run_id: str
    expected_checksum: str
    actual_checksum: str
    variance_notes: Optional[str] = None
    tester: str = "mcp_client"
    run_timestamp: str = None
    status: ValidationStatus = ValidationStatus.PASS
    
    def __post_init__(self):
        if self.run_timestamp is None:
            self.run_timestamp = datetime.utcnow().isoformat()


class ExecutionAssuranceValidator:
    """Validator for execution assurance records."""
    
    def __init__(self, records_path: str = "/var/log/loco-mcp/execution_assurance.jsonl"):
        """Initialize the validator.
        
        Args:
            records_path: Path to the execution assurance records file
        """
        self.records_path = Path(records_path)
        self._ensure_directory()
    
    def _ensure_directory(self) -> None:
        """Ensure the directory for records exists."""
        self.records_path.parent.mkdir(parents=True, exist_ok=True)
    
    def calculate_checksum(self, data: Any) -> str:
        """Calculate SHA256 checksum of data.
        
        Args:
            data: Data to calculate checksum for
            
        Returns:
            SHA256 checksum as hex string
        """
        if isinstance(data, dict):
            # Sort keys for consistent hashing
            data_str = json.dumps(data, sort_keys=True)
        else:
            data_str = str(data)
        
        return hashlib.sha256(data_str.encode()).hexdigest()
    
    def record_execution(
        self,
        cli_id: str,
        expected_result: Dict[str, Any],
        actual_result: Dict[str, Any],
        tester: str = "mcp_client",
        variance_notes: Optional[str] = None
    ) -> ExecutionAssuranceRecord:
        """Record an execution assurance validation.
        
        Args:
            cli_id: CLI utility identifier
            expected_result: Expected result from CLI baseline
            actual_result: Actual result from MCP invocation
            tester: Name of the tester/operator
            variance_notes: Notes about any variances
            
        Returns:
            Execution assurance record
        """
        verification_run_id = f"{cli_id}_{datetime.utcnow().strftime('%Y%m%d_%H%M%S')}"
        
        expected_checksum = self.calculate_checksum(expected_result)
        actual_checksum = self.calculate_checksum(actual_result)
        
        # Determine status
        if expected_checksum == actual_checksum:
            status = ValidationStatus.PASS
        elif variance_notes:
            status = ValidationStatus.WAIVED
        else:
            status = ValidationStatus.FAIL
        
        record = ExecutionAssuranceRecord(
            cli_id=cli_id,
            verification_run_id=verification_run_id,
            expected_checksum=expected_checksum,
            actual_checksum=actual_checksum,
            variance_notes=variance_notes,
            tester=tester,
            status=status
        )
        
        self._persist_record(record)
        return record
    
    def _persist_record(self, record: ExecutionAssuranceRecord) -> None:
        """Persist an execution assurance record to disk.
        
        Args:
            record: Execution assurance record to persist
        """
        try:
            # Convert to dict and handle enum serialization
            record_dict = asdict(record)
            record_dict["status"] = record.status.value
            
            with open(self.records_path, "a", encoding="utf-8") as f:
                f.write(json.dumps(record_dict) + "\n")
            
            logger.info(f"Execution assurance record persisted: {record.verification_run_id}")
            
        except Exception as e:
            logger.error(f"Failed to persist execution assurance record: {e}")
            raise
    
    def get_records_for_cli(self, cli_id: str) -> List[ExecutionAssuranceRecord]:
        """Get all execution assurance records for a CLI utility.
        
        Args:
            cli_id: CLI utility identifier
            
        Returns:
            List of execution assurance records
        """
        records = []
        
        if not self.records_path.exists():
            return records
        
        try:
            with open(self.records_path, "r", encoding="utf-8") as f:
                for line in f:
                    if line.strip():
                        record_data = json.loads(line)
                        if record_data.get("cli_id") == cli_id:
                            # Convert status back to enum
                            record_data["status"] = ValidationStatus(record_data["status"])
                            records.append(ExecutionAssuranceRecord(**record_data))
            
        except Exception as e:
            logger.error(f"Failed to read execution assurance records: {e}")
        
        return records
    
    def get_latest_record(self, cli_id: str) -> Optional[ExecutionAssuranceRecord]:
        """Get the latest execution assurance record for a CLI utility.
        
        Args:
            cli_id: CLI utility identifier
            
        Returns:
            Latest execution assurance record or None
        """
        records = self.get_records_for_cli(cli_id)
        if not records:
            return None
        
        # Sort by timestamp and return the latest
        records.sort(key=lambda r: r.run_timestamp, reverse=True)
        return records[0]
    
    def validate_parity(
        self,
        cli_id: str,
        expected_result: Dict[str, Any],
        actual_result: Dict[str, Any],
        allow_variance: bool = False
    ) -> bool:
        """Validate parity between expected and actual results.
        
        Args:
            cli_id: CLI utility identifier
            expected_result: Expected result from CLI baseline
            actual_result: Actual result from MCP invocation
            allow_variance: Whether to allow variance with notes
            
        Returns:
            True if validation passes, False otherwise
        """
        record = self.record_execution(
            cli_id=cli_id,
            expected_result=expected_result,
            actual_result=actual_result,
            variance_notes="Variance allowed" if allow_variance else None
        )
        
        return record.status in [ValidationStatus.PASS, ValidationStatus.WAIVED]
    
    def get_validation_summary(self, cli_id: str) -> Dict[str, Any]:
        """Get validation summary for a CLI utility.
        
        Args:
            cli_id: CLI utility identifier
            
        Returns:
            Validation summary dictionary
        """
        records = self.get_records_for_cli(cli_id)
        
        if not records:
            return {
                "cli_id": cli_id,
                "total_validations": 0,
                "pass_count": 0,
                "fail_count": 0,
                "waived_count": 0,
                "pass_rate": 0.0,
                "last_validation": None,
                "status": "no_data"
            }
        
        pass_count = sum(1 for r in records if r.status == ValidationStatus.PASS)
        fail_count = sum(1 for r in records if r.status == ValidationStatus.FAIL)
        waived_count = sum(1 for r in records if r.status == ValidationStatus.WAIVED)
        
        pass_rate = (pass_count + waived_count) / len(records) * 100
        
        # Get latest record
        latest_record = max(records, key=lambda r: r.run_timestamp)
        
        return {
            "cli_id": cli_id,
            "total_validations": len(records),
            "pass_count": pass_count,
            "fail_count": fail_count,
            "waived_count": waived_count,
            "pass_rate": round(pass_rate, 2),
            "last_validation": latest_record.run_timestamp,
            "status": latest_record.status.value
        }


# Global validator instance
# Use a test-safe records path by default
import os
records_path = os.environ.get("LOCO_MCP_RECORDS_PATH", "/tmp/loco-mcp-records.json")
validator = ExecutionAssuranceValidator(records_path)


def validate_mcp_cli_parity(
    cli_id: str,
    expected_result: Dict[str, Any],
    actual_result: Dict[str, Any],
    tester: str = "mcp_client"
) -> bool:
    """Convenience function to validate MCP-CLI parity.
    
    Args:
        cli_id: CLI utility identifier
        expected_result: Expected result from CLI baseline
        actual_result: Actual result from MCP invocation
        tester: Name of the tester/operator
        
    Returns:
        True if validation passes, False otherwise
    """
    return validator.validate_parity(cli_id, expected_result, actual_result)


def get_execution_assurance_summary(cli_id: str) -> Dict[str, Any]:
    """Convenience function to get execution assurance summary.
    
    Args:
        cli_id: CLI utility identifier
        
    Returns:
        Validation summary dictionary
    """
    return validator.get_validation_summary(cli_id)
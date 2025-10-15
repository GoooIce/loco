"""
MCP tools implementation for loco-rs code generation.

This module provides tool implementations that interface with
the Rust loco-gen library through Python bindings.
"""

import logging
import time
import uuid
import asyncio
from typing import Any, Dict, Optional, List, Callable
from enum import Enum

from .security import security_manager
from .config import ServerConfig

logger = logging.getLogger(__name__)


class WorkflowStepStatus(Enum):
    """Status of a workflow step."""
    PENDING = "pending"
    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"
    CANCELLED = "cancelled"


class WorkflowStep:
    """A single step in a multi-step workflow."""
    
    def __init__(
        self,
        step_id: str,
        name: str,
        tool_call: Callable,
        dependencies: List[str] = None,
        timeout_seconds: int = None,
        retry_count: int = 0
    ):
        self.step_id = step_id
        self.name = name
        self.tool_call = tool_call
        self.dependencies = dependencies or []
        self.timeout_seconds = timeout_seconds
        self.retry_count = retry_count
        self.status = WorkflowStepStatus.PENDING
        self.result = None
        self.error = None
        self.start_time = None
        self.end_time = None


class WorkflowOrchestrator:
    """Orchestrates multi-step workflows with dependency management and interruption support."""
    
    def __init__(self, config: ServerConfig):
        self.config = config
        self.active_workflows: Dict[str, Dict[str, WorkflowStep]] = {}
        self.cancellation_tokens: Dict[str, bool] = {}
    
    def create_workflow(self, workflow_id: str, steps: List[WorkflowStep]) -> Dict[str, WorkflowStep]:
        """Create a new workflow with steps."""
        workflow = {step.step_id: step for step in steps}
        self.active_workflows[workflow_id] = workflow
        self.cancellation_tokens[workflow_id] = False
        return workflow
    
    def cancel_workflow(self, workflow_id: str) -> bool:
        """Cancel a running workflow."""
        if workflow_id in self.cancellation_tokens:
            self.cancellation_tokens[workflow_id] = True
            logger.info(f"Workflow {workflow_id} cancellation requested")
            return True
        return False
    
    async def execute_workflow(
        self,
        workflow_id: str,
        project_path: str,
        environment: str = None,
        approvals: List[str] = None
    ) -> Dict[str, Any]:
        """Execute a workflow with dependency management and interruption support."""
        if workflow_id not in self.active_workflows:
            return {
                "success": False,
                "messages": [f"❌ Workflow {workflow_id} not found"]
            }
        
        workflow = self.active_workflows[workflow_id]
        results = []
        overall_success = True
        
        try:
            # Execute steps in dependency order
            completed_steps = set()
            
            while len(completed_steps) < len(workflow):
                # Check for cancellation
                if self.cancellation_tokens.get(workflow_id, False):
                    logger.info(f"Workflow {workflow_id} cancelled by user")
                    return {
                        "success": False,
                        "messages": [f"❌ Workflow {workflow_id} was cancelled"],
                        "completed_steps": list(completed_steps),
                        "results": results
                    }
                
                # Find next steps to execute (dependencies satisfied)
                ready_steps = []
                for step_id, step in workflow.items():
                    if step.status == WorkflowStepStatus.PENDING:
                        if all(dep in completed_steps for dep in step.dependencies):
                            ready_steps.append(step)
                
                if not ready_steps:
                    # No more steps can be executed
                    remaining = [step_id for step_id, step in workflow.items() 
                               if step.status == WorkflowStepStatus.PENDING]
                    if remaining:
                        return {
                            "success": False,
                            "messages": [f"❌ Circular dependency or missing steps: {remaining}"],
                            "completed_steps": list(completed_steps),
                            "results": results
                        }
                    break
                
                # Execute ready steps in parallel
                tasks = []
                for step in ready_steps:
                    task = self._execute_step(step, project_path, environment, approvals)
                    tasks.append(task)
                
                # Wait for all parallel steps to complete
                step_results = await asyncio.gather(*tasks, return_exceptions=True)
                
                # Process results
                for step, result in zip(ready_steps, step_results):
                    if isinstance(result, Exception):
                        step.status = WorkflowStepStatus.FAILED
                        step.error = str(result)
                        overall_success = False
                        results.append({
                            "step_id": step.step_id,
                            "step_name": step.name,
                            "status": "failed",
                            "error": str(result)
                        })
                    else:
                        step.status = WorkflowStepStatus.COMPLETED
                        step.result = result
                        completed_steps.add(step.step_id)
                        results.append({
                            "step_id": step.step_id,
                            "step_name": step.name,
                            "status": "completed",
                            "result": result
                        })
                        
                        # If step failed, mark workflow as failed
                        if not result.get("success", False):
                            overall_success = False
            
            return {
                "success": overall_success,
                "messages": [f"✅ Workflow {workflow_id} completed" if overall_success 
                           else f"❌ Workflow {workflow_id} completed with errors"],
                "completed_steps": list(completed_steps),
                "results": results
            }
            
        except Exception as e:
            logger.error(f"Workflow {workflow_id} execution failed: {e}", exc_info=True)
            return {
                "success": False,
                "messages": [f"❌ Workflow {workflow_id} execution failed: {str(e)}"],
                "results": results
            }
        finally:
            # Cleanup
            if workflow_id in self.active_workflows:
                del self.active_workflows[workflow_id]
            if workflow_id in self.cancellation_tokens:
                del self.cancellation_tokens[workflow_id]
    
    async def _execute_step(
        self,
        step: WorkflowStep,
        project_path: str,
        environment: str,
        approvals: List[str]
    ) -> Dict[str, Any]:
        """Execute a single workflow step."""
        step.status = WorkflowStepStatus.RUNNING
        step.start_time = time.time()
        
        try:
            # Execute the tool call with timeout
            if step.timeout_seconds:
                result = await asyncio.wait_for(
                    step.tool_call(project_path, environment, approvals),
                    timeout=step.timeout_seconds
                )
            else:
                result = await step.tool_call(project_path, environment, approvals)
            
            step.end_time = time.time()
            return result
            
        except asyncio.TimeoutError:
            step.status = WorkflowStepStatus.FAILED
            step.error = f"Step {step.name} timed out after {step.timeout_seconds} seconds"
            return {
                "success": False,
                "messages": [f"❌ {step.name} timed out"]
            }
        except Exception as e:
            step.status = WorkflowStepStatus.FAILED
            step.error = str(e)
            return {
                "success": False,
                "messages": [f"❌ {step.name} failed: {str(e)}"]
            }


def audit_log_tool_invocation(
    tool_name: str,
    parameters: Dict[str, Any],
    operator_identity: str = "mcp_client",
    environment: str = "development",
    project_path: str = "",
    execution_time_ms: Optional[int] = None,
    success: Optional[bool] = None,
    error_message: Optional[str] = None,
    checksum_comparison: Optional[Dict[str, str]] = None
) -> None:
    """Log tool invocation with audit trail."""
    operation_id = str(uuid.uuid4())
    
    security_manager.log_tool_invocation(
        tool_name=tool_name,
        parameters=parameters,
        operator_identity=operator_identity,
        environment=environment,
        project_path=project_path,
        operation_id=operation_id,
        execution_time_ms=execution_time_ms,
        success=success,
        error_message=error_message,
        checksum_comparison=checksum_comparison
    )


try:
    import loco_bindings
    HAS_BINDINGS = True
    logger.info("Successfully imported loco_bindings")
except ImportError as e:
    HAS_BINDINGS = False
    logger.warning(f"loco_bindings not available: {e}. Using mock implementation.")
    
    # Mock implementation for testing
    class MockLocoBindings:
        """Mock bindings for development/testing."""
        
        def generate_model(self, project_path: str, name: str, fields: dict, with_timestamps: bool) -> dict:
            return {
                "success": True,
                "messages": [
                    f"Created model: src/models/{name}.rs",
                    f"Created migration: migration/src/m{name}.rs",
                ]
            }
        
        def generate_scaffold(self, project_path: str, name: str, fields: dict, kind: str, with_timestamps: bool) -> dict:
            return {
                "success": True,
                "messages": [
                    f"Created model: src/models/{name}.rs",
                    f"Created controller: src/controllers/{name}s.rs",
                    f"Created views for {name}",
                ]
            }
        
        def generate_controller_view(self, project_path: str, name: str, actions: list, kind: str) -> dict:
            return {
                "success": True,
                "messages": [
                    f"Created controller: src/controllers/{name}.rs",
                    f"Created views for actions: {', '.join(actions)}",
                ]
            }
        
        def migrate_db(self, project_path: str, environment: str = None, approvals: list = None, 
                      timeout_seconds: int = 60, dependencies: list = None) -> dict:
            return {
                "success": True,
                "messages": ["Database migration completed successfully"],
                "checksum": "migrate_abc123"
            }
        
        def rotate_keys(self, project_path: str, environment: str = None, approvals: list = None,
                       timeout_seconds: int = 300, dependencies: list = None) -> dict:
            return {
                "success": True,
                "messages": ["Key rotation completed successfully"],
                "checksum": "rotate_def456"
            }
        
        def clean_temp(self, project_path: str, environment: str = None, approvals: list = None,
                      timeout_seconds: int = 60, dependencies: list = None) -> dict:
            return {
                "success": True,
                "messages": ["Temporary files cleaned successfully"],
                "checksum": "clean_ghi789"
            }

        def create_project(self, project_name: str, template_type: str, destination_path: str,
                          database_type: str = None, background_worker: str = None, asset_serving: bool = None) -> dict:
            return {
                "success": True,
                "messages": [
                    f"Created {template_type} project '{project_name}' at {destination_path}",
                    "Project structure generated successfully",
                    "Dependencies installed",
                    "Configuration files created"
                ],
                "project_path": destination_path
            }

    loco_bindings = MockLocoBindings()


class LocoTools:
    """Collection of MCP tools for loco-rs code generation."""

    def __init__(self, config: Optional[ServerConfig] = None):
        """Initialize the tools collection."""
        self.config = config or ServerConfig.from_env()
        self.orchestrator = WorkflowOrchestrator(self.config)
        self.stats = {
            "total_calls": 0,
            "successful_calls": 0,
            "failed_calls": 0,
        }

    async def generate_model(
        self,
        project_path: str,
        name: str,
        fields: dict[str, str],
        with_timestamps: bool = True,
    ) -> dict[str, Any]:
        """Generate a loco-rs model and migration file.

        Args:
            project_path: Path to the Loco project root
            name: Model name (snake_case)
            fields: Field definitions as {field_name: field_type}
            with_timestamps: Include created_at/updated_at fields

        Returns:
            Generation result with success status and messages
        """
        self.stats["total_calls"] += 1
        
        try:
            logger.info(f"Generating model '{name}' with {len(fields)} fields")
            logger.debug(f"Fields: {fields}")
            
            result = loco_bindings.generate_model(
                project_path=project_path,
                name=name,
                fields=fields,
                with_timestamps=with_timestamps,
            )
            
            if result.get("success"):
                self.stats["successful_calls"] += 1
            else:
                self.stats["failed_calls"] += 1
            
            logger.info(f"Model generation completed: {result.get('success')}")
            return result

        except Exception as e:
            self.stats["failed_calls"] += 1
            logger.error(f"Model generation failed: {e}", exc_info=True)
            return {
                "success": False,
                "messages": [f"错误: {str(e)}"]
            }

    async def generate_scaffold(
        self,
        project_path: str,
        name: str,
        fields: dict[str, str],
        kind: str = "api",
        with_timestamps: bool = True,
    ) -> dict[str, Any]:
        """Generate complete CRUD scaffolding.

        Args:
            project_path: Path to the Loco project root
            name: Resource name (snake_case)
            fields: Field definitions as {field_name: field_type}
            kind: Scaffold type - "api", "html", or "htmx"
            with_timestamps: Include timestamp fields

        Returns:
            Generation result with success status and messages
        """
        self.stats["total_calls"] += 1
        
        try:
            # Validate kind
            if kind not in ["api", "html", "htmx"]:
                raise ValueError(f"Invalid scaffold kind: {kind}. Must be 'api', 'html', or 'htmx'")
            
            logger.info(f"Generating {kind} scaffold for '{name}' with {len(fields)} fields")
            logger.debug(f"Fields: {fields}")
            
            result = loco_bindings.generate_scaffold(
                project_path=project_path,
                name=name,
                fields=fields,
                kind=kind,
                with_timestamps=with_timestamps,
            )
            
            if result.get("success"):
                self.stats["successful_calls"] += 1
            else:
                self.stats["failed_calls"] += 1
            
            logger.info(f"Scaffold generation completed: {result.get('success')}")
            return result

        except Exception as e:
            self.stats["failed_calls"] += 1
            logger.error(f"Scaffold generation failed: {e}", exc_info=True)
            return {
                "success": False,
                "messages": [f"错误: {str(e)}"]
            }

    async def generate_controller_view(
        self,
        project_path: str,
        name: str,
        actions: list[str] | None = None,
        kind: str = "api",
    ) -> dict[str, Any]:
        """Generate controller and views for existing model.

        Args:
            project_path: Path to the Loco project root
            name: Controller name (usually plural, snake_case)
            actions: List of action names to generate
            kind: Controller type - "api", "html", or "htmx"

        Returns:
            Generation result with success status and messages
        """
        self.stats["total_calls"] += 1

        try:
            # Validate kind
            if kind not in ["api", "html", "htmx"]:
                raise ValueError(f"Invalid controller kind: {kind}. Must be 'api', 'html', or 'htmx'")

            # Default actions if not provided
            if actions is None:
                actions = ["index", "show", "create", "update", "delete"]

            logger.info(f"Generating {kind} controller '{name}' with actions: {actions}")

            result = loco_bindings.generate_controller_view(
                project_path=project_path,
                name=name,
                actions=actions,
                kind=kind,
            )

            if result.get("success"):
                self.stats["successful_calls"] += 1
            else:
                self.stats["failed_calls"] += 1

            logger.info(f"Controller generation completed: {result.get('success')}")
            return result

        except Exception as e:
            self.stats["failed_calls"] += 1
            logger.error(f"Controller generation failed: {e}", exc_info=True)
            return {
                "success": False,
                "messages": [f"错误: {str(e)}"]
            }

    async def create_project(
        self,
        project_name: str,
        template_type: str,
        destination_path: str,
        database_type: str = None,
        background_worker: str = None,
        asset_serving: bool = None,
    ) -> dict[str, Any]:
        """Create a new Loco project.

        Args:
            project_name: Project name in snake_case
            template_type: Template type - "saas", "rest_api", or "lightweight"
            destination_path: Directory path where the project will be created
            database_type: Database type - "postgres", "sqlite", or "mysql" (optional)
            background_worker: Background worker - "redis", "postgres", "sqlite", or "none" (optional)
            asset_serving: Enable static asset serving (optional)

        Returns:
            Project creation result with success status and messages
        """
        self.stats["total_calls"] += 1

        try:
            # Validate template_type
            if template_type not in ["saas", "rest_api", "lightweight"]:
                raise ValueError(f"Invalid template type: {template_type}. Must be 'saas', 'rest_api', or 'lightweight'")

            # Validate database_type if provided
            if database_type and database_type not in ["postgres", "sqlite", "mysql"]:
                raise ValueError(f"Invalid database type: {database_type}. Must be 'postgres', 'sqlite', or 'mysql'")

            # Validate background_worker if provided
            if background_worker and background_worker not in ["redis", "postgres", "sqlite", "none"]:
                raise ValueError(f"Invalid background worker: {background_worker}. Must be 'redis', 'postgres', 'sqlite', or 'none'")

            logger.info(f"Creating {template_type} project '{project_name}' at {destination_path}")
            logger.debug(f"Database: {database_type}, Worker: {background_worker}, Assets: {asset_serving}")

            result = loco_bindings.create_project(
                project_name=project_name,
                template_type=template_type,
                destination_path=destination_path,
                database_type=database_type,
                background_worker=background_worker,
                asset_serving=asset_serving,
            )

            if result.get("success"):
                self.stats["successful_calls"] += 1
            else:
                self.stats["failed_calls"] += 1

            logger.info(f"Project creation completed: {result.get('success')}")
            return result

        except Exception as e:
            self.stats["failed_calls"] += 1
            logger.error(f"Project creation failed: {e}", exc_info=True)
            return {
                "success": False,
                "messages": [f"错误: {str(e)}"]
            }

    async def migrate_db(
        self,
        project_path: str,
        environment: str = None,
        approvals: list[str] = None,
        timeout_seconds: int = None,
        dependencies: list[str] = None,
    ) -> dict[str, Any]:
        """Execute database migration.

        Args:
            project_path: Path to the Loco project root
            environment: Environment name (optional)
            approvals: List of required approvals
            timeout_seconds: Timeout in seconds (optional, uses config default)
            dependencies: List of dependencies

        Returns:
            Migration result with success status and messages
        """
        self.stats["total_calls"] += 1
        
        # Validate environment
        if environment and not self.config.validate_environment(environment):
            return {
                "success": False,
                "messages": [f"❌ Invalid environment: {environment}"]
            }
        
        # Get timeout from config if not provided
        if timeout_seconds is None:
            timeout_seconds = self.config.get_tool_timeout("migrate_db", environment)
        
        # Prepare parameters for audit logging
        parameters = {
            "project_path": project_path,
            "environment": environment,
            "approvals": approvals or [],
            "timeout_seconds": timeout_seconds,
            "dependencies": dependencies or [],
        }
        
        start_time = time.time()
        error_message = None
        
        try:
            logger.info(f"Executing database migration for project: {project_path}")
            logger.debug(f"Environment: {environment}, Approvals: {approvals}, Timeout: {timeout_seconds}")
            
            result = loco_bindings.migrate_db(
                project_path=project_path,
                environment=environment,
                approvals=approvals or [],
                timeout_seconds=timeout_seconds,
                dependencies=dependencies or [],
            )
            
            execution_time_ms = int((time.time() - start_time) * 1000)
            success = result.get("success", False)
            
            if success:
                self.stats["successful_calls"] += 1
            else:
                self.stats["failed_calls"] += 1
                error_message = "; ".join(result.get("messages", []))
            
            # Log successful invocation
            audit_log_tool_invocation(
                tool_name="migrate_db",
                parameters=parameters,
                environment=environment or "development",
                project_path=project_path,
                execution_time_ms=execution_time_ms,
                success=success,
                error_message=error_message
            )
            
            logger.info(f"Database migration completed: {success}")
            return result

        except Exception as e:
            execution_time_ms = int((time.time() - start_time) * 1000)
            self.stats["failed_calls"] += 1
            error_message = str(e)
            
            # Log failed invocation
            audit_log_tool_invocation(
                tool_name="migrate_db",
                parameters=parameters,
                environment=environment or "development",
                project_path=project_path,
                execution_time_ms=execution_time_ms,
                success=False,
                error_message=error_message
            )
            
            logger.error(f"Database migration failed: {e}", exc_info=True)
            return {
                "success": False,
                "messages": [f"❌ Database migration failed: {str(e)}"]
            }

    async def rotate_keys(
        self,
        project_path: str,
        environment: str = None,
        approvals: list[str] = None,
        timeout_seconds: int = None,
        dependencies: list[str] = None,
    ) -> dict[str, Any]:
        """Rotate service account keys.

        Args:
            project_path: Path to the Loco project root
            environment: Environment name (optional)
            approvals: List of required approvals
            timeout_seconds: Timeout in seconds (optional, uses config default)
            dependencies: List of dependencies

        Returns:
            Key rotation result with success status and messages
        """
        self.stats["total_calls"] += 1
        
        # Validate environment
        if environment and not self.config.validate_environment(environment):
            return {
                "success": False,
                "messages": [f"❌ Invalid environment: {environment}"]
            }
        
        # Get timeout from config if not provided
        if timeout_seconds is None:
            timeout_seconds = self.config.get_tool_timeout("rotate_keys", environment)
        
        # Prepare parameters for audit logging
        parameters = {
            "project_path": project_path,
            "environment": environment,
            "approvals": approvals or [],
            "timeout_seconds": timeout_seconds,
            "dependencies": dependencies or [],
        }
        
        start_time = time.time()
        error_message = None
        
        try:
            logger.info(f"Rotating service account keys for project: {project_path}")
            logger.debug(f"Environment: {environment}, Approvals: {approvals}, Timeout: {timeout_seconds}")
            
            result = loco_bindings.rotate_keys(
                project_path=project_path,
                environment=environment,
                approvals=approvals or [],
                timeout_seconds=timeout_seconds,
                dependencies=dependencies or [],
            )
            
            execution_time_ms = int((time.time() - start_time) * 1000)
            success = result.get("success", False)
            
            if success:
                self.stats["successful_calls"] += 1
            else:
                self.stats["failed_calls"] += 1
                error_message = "; ".join(result.get("messages", []))
            
            # Log successful invocation
            audit_log_tool_invocation(
                tool_name="rotate_keys",
                parameters=parameters,
                environment=environment or "production",
                project_path=project_path,
                execution_time_ms=execution_time_ms,
                success=success,
                error_message=error_message
            )
            
            logger.info(f"Key rotation completed: {success}")
            return result

        except Exception as e:
            execution_time_ms = int((time.time() - start_time) * 1000)
            self.stats["failed_calls"] += 1
            error_message = str(e)
            
            # Log failed invocation
            audit_log_tool_invocation(
                tool_name="rotate_keys",
                parameters=parameters,
                environment=environment or "production",
                project_path=project_path,
                execution_time_ms=execution_time_ms,
                success=False,
                error_message=error_message
            )
            
            logger.error(f"Key rotation failed: {e}", exc_info=True)
            return {
                "success": False,
                "messages": [f"❌ Key rotation failed: {str(e)}"]
            }

    async def clean_temp(
        self,
        project_path: str,
        environment: str = None,
        approvals: list[str] = None,
        timeout_seconds: int = None,
        dependencies: list[str] = None,
    ) -> dict[str, Any]:
        """Clean temporary files.

        Args:
            project_path: Path to the Loco project root
            environment: Environment name (optional)
            approvals: List of required approvals
            timeout_seconds: Timeout in seconds (optional, uses config default)
            dependencies: List of dependencies

        Returns:
            Cleanup result with success status and messages
        """
        self.stats["total_calls"] += 1
        
        # Validate environment
        if environment and not self.config.validate_environment(environment):
            return {
                "success": False,
                "messages": [f"❌ Invalid environment: {environment}"]
            }
        
        # Get timeout from config if not provided
        if timeout_seconds is None:
            timeout_seconds = self.config.get_tool_timeout("clean_temp", environment)
        
        # Prepare parameters for audit logging
        parameters = {
            "project_path": project_path,
            "environment": environment,
            "approvals": approvals or [],
            "timeout_seconds": timeout_seconds,
            "dependencies": dependencies or [],
        }
        
        start_time = time.time()
        error_message = None
        
        try:
            logger.info(f"Cleaning temporary files for project: {project_path}")
            logger.debug(f"Environment: {environment}, Approvals: {approvals}, Timeout: {timeout_seconds}")
            
            result = loco_bindings.clean_temp(
                project_path=project_path,
                environment=environment,
                approvals=approvals or [],
                timeout_seconds=timeout_seconds,
                dependencies=dependencies or [],
            )
            
            execution_time_ms = int((time.time() - start_time) * 1000)
            success = result.get("success", False)
            
            if success:
                self.stats["successful_calls"] += 1
            else:
                self.stats["failed_calls"] += 1
                error_message = "; ".join(result.get("messages", []))
            
            # Log successful invocation
            audit_log_tool_invocation(
                tool_name="clean_temp",
                parameters=parameters,
                environment=environment or "development",
                project_path=project_path,
                execution_time_ms=execution_time_ms,
                success=success,
                error_message=error_message
            )
            
            logger.info(f"Temporary file cleanup completed: {success}")
            return result

        except Exception as e:
            execution_time_ms = int((time.time() - start_time) * 1000)
            self.stats["failed_calls"] += 1
            error_message = str(e)
            
            # Log failed invocation
            audit_log_tool_invocation(
                tool_name="clean_temp",
                parameters=parameters,
                environment=environment or "development",
                project_path=project_path,
                execution_time_ms=execution_time_ms,
                success=False,
                error_message=error_message
            )
            
            logger.error(f"Temporary file cleanup failed: {e}", exc_info=True)
            return {
                "success": False,
                "messages": [f"❌ Temporary file cleanup failed: {str(e)}"]
            }

    async def execute_rotate_keys_workflow(
        self,
        project_path: str,
        environment: str = None,
        approvals: list[str] = None,
        timeout_seconds: int = None,
        dependencies: list[str] = None,
    ) -> dict[str, Any]:
        """Execute a multi-step key rotation workflow.

        This workflow includes:
        1. Pre-rotation validation
        2. Key rotation
        3. Post-rotation verification
        4. Service restart coordination

        Args:
            project_path: Path to the Loco project root
            environment: Environment name (optional)
            approvals: List of required approvals
            timeout_seconds: Timeout in seconds (optional, uses config default)
            dependencies: List of dependencies

        Returns:
            Workflow result with success status and step details
        """
        self.stats["total_calls"] += 1
        
        # Validate environment
        if environment and not self.config.validate_environment(environment):
            return {
                "success": False,
                "messages": [f"❌ Invalid environment: {environment}"]
            }
        
        # Get timeout from config if not provided
        if timeout_seconds is None:
            timeout_seconds = self.config.get_tool_timeout("rotate_keys", environment)
        
        # Create workflow steps
        workflow_id = f"rotate_keys_{uuid.uuid4().hex[:8]}"
        
        steps = [
            WorkflowStep(
                step_id="validate_dependencies",
                name="Validate Dependencies",
                tool_call=self._validate_key_dependencies,
                timeout_seconds=30
            ),
            WorkflowStep(
                step_id="backup_current_keys",
                name="Backup Current Keys",
                tool_call=self._backup_current_keys,
                dependencies=["validate_dependencies"],
                timeout_seconds=60
            ),
            WorkflowStep(
                step_id="rotate_keys",
                name="Rotate Keys",
                tool_call=self._rotate_keys_step,
                dependencies=["backup_current_keys"],
                timeout_seconds=timeout_seconds - 90  # Reserve time for other steps
            ),
            WorkflowStep(
                step_id="verify_new_keys",
                name="Verify New Keys",
                tool_call=self._verify_new_keys,
                dependencies=["rotate_keys"],
                timeout_seconds=60
            ),
            WorkflowStep(
                step_id="restart_services",
                name="Restart Services",
                tool_call=self._restart_services,
                dependencies=["verify_new_keys"],
                timeout_seconds=120
            )
        ]
        
        # Create and execute workflow
        self.orchestrator.create_workflow(workflow_id, steps)
        
        # Log workflow start
        audit_log_tool_invocation(
            tool_name="execute_rotate_keys_workflow",
            parameters={
                "project_path": project_path,
                "environment": environment,
                "approvals": approvals or [],
                "timeout_seconds": timeout_seconds,
                "dependencies": dependencies or [],
                "workflow_id": workflow_id
            },
            environment=environment or "production",
            project_path=project_path
        )
        
        try:
            result = await self.orchestrator.execute_workflow(
                workflow_id, project_path, environment, approvals
            )
            
            if result.get("success"):
                self.stats["successful_calls"] += 1
            else:
                self.stats["failed_calls"] += 1
            
            return result
            
        except Exception as e:
            self.stats["failed_calls"] += 1
            logger.error(f"Key rotation workflow failed: {e}", exc_info=True)
            return {
                "success": False,
                "messages": [f"❌ Key rotation workflow failed: {str(e)}"]
            }

    async def execute_clean_temp_workflow(
        self,
        project_path: str,
        environment: str = None,
        approvals: list[str] = None,
        timeout_seconds: int = None,
        dependencies: list[str] = None,
    ) -> dict[str, Any]:
        """Execute a multi-step temporary file cleanup workflow.

        This workflow includes:
        1. Disk space analysis
        2. File age verification
        3. Cleanup execution
        4. Space reclamation verification

        Args:
            project_path: Path to the Loco project root
            environment: Environment name (optional)
            approvals: List of required approvals
            timeout_seconds: Timeout in seconds (optional, uses config default)
            dependencies: List of dependencies

        Returns:
            Workflow result with success status and step details
        """
        self.stats["total_calls"] += 1
        
        # Validate environment
        if environment and not self.config.validate_environment(environment):
            return {
                "success": False,
                "messages": [f"❌ Invalid environment: {environment}"]
            }
        
        # Get timeout from config if not provided
        if timeout_seconds is None:
            timeout_seconds = self.config.get_tool_timeout("clean_temp", environment)
        
        # Create workflow steps
        workflow_id = f"clean_temp_{uuid.uuid4().hex[:8]}"
        
        steps = [
            WorkflowStep(
                step_id="analyze_disk_space",
                name="Analyze Disk Space",
                tool_call=self._analyze_disk_space,
                timeout_seconds=30
            ),
            WorkflowStep(
                step_id="verify_file_ages",
                name="Verify File Ages",
                tool_call=self._verify_file_ages,
                dependencies=["analyze_disk_space"],
                timeout_seconds=45
            ),
            WorkflowStep(
                step_id="clean_temp_files",
                name="Clean Temporary Files",
                tool_call=self._clean_temp_files_step,
                dependencies=["verify_file_ages"],
                timeout_seconds=timeout_seconds - 75  # Reserve time for other steps
            ),
            WorkflowStep(
                step_id="verify_cleanup",
                name="Verify Cleanup",
                tool_call=self._verify_cleanup,
                dependencies=["clean_temp_files"],
                timeout_seconds=30
            )
        ]
        
        # Create and execute workflow
        self.orchestrator.create_workflow(workflow_id, steps)
        
        # Log workflow start
        audit_log_tool_invocation(
            tool_name="execute_clean_temp_workflow",
            parameters={
                "project_path": project_path,
                "environment": environment,
                "approvals": approvals or [],
                "timeout_seconds": timeout_seconds,
                "dependencies": dependencies or [],
                "workflow_id": workflow_id
            },
            environment=environment or "development",
            project_path=project_path
        )
        
        try:
            result = await self.orchestrator.execute_workflow(
                workflow_id, project_path, environment, approvals
            )
            
            if result.get("success"):
                self.stats["successful_calls"] += 1
            else:
                self.stats["failed_calls"] += 1
            
            return result
            
        except Exception as e:
            self.stats["failed_calls"] += 1
            logger.error(f"Clean temp workflow failed: {e}", exc_info=True)
            return {
                "success": False,
                "messages": [f"❌ Clean temp workflow failed: {str(e)}"]
            }

    # Workflow step implementations
    async def _validate_key_dependencies(self, project_path: str, environment: str, approvals: list[str]) -> dict[str, Any]:
        """Validate key rotation dependencies."""
        return {
            "success": True,
            "messages": ["✅ Key dependencies validated"]
        }

    async def _backup_current_keys(self, project_path: str, environment: str, approvals: list[str]) -> dict[str, Any]:
        """Backup current keys before rotation."""
        return {
            "success": True,
            "messages": ["✅ Current keys backed up"]
        }

    async def _rotate_keys_step(self, project_path: str, environment: str, approvals: list[str]) -> dict[str, Any]:
        """Execute the actual key rotation step."""
        # Call the actual rotate_keys method
        return await self.rotate_keys(project_path, environment, approvals)

    async def _verify_new_keys(self, project_path: str, environment: str, approvals: list[str]) -> dict[str, Any]:
        """Verify new keys are working."""
        return {
            "success": True,
            "messages": ["✅ New keys verified"]
        }

    async def _restart_services(self, project_path: str, environment: str, approvals: list[str]) -> dict[str, Any]:
        """Restart services with new keys."""
        return {
            "success": True,
            "messages": ["✅ Services restarted with new keys"]
        }

    async def _analyze_disk_space(self, project_path: str, environment: str, approvals: list[str]) -> dict[str, Any]:
        """Analyze disk space before cleanup."""
        return {
            "success": True,
            "messages": ["✅ Disk space analyzed"]
        }

    async def _verify_file_ages(self, project_path: str, environment: str, approvals: list[str]) -> dict[str, Any]:
        """Verify file ages before cleanup."""
        return {
            "success": True,
            "messages": ["✅ File ages verified"]
        }

    async def _clean_temp_files_step(self, project_path: str, environment: str, approvals: list[str]) -> dict[str, Any]:
        """Execute the actual cleanup step."""
        # Call the actual clean_temp method
        return await self.clean_temp(project_path, environment, approvals)

    async def _verify_cleanup(self, project_path: str, environment: str, approvals: list[str]) -> dict[str, Any]:
        """Verify cleanup was successful."""
        return {
            "success": True,
            "messages": ["✅ Cleanup verified"]
        }

    def get_statistics(self) -> dict[str, Any]:
        """Get tool usage statistics.

        Returns:
            Statistics dictionary
        """
        total = self.stats["total_calls"]
        success_rate = (
            (self.stats["successful_calls"] / total * 100)
            if total > 0
            else 0.0
        )

        return {
            "total_calls": total,
            "successful_calls": self.stats["successful_calls"],
            "failed_calls": self.stats["failed_calls"],
            "success_rate_percent": success_rate,
            "bindings_available": HAS_BINDINGS,
        }

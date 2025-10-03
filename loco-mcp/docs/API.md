# API Documentation

This document provides comprehensive API documentation for the loco-mcp-server and loco-bindings libraries.

## Table of Contents

- [loco-mcp-server API](#loco-mcp-server-api)
  - [LocoMCPServer](#locomcpserver)
  - [MCP Tools](#mcp-tools)
- [loco-bindings API](#loco-bindings-api)
  - [Core Functions](#core-functions)
  - [Error Types](#error-types)
  - [Field Types](#field-types)
- [Examples](#examples)

## loco-mcp-server API

### LocoMCPServer

The main MCP server class that handles communication with Claude Code Agent.

#### Constructor

```python
def __init__(self, config: Optional[ServerConfig] = None) -> None:
    """Initialize the MCP server.

    Args:
        config: Server configuration. If None, uses default configuration.

    Example:
        server = LocoMCPServer()
        # or with custom config
        config = ServerConfig(host="0.0.0.0", port=8080)
        server = LocoMCPServer(config)
    """
```

#### Methods

##### start()

```python
async def start(self) -> None:
    """Start the MCP server.

    This method initializes the underlying MCP server, registers all tools,
    and starts listening for incoming connections from Claude Code Agent.

    Raises:
        Exception: If server fails to start due to configuration or network issues.

    Example:
        await server.start()
    """
```

##### shutdown()

```python
async def shutdown(self) -> None:
    """Gracefully shutdown the server.

    This method stops the server and closes all active connections.
    It's recommended to call this method before exiting the application.

    Example:
        await server.shutdown()
    """
```

##### get_health_status()

```python
def get_health_status(self) -> Dict[str, Any]:
    """Get server health and performance status.

    Returns:
        Dict containing:
        - status: "healthy" or "stopped"
        - uptime_seconds: Server uptime in seconds
        - requests_handled: Total number of requests processed
        - errors: Total number of errors encountered
        - error_rate_percent: Error rate as percentage
        - tools_available: Number of available tools
        - version: Server version
        - performance: Performance metrics

    Example:
        status = server.get_health_status()
        print(f"Server status: {status['status']}")
        print(f"Uptime: {status['uptime_seconds']}s")
        print(f"Error rate: {status['error_rate_percent']:.1f}%")
    """
```

### MCP Tools

The server exposes three main MCP tools for loco-rs code generation.

#### loco.generate_model

Generates a loco-rs model and migration file.

**Parameters:**
```json
{
  "model_name": {
    "type": "string",
    "required": true,
    "description": "Model name in snake_case (e.g., 'user', 'blog_post')"
  },
  "fields": {
    "type": "array<string>",
    "required": true,
    "description": "Array of field definitions in format 'name:type[:constraint]'"
  },
  "project_path": {
    "type": "string",
    "required": false,
    "description": "Path to loco-rs project directory (default: current directory)"
  }
}
```

**Returns:**
```json
{
  "success": "boolean",
  "created_files": [
    {
      "path": "string",
      "type": "string",
      "size_bytes": "integer"
    }
  ],
  "modified_files": [
    {
      "path": "string",
      "type": "string"
    }
  ],
  "errors": ["string"],
  "processing_time_ms": "number"
}
```

**Example:**
```json
{
  "model_name": "user",
  "fields": ["name:string", "email:string:unique", "created_at:datetime"],
  "project_path": "/path/to/project"
}
```

#### loco.generate_scaffold

Generates complete CRUD scaffolding including model, controller, views, and routes.

**Parameters:**
```json
{
  "model_name": {
    "type": "string",
    "required": true,
    "description": "Model name in snake_case"
  },
  "fields": {
    "type": "array<string>",
    "required": true,
    "description": "Array of field definitions"
  },
  "include_views": {
    "type": "boolean",
    "required": false,
    "default": true,
    "description": "Generate view templates"
  },
  "include_controllers": {
    "type": "boolean",
    "required": false,
    "default": true,
    "description": "Generate controller"
  },
  "api_only": {
    "type": "boolean",
    "required": false,
    "default": false,
    "description": "API-only mode (no views)"
  },
  "project_path": {
    "type": "string",
    "required": false,
    "description": "Path to loco-rs project"
  }
}
```

**Example:**
```json
{
  "model_name": "blog_post",
  "fields": ["title:string", "content:text", "published:boolean"],
  "include_views": true,
  "api_only": false
}
```

#### loco.generate_controller_view

Generates controller and views for an existing model.

**Parameters:**
```json
{
  "model_name": {
    "type": "string",
    "required": true,
    "description": "Name of existing model"
  },
  "actions": {
    "type": "array<string>",
    "required": false,
    "default": ["index", "show", "create", "update", "delete"],
    "description": "Controller actions to include"
  },
  "view_types": {
    "type": "array<string>",
    "required": false,
    "default": ["list", "show", "form", "edit"],
    "description": "View templates to generate"
  },
  "project_path": {
    "type": "string",
    "required": false,
    "description": "Path to loco-rs project"
  }
}
```

## loco-bindings API

### Core Functions

These functions are exposed by the Rust bindings and provide the core code generation functionality.

#### generate_model()

```python
def generate_model(params: Dict[str, Any]) -> Dict[str, Any]:
    """Generate a model and migration file.

    Args:
        params: Dictionary containing:
        - model_name (str, required): Model name in snake_case
        - fields (List[str], required): Field definitions
        - project_path (str, optional): Project directory path

    Returns:
        Dictionary with success status, created/modified files, and errors.

    Raises:
        ValidationError: If model name or fields are invalid
        ProjectError: If not in a valid loco-rs project
        FileOperationError: If file operations fail

    Example:
        result = generate_model({
            "model_name": "user",
            "fields": ["name:string", "email:string:unique"]
        })
    """
```

#### generate_scaffold()

```python
def generate_scaffold(params: Dict[str, Any]) -> Dict[str, Any]:
    """Generate complete CRUD scaffolding.

    Args:
        params: Dictionary containing:
        - model_name (str, required): Model name
        - fields (List[str], required): Field definitions
        - include_views (bool, optional): Generate views (default: True)
        - include_controllers (bool, optional): Generate controller (default: True)
        - api_only (bool, optional): API-only mode (default: False)
        - project_path (str, optional): Project path

    Returns:
        Dictionary with operation results.

    Example:
        result = generate_scaffold({
            "model_name": "blog_post",
            "fields": ["title:string", "content:text"],
            "api_only": False
        })
    """
```

#### generate_controller_view()

```python
def generate_controller_view(params: Dict[str, Any]) -> Dict[str, Any]:
    """Generate controller and views for existing model.

    Args:
        params: Dictionary containing:
        - model_name (str, required): Existing model name
        - actions (List[str], optional): Controller actions
        - view_types (List[str], optional): View types to generate
        - project_path (str, optional): Project path

    Returns:
        Dictionary with operation results.

    Example:
        result = generate_controller_view({
            "model_name": "user",
            "actions": ["index", "show"]
        })
    """
```

#### get_performance_metrics()

```python
def get_performance_metrics() -> Dict[str, Any]:
    """Get performance metrics for the binding library.

    Returns:
        Dictionary containing:
        - total_calls: Total number of function calls
        - avg_duration_ms: Average response time in milliseconds
        - cache_hit_rate: Template cache hit rate as percentage
        - memory_usage_mb: Estimated memory usage in MB

    Example:
        metrics = get_performance_metrics()
        print(f"Average response time: {metrics['avg_duration_ms']:.2f}ms")
    """
```

### Error Types

The library defines several error types for different failure scenarios.

#### ValidationError

Raised when input parameters don't meet validation requirements.

```python
class ValidationError(Exception):
    """Raised when input validation fails.

    Attributes:
        message: Error description
        suggestions: List of suggestions to fix the error
        context: Additional context information
    """
```

#### FileOperationError

Raised when file system operations fail.

```python
class FileOperationError(Exception):
    """Raised when file operations fail.

    Attributes:
        message: Error description
        file_path: Path of the file that caused the error
        operation: Type of operation that failed
    """
```

#### ProjectError

Raised when project structure validation fails.

```python
class ProjectError(Exception):
    """Raised when project validation fails.

    Attributes:
        message: Error description
        project_path: Path of the invalid project
        missing_elements: List of missing required elements
    """
```

### Field Types

Supported field types in field definitions:

| Type | Description | Database Mapping |
|------|-------------|------------------|
| `string` | String with 255 character limit | VARCHAR(255) |
| `text` | Unlimited text content | TEXT |
| `i32` | 32-bit integer | INTEGER |
| `i64` | 64-bit integer | BIGINT |
| `f32` | 32-bit float | FLOAT |
| `f64` | 64-bit float | DOUBLE |
| `boolean` | Boolean value | BOOLEAN |
| `datetime` | DateTime with timezone | TIMESTAMP WITH TIME ZONE |
| `uuid` | UUID identifier | UUID |
| `json` | JSON data | JSON |

### Field Constraints

Supported constraints for field definitions:

| Constraint | Description | Example |
|------------|-------------|---------|
| `unique` | Unique constraint on the field | `email:string:unique` |
| `nullable` | Allows NULL values | `description:text:nullable` |
| `primary_key` | Primary key (auto-increment) | `id:i32:primary_key` |
| `default:<value>` | Default value | `status:string:default:active` |
| `foreign_key:<table>` | Foreign key reference | `user_id:i64:foreign_key:users` |

## Examples

### Basic Usage

```python
import loco_bindings

# Generate a simple model
result = loco_bindings.generate_model({
    "model_name": "user",
    "fields": [
        "name:string",
        "email:string:unique",
        "created_at:datetime"
    ]
})

if result["success"]:
    print("Model generated successfully!")
    for file in result["created_files"]:
        print(f"Created: {file['path']}")
else:
    print("Errors:", result["errors"])
```

### Complete Scaffold

```python
# Generate full CRUD scaffold
result = loco_bindings.generate_scaffold({
    "model_name": "blog_post",
    "fields": [
        "title:string",
        "content:text",
        "published:boolean",
        "author_id:i64:foreign_key:users"
    ],
    "include_views": True,
    "include_controllers": True
})
```

### API-Only Mode

```python
# Generate API-only scaffold
result = loco_bindings.generate_scaffold({
    "model_name": "api_key",
    "fields": [
        "key:string:unique",
        "name:string",
        "permissions:json"
    ],
    "api_only": True
})
```

### Error Handling

```python
try:
    result = loco_bindings.generate_model({
        "model_name": "user",
        "fields": ["name:string", "email:string"]
    })
except ValidationError as e:
    print(f"Validation error: {e}")
    print(f"Suggestions: {e.suggestions}")
except ProjectError as e:
    print(f"Project error: {e}")
    print(f"Missing elements: {e.missing_elements}")
except FileOperationError as e:
    print(f"File operation error: {e}")
    print(f"File: {e.file_path}")
```

### Performance Monitoring

```python
# Check performance metrics
metrics = loco_bindings.get_performance_metrics()
print(f"Total calls: {metrics['total_calls']}")
print(f"Average time: {metrics['avg_duration_ms']:.2f}ms")
print(f"Cache hit rate: {metrics['hit_rate']:.1f}%")
print(f"Memory usage: {metrics['memory_usage_mb']:.1f}MB")
```

## Integration with loco-mcp-server

```python
from loco_mcp_server import LocoMCPServer

# Create server with custom configuration
config = ServerConfig(
    host="localhost",
    port=8080,
    log_level="INFO",
    default_project_path="/path/to/loco/project"
)

server = LocoMCPServer(config)

# Start server
await server.start()

# Check server health
status = server.get_health_status()
print(f"Server status: {status['status']}")

# Graceful shutdown
await server.shutdown()
```

## Best Practices

1. **Model Naming**: Use snake_case for model names (e.g., `user_profile`, `blog_post`)
2. **Field Definitions**: Be specific with types and constraints
3. **Error Handling**: Always check the `success` field and handle errors appropriately
4. **Performance**: Monitor metrics to ensure <10ms response times
5. **Project Structure**: Ensure you're in a valid loco-rs project directory

## Troubleshooting

### Common Issues

1. **Import Errors**: Ensure loco-bindings is properly installed
2. **Project Validation**: Check that you're in a valid loco-rs project
3. **Permission Errors**: Ensure write permissions in the project directory
4. **Performance Issues**: Check metrics and consider template warmup

### Debug Mode

Enable debug logging for troubleshooting:

```python
import logging
logging.basicConfig(level=logging.DEBUG)

# Or use server configuration
config = ServerConfig(log_level="DEBUG")
server = LocoMCPServer(config)
```
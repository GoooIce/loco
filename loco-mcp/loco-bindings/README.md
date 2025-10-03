# loco-bindings

High-performance Python bindings for loco-rs code generation functionality, enabling Claude Code Agent to generate scaffolding, models, and controllers through direct function calls rather than CLI operations.

## Overview

`loco-bindings` provides Rust-Python bindings that expose loco-rs generate functionality as Python functions, bypassing CLI overhead for sub-10ms response times. This is the core binding library used by the loco-mcp-server.

## Features

- **🚀 High Performance**: Direct Rust function calls with <10ms response times
- **🔧 Complete Generation**: Models, scaffolds, controllers, views, and migrations
- **🛡️ Type Safe**: Full type validation and error handling
- **📁 Project Aware**: Automatic loco-rs project detection and validation
- **🎯 Convention Following**: Respects loco-rs naming and structure conventions

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/loco-rs/loco.git
cd loco/loco-mcp/loco-bindings

# Build the Python bindings
maturin develop

# Or install in development mode
pip install -e .
```

### From PyPI (when published)

```bash
pip install loco-bindings
```

## Requirements

- Rust 1.70+
- Python 3.11+
- PyO3 0.22+
- Maturin (for building)

## Quick Start

```python
import loco_bindings

# Generate a model
params = {
    "model_name": "user",
    "fields": ["name:string", "email:string:unique", "created_at:datetime"],
    "project_path": "/path/to/your/loco/project"
}

result = loco_bindings.generate_model(params)
if result["success"]:
    print("Model generated successfully!")
    for file in result["created_files"]:
        print(f"Created: {file['path']}")
else:
    print("Error:", result["errors"])
```

## API Reference

### Core Functions

#### `generate_model(params: dict) -> dict`

Generate a model and migration file.

**Parameters:**
- `model_name` (str): Name of the model (snake_case)
- `fields` (list[str]): Field definitions in format "name:type[:constraint]"
- `project_path` (str, optional): Path to loco-rs project (default: current directory)

**Returns:**
```python
{
    "success": bool,
    "created_files": [{"path": str, "type": str, "size_bytes": int}],
    "modified_files": [{"path": str, "type": str}],
    "errors": [str]
}
```

#### `generate_scaffold(params: dict) -> dict`

Generate complete CRUD scaffolding (model, migration, controller, views).

**Parameters:**
- `model_name` (str): Name of the model
- `fields` (list[str]): Field definitions
- `include_views` (bool, optional): Generate view templates (default: True)
- `include_controllers` (bool, optional): Generate controller (default: True)
- `api_only` (bool, optional): API-only mode (default: False)
- `project_path` (str, optional): Project path (default: current directory)

#### `generate_controller_view(params: dict) -> dict`

Generate controller and views for existing model.

**Parameters:**
- `model_name` (str): Name of existing model
- `actions` (list[str], optional): Controller actions (default: ["index", "show", "create", "update", "delete"])
- `view_types` (list[str], optional): View types (default: ["list", "show", "form", "edit"])
- `project_path` (str, optional): Project path (default: current directory)

### Field Types

Supported field types in field definitions:

- `string` - String with 255 character limit
- `text` - Unlimited text content
- `i32` - 32-bit integer
- `i64` - 64-bit integer
- `f32` - 32-bit float
- `f64` - 64-bit float
- `boolean` - Boolean value
- `datetime` - DateTime with timezone
- `uuid` - UUID identifier
- `json` - JSON data

### Field Constraints

Supported constraints:

- `unique` - Unique constraint
- `nullable` - Allows NULL values
- `primary_key` - Primary key (auto-increment)
- `default:<value>` - Default value
- `foreign_key:<table>` - Foreign key reference

## Usage Examples

### Basic Model Generation

```python
import loco_bindings

# Simple user model
result = loco_bindings.generate_model({
    "model_name": "user",
    "fields": [
        "name:string",
        "email:string:unique",
        "age:i32",
        "is_active:boolean"
    ]
})
```

### Complete Scaffold Generation

```python
# Full CRUD with views
result = loco_bindings.generate_scaffold({
    "model_name": "blog_post",
    "fields": [
        "title:string",
        "content:text",
        "published:boolean",
        "author_id:i64"
    ],
    "include_views": True,
    "include_controllers": True
})
```

### API-Only Scaffold

```python
# API-only mode (no views)
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

### Controller for Existing Model

```python
# Add controller to existing model
result = loco_bindings.generate_controller_view({
    "model_name": "user",
    "actions": ["index", "show", "create", "update"],
    "view_types": ["list", "show", "form"]
})
```

## Performance

The bindings are optimized for performance:

- **Template Caching**: Frequently used templates are cached
- **Zero-Copy Operations**: Minimized data copying between Rust and Python
- **Direct Function Calls**: No CLI overhead
- **Target**: <10ms response time for most operations

### Performance Monitoring

```python
# Get performance metrics
metrics = loco_bindings.get_performance_metrics()
print(f"Average response time: {metrics['avg_duration_ms']:.2f}ms")
print(f"Cache hit rate: {metrics['hit_rate']:.1%}")
```

## Error Handling

All functions return structured responses with detailed error information:

```python
result = loco_bindings.generate_model(params)

if not result["success"]:
    for error in result["errors"]:
        print(f"Error: {error}")

    # Error includes suggestions for common issues
    if "Model name must start with a letter" in str(result["errors"]):
        print("Suggestion: Use snake_case naming like 'user_profile'")
```

## Integration with loco-mcp-server

This library is designed to be used by the loco-mcp-server:

```python
# In loco-mcp-server
from loco_bindings import generate_model, generate_scaffold

# Expose as MCP tools
@app.tool("loco.generate_model")
def generate_model_tool(params):
    return generate_model(params)
```

## Development

### Building

```bash
# Development build
maturin develop

# Release build
maturin build --release

# Run tests
cargo test
pytest tests/
```

### Testing

```bash
# Rust tests
cargo test

# Python tests
pytest tests/

# Integration tests
python -m pytest tests/test_integration.py
```

## Architecture

The library consists of several modules:

- **`generate.rs`**: Core generation logic
- **`template.rs`**: Template rendering with caching
- **`field.rs`**: Field parsing and validation
- **`error.rs`**: Error handling types
- **`bindings.rs`**: Python bindings (PyO3)
- **`performance.rs`**: Performance monitoring
- **`template_cache.rs`**: Template caching system

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Run `cargo fmt` and `cargo clippy`
6. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Support

- **Issues**: [GitHub Issues](https://github.com/loco-rs/loco/issues)
- **Documentation**: [Loco Framework Docs](https://loco.rs)
- **Community**: [Loco Discord](https://discord.gg/loco)
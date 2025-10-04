# Loco Bindings - Python Bindings for Loco-rs

Python bindings for the [Loco-rs](https://loco.rs) code generator. This provides a thin wrapper around the Loco generator, allowing you to generate models, controllers, and scaffolds from Python.

## Features

- 🚀 **Direct Integration** - Uses loco-gen directly for 100% compatibility
- 🎯 **Simple API** - Clean Python interface to Loco's generator
- ⚡ **High Performance** - Native Rust implementation with Python bindings
- 🔄 **Auto-sync** - Always up-to-date with loco-gen improvements

## Installation

```bash
# Development installation
cd loco-bindings
maturin develop
```

## Usage

### Generate a Model

```python
import loco_bindings

result = loco_bindings.generate_model(
    project_path="/path/to/loco/project",
    name="user",
    fields={
        "name": "string",
        "email": "string^",  # unique
        "age": "int",
    },
    with_timestamps=True
)

print(result["messages"])
```

### Generate a Scaffold

Generate a complete scaffold with model, controller, and views:

```python
result = loco_bindings.generate_scaffold(
    project_path="/path/to/loco/project",
    name="post",
    fields={
        "title": "string",
        "content": "text",
        "published": "bool",
    },
    kind="api",  # Options: "api", "html", "htmx"
    with_timestamps=True
)
```

### Generate a Controller

```python
result = loco_bindings.generate_controller_view(
    project_path="/path/to/loco/project",
    name="users",
    actions=["index", "show", "create", "update", "delete"],
    kind="api"  # Options: "api", "html", "htmx"
)
```

## Field Types

The `fields` dictionary uses Loco's field type syntax:

| Type | Example | Description |
|------|---------|-------------|
| `string` | `"name": "string"` | String field |
| `string^` | `"email": "string^"` | Unique string |
| `string!` | `"title": "string!"` | Non-null string |
| `text` | `"content": "text"` | Text field |
| `int` | `"age": "int"` | Integer |
| `int^` | `"code": "int^"` | Unique integer |
| `float` | `"price": "float"` | Float |
| `bool` | `"active": "bool"` | Boolean |
| `ts` | `"published_at": "ts"` | Timestamp |
| `uuid` | `"id": "uuid"` | UUID |
| `json` | `"data": "json"` | JSON field |
| `array:string` | `"tags": "array:string"` | Array of strings |

See [Loco field types documentation](https://loco.rs/docs/the-app/models/) for more.

## Error Handling

```python
from loco_bindings import ValidationError, FileOperationError, ProjectError

try:
    result = loco_bindings.generate_model(
        project_path="/invalid/path",
        name="user",
        fields={"name": "string"},
        with_timestamps=True
    )
except FileOperationError as e:
    print(f"File error: {e}")
except ValidationError as e:
    print(f"Validation error: {e}")
except ProjectError as e:
    print(f"Project error: {e}")
```

## Architecture

This package is a **thin binding layer** (~250 lines) that directly exposes `loco-gen`'s functionality:

```
Python Application
    ↓
loco_bindings (thin wrapper)
    ↓
loco-gen (Rust core)
    ↓
File system
```

All generation logic, template processing, and validations are handled by `loco-gen` itself, ensuring complete compatibility with the Loco CLI.

## Development

### Build

```bash
maturin develop
```

### Build for release

```bash
maturin build --release
```

## License

MIT OR Apache-2.0

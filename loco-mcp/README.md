# Loco MCP Server

A Model Context Protocol (MCP) server that provides Loco framework code generation capabilities to AI assistants.

## Overview

The Loco MCP Server exposes Loco's powerful code generation features through the MCP protocol, allowing AI assistants to create Loco projects, generate models, scaffolds, controllers, and perform common development tasks.

## Features

### Core Generation Tools
- **`loco_create_project`**: Create new Loco projects with different templates
- **`loco_generate_model`**: Generate database models with migrations
- **`loco_generate_scaffold`**: Generate complete CRUD scaffolding
- **`loco_generate_controller_view`**: Generate controllers and views

### Utility Tools
- **`migrate_db`**: Execute database migrations with approval workflow
- **`rotate_keys`**: Rotate service account keys with security approvals
- **`clean_temp`**: Clean temporary files with operational approvals

## Installation

### Prerequisites
- Rust 1.70+
- Python 3.11+
- PyO3 for Python bindings

### Building from Source

1. **Clone the repository**:
   ```bash
   git clone https://github.com/loco-rs/loco.git
   cd loco/loco-mcp
   ```

2. **Build Python bindings**:
   ```bash
   cd loco-bindings
   cargo build --release
   ```

3. **Install Python dependencies**:
   ```bash
   cd ../loco-mcp-server
   pip install -e .
   ```

## Usage

### Running the MCP Server

```bash
python -m loco_mcp_server.src.server
```

The server runs on stdio and communicates with MCP-compatible clients.

### Available Tools

#### loco_create_project
Create a new Loco project from scratch.

**Parameters:**
- `project_name` (required): Project name in snake_case (e.g., "my_app", "blog_platform")
- `template_type` (required): Template type - "saas", "rest_api", or "lightweight"
- `destination_path` (required): Directory path where the project will be created
- `database_type` (optional): Database type - "postgres", "sqlite", or "mysql" (default: varies by template)
- `background_worker` (optional): Background worker - "redis", "postgres", "sqlite", or "none" (default: varies by template)
- `asset_serving` (optional): Enable static asset serving (default: varies by template)

**Example:**
```json
{
  "project_name": "my_blog",
  "template_type": "saas",
  "destination_path": "/path/to/projects/my_blog",
  "database_type": "postgres",
  "background_worker": "redis",
  "asset_serving": true
}
```

#### Template Types

**SaaS Template** (`saas`)
- Full-featured web application
- Default: PostgreSQL database, Redis background worker, local asset serving
- Includes: Models, controllers, views, authentication, background jobs

**REST API Template** (`rest_api`)
- API-only application
- Default: PostgreSQL database, no background worker, no asset serving
- Includes: Models, controllers, API documentation

**Lightweight Template** (`lightweight`)
- Minimal service/application
- Default: SQLite database, no background worker, no asset serving
- Includes: Basic structure and configuration

#### loco_generate_model
Generate a Loco model and migration file.

**Parameters:**
- `project_path` (required): Path to the Loco project root
- `name` (required): Model name in snake_case (e.g., "user", "blog_post")
- `fields` (required): Field definitions as key-value pairs
- `with_timestamps` (optional): Include created_at/updated_at fields (default: true)

**Example:**
```json
{
  "project_path": "/path/to/my_project",
  "name": "user",
  "fields": {
    "name": "string",
    "email": "string",
    "age": "integer"
  }
}
```

#### loco_generate_scaffold
Generate complete CRUD scaffolding including model, controller, and views.

**Parameters:**
- `project_path` (required): Path to the Loco project root
- `name` (required): Resource name in snake_case
- `fields` (required): Field definitions as key-value pairs
- `kind` (optional): Scaffold type - "api", "html", or "htmx" (default: "api")
- `with_timestamps` (optional): Include timestamp fields (default: true)

#### loco_generate_controller_view
Generate controller and views for an existing model.

**Parameters:**
- `project_path` (required): Path to the Loco project root
- `name` (required): Controller name (usually plural, snake_case)
- `actions` (optional): List of actions to generate (default: ["index", "show", "create", "update", "delete"])
- `kind` (optional): Controller type - "api", "html", or "htmx" (default: "api")

### Utility Tools

#### migrate_db
Execute database schema migration with approval workflow.

**Parameters:**
- `project_path` (required): Path to the Loco project root
- `approvals` (required): Approval chain - ["ops_lead", "security_officer"]
- `environment` (optional): Environment name (default: "development")
- `timeout_seconds` (optional): Timeout in seconds (default: 60)
- `dependencies` (optional): Required dependencies (default: ["postgres", "redis"])

#### rotate_keys
Rotate service account keys with security approvals.

**Parameters:**
- `project_path` (required): Path to the Loco project root
- `approvals` (required): Approval chain - ["security_officer", "cto"]
- `environment` (optional): Environment name (default: "production")
- `timeout_seconds` (optional): Timeout in seconds (default: 300)
- `dependencies` (optional): Required dependencies (default: ["kms"])

#### clean_temp
Clean temporary files with operational approvals.

**Parameters:**
- `project_path` (required): Path to the Loco project root
- `approvals` (required): Approval chain - ["ops_lead"]
- `environment` (optional): Environment name (default: "development")
- `timeout_seconds` (optional): Timeout in seconds (default: 60)
- `dependencies` (optional): Required dependencies (default: ["fs-local"])

## Configuration

The MCP server can be configured through environment variables:

- `LOCO_LOG_LEVEL`: Logging level (debug, info, warn, error) - default: info
- `LOCO_ENVIRONMENT`: Environment name (development, staging, production) - default: development

## Architecture

### Components

1. **Rust Bindings** (`loco-bindings/`): PyO3 bindings to loco-gen library
2. **MCP Server** (`loco-mcp-server/`): Python MCP server implementation
3. **Tools Layer**: Business logic for each tool
4. **Security Layer**: Approval workflows and audit logging

### Security Features

- **Approval Workflows**: Multi-level approval for sensitive operations
- **Audit Logging**: Comprehensive logging of all tool invocations
- **Input Validation**: Strict validation of all parameters
- **Error Handling**: Secure error reporting without sensitive information

## Development

### Running Tests

```bash
# Python tests
cd loco-mcp-server
pytest

# Rust tests
cd loco-bindings
cargo test
```

### Building for Distribution

```bash
# Build Rust bindings
cd loco-bindings
cargo build --release

# Build Python package
cd ../loco-mcp-server
python -m build
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Run the test suite
6. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Support

- **Documentation**: [docs/API.md](docs/API.md)
- **Issues**: [GitHub Issues](https://github.com/loco-rs/loco/issues)
- **Discord**: [Loco Discord](https://discord.gg/loco)

## Related Projects

- [Loco Framework](https://github.com/loco-rs/loco): The Rust web framework
- [Loco CLI](https://github.com/loco-rs/loco-cli): Command-line interface for Loco
- [Model Context Protocol](https://modelcontextprotocol.io): The protocol this server implements
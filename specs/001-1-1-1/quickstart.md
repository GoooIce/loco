# Loco-MCP Quick Start Guide

**Version**: 1.0
**Created**: 2025-10-03
**Target Audience**: Claude Code users working with loco-rs projects

## Prerequisites

- Python 3.11+
- Rust 1.70+
- uv (Python package manager)
- A loco-rs project (created with `cargo loco new`)

## Installation

### 1. Set up Python Environment

```bash
# Create virtual environment with uv
uv venv loco-mcp-env
source loco-mcp-env/bin/activate  # On Windows: loco-mcp-env\Scripts\activate

# Install Python dependencies
uv pip install claude-agent-py-sdk
```

### 2. Build and Install Rust Bindings

```bash
# Clone the loco-mcp repository
git clone <repository-url>
cd loco-mcp

# Build and install Rust bindings
cd loco-bindings
maturin develop

# Return to project root
cd ..
```

### 3. Install MCP Server

```bash
cd loco-mcp-server
uv pip install -e .
cd ..
```

## Configuration

### Claude Code Configuration

Add to your Claude Code configuration:

```json
{
  "mcpServers": {
    "loco-mcp": {
      "command": "python",
      "args": ["-m", "loco_mcp_server.server"],
      "cwd": "/path/to/your/loco-rs/project"
    }
  }
}
```

## Basic Usage

### Scenario 1: Create a Simple Model

**What you say to Claude**:
> "Create a loco model called Product with fields name (string), price (i32), and sku (string, unique)"

**Claude's internal process**:
1. Calls `loco.generate_model` tool
2. Passes: `model_name="product"`, `fields=["name:string", "price:i32", "sku:string:unique"]`
3. Receives structured response with created files

**Expected result**:
```rust
// src/models/product.rs
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "products")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(column_type = "String(Some(255))")]
    pub name: String,
    pub price: i32,
    #[sea_orm(column_type = "String(Some(100))", unique)]
    pub sku: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
```

### Scenario 2: Generate Complete CRUD Scaffolding

**What you say to Claude**:
> "Generate complete CRUD scaffolding for a User model with email (string, unique), name (string), and active (boolean) fields"

**Claude's internal process**:
1. Calls `loco.generate_scaffold` tool
2. Passes: `model_name="user"`, `fields=["email:string:unique", "name:string", "active:boolean"]`
3. Receives all generated files

**Expected result**:
- Model file: `src/models/user.rs`
- Migration file: `migration/src/m_YYYYMMDD_HHMMSS_create_users.rs`
- Controller file: `src/controllers/users.rs`
- View templates: `src/views/users/` (index, show, form)
- Route definitions added to `src/routes/mod.rs`

### Scenario 3: Add Controller to Existing Model

**What you say to Claude**:
> "I already have a Product model, now generate a controller and views for it"

**Claude's internal process**:
1. Calls `loco.generate_controller_view` tool
2. Passes: `model_name="product"`
3. Receives generated controller and view files

## Advanced Usage

### Custom Field Types and Constraints

```bash
# Supported field types
string, i32, i64, f32, f64, boolean, datetime, uuid, json, text

# Supported constraints
:unique, :primary_key, :nullable, :optional, :default<value>

# Examples
["title:string", "content:text", "published_at:datetime:nullable", "views:i32:default:0"]
```

### API-Only Scaffolding

**What you say to Claude**:
> "Generate API-only scaffolding for an API model with json_data (json) and created_at (datetime) fields"

**Claude's internal process**:
1. Calls `loco.generate_scaffold` with `api_only=true`
2. Generates controller without view templates

### Selective Controller Actions

**What you say to Claude**:
> "Generate a controller for the Category model with only index and show actions"

**Claude's internal process**:
1. Calls `loco.generate_controller_view` with `actions=["index", "show"]`
2. Generates only specified actions

## Error Handling

Common errors and their solutions:

### Validation Errors
```
Error: Invalid model name: '123invalid' must start with a letter
Solution: Use valid Rust identifier (snake_case, starts with letter)
```

### File Already Exists
```
Error: Model file already exists: src/models/user.rs
Solution: Use different model name or remove existing file
```

### Not a Loco Project
```
Error: Not a valid loco-rs project directory
Solution: Run commands from within a loco-rs project created with 'cargo loco new'
```

## Performance Tips

1. **Batch Operations**: Generate multiple models in sequence for better performance
2. **Project Structure**: Keep your loco-rs project well-organized for faster file operations
3. **Memory Usage**: Large models (>50 fields) may take slightly longer but should still be <10ms

## Integration with Development Workflow

### Git Integration
```bash
# After generating scaffolding
git add .
git commit -m "Add Product model scaffolding"
```

### Database Migration
```bash
# After generating models
cargo loco db migrate
```

### Testing
```bash
# Run tests to verify generated code
cargo test
```

## Troubleshooting

### Common Issues

1. **MCP Server Not Starting**
   - Check Python path in Claude configuration
   - Verify uv environment is activated
   - Check that dependencies are installed

2. **Generation Failures**
   - Ensure you're in a valid loco-rs project directory
   - Check that model names don't conflict with existing models
   - Verify field types and constraints are valid

3. **Performance Issues**
   - Check available disk space
   - Verify file permissions
   - Consider closing other resource-intensive applications

### Getting Help

- Check the generated files for compilation errors
- Review the MCP server logs for detailed error messages
- Ensure your loco-rs project follows standard conventions

## Next Steps

After generating scaffolding:

1. **Customize Models**: Add relationships, validations, and custom methods
2. **Implement Business Logic**: Add custom controller actions and validation
3. **Write Tests**: Add unit and integration tests for your models
4. **Configure Routes**: Customize routing and middleware as needed
5. **Database Setup**: Run migrations and configure database connections

## Example: Complete Workflow

```bash
# 1. Create new loco project
cargo loco new my_app
cd my_app

# 2. Start Loco-MCP server (configured in Claude)
# 3. Tell Claude: "Create a BlogPost model with title (string), content (text), published (boolean), and published_at (datetime)"
# 4. Tell Claude: "Generate complete scaffolding for BlogPost"
# 5. Run migrations
cargo loco db migrate
# 6. Start development server
cargo loco start
```

This workflow gives you a complete, working CRUD interface for your BlogPost model in under 30 seconds, with all files properly structured and following loco-rs conventions.
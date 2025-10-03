# Claude Code Integration Guide

This guide provides step-by-step instructions for integrating loco-mcp-server with Claude Code.

## Quick Setup

### 1. Install loco-mcp-server

```bash
# Clone and build
git clone https://github.com/loco-rs/loco.git
cd loco/loco-mcp
make install

# Or install using the script
chmod +x install.sh
./install.sh
```

### 2. Configure Claude Code

#### Option A: Using Claude Code Interface (Recommended)

1. **Open Claude Code**
2. **Go to Settings** (click the gear icon or use `Cmd+Shift+P`)
3. **Select "MCP Servers"**
4. **Click "Add Server"**
5. **Enter Configuration**:
   ```
   Name: loco
   Description: Loco framework code generation server
   Command: loco-mcp-server
   Arguments: (leave empty)
   ```
6. **Save and Restart Claude Code**

#### Option B: Manual Configuration File

Edit your Claude Code configuration file:

**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows**: `%APPDATA%\Claude\claude_desktop_config.json`
**Linux**: `~/.config/Claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "loco": {
      "command": "loco-mcp-server",
      "args": []
    }
  }
}
```

#### Option C: Virtual Environment (Development)

If you installed in a virtual environment:

```json
{
  "mcpServers": {
    "loco": {
      "command": "/path/to/your/venv/bin/python",
      "args": ["-m", "loco_mcp_server"]
    }
  }
}
```

### 3. Verify Installation

1. **Restart Claude Code** completely
2. **Check MCP Status**:
   - Go to Settings → MCP Servers
   - You should see "loco" server listed as "Connected"
3. **Test Tools**:
   - The loco tools should appear in your tools panel
   - Try a simple prompt: *"Create a test model with name and email fields"*

## Usage Examples

### Basic Model Generation

**Prompt**: "Create a user model with name, email, and password fields"

**Result**: Generates `src/models/user.rs` and migration file

### Complete CRUD Scaffold

**Prompt**: "Generate a complete scaffold for blog posts with title, content, and published status"

**Result**: Creates model, controller, views, and routes

### API-Only Development

**Prompt**: "Create an API-only scaffold for authentication tokens"

**Result**: Generates model and JSON endpoints (no views)

## Working with loco-rs Projects

### Project Requirements

The MCP server works with existing loco-rs projects. Make sure you're in a loco-rs project directory:

```bash
# Create new project
cargo install loco-cli
loco new my_app --starter saas
cd my_app

# Or navigate to existing project
cd path/to/existing/loco-project
```

### Project Detection

The server automatically detects loco-rs projects by looking for:
- `src/models/` directory
- `migration/` directory
- `Cargo.toml` with loco dependencies

### Natural Language Commands

Use natural language to generate code:

- **Models**: "Create a [model_name] model with [fields]"
- **Scaffolds**: "Generate a CRUD scaffold for [model_name]"
- **Controllers**: "Add a controller for existing [model_name]"
- **API**: "Create API-only endpoints for [model_name]"

## Field Definition Format

Fields use the format: `name:type[:constraint]`

### Types
- `string` - Short text (255 chars)
- `text` - Long text content
- `i32`/`i64` - Integers
- `f32`/`f64` - Float numbers
- `boolean` - True/false values
- `datetime` - Date and time
- `uuid` - Unique identifiers
- `json` - JSON data

### Constraints
- `:unique` - Unique values only
- `:nullable` - Allows null values
- `:default:value` - Default value
- `:primary_key` - Primary key field

## Troubleshooting

### Server Not Connected

**Symptoms**: loco tools don't appear in tools panel

**Solutions**:
1. Restart Claude Code completely
2. Check configuration file syntax
3. Verify loco-mcp-server is installed and accessible
4. Check system PATH includes the installation directory

### Command Not Found

**Symptoms**: "loco-mcp-server command not found"

**Solutions**:
```bash
# Check installation
which loco-mcp-server

# If not found, try reinstalling
cd loco-mcp
make install

# Or use virtual environment
source venv/bin/activate
python -m loco_mcp_server
```

### Project Not Detected

**Symptoms**: "Not a valid loco-rs project" error

**Solutions**:
1. Navigate to a loco-rs project directory
2. Verify project structure with `loco doctor`
3. Create new project: `loco new my_app`

### Performance Issues

**Symptoms**: Slow response times

**Solutions**:
1. Check system resources
2. Verify Rust installation is up to date
3. Restart Claude Code and MCP server

## Advanced Configuration

### Custom Project Path

If your loco project is in a different directory:

```json
{
  "mcpServers": {
    "loco": {
      "command": "loco-mcp-server",
      "args": ["--project-path", "/absolute/path/to/project"]
    }
  }
}
```

### Debug Mode

Enable debug logging:

```json
{
  "mcpServers": {
    "loco": {
      "command": "loco-mcp-server",
      "args": ["--debug", "--log-level", "DEBUG"]
    }
  }
}
```

### Custom Server Port

For remote development:

```json
{
  "mcpServers": {
    "loco": {
      "command": "loco-mcp-server",
      "args": ["--port", "8080", "--host", "0.0.0.0"]
    }
  }
}
```

## Best Practices

### Project Organization

1. **One project per directory**: Each loco-rs project should be in its own directory
2. **Use descriptive names**: Clear model and field names improve maintainability
3. **Plan relationships**: Think about data relationships before generating models

### Development Workflow

1. **Start simple**: Create basic models first
2. **Add relationships**: Connect models with foreign keys
3. **Generate scaffolds**: Create CRUD operations
4. **Customize templates**: Modify generated code as needed
5. **Test incrementally**: Verify each generation step

### Error Prevention

1. **Validate names**: Use proper snake_case naming
2. **Check types**: Use supported field types
3. **Verify constraints**: Apply appropriate constraints
4. **Test migrations**: Run migrations after generation

## Getting Help

- **Documentation**: [Loco Framework Docs](https://loco.rs)
- **Issues**: [GitHub Issues](https://github.com/loco-rs/loco/issues)
- **Community**: [Loco Discord](https://discord.gg/loco)
- **Examples**: See `examples/` directory for workflow examples

## Next Steps

After setting up the integration:

1. **Explore examples**: Check `examples/basic_workflow.md`
2. **Read API docs**: See `docs/API.md` for complete reference
3. **Run tests**: `make test` to verify installation
4. **Build applications**: Start creating loco-rs applications with Claude Code!
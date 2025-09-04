# MCP Server Implementation for Loco Framework

This document provides a comprehensive guide to implementing an MCP (Model Context Protocol) server using the Loco framework.

## Overview

The MCP server implementation integrates seamlessly with the Loco framework, leveraging its async patterns, background processing, and configuration management to provide a robust, production-ready MCP server.

## Features

- **JSON-RPC 2.0 Protocol**: Full compliance with MCP specification
- **HTTP/WebSocket Transport**: Support for both HTTP and WebSocket connections
- **Tool System**: Extensible tool system with BackgroundWorker integration
- **Configuration Management**: YAML-based configuration with environment support
- **Authentication**: Optional API key and JWT authentication
- **CORS Support**: Configurable CORS policies
- **Logging**: Integrated with Loco's logging system
- **Testing**: Comprehensive testing utilities

## Quick Start

### 1. Add Dependencies

```toml
[dependencies]
loco-rs = { version = "0.16", features = ["mcp"] }
```

### 2. Basic Configuration

Create a configuration file (`config/development.yaml`):

```yaml
mcp:
  enable: true
  host: "127.0.0.1"
  port: 8080
  timeout: 30
  max_request_size: 1048576  # 1MB
  enable_websocket: true
  cors:
    allow_origins:
      - "http://localhost:3000"
    allow_methods:
      - "POST"
      - "GET"
      - "OPTIONS"
    allow_headers:
      - "Content-Type"
      - "Authorization"
    allow_credentials: false
  auth:
    enable: false
    # api_key: "your-api-key"
    # jwt_secret: "your-jwt-secret"
    # token_expiration: 3600
```

### 3. Create MCP Server

```rust
use loco_rs::mcp::McpServer;
use loco_rs::config::Config;
use loco_rs::environment::Environment;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = Config::new(&Environment::Development)?;
    let environment = Environment::Development;

    // Create MCP server
    let mcp_server = McpServer::new(config, &environment).await?;

    // Start the server
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 8080));
    mcp_server.start(addr).await?;

    Ok(())
}
```

### 4. Integrate with Existing Loco App

```rust
use loco_rs::{
    app::{AppContext, Hooks},
    boot::{create_app, BootResult, StartMode},
    config::Config,
    controller::AppRoutes,
    environment::Environment,
    mcp::McpServer,
    prelude::*,
    task::Tasks,
    Result,
};

pub struct App;

#[async_trait]
impl Hooks for App {
    fn app_name() -> &'static str {
        "myapp"
    }

    fn routes(ctx: &AppContext) -> AppRoutes {
        AppRoutes::with_default_routes()
            .add_route(controllers::home::routes())
            // Add MCP router
            .add_route(ctx.mcp_server().unwrap().router())
    }

    async fn boot(mode: StartMode, environment: &Environment, config: Config) -> Result<BootResult> {
        create_app::<Self, Migrator>(mode, environment, config).await
    }

    async fn connect_workers(ctx: &AppContext, queue: &Queue) -> Result<()> {
        // Initialize MCP server
        if let Some(mcp_server) = ctx.mcp_server() {
            // Register custom tools
            mcp_server.register_tool("custom_tool", CustomTool).await?;
        }

        Ok(())
    }
}
```

## Tool System

### Built-in Tools

The MCP server includes built-in tools:

1. **Echo Tool**: Echo back input text with optional uppercase conversion
2. **Calculate Tool**: Perform basic mathematical calculations

### Creating Custom Tools

```rust
use loco_rs::mcp::{McpTool, CallToolResponse, Content, Tool};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

pub struct WeatherTool {
    api_key: String,
}

impl WeatherTool {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

#[async_trait]
impl McpTool for WeatherTool {
    fn tool_def(&self) -> Tool {
        Tool {
            name: "get_weather".to_string(),
            description: "Get current weather for a location".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "location": {
                        "type": "string",
                        "description": "City name or zip code"
                    },
                    "units": {
                        "type": "string",
                        "enum": ["metric", "imperial"],
                        "default": "metric"
                    }
                },
                "required": ["location"]
            }),
            output_schema: None,
            metadata: None,
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<CallToolResponse> {
        let location = args.get("location")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Message("Missing location".to_string()))?;

        let units = args.get("units")
            .and_then(|v| v.as_str())
            .unwrap_or("metric");

        // Call weather API
        let weather_data = self.fetch_weather(location, units).await?;

        Ok(CallToolResponse {
            content: vec![Content::Text {
                text: format!("Weather in {}: {}", location, weather_data),
            }],
            is_progress: None,
        })
    }

    fn validate_args(&self, args: &HashMap<String, Value>) -> Result<()> {
        if !args.contains_key("location") {
            return Err(Error::Message("Missing required 'location' argument".to_string()));
        }

        if let Some(units) = args.get("units") {
            if !units.is_string() {
                return Err(Error::Message("'units' must be a string".to_string()));
            }
            let units_str = units.as_str().unwrap();
            if units_str != "metric" && units_str != "imperial" {
                return Err(Error::Message("'units' must be 'metric' or 'imperial'".to_string()));
            }
        }

        Ok(())
    }
}

impl WeatherTool {
    async fn fetch_weather(&self, location: &str, units: &str) -> Result<String> {
        // Implement actual weather API call
        Ok(format!("22°C, sunny in {}", location))
    }
}
```

### Registering Custom Tools

```rust
// In your app's connect_workers method
async fn connect_workers(ctx: &AppContext, queue: &Queue) -> Result<()> {
    if let Some(mcp_server) = ctx.mcp_server() {
        // Register weather tool
        let weather_tool = WeatherTool::new("your-api-key".to_string());
        mcp_server.register_tool("weather", weather_tool).await?;

        // Register file system tool
        mcp_server.register_tool("file_system", FileSystemTool).await?;
    }

    Ok(())
}
```

## Transport Layer

### HTTP Transport

The MCP server supports HTTP POST requests:

```bash
curl -X POST http://localhost:8080/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": "1",
    "method": "tools/list",
    "params": {}
  }'
```

### WebSocket Transport

WebSocket support provides real-time communication:

```javascript
const ws = new WebSocket('ws://localhost:8080/mcp/ws');

ws.onopen = () => {
    ws.send(JSON.stringify({
        jsonrpc: "2.0",
        id: "1",
        method: "tools/call",
        params: {
            name: "echo",
            arguments: {
                text: "Hello, MCP!"
            }
        }
    }));
};

ws.onmessage = (event) => {
    const response = JSON.parse(event.data);
    console.log('Received:', response);
};
```

## Configuration

### MCP Configuration Options

```yaml
mcp:
  enable: true                    # Enable MCP server
  host: "127.0.0.1"              # Server host
  port: 8080                     # Server port
  path: "/mcp"                   # HTTP endpoint path
  ws_path: "/mcp/ws"              # WebSocket endpoint path
  timeout: 30                    # Tool execution timeout (seconds)
  max_request_size: 1048576     # Maximum request size (bytes)
  enable_websocket: true        # Enable WebSocket transport
  
  # CORS configuration
  cors:
    allow_origins:
      - "http://localhost:3000"
      - "https://yourdomain.com"
    allow_methods:
      - "POST"
      - "GET"
      - "OPTIONS"
    allow_headers:
      - "Content-Type"
      - "Authorization"
      - "X-Requested-With"
    allow_credentials: false
  
  # Authentication configuration
  auth:
    enable: false
    api_key: "your-api-key"      # API key authentication
    jwt_secret: "your-secret"    # JWT secret key
    token_expiration: 3600       # Token expiration (seconds)
```

### Environment Variables

You can override configuration using environment variables:

```bash
export MCP_ENABLE=true
export MCP_HOST=0.0.0.0
export MCP_PORT=8080
export MCP_AUTH_ENABLE=true
export MCP_AUTH_API_KEY=your-api-key
```

## Authentication

### API Key Authentication

```yaml
mcp:
  auth:
    enable: true
    api_key: "your-secret-api-key"
```

Then include the API key in requests:

```bash
curl -X POST http://localhost:8080/mcp \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-secret-api-key" \
  -d '...'
```

### JWT Authentication

```yaml
mcp:
  auth:
    enable: true
    jwt_secret: "your-jwt-secret"
    token_expiration: 3600
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use loco_rs::config::Config;
    use loco_rs::environment::Environment;

    #[tokio::test]
    async fn test_mcp_server_creation() {
        let config = Config::default();
        let environment = Environment::Development;

        let server = McpServer::new(config, &environment).await;
        assert!(server.is_ok());
    }

    #[tokio::test]
    async fn test_tool_execution() {
        let config = Config::default();
        let environment = Environment::Development;

        let mut server = McpServer::new(config, &environment).await.unwrap();
        
        // Test echo tool
        let response = server.protocol_handler.handle_request(McpRequest::new(
            "tools/call".to_string(),
            Some(serde_json::json!({
                "name": "echo",
                "arguments": {
                    "text": "Hello, World!"
                }
            })),
        )).await;

        assert!(response.result.is_some());
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_mcp_http_endpoint() {
    use axum_test::TestServer;
    
    let config = Config::default();
    let environment = Environment::Development;
    let mcp_server = McpServer::new(config, &environment).await.unwrap();
    
    let server = TestServer::new(mcp_server.router()).unwrap();
    
    let response = server
        .post("/mcp")
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": "1",
            "method": "tools/list",
            "params": {}
        }))
        .await;

    response.assert_status_ok();
}
```

## Performance Considerations

### Tool Execution Timeout

Each tool has a configurable timeout. Set appropriate timeouts for your tools:

```rust
impl McpTool for MySlowTool {
    fn timeout(&self) -> Duration {
        Duration::from_secs(120) // 2 minutes for slow operations
    }
}
```

### Concurrent Tool Execution

The MCP server handles concurrent tool execution using Tokio's async runtime:

```rust
// Multiple tools can execute concurrently
let tool1 = server.execute_tool("tool1", args1);
let tool2 = server.execute_tool("tool2", args2);
let tool3 = server.execute_tool("tool3", args3);

let (result1, result2, result3) = tokio::join!(tool1, tool2, tool3);
```

### Memory Management

- Use `Arc` for sharing tool instances
- Implement proper cleanup in tool destructors
- Monitor memory usage for long-running tools

## Security Best Practices

### Input Validation

Always validate tool inputs:

```rust
impl McpTool for MyTool {
    fn validate_args(&self, args: &HashMap<String, Value>) -> Result<()> {
        if !args.contains_key("required_param") {
            return Err(Error::Message("Missing required parameter".to_string()));
        }
        
        // Validate input types and ranges
        if let Some(param) = args.get("numeric_param") {
            if !param.is_number() {
                return Err(Error::Message("Parameter must be numeric".to_string()));
            }
        }
        
        Ok(())
    }
}
```

### Tool Isolation

- Use separate processes for potentially dangerous tools
- Implement resource limits
- Monitor tool execution time and memory usage

### Authentication

- Always use authentication in production
- Use HTTPS for all communications
- Implement proper token validation and expiration

## Deployment

### Docker Deployment

```dockerfile
FROM rust:1.70 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/myapp /usr/local/bin/

EXPOSE 8080

CMD ["myapp", "mcp", "start"]
```

### Systemd Service

```ini
[Unit]
Description=Loco MCP Server
After=network.target

[Service]
Type=simple
User=loco
WorkingDirectory=/opt/loco
ExecStart=/opt/loco/myapp mcp start
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

## Monitoring and Logging

### Structured Logging

The MCP server integrates with Loco's logging system:

```rust
use tracing::{info, warn, error};

impl McpTool for MyTool {
    async fn execute(&self, args: HashMap<String, Value>) -> Result<CallToolResponse> {
        info!("Executing tool with args: {:?}", args);
        
        match self.do_work(args).await {
            Ok(result) => {
                info!("Tool execution completed successfully");
                Ok(result)
            }
            Err(e) => {
                error!("Tool execution failed: {}", e);
                Err(e)
            }
        }
    }
}
```

### Metrics

Add metrics collection to monitor tool usage:

```rust
use std::sync::atomic::{AtomicU64, Ordering};

static TOOL_EXECUTION_COUNT: AtomicU64 = AtomicU64::new(0);
static TOOL_EXECUTION_TIME: AtomicU64 = AtomicU64::new(0);

impl McpTool for MyTool {
    async fn execute(&self, args: HashMap<String, Value>) -> Result<CallToolResponse> {
        let start_time = std::time::Instant::now();
        
        TOOL_EXECUTION_COUNT.fetch_add(1, Ordering::Relaxed);
        
        let result = self.do_work(args).await;
        
        let execution_time = start_time.elapsed().as_millis() as u64;
        TOOL_EXECUTION_TIME.fetch_add(execution_time, Ordering::Relaxed);
        
        result
    }
}
```

## Troubleshooting

### Common Issues

1. **Tool Timeout**: Increase timeout in configuration or optimize tool execution
2. **Memory Usage**: Monitor tool memory usage and implement proper cleanup
3. **Connection Issues**: Check firewall and network configuration
4. **Authentication Problems**: Verify API keys and JWT configuration

### Debug Mode

Enable debug logging:

```yaml
logger:
  enable: true
  level: debug
  pretty_backtrace: true
```

### Health Checks

The MCP server includes health check endpoints:

```bash
curl http://localhost:8080/ping
```

## Conclusion

The Loco MCP server implementation provides a robust, production-ready foundation for building MCP-compatible applications. By leveraging Loco's async patterns, background processing, and configuration management, you can quickly develop and deploy MCP servers with minimal boilerplate code.

For more information about the MCP specification, visit the official documentation.

For issues or feature requests, please open an issue on the Loco framework GitHub repository.
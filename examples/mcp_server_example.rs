//! MCP Server Example Application
//!
//! This example demonstrates how to create and run an MCP server using the Loco framework.

use loco_rs::mcp::{McpServer, McpTool};
use loco_rs::config::Config;
use loco_rs::environment::Environment;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    println!("🚀 Starting Loco MCP Server Example");

    // Load configuration
    let config = Config::new(&Environment::Development)?;
    let environment = Environment::Development;

    // Create MCP server
    let mut mcp_server = McpServer::new(config, &environment).await?;

    // Register custom tools
    mcp_server.register_tool("greet", GreetTool).await?;
    mcp_server.register_tool("file_info", FileInfoTool).await?;

    println!("✅ MCP Server created successfully");
    println!("📋 Available tools:");
    
    // List available tools
    let server_info = mcp_server.server_info();
    println!("   - Server: {}", server_info.name);
    println!("   - Version: {}", server_info.version);
    println!("   - Protocol: {}", server_info.protocol_version);

    // Start the server
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("\n🌐 Starting MCP server on: {}", addr);
    println!("📡 HTTP endpoint: http://localhost:8080/mcp");
    println!("🔌 WebSocket endpoint: ws://localhost:8080/mcp/ws");
    println!("\nPress Ctrl+C to stop the server");

    mcp_server.start(addr).await?;

    Ok(())
}

/// Custom greet tool
pub struct GreetTool;

#[async_trait]
impl McpTool for GreetTool {
    fn tool_def(&self) -> loco_rs::mcp::Tool {
        loco_rs::mcp::Tool {
            name: "greet".to_string(),
            description: "Greet someone with a personalized message".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Name of the person to greet"
                    },
                    "style": {
                        "type": "string",
                        "enum": ["formal", "casual", "friendly"],
                        "default": "friendly"
                    }
                },
                "required": ["name"]
            }),
            output_schema: None,
            metadata: None,
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<loco_rs::mcp::CallToolResponse, loco_rs::Error> {
        let name = args.get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| loco_rs::Error::Message("Missing 'name' argument".to_string()))?;

        let style = args.get("style")
            .and_then(|v| v.as_str())
            .unwrap_or("friendly");

        let greeting = match style {
            "formal" => format!("Good day, {}. How may I assist you today?", name),
            "casual" => format!("Hey {}! What's up?", name),
            "friendly" => format!("Hello {}! Nice to meet you! 😊", name),
            _ => format!("Hello {}!", name),
        };

        Ok(loco_rs::mcp::CallToolResponse {
            content: vec![loco_rs::mcp::Content::Text { text: greeting }],
            is_progress: None,
        })
    }

    fn validate_args(&self, args: &HashMap<String, Value>) -> Result<(), loco_rs::Error> {
        if !args.contains_key("name") {
            return Err(loco_rs::Error::Message("Missing required 'name' argument".to_string()));
        }

        if let Some(name_value) = args.get("name") {
            if !name_value.is_string() {
                return Err(loco_rs::Error::Message("'name' must be a string".to_string()));
            }
        }

        if let Some(style_value) = args.get("style") {
            if !style_value.is_string() {
                return Err(loco_rs::Error::Message("'style' must be a string".to_string()));
            }
        }

        Ok(())
    }
}

/// Custom file info tool
pub struct FileInfoTool;

#[async_trait]
impl McpTool for FileInfoTool {
    fn tool_def(&self) -> loco_rs::mcp::Tool {
        loco_rs::mcp::Tool {
            name: "file_info".to_string(),
            description: "Get information about a file".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file"
                    },
                    "include_content": {
                        "type": "boolean",
                        "description": "Include file content in response",
                        "default": false
                    }
                },
                "required": ["path"]
            }),
            output_schema: None,
            metadata: None,
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<loco_rs::mcp::CallToolResponse, loco_rs::Error> {
        let path = args.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| loco_rs::Error::Message("Missing 'path' argument".to_string()))?;

        let include_content = args.get("include_content")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Get file metadata
        let metadata = std::fs::metadata(path).map_err(|e| {
            loco_rs::Error::Message(format!("Failed to read file metadata: {}", e))
        })?;

        let mut info = format!("File: {}\n", path);
        info.push_str(&format!("Size: {} bytes\n", metadata.len()));
        info.push_str(&format!("Type: {}\n", if metadata.is_file() { "File" } else { "Directory" }));
        info.push_str(&format!("Read-only: {}\n", metadata.permissions().readonly()));
        
        if metadata.is_file() && include_content {
            match std::fs::read_to_string(path) {
                Ok(content) => {
                    info.push_str(&format!("\nContent:\n```\n{}\n```", content));
                }
                Err(_) => {
                    info.push_str("\nContent: [Binary or unreadable file]");
                }
            }
        }

        Ok(loco_rs::mcp::CallToolResponse {
            content: vec![loco_rs::mcp::Content::Text { text: info }],
            is_progress: None,
        })
    }

    fn validate_args(&self, args: &HashMap<String, Value>) -> Result<(), loco_rs::Error> {
        if !args.contains_key("path") {
            return Err(loco_rs::Error::Message("Missing required 'path' argument".to_string()));
        }

        if let Some(path_value) = args.get("path") {
            if !path_value.is_string() {
                return Err(loco_rs::Error::Message("'path' must be a string".to_string()));
            }
        }

        if let Some(include_content_value) = args.get("include_content") {
            if !include_content_value.is_boolean() {
                return Err(loco_rs::Error::Message("'include_content' must be a boolean".to_string()));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_greet_tool() {
        let tool = GreetTool;
        
        let mut args = HashMap::new();
        args.insert("name".to_string(), serde_json::Value::String("Alice".to_string()));
        
        let response = tool.execute(args).await.unwrap();
        assert_eq!(response.content.len(), 1);
        
        if let loco_rs::mcp::Content::Text { text } = &response.content[0] {
            assert!(text.contains("Alice"));
        }
    }

    #[tokio::test]
    async fn test_file_info_tool() {
        let tool = FileInfoTool;
        
        let mut args = HashMap::new();
        args.insert("path".to_string(), serde_json::Value::String("Cargo.toml".to_string()));
        
        let response = tool.execute(args).await.unwrap();
        assert_eq!(response.content.len(), 1);
        
        if let loco_rs::mcp::Content::Text { text } = &response.content[0] {
            assert!(text.contains("Cargo.toml"));
        }
    }
}
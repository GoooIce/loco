//! MCP protocol types and structures
//!
//! This module defines the core types and structures used by the MCP server,
//! following the Model Context Protocol specification.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// MCP Request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    /// JSON-RPC 2.0 request ID
    pub id: RequestId,
    /// JSON-RPC 2.0 method name
    pub method: String,
    /// Method parameters
    pub params: Option<serde_json::Value>,
    /// Request metadata
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// MCP Response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    /// JSON-RPC 2.0 response ID
    pub id: RequestId,
    /// Response result
    pub result: Option<serde_json::Value>,
    /// Error response
    pub error: Option<McpError>,
    /// Response metadata
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// MCP Error structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    /// Error code
    pub code: i32,
    /// Error message
    pub message: String,
    /// Error data
    pub data: Option<serde_json::Value>,
}

/// Request ID type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestId {
    /// String ID
    String(String),
    /// Number ID
    Number(u64),
    /// Null ID (for notifications)
    Null,
}

/// MCP Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// Input schema
    pub input_schema: serde_json::Value,
    /// Output schema
    pub output_schema: Option<serde_json::Value>,
    /// Tool metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// MCP Resource definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    /// Resource URI
    pub uri: String,
    /// Resource name
    pub name: String,
    /// Resource description
    pub description: Option<String>,
    /// Resource MIME type
    pub mime_type: Option<String>,
    /// Resource metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// MCP Prompt definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    /// Prompt name
    pub name: String,
    /// Prompt description
    pub description: String,
    /// Prompt arguments
    pub arguments: Vec<PromptArgument>,
    /// Prompt metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Prompt argument definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptArgument {
    /// Argument name
    pub name: String,
    /// Argument description
    pub description: String,
    /// Whether the argument is required
    pub required: bool,
    /// Argument default value
    pub default: Option<serde_json::Value>,
}

/// MCP Server capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    /// Available tools
    pub tools: Option<ToolCapabilities>,
    /// Available resources
    pub resources: Option<ResourceCapabilities>,
    /// Available prompts
    pub prompts: Option<PromptCapabilities>,
    /// Logging capabilities
    pub logging: Option<LoggingCapabilities>,
}

/// Tool capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCapabilities {
    /// Whether tools are supported
    pub list_changed: Option<bool>,
}

/// Resource capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceCapabilities {
    /// Whether resources are supported
    pub subscribe: Option<bool>,
    /// Whether resource lists can change
    pub list_changed: Option<bool>,
}

/// Prompt capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptCapabilities {
    /// Whether prompts are supported
    pub list_changed: Option<bool>,
}

/// Logging capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingCapabilities {
    /// Logging level
    pub level: Option<String>,
}

/// MCP Server info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    /// Server name
    pub name: String,
    /// Server version
    pub version: String,
    /// Server protocol version
    pub protocol_version: String,
    /// Server capabilities
    pub capabilities: ServerCapabilities,
    /// Server metadata
    pub server_info: Option<HashMap<String, serde_json::Value>>,
}

/// Initialize request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeRequest {
    /// Protocol version
    pub protocol_version: String,
    /// Client capabilities
    pub capabilities: ClientCapabilities,
    /// Client info
    pub client_info: ClientInfo,
}

/// Client capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCapabilities {
    /// Experimental features
    pub experimental: Option<HashMap<String, serde_json::Value>>,
}

/// Client info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    /// Client name
    pub name: String,
    /// Client version
    pub version: String,
}

/// Initialize response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResponse {
    /// Protocol version
    pub protocol_version: String,
    /// Server capabilities
    pub capabilities: ServerCapabilities,
    /// Server info
    pub server_info: ServerInfo,
}

/// Tool call request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolRequest {
    /// Tool name
    pub name: String,
    /// Tool arguments
    pub arguments: Option<HashMap<String, serde_json::Value>>,
}

/// Tool call response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolResponse {
    /// Tool result content
    pub content: Vec<Content>,
    /// Whether the tool call is a progress update
    pub is_progress: Option<bool>,
}

/// Content block
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Content {
    /// Text content
    Text { text: String },
    /// Image content
    Image { data: String, mime_type: String },
    /// Embedded resource
    Resource { resource: Resource, text: Option<String> },
}

/// Resource list request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResourcesRequest {
    /// Cursor for pagination
    pub cursor: Option<String>,
}

/// Resource list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResourcesResponse {
    /// List of resources
    pub resources: Vec<Resource>,
    /// Next cursor for pagination
    pub next_cursor: Option<String>,
}

/// Tool list request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListToolsRequest {
    /// Cursor for pagination
    pub cursor: Option<String>,
}

/// Tool list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListToolsResponse {
    /// List of tools
    pub tools: Vec<Tool>,
    /// Next cursor for pagination
    pub next_cursor: Option<String>,
}

/// Prompt list request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPromptsRequest {
    /// Cursor for pagination
    pub cursor: Option<String>,
}

/// Prompt list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPromptsResponse {
    /// List of prompts
    pub prompts: Vec<Prompt>,
    /// Next cursor for pagination
    pub next_cursor: Option<String>,
}

/// Get prompt request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPromptRequest {
    /// Prompt name
    pub name: String,
    /// Prompt arguments
    pub arguments: Option<HashMap<String, serde_json::Value>>,
}

/// Get prompt response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPromptResponse {
    /// Prompt description
    pub description: String,
    /// Prompt messages
    pub messages: Vec<PromptMessage>,
}

/// Prompt message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptMessage {
    /// Message role
    pub role: String,
    /// Message content
    pub content: Content,
}

impl McpError {
    /// Create a new MCP error
    pub fn new(code: i32, message: String) -> Self {
        Self {
            code,
            message,
            data: None,
        }
    }

    /// Create a new MCP error with data
    pub fn with_data(code: i32, message: String, data: serde_json::Value) -> Self {
        Self {
            code,
            message,
            data: Some(data),
        }
    }

    /// Parse error (-32700)
    pub fn parse_error(message: String) -> Self {
        Self::new(-32700, message)
    }

    /// Invalid request (-32600)
    pub fn invalid_request(message: String) -> Self {
        Self::new(-32600, message)
    }

    /// Method not found (-32601)
    pub fn method_not_found(message: String) -> Self {
        Self::new(-32601, message)
    }

    /// Invalid params (-32602)
    pub fn invalid_params(message: String) -> Self {
        Self::new(-32602, message)
    }

    /// Internal error (-32603)
    pub fn internal_error(message: String) -> Self {
        Self::new(-32603, message)
    }
}

impl McpRequest {
    /// Create a new MCP request
    pub fn new(method: String, params: Option<serde_json::Value>) -> Self {
        Self {
            id: RequestId::String(Uuid::new_v4().to_string()),
            method,
            params,
            meta: None,
        }
    }

    /// Create a new notification (no response expected)
    pub fn notification(method: String, params: Option<serde_json::Value>) -> Self {
        Self {
            id: RequestId::Null,
            method,
            params,
            meta: None,
        }
    }

    /// Check if this is a notification
    pub fn is_notification(&self) -> bool {
        matches!(self.id, RequestId::Null)
    }
}

impl McpResponse {
    /// Create a successful response
    pub fn success(id: RequestId, result: serde_json::Value) -> Self {
        Self {
            id,
            result: Some(result),
            error: None,
            meta: None,
        }
    }

    /// Create an error response
    pub fn error(id: RequestId, error: McpError) -> Self {
        Self {
            id,
            result: None,
            error: Some(error),
            meta: None,
        }
    }
}
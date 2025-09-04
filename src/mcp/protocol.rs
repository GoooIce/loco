//! JSON-RPC 2.0 Protocol Handler for MCP
//!
//! This module implements the JSON-RPC 2.0 protocol handler for MCP requests,
//! providing request parsing, validation, and response generation.

use crate::{
    app::AppContext,
    errors::Error,
    mcp::{
        tools::ToolRegistry,
        types::{InitializeRequest, InitializeResponse, McpError, McpRequest, McpResponse},
    },
    Result,
};
use serde_json::Value;

/// MCP Protocol Handler
#[derive(Clone)]
pub struct ProtocolHandler {
    app_context: AppContext,
    server_info: crate::mcp::types::ServerInfo,
    tool_registry: std::sync::Arc<tokio::sync::RwLock<ToolRegistry>>,
}

impl ProtocolHandler {
    /// Create a new protocol handler
    pub fn new(
        app_context: AppContext,
        server_info: crate::mcp::types::ServerInfo,
        tool_registry: std::sync::Arc<tokio::sync::RwLock<ToolRegistry>>,
    ) -> Self {
        Self {
            app_context,
            server_info,
            tool_registry,
        }
    }

    /// Handle an incoming MCP request
    pub async fn handle_request(&self, request: McpRequest) -> McpResponse {
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request).await,
            "initialized" => self.handle_initialized(request).await,
            "tools/list" => self.handle_tools_list(request).await,
            "tools/call" => self.handle_tools_call(request).await,
            "resources/list" => self.handle_resources_list(request).await,
            "resources/read" => self.handle_resources_read(request).await,
            "prompts/list" => self.handle_prompts_list(request).await,
            "prompts/get" => self.handle_prompts_get(request).await,
            "ping" => self.handle_ping(request).await,
            _ => McpResponse::error(
                request.id,
                McpError::method_not_found(format!("Method '{}' not found", request.method)),
            ),
        }
    }

    /// Handle initialize request
    async fn handle_initialize(&self, request: McpRequest) -> McpResponse {
        let init_request: Result<InitializeRequest> = request
            .params
            .ok_or_else(|| Error::Message("Missing parameters".to_string()))
            .and_then(|params| serde_json::from_value(params).map_err(Into::into));

        match init_request {
            Ok(init_req) => {
                // Validate protocol version
                if init_req.protocol_version != "2024-11-05" {
                    return McpResponse::error(
                        request.id,
                        McpError::invalid_request("Unsupported protocol version".to_string()),
                    );
                }

                let response = InitializeResponse {
                    protocol_version: "2024-11-05".to_string(),
                    capabilities: self.server_info.capabilities.clone(),
                    server_info: self.server_info.clone(),
                };

                McpResponse::success(request.id, serde_json::to_value(response).unwrap())
            }
            Err(e) => McpResponse::error(
                request.id,
                McpError::invalid_params(format!("Invalid initialize request: {}", e)),
            ),
        }
    }

    /// Handle initialized notification
    async fn handle_initialized(&self, request: McpRequest) -> McpResponse {
        // This is a notification, so we don't need to return anything meaningful
        McpResponse::success(request.id, Value::Object(serde_json::Map::new()))
    }

    /// Handle tools/list request
    async fn handle_tools_list(&self, request: McpRequest) -> McpResponse {
        let list_request: Result<crate::mcp::types::ListToolsRequest> = request
            .params
            .map(|params| serde_json::from_value(params))
            .unwrap_or_else(|| Ok(crate::mcp::types::ListToolsRequest { cursor: None }))
            .map_err(|e| Error::Message(format!("Invalid list tools request: {}", e)));

        match list_request {
            Ok(_list_req) => {
                // Get tools from the registry
                let registry = self.tool_registry.read().await;
                let tools = registry.list_tools();
                
                let response = crate::mcp::types::ListToolsResponse {
                    tools,
                    next_cursor: None,
                };

                McpResponse::success(request.id, serde_json::to_value(response).unwrap())
            }
            Err(e) => McpResponse::error(
                request.id,
                McpError::invalid_params(e.to_string()),
            ),
        }
    }

    /// Handle tools/call request
    async fn handle_tools_call(&self, request: McpRequest) -> McpResponse {
        let call_request: Result<crate::mcp::types::CallToolRequest> = request
            .params
            .ok_or_else(|| Error::Message("Missing parameters".to_string()))
            .and_then(|params| serde_json::from_value(params).map_err(Into::into));

        match call_request {
            Ok(call_req) => {
                let registry = self.tool_registry.read().await;
                let args = call_req.arguments.unwrap_or_default();
                
                match registry.execute_tool(&call_req.name, args).await {
                    Ok(result) => McpResponse::success(request.id, serde_json::to_value(result).unwrap()),
                    Err(e) => McpResponse::error(
                        request.id,
                        McpError::internal_error(format!("Tool execution failed: {}", e)),
                    ),
                }
            }
            Err(e) => McpResponse::error(
                request.id,
                McpError::invalid_params(format!("Invalid call tool request: {}", e)),
            ),
        }
    }

    /// Handle resources/list request
    async fn handle_resources_list(&self, request: McpRequest) -> McpResponse {
        let list_request: Result<crate::mcp::types::ListResourcesRequest> = request
            .params
            .map(|params| serde_json::from_value(params))
            .unwrap_or_else(|| Ok(crate::mcp::types::ListResourcesRequest { cursor: None }))
            .map_err(|e| Error::Message(format!("Invalid list resources request: {}", e)));

        match list_request {
            Ok(_list_req) => {
                let resources = self.get_available_resources().await;
                
                let response = crate::mcp::types::ListResourcesResponse {
                    resources,
                    next_cursor: None,
                };

                McpResponse::success(request.id, serde_json::to_value(response).unwrap())
            }
            Err(e) => McpResponse::error(
                request.id,
                McpError::invalid_params(e.to_string()),
            ),
        }
    }

    /// Handle resources/read request
    async fn handle_resources_read(&self, request: McpRequest) -> McpResponse {
        // For now, return a placeholder response
        McpResponse::error(
            request.id,
            McpError::method_not_found("Resource reading not implemented".to_string()),
        )
    }

    /// Handle prompts/list request
    async fn handle_prompts_list(&self, request: McpRequest) -> McpResponse {
        let list_request: Result<crate::mcp::types::ListPromptsRequest> = request
            .params
            .map(|params| serde_json::from_value(params))
            .unwrap_or_else(|| Ok(crate::mcp::types::ListPromptsRequest { cursor: None }))
            .map_err(|e| Error::Message(format!("Invalid list prompts request: {}", e)));

        match list_request {
            Ok(_list_req) => {
                let prompts = self.get_available_prompts().await;
                
                let response = crate::mcp::types::ListPromptsResponse {
                    prompts,
                    next_cursor: None,
                };

                McpResponse::success(request.id, serde_json::to_value(response).unwrap())
            }
            Err(e) => McpResponse::error(
                request.id,
                McpError::invalid_params(e.to_string()),
            ),
        }
    }

    /// Handle prompts/get request
    async fn handle_prompts_get(&self, request: McpRequest) -> McpResponse {
        // For now, return a placeholder response
        McpResponse::error(
            request.id,
            McpError::method_not_found("Prompt generation not implemented".to_string()),
        )
    }

    /// Handle ping request
    async fn handle_ping(&self, request: McpRequest) -> McpResponse {
        McpResponse::success(request.id, Value::Object(serde_json::Map::new()))
    }

    /// Get available resources from the app context
    async fn get_available_resources(&self) -> Vec<crate::mcp::types::Resource> {
        // This would typically query the resource registry
        // For now, return an empty list
        vec![]
    }

    /// Get available prompts from the app context
    async fn get_available_prompts(&self) -> Vec<crate::mcp::types::Prompt> {
        // This would typically query the prompt registry
        // For now, return an empty list
        vec![]
    }
}

/// Parse JSON-RPC request from bytes
pub fn parse_request(data: &[u8]) -> Result<McpRequest> {
    let request: McpRequest = serde_json::from_slice(data)
        .map_err(|e| Error::Message(format!("Failed to parse request: {}", e)))?;
    Ok(request)
}

/// Serialize MCP response to bytes
pub fn serialize_response(response: &McpResponse) -> Result<Vec<u8>> {
    let bytes = serde_json::to_vec(response)
        .map_err(|e| Error::Message(format!("Failed to serialize response: {}", e)))?;
    Ok(bytes)
}
//! Transport layer for MCP server
//!
//! This module provides HTTP and WebSocket transport implementations
//! for the MCP server using Axum.

use crate::{
    app::AppContext,
    errors::Error,
    mcp::{protocol::ProtocolHandler, types::McpRequest, McpResponse},
    Result,
};
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;

/// MCP Transport layer
pub struct McpTransport {
    app_context: AppContext,
    protocol_handler: Arc<ProtocolHandler>,
}

impl McpTransport {
    /// Create a new MCP transport
    pub fn new(app_context: AppContext, protocol_handler: ProtocolHandler) -> Self {
        Self {
            app_context,
            protocol_handler: Arc::new(protocol_handler),
        }
    }

    /// Create Axum router for MCP endpoints
    pub fn create_router(&self) -> Router {
        let protocol_handler = self.protocol_handler.clone();
        
        Router::new()
            .route("/mcp", post(handle_mcp_request))
            .route("/mcp/ws", get(handle_websocket_upgrade))
            .with_state(McpTransportState {
                protocol_handler,
                app_context: self.app_context.clone(),
            })
    }
}

/// State for MCP transport
#[derive(Clone)]
struct McpTransportState {
    protocol_handler: Arc<ProtocolHandler>,
    app_context: AppContext,
}

/// Handle HTTP MCP request
async fn handle_mcp_request(
    State(state): State<McpTransportState>,
    Json(request): Json<McpRequest>,
) -> Result<Json<McpResponse>, StatusCode> {
    match state.protocol_handler.handle_request(request).await {
        response => Ok(Json(response)),
    }
}

/// Handle WebSocket upgrade for MCP - placeholder for now
async fn handle_websocket_upgrade(
    State(_state): State<McpTransportState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // For now, return a message indicating WebSocket is not implemented
    Ok(Json(serde_json::json!({
        "error": "WebSocket support not yet implemented",
        "message": "Use HTTP endpoint instead"
    })))
}

// TODO: Implement WebSocket connection handling
// This function is commented out for now to fix compilation issues
/*
async fn handle_websocket_connection(
    mut socket: WebSocket,
    state: McpTransportState,
) {
    let (mut sender, mut receiver) = socket.split();

    // Handle incoming messages
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(axum::extract::ws::Message::Text(text)) => {
                // Parse MCP request
                match serde_json::from_str::<McpRequest>(&text) {
                    Ok(request) => {
                        // Handle the request
                        let response = state.protocol_handler.handle_request(request).await;
                        
                        // Send response back
                        match serde_json::to_string(&response) {
                            Ok(response_text) => {
                                if let Err(e) = sender
                                    .send(axum::extract::ws::Message::Text(response_text))
                                    .await
                                {
                                    tracing::error!("Failed to send WebSocket response: {}", e);
                                    break;
                                }
                            }
                            Err(e) => {
                                tracing::error!("Failed to serialize response: {}", e);
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to parse WebSocket message: {}", e);
                        let error_response = McpResponse::error(
                            crate::mcp::types::RequestId::Null,
                            crate::mcp::types::McpError::parse_error(format!("Invalid JSON: {}", e)),
                        );
                        
                        if let Ok(response_text) = serde_json::to_string(&error_response) {
                            let _ = sender
                                .send(axum::extract::ws::Message::Text(response_text))
                                .await;
                        }
                    }
                }
            }
            Ok(axum::extract::ws::Message::Binary(data)) => {
                // Handle binary messages if needed
                tracing::warn!("Binary WebSocket message received but not supported");
            }
            Ok(axum::extract::ws::Message::Close(_)) => {
                // Connection closed
                break;
            }
            Err(e) => {
                tracing::error!("WebSocket error: {}", e);
                break;
            }
        }
    }
}
*/

/// HTTP client for MCP
pub struct McpHttpClient {
    base_url: String,
    client: reqwest::Client,
}

impl McpHttpClient {
    /// Create a new MCP HTTP client
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }

    /// Send an MCP request via HTTP
    pub async fn send_request(&self, request: McpRequest) -> Result<McpResponse> {
        let url = format!("{}/mcp", self.base_url);
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::Message(format!("Failed to send request: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::Message(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        let mcp_response: McpResponse = response
            .json()
            .await
            .map_err(|e| Error::Message(format!("Failed to parse response: {}", e)))?;

        Ok(mcp_response)
    }
}

/// WebSocket client for MCP
pub struct McpWebSocketClient {
    url: String,
}

impl McpWebSocketClient {
    /// Create a new MCP WebSocket client
    pub fn new(url: String) -> Self {
        Self { url }
    }

    /// Connect to MCP server via WebSocket
    pub async fn connect(&self) -> Result<McpWebSocketConnection> {
        let (ws_stream, _) = tokio_tungstenite::connect_async(&self.url)
            .await
            .map_err(|e| Error::Message(format!("Failed to connect to WebSocket: {}", e)))?;

        Ok(McpWebSocketConnection { ws_stream })
    }
}

/// MCP WebSocket connection
pub struct McpWebSocketConnection {
    ws_stream: tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
}

impl McpWebSocketConnection {
    /// Send an MCP request via WebSocket
    pub async fn send_request(&mut self, request: McpRequest) -> Result<McpResponse> {
        let request_text = serde_json::to_string(&request)
            .map_err(|e| Error::Message(format!("Failed to serialize request: {}", e)))?;

        self.ws_stream
            .send(tokio_tungstenite::tungstenite::Message::Text(request_text))
            .await
            .map_err(|e| Error::Message(format!("Failed to send WebSocket message: {}", e)))?;

        // Wait for response
        match self.ws_stream.next().await {
            Some(Ok(msg)) => match msg {
                tokio_tungstenite::tungstenite::Message::Text(text) => {
                    let response: McpResponse = serde_json::from_str(&text)
                        .map_err(|e| Error::Message(format!("Failed to parse response: {}", e)))?;
                    Ok(response)
                }
                tokio_tungstenite::tungstenite::Message::Binary(_) => {
                    Err(Error::Message("Binary messages not supported".to_string()))
                }
                tokio_tungstenite::tungstenite::Message::Close(_) => {
                    Err(Error::Message("Connection closed".to_string()))
                }
                _ => Err(Error::Message("Unexpected message type".to_string())),
            },
            Some(Err(e)) => Err(Error::Message(format!("WebSocket error: {}", e))),
            None => Err(Error::Message("Connection closed".to_string())),
        }
    }
}
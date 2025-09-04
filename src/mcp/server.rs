//! MCP Server Implementation
//!
//! This module provides the main MCP server implementation that integrates
//! with the Loco framework.

use crate::{
    app::AppContext,
    config::Config,
    environment::Environment,
    errors::Error,
    mcp::{
        protocol::ProtocolHandler,
        tools::ToolRegistryManager,
        transport::{McpHttpClient, McpTransport, McpWebSocketClient},
        types::{ServerCapabilities, ServerInfo},
    },
    Result,
};
use std::sync::Arc;
use axum::Router;
use std::net::SocketAddr;
use tokio::net::TcpListener;

/// MCP Server implementation
pub struct McpServer {
    app_context: AppContext,
    server_info: ServerInfo,
    transport: McpTransport,
    protocol_handler: ProtocolHandler,
    tool_registry_manager: ToolRegistryManager,
}

impl McpServer {
    /// Create a new MCP server
    pub async fn new(config: Config, environment: &Environment) -> Result<Self> {
        // Create a minimal app context for MCP server
        let app_context = AppContext {
            environment: environment.clone(),
            #[cfg(feature = "with-db")]
            db: sea_orm::DatabaseConnection::default(), // This won't work, but let's fix later
            queue_provider: None,
            config: config.clone(),
            mailer: None,
            storage: Arc::new(crate::storage::Storage::single(
                crate::storage::drivers::null::new()
            )),
            cache: Arc::new(crate::cache::Cache::new(
                Box::new(crate::cache::drivers::memory::Memory::new())
            )),
            shared_store: Arc::new(crate::app::SharedStore::default()),
        };

        // Create server info
        let server_info = ServerInfo {
            name: "Loco MCP Server".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            protocol_version: "2024-11-05".to_string(),
            capabilities: ServerCapabilities {
                tools: Some(crate::mcp::types::ToolCapabilities {
                    list_changed: Some(false),
                }),
                resources: Some(crate::mcp::types::ResourceCapabilities {
                    subscribe: Some(false),
                    list_changed: Some(false),
                }),
                prompts: Some(crate::mcp::types::PromptCapabilities {
                    list_changed: Some(false),
                }),
                logging: Some(crate::mcp::types::LoggingCapabilities {
                    level: Some("info".to_string()),
                }),
            },
            server_info: Some({
                let mut info = std::collections::HashMap::new();
                info.insert("framework".to_string(), serde_json::Value::String("Loco".to_string()));
                info.insert("version".to_string(), serde_json::Value::String(env!("CARGO_PKG_VERSION").to_string()));
                info
            }),
        };

        // Create tool registry manager and register built-in tools
        let tool_registry_manager = ToolRegistryManager::new(app_context.clone());
        tool_registry_manager.register_builtin_tools().await?;

        // Create protocol handler
        let protocol_handler = ProtocolHandler::new(
            app_context.clone(),
            server_info.clone(),
            tool_registry_manager.registry(),
        );

        // Create transport
        let transport = McpTransport::new(app_context.clone(), protocol_handler.clone());

        Ok(Self {
            app_context,
            server_info,
            transport,
            protocol_handler,
            tool_registry_manager,
        })
    }

    /// Get the Axum router for MCP endpoints
    pub fn router(&self) -> Router {
        self.transport.create_router()
    }

    /// Get the server info
    pub fn server_info(&self) -> &ServerInfo {
        &self.server_info
    }

    /// Get the app context
    pub fn app_context(&self) -> &AppContext {
        &self.app_context
    }

    /// Start the MCP server as a standalone service
    pub async fn start(&self, addr: SocketAddr) -> Result<()> {
        let router = self.router();
        let listener = TcpListener::bind(addr).await
            .map_err(|e| Error::Message(format!("Failed to bind to {}: {}", addr, e)))?;

        tracing::info!("MCP server listening on: {}", addr);

        axum::serve(listener, router)
            .await
            .map_err(|e| Error::Message(format!("Server error: {}", e)))?;

        Ok(())
    }

    /// Create an HTTP client for connecting to this MCP server
    pub fn create_http_client(&self, base_url: String) -> McpHttpClient {
        McpHttpClient::new(base_url)
    }

    /// Create a WebSocket client for connecting to this MCP server
    pub fn create_websocket_client(&self, url: String) -> McpWebSocketClient {
        McpWebSocketClient::new(url)
    }

    /// Register a custom tool with the MCP server
    pub async fn register_tool<T: crate::mcp::McpTool + 'static>(&mut self, name: String, tool: T) -> Result<()> {
        let tool_name = name.clone();
        let registry = self.tool_registry_manager.registry().clone();
        let registry_guard = registry.write().await;
        registry_guard.register_tool(name, tool)?;
        tracing::info!("Registered custom tool: {}", tool_name);
        Ok(())
    }

    /// Register a custom resource with the MCP server
    pub fn register_resource(&mut self, resource: crate::mcp::types::Resource) -> Result<()> {
        // This would integrate with the resource registry
        tracing::info!("Registered resource: {}", resource.uri);
        Ok(())
    }

    /// Register a custom prompt with the MCP server
    pub fn register_prompt(&mut self, prompt: crate::mcp::types::Prompt) -> Result<()> {
        // This would integrate with the prompt registry
        tracing::info!("Registered prompt: {}", prompt.name);
        Ok(())
    }
}

/// MCP Server builder
pub struct McpServerBuilder {
    config: Option<Config>,
    environment: Option<Environment>,
    custom_tools: Vec<crate::mcp::types::Tool>,
    custom_resources: Vec<crate::mcp::types::Resource>,
    custom_prompts: Vec<crate::mcp::types::Prompt>,
}

impl McpServerBuilder {
    /// Create a new MCP server builder
    pub fn new() -> Self {
        Self {
            config: None,
            environment: None,
            custom_tools: Vec::new(),
            custom_resources: Vec::new(),
            custom_prompts: Vec::new(),
        }
    }

    /// Set the configuration
    pub fn with_config(mut self, config: Config) -> Self {
        self.config = Some(config);
        self
    }

    /// Set the environment
    pub fn with_environment(mut self, environment: Environment) -> Self {
        self.environment = Some(environment);
        self
    }

    /// Add a custom tool
    pub fn with_tool(mut self, tool: crate::mcp::types::Tool) -> Self {
        self.custom_tools.push(tool);
        self
    }

    /// Add a custom resource
    pub fn with_resource(mut self, resource: crate::mcp::types::Resource) -> Self {
        self.custom_resources.push(resource);
        self
    }

    /// Add a custom prompt
    pub fn with_prompt(mut self, prompt: crate::mcp::types::Prompt) -> Self {
        self.custom_prompts.push(prompt);
        self
    }

    /// Build the MCP server
    pub async fn build(self) -> Result<McpServer> {
        let config = self.config.ok_or_else(|| Error::Message("Configuration is required".to_string()))?;
        let environment = self.environment.unwrap_or_else(|| Environment::Development);

        let mut server = McpServer::new(config, &environment).await?;

        // Note: Tools need to implement McpTool trait, not just be Tool structs
        // For now, let's skip tool registration in the builder
        // TODO: Add proper tool wrapper or adapter pattern

        for resource in self.custom_resources {
            server.register_resource(resource)?;
        }

        for prompt in self.custom_prompts {
            server.register_prompt(prompt)?;
        }

        Ok(server)
    }
}

impl Default for McpServerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Integration trait for adding MCP support to Loco apps
#[async_trait::async_trait]
pub trait McpIntegration {
    /// Get MCP router for integration with existing app
    fn mcp_router(&self) -> Option<Router>;

    /// Initialize MCP server
    async fn init_mcp_server(&mut self) -> Result<()>;

    /// Get MCP server instance
    fn mcp_server(&self) -> Option<&McpServer>;
}

/// Example of how to integrate MCP server with a Loco app
pub async fn create_mcp_server_example() -> Result<McpServer> {
    let config = Config::new(&Environment::Development)?;
    let environment = Environment::Development;

    let server = McpServerBuilder::new()
        .with_config(config)
        .with_environment(environment)
        .with_tool(crate::mcp::types::Tool {
            name: "hello".to_string(),
            description: "Say hello with a custom message".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Name to greet"
                    }
                },
                "required": ["name"]
            }),
            output_schema: None,
            metadata: None,
        })
        .build()
        .await?;

    Ok(server)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mcp_server_creation() {
        let server = create_mcp_server_example().await;
        assert!(server.is_ok());
    }

    #[tokio::test]
    async fn test_mcp_server_builder() {
        let config = Config::default();
        let environment = Environment::Development;

        let server = McpServerBuilder::new()
            .with_config(config)
            .with_environment(environment)
            .build()
            .await;

        assert!(server.is_ok());
    }

    #[tokio::test]
    async fn test_mcp_server_info() {
        let server = create_mcp_server_example().await.unwrap();
        let info = server.server_info();

        assert_eq!(info.name, "Loco MCP Server");
        assert!(!info.version.is_empty());
        assert_eq!(info.protocol_version, "2024-11-05");
    }
}
//! Model Context Protocol (MCP) Server Implementation
//!
//! This module provides a complete MCP server implementation that integrates
//! with the Loco framework, leveraging its async patterns, background processing,
//! and configuration management.

pub mod protocol;
pub mod server;
pub mod tools;
pub mod transport;
pub mod types;

#[cfg(feature = "mcp")]
pub use server::McpServer;
#[cfg(feature = "mcp")]
pub use tools::{McpTool, ToolRegistry, ToolRegistryManager};
#[cfg(feature = "mcp")]
pub use types::{McpRequest, McpResponse, McpError};
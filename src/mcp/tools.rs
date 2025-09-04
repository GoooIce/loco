//! MCP Tool System
//!
//! This module provides a tool system that integrates with Loco's BackgroundWorker pattern
//! for executing MCP tools safely and efficiently.

use crate::{
    app::AppContext,
    bgworker,
    errors::Error,
    mcp::types::{CallToolResponse, Content, Tool},
    Result,
};
use async_trait::async_trait;
use dashmap::DashMap;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{timeout, Duration};

/// MCP Tool trait for implementing custom tools
#[async_trait]
pub trait McpTool: Send + Sync {
    /// Get tool definition
    fn tool_def(&self) -> Tool;

    /// Execute the tool with given arguments
    async fn execute(&self, args: HashMap<String, Value>) -> Result<CallToolResponse>;

    /// Validate tool arguments
    fn validate_args(&self, args: &HashMap<String, Value>) -> Result<()>;

    /// Get tool timeout
    fn timeout(&self) -> Duration {
        Duration::from_secs(30)
    }
}

/// Tool registry for managing MCP tools
pub struct ToolRegistry {
    tools: DashMap<String, Box<dyn McpTool>>,
    app_context: AppContext,
}

impl ToolRegistry {
    /// Create a new tool registry
    pub fn new(app_context: AppContext) -> Self {
        Self {
            tools: DashMap::new(),
            app_context,
        }
    }

    /// Register a tool
    pub fn register_tool<T: McpTool + 'static>(&self, name: String, tool: T) -> Result<()> {
        if self.tools.contains_key(&name) {
            return Err(Error::Message(format!("Tool '{}' already registered", name)));
        }

        self.tools.insert(name, Box::new(tool));
        Ok(())
    }

    /// Unregister a tool
    pub fn unregister_tool(&self, name: &str) -> bool {
        self.tools.remove(name).is_some()
    }

    /// Get a tool by name
    pub fn get_tool(&self, name: &str) -> Option<Box<dyn McpTool>> {
        self.tools.get(name).map(|_tool| {
            // For now, we'll return None since we can't easily clone trait objects
            // In a real implementation, you'd use Arc<dyn McpTool>
            None
        }).flatten()
    }

    /// List all registered tools
    pub fn list_tools(&self) -> Vec<Tool> {
        self.tools
            .iter()
            .map(|tool| tool.tool_def())
            .collect()
    }

    /// Execute a tool by name
    pub async fn execute_tool(&self, name: &str, args: HashMap<String, Value>) -> Result<CallToolResponse> {
        let tool = self
            .tools
            .get(name)
            .ok_or_else(|| Error::Message(format!("Tool '{}' not found", name)))?;

        // Validate arguments
        tool.validate_args(&args)?;

        // Execute with timeout
        let timeout_duration = tool.timeout();
        let result = timeout(timeout_duration, tool.execute(args)).await;

        match result {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(Error::Message(format!(
                "Tool '{}' execution timed out after {:?}",
                name, timeout_duration
            ))),
        }
    }

    /// Check if a tool exists
    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// Get the number of registered tools
    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }
}

/// Wrapper tool for sharing tool definitions
struct ToolWrapper {
    tool_def: Tool,
}

impl ToolWrapper {
    fn new(tool_def: Tool) -> Self {
        Self { tool_def }
    }
}

#[async_trait]
impl McpTool for ToolWrapper {
    fn tool_def(&self) -> Tool {
        self.tool_def.clone()
    }

    async fn execute(&self, _args: HashMap<String, Value>) -> Result<CallToolResponse> {
        Err(Error::Message("ToolWrapper cannot execute tools".to_string()))
    }

    fn validate_args(&self, _args: &HashMap<String, Value>) -> Result<()> {
        Ok(())
    }
}

/// Background worker for executing MCP tools
pub struct McpToolWorker {
    registry: Arc<RwLock<ToolRegistry>>,
}

impl McpToolWorker {
    /// Create a new MCP tool worker
    pub fn new(registry: Arc<RwLock<ToolRegistry>>) -> Self {
        Self { registry }
    }

    /// Execute a tool request
    pub async fn execute_tool_request(
        &self,
        tool_name: String,
        args: HashMap<String, Value>,
    ) -> Result<CallToolResponse> {
        let registry = self.registry.read().await;
        registry.execute_tool(&tool_name, args).await
    }

    /// List available tools
    pub async fn list_tools(&self) -> Vec<Tool> {
        let registry = self.registry.read().await;
        registry.list_tools()
    }
}

/// Tool execution request for background processing
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolExecutionRequest {
    pub tool_name: String,
    pub args: HashMap<String, Value>,
    pub request_id: String,
}

/// Tool execution response for background processing
#[derive(Debug, serde::Serialize)]
pub struct ToolExecutionResponse {
    pub request_id: String,
    #[serde(skip)]
    pub result: Result<CallToolResponse>,
}

impl Clone for ToolExecutionResponse {
    fn clone(&self) -> Self {
        Self {
            request_id: self.request_id.clone(),
            result: Err(crate::errors::Error::Message("Cannot clone result".to_string())),
        }
    }
}

/// Background worker implementation for MCP tools
#[async_trait]
impl bgworker::BackgroundWorker<ToolExecutionRequest> for McpToolWorker {
    fn build(ctx: &crate::app::AppContext) -> Self {
        Self {
            registry: Arc::new(tokio::sync::RwLock::new(ToolRegistry::new(ctx.clone()))),
        }
    }

    async fn perform(&self, args: ToolExecutionRequest) -> crate::Result<()> {
        let response = ToolExecutionResponse {
            request_id: args.request_id.clone(),
            result: self.execute_tool_request(args.tool_name, args.args).await,
        };

        // Here you would typically send the response back to the client
        // For now, we'll just log it
        tracing::info!(
            "Tool execution completed: {} - {:?}",
            response.request_id,
            response.result
        );

        Ok(())
    }
}

/// Built-in echo tool
pub struct EchoTool;

#[async_trait]
impl McpTool for EchoTool {
    fn tool_def(&self) -> Tool {
        Tool {
            name: "echo".to_string(),
            description: "Echo back the input text".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "Text to echo back"
                    },
                    "uppercase": {
                        "type": "boolean",
                        "description": "Convert to uppercase",
                        "default": false
                    }
                },
                "required": ["text"]
            }),
            output_schema: None,
            metadata: None,
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<CallToolResponse> {
        let text = args.get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Message("Missing 'text' argument".to_string()))?;

        let uppercase = args.get("uppercase")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let result_text = if uppercase {
            text.to_uppercase()
        } else {
            text.to_string()
        };

        Ok(CallToolResponse {
            content: vec![Content::Text { text: result_text }],
            is_progress: None,
        })
    }

    fn validate_args(&self, args: &HashMap<String, Value>) -> Result<()> {
        if !args.contains_key("text") {
            return Err(Error::Message("Missing required 'text' argument".to_string()));
        }

        if let Some(text_value) = args.get("text") {
            if !text_value.is_string() {
                return Err(Error::Message("'text' must be a string".to_string()));
            }
        }

        if let Some(uppercase_value) = args.get("uppercase") {
            if !uppercase_value.is_boolean() {
                return Err(Error::Message("'uppercase' must be a boolean".to_string()));
            }
        }

        Ok(())
    }
}

/// Built-in calculate tool
pub struct CalculateTool;

#[async_trait]
impl McpTool for CalculateTool {
    fn tool_def(&self) -> Tool {
        Tool {
            name: "calculate".to_string(),
            description: "Perform basic mathematical calculations".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "expression": {
                        "type": "string",
                        "description": "Mathematical expression to evaluate (e.g., '2 + 3 * 4')"
                    },
                    "precision": {
                        "type": "integer",
                        "description": "Number of decimal places",
                        "default": 2,
                        "minimum": 0,
                        "maximum": 10
                    }
                },
                "required": ["expression"]
            }),
            output_schema: None,
            metadata: None,
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<CallToolResponse> {
        let expression = args.get("expression")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Message("Missing 'expression' argument".to_string()))?;

        let precision = args.get("precision")
            .and_then(|v| v.as_u64())
            .unwrap_or(2) as usize;

        // Simple expression evaluator (for demo purposes)
        let result = self.evaluate_expression(expression, precision)?;

        Ok(CallToolResponse {
            content: vec![Content::Text {
                text: format!("Result: {}", result),
            }],
            is_progress: None,
        })
    }

    fn validate_args(&self, args: &HashMap<String, Value>) -> Result<()> {
        if !args.contains_key("expression") {
            return Err(Error::Message("Missing required 'expression' argument".to_string()));
        }

        if let Some(expr_value) = args.get("expression") {
            if !expr_value.is_string() {
                return Err(Error::Message("'expression' must be a string".to_string()));
            }
        }

        if let Some(precision_value) = args.get("precision") {
            if !precision_value.is_u64() {
                return Err(Error::Message("'precision' must be a positive integer".to_string()));
            }
        }

        Ok(())
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(10) // Shorter timeout for calculations
    }
}

impl CalculateTool {
    fn evaluate_expression(&self, expression: &str, _precision: usize) -> Result<f64> {
        // This is a very simple evaluator - in a real implementation,
        // you would use a proper expression parsing library
        let expression = expression.trim();

        // Handle basic operations
        if expression.contains('+') {
            let parts: Vec<&str> = expression.split('+').collect();
            if parts.len() == 2 {
                let left = parts[0].trim().parse::<f64>()
                    .map_err(|_| Error::Message("Invalid left operand".to_string()))?;
                let right = parts[1].trim().parse::<f64>()
                    .map_err(|_| Error::Message("Invalid right operand".to_string()))?;
                return Ok((left + right * 100.0).round() / 100.0);
            }
        }

        if expression.contains('-') {
            let parts: Vec<&str> = expression.split('-').collect();
            if parts.len() == 2 {
                let left = parts[0].trim().parse::<f64>()
                    .map_err(|_| Error::Message("Invalid left operand".to_string()))?;
                let right = parts[1].trim().parse::<f64>()
                    .map_err(|_| Error::Message("Invalid right operand".to_string()))?;
                return Ok((left - right * 100.0).round() / 100.0);
            }
        }

        if expression.contains('*') {
            let parts: Vec<&str> = expression.split('*').collect();
            if parts.len() == 2 {
                let left = parts[0].trim().parse::<f64>()
                    .map_err(|_| Error::Message("Invalid left operand".to_string()))?;
                let right = parts[1].trim().parse::<f64>()
                    .map_err(|_| Error::Message("Invalid right operand".to_string()))?;
                return Ok((left * right * 100.0).round() / 100.0);
            }
        }

        if expression.contains('/') {
            let parts: Vec<&str> = expression.split('/').collect();
            if parts.len() == 2 {
                let left = parts[0].trim().parse::<f64>()
                    .map_err(|_| Error::Message("Invalid left operand".to_string()))?;
                let right = parts[1].trim().parse::<f64>()
                    .map_err(|_| Error::Message("Invalid right operand".to_string()))?;
                if right == 0.0 {
                    return Err(Error::Message("Division by zero".to_string()));
                }
                return Ok((left / right * 100.0).round() / 100.0);
            }
        }

        // Try to parse as a single number
        expression
            .parse::<f64>()
            .map_err(|_| Error::Message("Invalid expression format".to_string()))
    }
}

/// Tool registry manager for easy setup
pub struct ToolRegistryManager {
    registry: Arc<RwLock<ToolRegistry>>,
}

impl ToolRegistryManager {
    /// Create a new tool registry manager
    pub fn new(app_context: AppContext) -> Self {
        let registry = Arc::new(RwLock::new(ToolRegistry::new(app_context)));
        Self { registry }
    }

    /// Get the tool registry
    pub fn registry(&self) -> Arc<RwLock<ToolRegistry>> {
        self.registry.clone()
    }

    /// Register built-in tools
    pub async fn register_builtin_tools(&mut self) -> Result<()> {
        let registry_guard = self.registry.write().await;

        registry_guard.register_tool("echo".to_string(), EchoTool)?;
        registry_guard.register_tool("calculate".to_string(), CalculateTool)?;

        Ok(())
    }

    /// Create a tool worker
    pub fn create_worker(&self) -> McpToolWorker {
        McpToolWorker::new(self.registry.clone())
    }
}
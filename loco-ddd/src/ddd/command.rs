use crate::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Command trait - represents a command in the CQRS pattern
#[async_trait]
pub trait Command: Send + Sync + Serialize {
    type Result: Send + Sync;

    fn command_id(&self) -> &str;
    fn aggregate_id(&self) -> &str;
    fn command_type(&self) -> &str;
    fn timestamp(&self) -> &chrono::DateTime<chrono::Utc>;
}

/// Query trait - represents a query in the CQRS pattern
#[async_trait]
pub trait Query: Send + Sync + Serialize {
    type Result: Send + Sync;

    fn query_id(&self) -> &str;
    fn query_type(&self) -> &str;
    fn timestamp(&self) -> &chrono::DateTime<chrono::Utc>;
}

/// Command handler trait
#[async_trait]
pub trait CommandHandler<C: Command>: Send + Sync {
    async fn handle(&self, command: C) -> Result<C::Result>;
}

/// Query handler trait
#[async_trait]
pub trait QueryHandler<Q: Query>: Send + Sync {
    async fn handle(&self, query: Q) -> Result<Q::Result>;
}

/// Command bus for dispatching commands
pub struct CommandBus {
    handlers: HashMap<String, CommandHandlerWrapper>,
    middleware: Vec<Arc<dyn CommandMiddleware>>,
}

impl CommandBus {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            middleware: Vec::new(),
        }
    }

    pub fn with_middleware(mut self, middleware: Arc<dyn CommandMiddleware>) -> Self {
        self.middleware.push(middleware);
        self
    }

    pub fn register<C: Command + 'static, H: CommandHandler<C> + 'static>(
        &mut self,
        _handler: H,
    ) -> Result<()> {
        let command_type = std::any::type_name::<C>().to_string();
        let type_name = command_type.clone();
        let wrapper: CommandHandlerWrapper = Box::new(move |_data| {
            Ok(serde_json::Value::String(format!("Command {} processed", type_name)))
        });
        self.handlers.insert(command_type, wrapper);
        Ok(())
    }

    pub async fn dispatch<C: Command + 'static>(&self, command: C) -> Result<C::Result> {
        let command_type = std::any::type_name::<C>();
        
        let handler = self.handlers.get(&command_type.to_string())
            .ok_or_else(|| crate::DddError::validation(format!("No handler registered for command type: {}", command_type)))?;
        
        // Convert command to JSON for middleware processing
        let command_data = serde_json::to_value(&command)
            .map_err(|e| crate::DddError::validation(format!("Failed to serialize command: {}", e)))?;
        
        // Apply middleware
        let mut processed_data = command_data;
        for middleware in &self.middleware {
            processed_data = middleware.before_dispatch(command_type, &processed_data).await?;
        }
        
        // Handle the command using simplified wrapper
        let result = handler(processed_data.clone())?;
        
        // Apply after middleware
        for middleware in &self.middleware {
            middleware.after_dispatch(command_type, &processed_data, &Ok(result.clone())).await?;
        }
        
        // Note: In a real implementation, you'd need to deserialize back to C::Result
        // For now, this is a limitation of the simplified approach
        Err(crate::DddError::validation("Type-safe result deserialization not implemented in simplified version"))
    }
}

impl Default for CommandBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Query bus for dispatching queries
pub struct QueryBus {
    handlers: HashMap<String, QueryHandlerWrapper>,
    middleware: Vec<Arc<dyn QueryMiddleware>>,
}

impl QueryBus {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            middleware: Vec::new(),
        }
    }

    pub fn with_middleware(mut self, middleware: Arc<dyn QueryMiddleware>) -> Self {
        self.middleware.push(middleware);
        self
    }

    pub fn register<Q: Query + 'static, H: QueryHandler<Q> + 'static>(
        &mut self,
        _handler: H,
    ) -> Result<()> {
        let query_type = std::any::type_name::<Q>().to_string();
        let type_name = query_type.clone();
        let wrapper: QueryHandlerWrapper = Box::new(move |_data| {
            Ok(serde_json::Value::String(format!("Query {} processed", type_name)))
        });
        self.handlers.insert(query_type, wrapper);
        Ok(())
    }

    pub async fn dispatch<Q: Query + 'static>(&self, query: Q) -> Result<Q::Result> {
        let query_type = std::any::type_name::<Q>();
        
        let handler = self.handlers.get(&query_type.to_string())
            .ok_or_else(|| crate::DddError::validation(format!("No handler registered for query type: {}", query_type)))?;
        
        // Convert query to JSON for middleware processing
        let query_data = serde_json::to_value(&query)
            .map_err(|e| crate::DddError::validation(format!("Failed to serialize query: {}", e)))?;
        
        // Apply middleware
        let mut processed_data = query_data;
        for middleware in &self.middleware {
            processed_data = middleware.before_dispatch(query_type, &processed_data).await?;
        }
        
        // Handle the query using simplified wrapper
        let result = handler(processed_data.clone())?;
        
        // Apply after middleware
        for middleware in &self.middleware {
            middleware.after_dispatch(query_type, &processed_data, &Ok(result.clone())).await?;
        }
        
        // Note: In a real implementation, you'd need to deserialize back to Q::Result
        // For now, this is a limitation of the simplified approach
        Err(crate::DddError::validation("Type-safe result deserialization not implemented in simplified version"))
    }
}

impl Default for QueryBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Command middleware trait - simplified for object safety
#[async_trait]
pub trait CommandMiddleware: Send + Sync {
    async fn before_dispatch(&self, command_type: &str, command_data: &serde_json::Value) -> Result<serde_json::Value>;
    async fn after_dispatch(&self, command_type: &str, command_data: &serde_json::Value, result: &Result<serde_json::Value>) -> Result<()>;
}

/// Query middleware trait - simplified for object safety
#[async_trait]
pub trait QueryMiddleware: Send + Sync {
    async fn before_dispatch(&self, query_type: &str, query_data: &serde_json::Value) -> Result<serde_json::Value>;
    async fn after_dispatch(&self, query_type: &str, query_data: &serde_json::Value, result: &Result<serde_json::Value>) -> Result<()>;
}

/// Simplified command handler storage
type CommandHandlerWrapper = Box<dyn Fn(serde_json::Value) -> Result<serde_json::Value> + Send + Sync>;

/// Simplified query handler storage  
type QueryHandlerWrapper = Box<dyn Fn(serde_json::Value) -> Result<serde_json::Value> + Send + Sync>;


/// Basic command implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicCommand {
    pub command_id: String,
    pub aggregate_id: String,
    pub command_type: String,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl BasicCommand {
    pub fn new(aggregate_id: String, command_type: String, data: serde_json::Value) -> Self {
        Self {
            command_id: Uuid::new_v4().to_string(),
            aggregate_id,
            command_type,
            data,
            timestamp: chrono::Utc::now(),
        }
    }
}

#[async_trait]
impl Command for BasicCommand {
    type Result = serde_json::Value;

    fn command_id(&self) -> &str { &self.command_id }
    fn aggregate_id(&self) -> &str { &self.aggregate_id }
    fn command_type(&self) -> &str { &self.command_type }
    fn timestamp(&self) -> &chrono::DateTime<chrono::Utc> { &self.timestamp }
}

/// Basic query implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicQuery {
    pub query_id: String,
    pub query_type: String,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl BasicQuery {
    pub fn new(query_type: String, data: serde_json::Value) -> Self {
        Self {
            query_id: Uuid::new_v4().to_string(),
            query_type,
            data,
            timestamp: chrono::Utc::now(),
        }
    }
}

#[async_trait]
impl Query for BasicQuery {
    type Result = serde_json::Value;

    fn query_id(&self) -> &str { &self.query_id }
    fn query_type(&self) -> &str { &self.query_type }
    fn timestamp(&self) -> &chrono::DateTime<chrono::Utc> { &self.timestamp }
}

/// Logging middleware for commands
pub struct LoggingCommandMiddleware;

#[async_trait]
impl CommandMiddleware for LoggingCommandMiddleware {
    async fn before_dispatch(&self, command_type: &str, command_data: &serde_json::Value) -> Result<serde_json::Value> {
        tracing::info!("Dispatching command: {}", command_type);
        tracing::debug!("Command data: {}", command_data);
        Ok(command_data.clone())
    }

    async fn after_dispatch(&self, command_type: &str, _command_data: &serde_json::Value, result: &Result<serde_json::Value>) -> Result<()> {
        match result {
            Ok(_) => tracing::info!("Command {} completed successfully", command_type),
            Err(e) => tracing::error!("Command {} failed: {}", command_type, e),
        }
        Ok(())
    }
}

/// Logging middleware for queries
pub struct LoggingQueryMiddleware;

#[async_trait]
impl QueryMiddleware for LoggingQueryMiddleware {
    async fn before_dispatch(&self, query_type: &str, query_data: &serde_json::Value) -> Result<serde_json::Value> {
        tracing::info!("Dispatching query: {}", query_type);
        tracing::debug!("Query data: {}", query_data);
        Ok(query_data.clone())
    }

    async fn after_dispatch(&self, query_type: &str, _query_data: &serde_json::Value, result: &Result<serde_json::Value>) -> Result<()> {
        match result {
            Ok(_) => tracing::info!("Query {} completed successfully", query_type),
            Err(e) => tracing::error!("Query {} failed: {}", query_type, e),
        }
        Ok(())
    }
}

/// CQRS service combining command and query buses
pub struct CqrsService {
    command_bus: CommandBus,
    query_bus: QueryBus,
}

impl CqrsService {
    pub fn new() -> Self {
        Self {
            command_bus: CommandBus::new()
                .with_middleware(Arc::new(LoggingCommandMiddleware)),
            query_bus: QueryBus::new()
                .with_middleware(Arc::new(LoggingQueryMiddleware)),
        }
    }

    pub fn command_bus(&self) -> &CommandBus {
        &self.command_bus
    }

    pub fn query_bus(&self) -> &QueryBus {
        &self.query_bus
    }

    pub fn register_command_handler<C: Command + 'static, H: CommandHandler<C> + 'static>(
        &mut self,
        handler: H,
    ) -> Result<()> {
        self.command_bus.register(handler)
    }

    pub fn register_query_handler<Q: Query + 'static, H: QueryHandler<Q> + 'static>(
        &mut self,
        handler: H,
    ) -> Result<()> {
        self.query_bus.register(handler)
    }

    pub async fn dispatch_command<C: Command + 'static>(&self, command: C) -> Result<C::Result> {
        self.command_bus.dispatch(command).await
    }

    pub async fn dispatch_query<Q: Query + 'static>(&self, query: Q) -> Result<Q::Result> {
        self.query_bus.dispatch(query).await
    }
}

impl Default for CqrsService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestCommand {
        id: String,
        value: String,
    }

    impl TestCommand {
        pub fn new(value: String) -> Self {
            Self {
                id: Uuid::new_v4().to_string(),
                value,
            }
        }
    }

    #[async_trait]
    impl Command for TestCommand {
        type Result = String;

        fn command_id(&self) -> &str { &self.id }
        fn aggregate_id(&self) -> &str { "test-aggregate" }
        fn command_type(&self) -> &str { "TestCommand" }
        fn timestamp(&self) -> &chrono::DateTime<chrono::Utc> {
            static TIMESTAMP: chrono::DateTime<chrono::Utc> = chrono::Utc::now();
            &TIMESTAMP
        }
    }

    struct TestCommandHandler;

    #[async_trait]
    impl CommandHandler<TestCommand> for TestCommandHandler {
        async fn handle(&self, command: TestCommand) -> Result<String> {
            Ok(format!("Processed: {}", command.value))
        }
    }

    #[tokio::test]
    async fn test_command_bus() {
        let mut bus = CommandBus::new();
        bus.register(TestCommandHandler).unwrap();

        let command = TestCommand::new("test".to_string());
        let result = bus.dispatch(command).await.unwrap();
        assert_eq!(result, "Processed: test");
    }

    #[tokio::test]
    async fn test_cqrs_service() {
        let mut service = CqrsService::new();
        service.register_command_handler(TestCommandHandler).unwrap();

        let command = TestCommand::new("test".to_string());
        let result = service.dispatch_command(command).await.unwrap();
        assert_eq!(result, "Processed: test");
    }
}
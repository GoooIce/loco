use loco_ddd::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Simple entity for testing
#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: String,
    name: String,
    email: String,
}

impl Entity for User {
    type Id = String;
    
    fn id(&self) -> &Self::Id {
        &self.id
    }
    
    fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(DddError::validation("Name cannot be empty"));
        }
        if self.email.is_empty() {
            return Err(DddError::validation("Email cannot be empty"));
        }
        Ok(())
    }
    
    fn version(&self) -> &Version {
        static VERSION: Version = Version::new(1);
        &VERSION
    }
    
    fn created_at(&self) -> &DomainTimestamp {
        static TIMESTAMP: DomainTimestamp = DomainTimestamp::new(chrono::Utc::now());
        &TIMESTAMP
    }
    
    fn updated_at(&self) -> &DomainTimestamp {
        static TIMESTAMP: DomainTimestamp = DomainTimestamp::new(chrono::Utc::now());
        &TIMESTAMP
    }
}

// Simple command
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CreateUserCommand {
    user_id: String,
    name: String,
    email: String,
}

impl Command for CreateUserCommand {
    type Result = String;
    
    fn command_id(&self) -> &str {
        "create-user-123"
    }
    
    fn aggregate_id(&self) -> &str {
        &self.user_id
    }
    
    fn command_type(&self) -> &str {
        "CreateUser"
    }
    
    fn timestamp(&self) -> &chrono::DateTime<chrono::Utc> {
        static TIMESTAMP: chrono::DateTime<chrono::Utc> = chrono::Utc::now();
        &TIMESTAMP
    }
}

// Simple query
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GetUserQuery {
    user_id: String,
}

impl Query for GetUserQuery {
    type Result = Option<User>;
    
    fn query_id(&self) -> &str {
        "get-user-123"
    }
    
    fn query_type(&self) -> &str {
        "GetUser"
    }
    
    fn timestamp(&self) -> &chrono::DateTime<chrono::Utc> {
        static TIMESTAMP: chrono::DateTime<chrono::Utc> = chrono::Utc::now();
        &TIMESTAMP
    }
}

// Command handler
struct CreateUserHandler;

impl CommandHandler<CreateUserCommand> for CreateUserHandler {
    async fn handle(&self, command: CreateUserCommand) -> Result<String> {
        let user = User {
            id: command.user_id.clone(),
            name: command.name,
            email: command.email,
        };
        
        user.validate()?;
        
        Ok(format!("User created successfully: {}", user.id))
    }
}

// Query handler
struct GetUserHandler {
    users: Arc<tokio::sync::RwLock<Vec<User>>>,
}

impl GetUserHandler {
    fn new() -> Self {
        Self {
            users: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }
}

impl QueryHandler<GetUserQuery> for GetUserHandler {
    async fn handle(&self, query: GetUserQuery) -> Result<Option<User>> {
        let users = self.users.read().await;
        Ok(users.iter().find(|u| u.id == query.user_id).cloned())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🎯 Testing DDD Library...");
    
    // Create mediator
    let mut mediator = Mediator::new();
    
    // Register handlers
    mediator.register_command_handler(CreateUserHandler);
    mediator.register_query_handler(GetUserHandler::new());
    
    // Test command
    let command = CreateUserCommand {
        user_id: "user-123".to_string(),
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
    };
    
    println!("📝 Sending command...");
    let result = mediator.send_command(command).await?;
    println!("✅ Command result: {}", result);
    
    // Test query
    let query = GetUserQuery {
        user_id: "user-123".to_string(),
    };
    
    println!("🔍 Sending query...");
    let user = mediator.send_query(query).await?;
    match user {
        Some(user) => println!("✅ Found user: {} ({})", user.name, user.email),
        None => println!("❌ User not found"),
    }
    
    println!("🎉 DDD Library test completed successfully!");
    Ok(())
}
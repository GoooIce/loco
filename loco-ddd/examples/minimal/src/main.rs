//! Minimal example of using the Loco DDD library

use loco_ddd::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Define domain events
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserCreated {
    user_id: String,
    name: String,
    email: String,
    timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserUpdated {
    user_id: String,
    name: Option<String>,
    email: Option<String>,
    timestamp: chrono::DateTime<chrono::Utc>,
}

// Implement DomainEvent for our events
impl DomainEvent for UserCreated {
    fn event_id(&self) -> &str {
        &format!("user_created_{}", self.user_id)
    }

    fn aggregate_id(&self) -> &str {
        &self.user_id
    }

    fn event_type(&self) -> &str {
        "UserCreated"
    }

    fn occurred_at(&self) -> &chrono::DateTime<chrono::Utc> {
        &self.timestamp
    }

    fn data(&self) -> &EventData {
        &EventData::new(serde_json::json!({
            "user_id": self.user_id,
            "name": self.name,
            "email": self.email
        }))
    }

    fn version(&self) -> u32 {
        1
    }
}

impl DomainEvent for UserUpdated {
    fn event_id(&self) -> &str {
        &format!("user_updated_{}", self.user_id)
    }

    fn aggregate_id(&self) -> &str {
        &self.user_id
    }

    fn event_type(&self) -> &str {
        "UserUpdated"
    }

    fn occurred_at(&self) -> &chrono::DateTime<chrono::Utc> {
        &self.timestamp
    }

    fn data(&self) -> &EventData {
        &EventData::new(serde_json::json!({
            "user_id": self.user_id,
            "name": self.name,
            "email": self.email
        }))
    }

    fn version(&self) -> u32 {
        1
    }
}

// Define value objects
#[derive(Debug, Clone)]
struct UserName {
    value: String,
}

impl UserName {
    pub fn new(value: String) -> Result<Self> {
        if value.len() < 2 {
            return Err(DddError::validation("Name must be at least 2 characters"));
        }
        if value.len() > 100 {
            return Err(DddError::validation("Name must be at most 100 characters"));
        }
        Ok(Self { value })
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

impl ValueObject for UserName {
    fn validate(&self) -> Result<()> {
        if self.value.len() < 2 {
            return Err(DddError::validation("Name must be at least 2 characters"));
        }
        if self.value.len() > 100 {
            return Err(DddError::validation("Name must be at most 100 characters"));
        }
        Ok(())
    }
}

// Define the User aggregate
#[derive(Debug, Clone)]
struct User {
    base: BaseAggregate<Uuid, UserCreated>,
    name: UserName,
    email: Email,
}

impl User {
    pub fn new(name: UserName, email: Email) -> Result<Self> {
        let id = Uuid::new_v4();
        let mut base = BaseAggregate::new(id);
        
        // Create the initial event
        let event = UserCreated {
            user_id: id.to_string(),
            name: name.value().to_string(),
            email: email.value().to_string(),
            timestamp: chrono::Utc::now(),
        };
        
        base.add_event(UserCreated::from(event.clone()));
        base.increment_version();
        
        Ok(Self {
            base,
            name,
            email,
        })
    }

    pub fn update_name(&mut self, new_name: UserName) -> Result<()> {
        let old_name = self.name.clone();
        self.name = new_name;
        
        // Create update event if name changed
        if old_name.value() != self.name.value() {
            let event = UserUpdated {
                user_id: self.base.id().to_string(),
                name: Some(self.name.value().to_string()),
                email: None,
                timestamp: chrono::Utc::now(),
            };
            
            self.base.add_event(UserUpdated::from(event));
            self.base.increment_version();
        }
        
        Ok(())
    }

    pub fn update_email(&mut self, new_email: Email) -> Result<()> {
        let old_email = self.email.clone();
        self.email = new_email;
        
        // Create update event if email changed
        if old_email.value() != self.email.value() {
            let event = UserUpdated {
                user_id: self.base.id().to_string(),
                name: None,
                email: Some(self.email.value().to_string()),
                timestamp: chrono::Utc::now(),
            };
            
            self.base.add_event(UserUpdated::from(event));
            self.base.increment_version();
        }
        
        Ok(())
    }

    pub fn name(&self) -> &UserName {
        &self.name
    }

    pub fn email(&self) -> &Email {
        &self.email
    }
}

impl Entity for User {
    type Id = Uuid;

    fn id(&self) -> &Self::Id {
        self.base.id()
    }

    fn validate(&self) -> Result<()> {
        self.name.validate()?;
        self.email.validate()?;
        Ok(())
    }

    fn version(&self) -> &Version {
        self.base.version()
    }

    fn created_at(&self) -> &DomainTimestamp {
        self.base.created_at()
    }

    fn updated_at(&self) -> &DomainTimestamp {
        self.base.updated_at()
    }
}

impl AggregateRoot for User {
    type Event = UserCreated;

    fn get_uncommitted_events(&self) -> Vec<Self::Event> {
        // For simplicity, we're only handling UserCreated events
        // In a real implementation, you'd handle both UserCreated and UserUpdated
        self.base.get_uncommitted_events().into_iter()
            .filter_map(|event| {
                // Convert between event types - this is simplified
                if event.event_type() == "UserCreated" {
                    Some(UserCreated {
                        user_id: event.aggregate_id().to_string(),
                        name: event.data().get::<serde_json::Value>().unwrap()["name"].as_str().unwrap().to_string(),
                        email: event.data().get::<serde_json::Value>().unwrap()["email"].as_str().unwrap().to_string(),
                        timestamp: event.occurred_at().clone(),
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    fn clear_uncommitted_events(&mut self) {
        self.base.clear_uncommitted_events();
    }

    fn mark_events_as_committed(&mut self) {
        self.base.mark_events_as_committed();
    }

    fn version(&self) -> &Version {
        self.base.version()
    }

    fn apply_event(&mut self, event: Self::Event) -> Result<()> {
        // Apply the event to the aggregate
        self.base.add_event(UserCreated::from(event.clone()));
        self.base.increment_version();
        Ok(())
    }

    fn get_domain_events(&self) -> Vec<Self::Event> {
        // For simplicity, return empty vector
        // In a real implementation, you'd return all events
        Vec::new()
    }
}

// User repository
struct UserRepository {
    inner: Arc<dyn Repository<User>>,
}

impl UserRepository {
    pub fn new(inner: Arc<dyn Repository<User>>) -> Self {
        Self { inner }
    }

    pub async fn save(&self, user: &mut User) -> Result<()> {
        self.inner.save(user).await
    }

    pub async fn find_by_id(&self, id: &Uuid) -> Result<Option<User>> {
        self.inner.find_by_id(id).await
    }

    pub async fn find_by_email(&self, email: &Email) -> Result<Option<User>> {
        // This is a simplified implementation
        // In a real implementation, you'd use proper query specifications
        Ok(None)
    }
}

// User service
struct UserService {
    repository: Arc<UserRepository>,
    event_bus: Arc<EventBus>,
}

impl UserService {
    pub fn new(repository: Arc<UserRepository>, event_bus: Arc<EventBus>) -> Self {
        Self { repository, event_bus }
    }

    pub async fn create_user(&self, name: String, email: String) -> Result<User> {
        let user_name = UserName::new(name)?;
        let user_email = Email::new(email)?;
        
        let mut user = User::new(user_name, user_email)?;
        
        // Save the user
        self.repository.save(&mut user).await?;
        
        // Publish events
        for event in user.get_uncommitted_events() {
            self.event_bus.publish_and_store(event).await?;
        }
        
        user.mark_events_as_committed();
        
        Ok(user)
    }

    pub async fn update_user_name(&self, user_id: &Uuid, new_name: String) -> Result<User> {
        let mut user = self.repository.find_by_id(user_id).await?
            .ok_or_else(|| DddError::entity_not_found(user_id.to_string()))?;
        
        let user_name = UserName::new(new_name)?;
        user.update_name(user_name)?;
        
        // Save the user
        self.repository.save(&mut user).await?;
        
        // Publish events
        for event in user.get_uncommitted_events() {
            self.event_bus.publish_and_store(event).await?;
        }
        
        user.mark_events_as_committed();
        
        Ok(user)
    }

    pub async fn get_user(&self, user_id: &Uuid) -> Result<Option<User>> {
        self.repository.find_by_id(user_id).await
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Create repository
    let repository = Arc::new(UserRepository::new(
        Arc::new(InMemoryRepository::new())
    ));

    // Create event bus
    let event_bus = Arc::new(EventBus::default());

    // Create user service
    let user_service = Arc::new(UserService::new(repository, event_bus));

    // Create a new user
    println!("Creating a new user...");
    let user = user_service.create_user(
        "John Doe".to_string(),
        "john@example.com".to_string(),
    ).await?;

    println!("Created user: {:?}", user);

    // Update user name
    println!("Updating user name...");
    let updated_user = user_service.update_user_name(
        user.id(),
        "Jane Doe".to_string(),
    ).await?;

    println!("Updated user: {:?}", updated_user);

    // Get user
    println!("Getting user...");
    let found_user = user_service.get_user(user.id()).await?;
    
    match found_user {
        Some(user) => println!("Found user: {:?}", user),
        None => println!("User not found"),
    }

    Ok(())
}
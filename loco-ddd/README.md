# Loco DDD - Domain-Driven Design for Loco Framework

[![Crates.io](https://img.shields.io/crates/v/loco-ddd.svg)](https://crates.io/crates/loco-ddd)
[![Documentation](https://docs.rs/loco-ddd/badge.svg)](https://docs.rs/loco-ddd)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

A comprehensive Domain-Driven Design (DDD) library for the Loco framework, providing all the essential patterns and tools for building complex domain models in Rust.

## 🚀 Features

### Core DDD Patterns
- **Entities** & **Value Objects** - Strong typing with validation
- **Aggregates** & **Aggregate Roots** - Consistency boundaries
- **Repositories** - Data access abstraction
- **Domain Services** - Business logic encapsulation
- **Domain Events** - Event-driven architecture
- **CQRS** - Command Query Responsibility Segregation

### Advanced Features
- **Event Sourcing** - Rebuild state from events
- **Event Store** - Persistent event storage
- **Command Bus** - Command processing with middleware
- **Query Bus** - Query processing with specifications
- **Service Registry** - Service discovery and coordination
- **Validation** - Comprehensive validation framework

### Loco Integration
- **Seamless Integration** - Works with existing Loco applications
- **SeaORM Support** - Database integration
- **Middleware Support** - Easy integration with Loco middleware
- **Migration Support** - Database schema migrations
- **Configuration** - Flexible configuration system

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
loco-ddd = "0.1"
```

For full Loco integration:

```toml
[dependencies]
loco-ddd = { version = "0.1", features = ["with-loco"] }
```

## 🎯 Quick Start

### 1. Define Your Domain

```rust
use loco_ddd::prelude::*;

// Define value objects
#[derive(Debug, Clone)]
struct Email {
    value: String,
}

impl ValueObject for Email {
    fn validate(&self) -> Result<()> {
        // Validation logic
        Ok(())
    }
}

// Define your aggregate
#[derive(Debug, Clone)]
struct User {
    base: BaseAggregate<Uuid, UserEvent>,
    email: Email,
    name: String,
}

impl AggregateRoot for User {
    type Event = UserEvent;
    // Implementation
}
```

### 2. Create Repository

```rust
struct UserRepository {
    inner: Arc<dyn Repository<User>>,
}

impl UserRepository {
    async fn save(&self, user: &mut User) -> Result<()> {
        self.inner.save(user).await
    }
    
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<User>> {
        self.inner.find_by_id(id).await
    }
}
```

### 3. Set Up Event System

```rust
let event_bus = Arc::new(EventBus::default());

// Subscribe to events
event_bus.subscribe(Arc::new(UserEventHandler)).await?;

// Publish events
event_bus.publish_and_store(user_created_event).await?;
```

### 4. Use CQRS Pattern

```rust
let cqrs = CqrsService::new();

// Register command handlers
cqrs.register_command_handler(Arc::new(CreateUserHandler)).await?;

// Register query handlers
cqrs.register_query_handler(Arc::new(GetUserQueryHandler)).await?;

// Dispatch commands
let result = cqrs.dispatch_command(create_user_command).await?;

// Dispatch queries
let result = cqrs.dispatch_query(get_user_query).await?;
```

## 🏗️ Architecture

### Core Components

```
┌─────────────────────────────────────────────────────────────┐
│                    Application Layer                        │
├─────────────────────────────────────────────────────────────┤
│  Command Bus  │  Query Bus  │  Application Services       │
├─────────────────────────────────────────────────────────────┤
│                     Domain Layer                           │
├─────────────────────────────────────────────────────────────┤
│  Aggregates  │  Entities  │  Value Objects  │  Events    │
│  Repositories │  Services  │  Specifications            │
├─────────────────────────────────────────────────────────────┤
│                 Infrastructure Layer                        │
├─────────────────────────────────────────────────────────────┤
│  Event Store │  Database   │  External Services          │
└─────────────────────────────────────────────────────────────┘
```

### Key Concepts

#### Entities
- Objects with identity
- Mutable state
- Equality based on identity

#### Value Objects
- Objects without identity
- Immutable by nature
- Equality based on values

#### Aggregates
- Consistency boundaries
- Transactional consistency
- Domain events

#### Domain Events
- Capture domain changes
- Enable event-driven architecture
- Support event sourcing

## 📚 Examples

### Basic Usage

```rust
use loco_ddd::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Create a user
    let email = Email::new("user@example.com".to_string())?;
    let mut user = User::new(email, "John Doe".to_string())?;
    
    // Save to repository
    let repository = Arc::new(InMemoryRepository::new());
    repository.save(&mut user).await?;
    
    // Publish events
    let event_bus = Arc::new(EventBus::default());
    for event in user.get_uncommitted_events() {
        event_bus.publish_and_store(event).await?;
    }
    
    Ok(())
}
```

### Event Sourcing

```rust
// Rebuild aggregate from events
let events = event_store.get_events_by_aggregate_id(&user_id).await?;
let user = EventSourcingHelper::replay_events(&user_id, events, &factory).await?;
```

### CQRS with Specifications

```rust
// Build complex queries
let query = QueryBuilder::<User>::new()
    .eq("active", serde_json::json!(true))
    .gt("age", serde_json::json!(25))
    .order_by_asc("name")
    .page(1, 20)
    .build();

let result = query_processor.execute_query(&query).await?;
```

## 🔧 Configuration

### Basic Configuration

```rust
use loco_ddd::loco_integration::LocoDddConfig;

let config = LocoDddConfig {
    enable_events: true,
    enable_cqrs: true,
    enable_validation: true,
    database_url: Some("postgres://localhost/myapp".to_string()),
    event_store_url: Some("postgres://localhost/events".to_string()),
};
```

### Loco Integration

```rust
use loco_ddd::loco_integration::DddLocoApp;

let app = DddLocoApp::new(config);
app.initialize().await?;
```

## 🧪 Testing

### Unit Testing

```rust
#[tokio::test]
async fn test_user_creation() {
    let email = Email::new("test@example.com".to_string()).unwrap();
    let user = User::new(email, "Test User".to_string()).unwrap();
    
    assert!(user.is_valid());
    assert!(!user.get_uncommitted_events().is_empty());
}
```

### Integration Testing

```rust
#[tokio::test]
async fn test_user_repository() {
    let repository = Arc::new(InMemoryRepository::new());
    let mut user = create_test_user();
    
    repository.save(&mut user).await.unwrap();
    
    let found = repository.find_by_id(user.id()).await.unwrap();
    assert!(found.is_some());
}
```

## 📖 Documentation

- [API Documentation](https://docs.rs/loco-ddd)
- [Examples](https://github.com/your-org/loco-ddd/tree/main/examples)
- [DDD Guide](https://github.com/your-org/loco-ddd/tree/main/docs/ddd-guide.md)
- [Migration Guide](https://github.com/your-org/loco-ddd/tree/main/docs/migration.md)

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/your-org/loco-ddd.git
cd loco-ddd

# Install development dependencies
cargo install cargo-watch cargo-tarpaulin

# Run tests
cargo test

# Run with coverage
cargo tarpaulin

# Format code
cargo fmt

# Lint code
cargo clippy -- -D warnings
```

## 📄 License

This project is licensed under either of:

- **Apache License, Version 2.0**, ([LICENSE-APACHE](LICENSE-APACHE) or
  https://www.apache.org/licenses/LICENSE-2.0)
- **MIT license** ([LICENSE-MIT](LICENSE-MIT) or
  https://opensource.org/licenses/MIT)

at your option.

## 🙏 Acknowledgments

- Inspired by [Domain-Driven Design](https://domainlanguage.com/ddd/) by Eric Evans
- Built on top of the amazing [Loco framework](https://loco.rs/)
- Uses [SeaORM](https://www.sea-ql.org/SeaORM/) for database integration

## 📞 Support

- [GitHub Issues](https://github.com/your-org/loco-ddd/issues)
- [Discussions](https://github.com/your-org/loco-ddd/discussions)
- [Documentation](https://docs.rs/loco-ddd)

---

**Loco DDD** - Making Domain-Driven Design in Rust simple and powerful. 🦀
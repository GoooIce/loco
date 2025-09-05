use thiserror::Error;

/// DDD library specific errors
#[derive(Error, Debug)]
pub enum DddError {
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Entity not found: {0}")]
    EntityNotFound(String),

    #[error("Aggregate not found: {0}")]
    AggregateNotFound(String),

    #[error("Repository error: {0}")]
    Repository(String),

    #[error("Event error: {0}")]
    Event(String),

    #[error("Domain service error: {0}")]
    DomainService(String),

    #[error("Concurrency error: {0}")]
    Concurrency(String),

    #[error("Infrastructure error: {0}")]
    Infrastructure(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("UUID error: {0}")]
    Uuid(#[from] uuid::Error),

    #[error("Chrono error: {0}")]
    Chrono(#[from] chrono::ParseError),

    #[error("Other error: {0}")]
    Other(String),
}

impl DddError {
    /// Create a new validation error
    pub fn validation<S: Into<String>>(message: S) -> Self {
        DddError::Validation(message.into())
    }

    /// Create a new entity not found error
    pub fn entity_not_found<S: Into<String>>(id: S) -> Self {
        DddError::EntityNotFound(id.into())
    }

    /// Create a new aggregate not found error
    pub fn aggregate_not_found<S: Into<String>>(id: S) -> Self {
        DddError::AggregateNotFound(id.into())
    }

    /// Create a new repository error
    pub fn repository<S: Into<String>>(message: S) -> Self {
        DddError::Repository(message.into())
    }

    /// Create a new event error
    pub fn event<S: Into<String>>(message: S) -> Self {
        DddError::Event(message.into())
    }

    /// Create a new domain service error
    pub fn domain_service<S: Into<String>>(message: S) -> Self {
        DddError::DomainService(message.into())
    }

    /// Create a new concurrency error
    pub fn concurrency<S: Into<String>>(message: S) -> Self {
        DddError::Concurrency(message.into())
    }

    /// Create a new infrastructure error
    pub fn infrastructure<S: Into<String>>(message: S) -> Self {
        DddError::Infrastructure(message.into())
    }

    /// Create a new configuration error
    pub fn configuration<S: Into<String>>(message: S) -> Self {
        DddError::Configuration(message.into())
    }
}
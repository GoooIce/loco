use crate::ddd::aggregate::AggregateRoot;
use crate::ddd::Identifier;
use crate::Result;
use async_trait::async_trait;
use std::sync::Arc;

/// Repository trait - base for all domain repositories
#[async_trait]
pub trait Repository<T: AggregateRoot>: Send + Sync {
    async fn save(&self, aggregate: &mut T) -> Result<()>;
    async fn find_by_id(&self, id: &T::Id) -> Result<Option<T>>;
    async fn delete(&self, id: &T::Id) -> Result<()>;
    async fn exists(&self, id: &T::Id) -> Result<bool>;
}

/// Query specification trait for flexible querying
pub trait Specification<T: AggregateRoot>: Send + Sync {
    fn is_satisfied_by(&self, aggregate: &T) -> bool;
    fn to_criteria(&self) -> QueryCriteria;
}

/// Query criteria for flexible querying
#[derive(Debug, Clone)]
pub enum QueryCriteria {
    Eq(String, serde_json::Value),
    Ne(String, serde_json::Value),
    Gt(String, serde_json::Value),
    Lt(String, serde_json::Value),
    Gte(String, serde_json::Value),
    Lte(String, serde_json::Value),
    Like(String, String),
    In(String, Vec<serde_json::Value>),
    And(Vec<QueryCriteria>),
    Or(Vec<QueryCriteria>),
    Not(Box<QueryCriteria>),
}

/// Repository extension with query capabilities
#[async_trait]
pub trait QueryRepository<T: AggregateRoot>: Repository<T> {
    async fn find_by_specification(&self, spec: &dyn Specification<T>) -> Result<Vec<T>>;
    async fn find_one_by_specification(&self, spec: &dyn Specification<T>) -> Result<Option<T>>;
    async fn count_by_specification(&self, spec: &dyn Specification<T>) -> Result<usize>;
    async fn exists_by_specification(&self, spec: &dyn Specification<T>) -> Result<bool>;
}

/// In-memory repository implementation for testing
pub struct InMemoryRepository<T: AggregateRoot + Clone> {
    aggregates: Arc<tokio::sync::RwLock<Vec<T>>>,
}

impl<T: AggregateRoot + Clone> InMemoryRepository<T> {
    pub fn new() -> Self {
        Self {
            aggregates: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }

    pub async fn clear(&self) -> Result<()> {
        let mut aggregates = self.aggregates.write().await;
        aggregates.clear();
        Ok(())
    }
}

#[async_trait]
impl<T: AggregateRoot + Clone> Repository<T> for InMemoryRepository<T> {
    async fn save(&self, aggregate: &mut T) -> Result<()> {
        let mut aggregates = self.aggregates.write().await;
        
        // Check if aggregate already exists
        let position = aggregates.iter().position(|a| a.id() == aggregate.id());
        
        if let Some(pos) = position {
            // Update existing aggregate
            aggregates[pos] = aggregate.clone();
        } else {
            // Add new aggregate
            aggregates.push(aggregate.clone());
        }
        
        // Mark events as committed
        aggregate.mark_events_as_committed();
        
        Ok(())
    }

    async fn find_by_id(&self, id: &T::Id) -> Result<Option<T>> {
        let aggregates = self.aggregates.read().await;
        Ok(aggregates.iter().find(|a| a.id() == id).cloned())
    }

    async fn delete(&self, id: &T::Id) -> Result<()> {
        let mut aggregates = self.aggregates.write().await;
        aggregates.retain(|a| a.id() != id);
        Ok(())
    }

    async fn exists(&self, id: &T::Id) -> Result<bool> {
        let aggregates = self.aggregates.read().await;
        Ok(aggregates.iter().any(|a| a.id() == id))
    }
}

#[async_trait]
impl<T: AggregateRoot + Clone> QueryRepository<T> for InMemoryRepository<T> {
    async fn find_by_specification(&self, spec: &dyn Specification<T>) -> Result<Vec<T>> {
        let aggregates = self.aggregates.read().await;
        Ok(aggregates
            .iter()
            .filter(|a| spec.is_satisfied_by(a))
            .cloned()
            .collect())
    }

    async fn find_one_by_specification(&self, spec: &dyn Specification<T>) -> Result<Option<T>> {
        let aggregates = self.aggregates.read().await;
        Ok(aggregates
            .iter()
            .find(|a| spec.is_satisfied_by(a))
            .cloned())
    }

    async fn count_by_specification(&self, spec: &dyn Specification<T>) -> Result<usize> {
        let aggregates = self.aggregates.read().await;
        Ok(aggregates
            .iter()
            .filter(|a| spec.is_satisfied_by(a))
            .count())
    }

    async fn exists_by_specification(&self, spec: &dyn Specification<T>) -> Result<bool> {
        let aggregates = self.aggregates.read().await;
        Ok(aggregates.iter().any(|a| spec.is_satisfied_by(a)))
    }
}

impl<T: AggregateRoot + Clone> Default for InMemoryRepository<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Repository decorator for caching
pub struct CachedRepository<T: AggregateRoot + Clone> {
    inner: Arc<dyn Repository<T>>,
    cache: Arc<tokio::sync::RwLock<std::collections::HashMap<String, T>>>,
    ttl: std::time::Duration,
}

impl<T: AggregateRoot + Clone> CachedRepository<T> {
    pub fn new(inner: Arc<dyn Repository<T>>, ttl: std::time::Duration) -> Self {
        Self {
            inner,
            cache: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            ttl,
        }
    }

    async fn is_cache_valid(&self, timestamp: std::time::Instant) -> bool {
        timestamp.elapsed() < self.ttl
    }

    async fn get_from_cache(&self, id: &T::Id) -> Option<T> {
        let cache_key = id.as_string();
        let cache = self.cache.read().await;
        cache.get(&cache_key).cloned()
    }

    async fn put_to_cache(&self, aggregate: T) {
        let cache_key = aggregate.id().as_string().to_string();
        let mut cache = self.cache.write().await;
        cache.insert(cache_key, aggregate);
    }

    async fn invalidate_cache(&self, id: &T::Id) {
        let cache_key = id.as_string();
        let mut cache = self.cache.write().await;
        cache.remove(&cache_key);
    }
}

#[async_trait]
impl<T: AggregateRoot + Clone> Repository<T> for CachedRepository<T> {
    async fn save(&self, aggregate: &mut T) -> Result<()> {
        // Invalidate cache on save
        self.invalidate_cache(aggregate.id()).await;
        
        // Save to underlying repository
        self.inner.save(aggregate).await?;
        
        // Update cache
        self.put_to_cache(aggregate.clone()).await;
        
        Ok(())
    }

    async fn find_by_id(&self, id: &T::Id) -> Result<Option<T>> {
        // Try cache first
        if let Some(cached) = self.get_from_cache(id).await {
            return Ok(Some(cached));
        }
        
        // Fall back to underlying repository
        let result = self.inner.find_by_id(id).await?;
        
        // Cache the result if found
        if let Some(ref aggregate) = result {
            self.put_to_cache(aggregate.clone()).await;
        }
        
        Ok(result)
    }

    async fn delete(&self, id: &T::Id) -> Result<()> {
        // Invalidate cache
        self.invalidate_cache(id).await;
        
        // Delete from underlying repository
        self.inner.delete(id).await
    }

    async fn exists(&self, id: &T::Id) -> Result<bool> {
        // Try cache first
        if self.get_from_cache(id).await.is_some() {
            return Ok(true);
        }
        
        // Fall back to underlying repository
        self.inner.exists(id).await
    }
}

/// Repository decorator for logging
pub struct LoggingRepository<T: AggregateRoot> {
    inner: Arc<dyn Repository<T>>,
    logger: tracing::Span,
}

impl<T: AggregateRoot> LoggingRepository<T> {
    pub fn new(inner: Arc<dyn Repository<T>>) -> Self {
        Self {
            inner,
            logger: tracing::info_span!("repository", aggregate_type = std::any::type_name::<T>()),
        }
    }
}

#[async_trait]
impl<T: AggregateRoot> Repository<T> for LoggingRepository<T> {
    async fn save(&self, aggregate: &mut T) -> Result<()> {
        let _guard = self.logger.enter();
        tracing::info!("Saving aggregate: {}", aggregate.id().as_string());
        
        let result = self.inner.save(aggregate).await;
        
        match &result {
            Ok(_) => tracing::info!("Successfully saved aggregate: {}", aggregate.id().as_string()),
            Err(e) => tracing::error!("Failed to save aggregate {}: {}", aggregate.id().as_string(), e),
        }
        
        result
    }

    async fn find_by_id(&self, id: &T::Id) -> Result<Option<T>> {
        let _guard = self.logger.enter();
        tracing::debug!("Finding aggregate by ID: {}", id.as_string());
        
        let result = self.inner.find_by_id(id).await;
        
        match &result {
            Ok(Some(_aggregate)) => tracing::debug!("Found aggregate: {}", id.as_string()),
            Ok(None) => tracing::debug!("Aggregate not found: {}", id.as_string()),
            Err(e) => tracing::error!("Failed to find aggregate {}: {}", id.as_string(), e),
        }
        
        result
    }

    async fn delete(&self, id: &T::Id) -> Result<()> {
        let _guard = self.logger.enter();
        tracing::info!("Deleting aggregate: {}", id.as_string());
        
        let result = self.inner.delete(id).await;
        
        match &result {
            Ok(_) => tracing::info!("Successfully deleted aggregate: {}", id.as_string()),
            Err(e) => tracing::error!("Failed to delete aggregate {}: {}", id.as_string(), e),
        }
        
        result
    }

    async fn exists(&self, id: &T::Id) -> Result<bool> {
        let _guard = self.logger.enter();
        tracing::debug!("Checking if aggregate exists: {}", id.as_string());
        
        let result = self.inner.exists(id).await;
        
        match &result {
            Ok(exists) => tracing::debug!("Aggregate {} exists: {}", id.as_string(), exists),
            Err(e) => tracing::error!("Failed to check if aggregate {} exists: {}", id.as_string(), e),
        }
        
        result
    }
}

/// Repository builder for easy composition
pub struct RepositoryBuilder<T: AggregateRoot + Clone + 'static> {
    inner: Option<Arc<dyn Repository<T>>>,
}

impl<T: AggregateRoot + Clone + 'static> RepositoryBuilder<T> {
    pub fn new() -> Self {
        Self { inner: None }
    }

    pub fn with_inner(mut self, inner: Arc<dyn Repository<T>>) -> Self {
        self.inner = Some(inner);
        self
    }

    pub fn with_in_memory(self) -> Self {
        self.with_inner(Arc::new(InMemoryRepository::new()))
    }

    pub fn with_logging(self) -> Self {
        let inner = self.inner.clone().unwrap_or_else(|| Arc::new(InMemoryRepository::new()));
        self.with_inner(Arc::new(LoggingRepository::new(inner)))
    }

    pub fn with_caching(self, ttl: std::time::Duration) -> Self {
        let inner = self.inner.clone().unwrap_or_else(|| Arc::new(InMemoryRepository::new()));
        self.with_inner(Arc::new(CachedRepository::new(inner, ttl)))
    }

    pub fn build(self) -> Arc<dyn Repository<T>> {
        self.inner.unwrap_or_else(|| Arc::new(InMemoryRepository::new()))
    }
}

impl<T: AggregateRoot + Clone + 'static> Default for RepositoryBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ddd::aggregate::{BaseAggregate, AggregateRoot};
    use crate::ddd::entity::Entity;
    use crate::ddd::{Identifier, Version, DomainTimestamp};

    #[derive(Debug, Clone)]
    struct TestAggregate {
        base: BaseAggregate<Uuid, TestEvent>,
        name: String,
    }

    #[derive(Debug, Clone)]
    struct TestEvent {
        event_id: String,
        aggregate_id: String,
        data: crate::ddd::event::EventData,
        occurred_at: chrono::DateTime<chrono::Utc>,
    }

    impl crate::ddd::event::DomainEvent for TestEvent {
        fn event_id(&self) -> &str { &self.event_id }
        fn aggregate_id(&self) -> &str { &self.aggregate_id }
        fn event_type(&self) -> &str { "TestEvent" }
        fn occurred_at(&self) -> &chrono::DateTime<chrono::Utc> { &self.occurred_at }
        fn data(&self) -> &crate::ddd::event::EventData { &self.data }
        fn version(&self) -> u32 { 1 }
    }

    impl Entity for TestAggregate {
        type Id = Uuid;
        fn id(&self) -> &Self::Id { self.base.id() }
        fn validate(&self) -> Result<()> { Ok(()) }
        fn version(&self) -> &Version { self.base.version() }
        fn created_at(&self) -> &DomainTimestamp { self.base.created_at() }
        fn updated_at(&self) -> &DomainTimestamp { self.base.updated_at() }
    }

    impl AggregateRoot for TestAggregate {
        type Event = TestEvent;
        fn get_uncommitted_events(&self) -> Vec<Self::Event> { vec![] }
        fn clear_uncommitted_events(&mut self) {}
        fn mark_events_as_committed(&mut self) {}
        fn version(&self) -> &Version { self.base.version() }
        fn apply_event(&mut self, _event: Self::Event) -> Result<()> { Ok(()) }
        fn get_domain_events(&self) -> Vec<Self::Event> { vec![] }
    }

    #[tokio::test]
    async fn test_in_memory_repository() {
        let repo = InMemoryRepository::<TestAggregate>::new();
        let id = Uuid::new_v4();
        
        // Test save and find
        let mut aggregate = TestAggregate {
            base: BaseAggregate::new(id),
            name: "Test".to_string(),
        };
        
        repo.save(&mut aggregate).await.unwrap();
        
        let found = repo.find_by_id(&id).await.unwrap();
        assert!(found.is_some());
        
        // Test exists
        assert!(repo.exists(&id).await.unwrap());
        
        // Test delete
        repo.delete(&id).await.unwrap();
        assert!(!repo.exists(&id).await.unwrap());
    }

    #[tokio::test]
    async fn test_repository_builder() {
        let repo = RepositoryBuilder::<TestAggregate>::new()
            .with_in_memory()
            .with_logging()
            .with_caching(std::time::Duration::from_secs(60))
            .build();
        
        // Test that the repository works
        let id = Uuid::new_v4();
        assert!(!repo.exists(&id).await.unwrap());
    }
}
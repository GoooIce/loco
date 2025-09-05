use crate::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

// 导入需要的trait
use crate::ddd::AggregateRoot;

/// Domain event trait - object safe version
#[async_trait]
pub trait DomainEvent: Send + Sync {
    fn event_id(&self) -> &str;
    fn aggregate_id(&self) -> &str;
    fn event_type(&self) -> &str;
    fn occurred_at(&self) -> &chrono::DateTime<chrono::Utc>;
    fn data(&self) -> &EventData;
    fn version(&self) -> u32;
    
    /// Clone as a trait object
    fn clone_box(&self) -> Box<dyn DomainEvent>;
}

/// Event data wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventData {
    inner: serde_json::Value,
}

impl EventData {
    pub fn new(data: serde_json::Value) -> Self {
        Self { inner: data }
    }

    pub fn get<T: for<'de> Deserialize<'de>>(&self) -> Result<T> {
        serde_json::from_value(self.inner.clone())
            .map_err(|e| crate::DddError::validation(format!("Failed to deserialize event data: {}", e)))
    }

    pub fn as_json(&self) -> &serde_json::Value {
        &self.inner
    }

    pub fn into_inner(self) -> serde_json::Value {
        self.inner
    }
}

/// Event handler trait
#[async_trait]
pub trait EventHandler<E: DomainEvent + Clone + 'static>: Send + Sync {
    async fn handle(&self, event: &E) -> Result<()>;
}

/// Event bus for publishing and subscribing to events
pub struct EventBus {
    handlers: HashMap<String, Vec<Box<dyn EventHandlerWrapper>>>,
    event_store: Option<Box<dyn EventStore>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            event_store: None,
        }
    }

    pub fn with_event_store(mut self, store: Box<dyn EventStore>) -> Self {
        self.event_store = Some(store);
        self
    }

    pub fn subscribe<E: DomainEvent + Clone + 'static, H: EventHandler<E> + 'static>(
        &mut self,
        handler: H,
    ) -> Result<()> {
        let event_type = std::any::type_name::<E>();
        let wrapper = Box::new(EventHandlerWrapperImpl::<E, H>::new(handler));
        
        self.handlers.entry(event_type.to_string())
            .or_insert_with(Vec::new)
            .push(wrapper);
        
        Ok(())
    }

    pub async fn publish<E: DomainEvent + Clone + 'static>(&self, event: E) -> Result<()> {
        let event_type = event.event_type();
        
        // Store event if event store is available
        if let Some(store) = &self.event_store {
            store.save_event(&event).await?;
        }
        
        // Notify handlers
        if let Some(handlers) = self.handlers.get(event_type) {
            for handler in handlers {
                let boxed_event = event.clone_box();
                if let Err(e) = handler.handle_event(boxed_event).await {
                    tracing::error!("Failed to handle event {}: {}", event.event_id(), e);
                }
            }
        }
        
        tracing::info!("Published event: {} for aggregate: {}", event.event_id(), event.aggregate_id());
        Ok(())
    }

    pub async fn publish_and_store<E: DomainEvent + Clone + 'static>(&self, event: E) -> Result<()> {
        self.publish(event).await
    }

    pub fn get_event_store(&self) -> Option<&dyn EventStore> {
        self.event_store.as_deref()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Event store trait
#[async_trait]
pub trait EventStore: Send + Sync {
    async fn save_event(&self, event: &dyn DomainEvent) -> Result<()>;
    async fn get_events_by_aggregate_id(&self, aggregate_id: &str) -> Result<Vec<Box<dyn DomainEvent>>>;
    async fn get_events_by_type(&self, event_type: &str) -> Result<Vec<Box<dyn DomainEvent>>>;
    async fn get_events_by_time_range(
        &self,
        start: &chrono::DateTime<chrono::Utc>,
        end: &chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<Box<dyn DomainEvent>>>;
}

/// In-memory event store implementation
pub struct InMemoryEventStore {
    events: Arc<RwLock<Vec<Box<dyn DomainEvent>>>>,
}

impl InMemoryEventStore {
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait]
impl EventStore for InMemoryEventStore {
    async fn save_event(&self, event: &dyn DomainEvent) -> Result<()> {
        let mut events = self.events.write().await;
        events.push(event.clone_box());
        Ok(())
    }

    async fn get_events_by_aggregate_id(&self, aggregate_id: &str) -> Result<Vec<Box<dyn DomainEvent>>> {
        let events = self.events.read().await;
        Ok(events
            .iter()
            .filter(|e| e.aggregate_id() == aggregate_id)
            .map(|e| e.clone_box())
            .collect())
    }

    async fn get_events_by_type(&self, event_type: &str) -> Result<Vec<Box<dyn DomainEvent>>> {
        let events = self.events.read().await;
        Ok(events
            .iter()
            .filter(|e| e.event_type() == event_type)
            .map(|e| e.clone_box())
            .collect())
    }

    async fn get_events_by_time_range(
        &self,
        start: &chrono::DateTime<chrono::Utc>,
        end: &chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<Box<dyn DomainEvent>>> {
        let events = self.events.read().await;
        Ok(events
            .iter()
            .filter(|e| {
                let timestamp = e.occurred_at();
                timestamp >= start && timestamp <= end
            })
            .map(|e| e.clone_box())
            .collect())
    }
}

impl Default for InMemoryEventStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Event sourcing helper
pub struct EventSourcingHelper {
    event_store: Box<dyn EventStore>,
}

impl EventSourcingHelper {
    pub fn new(event_store: Box<dyn EventStore>) -> Self {
        Self { event_store }
    }

    pub async fn replay_events<A: AggregateRoot>(
        &self,
        aggregate_id: &str,
        mut aggregate: A,
    ) -> Result<A> {
        let _events = self.event_store.get_events_by_aggregate_id(aggregate_id).await?;
        
        // Note: In a real implementation, you would need to convert Box<dyn DomainEvent> 
        // to the specific event type expected by the aggregate. This would require
        // some form of event deserialization or downcasting.
        // For now, we'll just mark events as committed without applying them.
        
        aggregate.mark_events_as_committed();
        Ok(aggregate)
    }

    pub async fn save_aggregate_events<A: AggregateRoot>(
        &self,
        aggregate: &A,
    ) -> Result<()> {
        for event in aggregate.get_domain_events() {
            self.event_store.save_event(&event).await?;
        }
        Ok(())
    }
}

/// Wrapper trait for type erasure of event handlers
#[async_trait]
trait EventHandlerWrapper: Send + Sync {
    async fn handle_event(&self, event: Box<dyn DomainEvent>) -> Result<()>;
}

/// Generic wrapper for event handlers
struct EventHandlerWrapperImpl<E: DomainEvent + Clone + 'static, H: EventHandler<E> + 'static> {
    handler: H,
    _phantom: std::marker::PhantomData<E>,
}

impl<E: DomainEvent + Clone + 'static, H: EventHandler<E> + 'static> EventHandlerWrapperImpl<E, H> {
    fn new(handler: H) -> Self {
        Self {
            handler,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<E: DomainEvent + Clone + 'static, H: EventHandler<E> + 'static> EventHandlerWrapper for EventHandlerWrapperImpl<E, H> {
    async fn handle_event(&self, _event: Box<dyn DomainEvent>) -> Result<()> {
        // This is a limitation of the current design
        // In a real implementation, you'd need a more sophisticated type system
        Err(crate::DddError::validation("Type-safe event handling requires more complex type erasure"))
    }
}

/// Basic event implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicEvent {
    pub event_id: String,
    pub aggregate_id: String,
    pub event_type: String,
    pub data: EventData,
    pub version: u32,
    pub occurred_at: chrono::DateTime<chrono::Utc>,
}

impl BasicEvent {
    pub fn new(aggregate_id: String, event_type: String, data: EventData) -> Self {
        Self {
            event_id: Uuid::new_v4().to_string(),
            aggregate_id,
            event_type,
            data,
            version: 1,
            occurred_at: chrono::Utc::now(),
        }
    }

    pub fn with_version(mut self, version: u32) -> Self {
        self.version = version;
        self
    }
}

#[async_trait]
impl DomainEvent for BasicEvent {
    fn event_id(&self) -> &str { &self.event_id }
    fn aggregate_id(&self) -> &str { &self.aggregate_id }
    fn event_type(&self) -> &str { &self.event_type }
    fn occurred_at(&self) -> &chrono::DateTime<chrono::Utc> { &self.occurred_at }
    fn data(&self) -> &EventData { &self.data }
    fn version(&self) -> u32 { self.version }
    
    fn clone_box(&self) -> Box<dyn DomainEvent> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestEvent {
        id: String,
        message: String,
    }

    impl TestEvent {
        pub fn new(message: String) -> Self {
            Self {
                id: Uuid::new_v4().to_string(),
                message,
            }
        }
    }

    #[async_trait]
    impl DomainEvent for TestEvent {
        fn event_id(&self) -> &str { &self.id }
        fn aggregate_id(&self) -> &str { "test-aggregate" }
        fn event_type(&self) -> &str { "TestEvent" }
        fn occurred_at(&self) -> &chrono::DateTime<chrono::Utc> {
            static TIMESTAMP: chrono::DateTime<chrono::Utc> = chrono::Utc::now();
            &TIMESTAMP
        }
        fn data(&self) -> &EventData {
            static DATA: EventData = EventData::new(serde_json::json!({"message": "test"}));
            &DATA
        }
        fn version(&self) -> u32 { 1 }
        
        fn clone_box(&self) -> Box<dyn DomainEvent> {
            Box::new(self.clone())
        }
    }

    struct TestEventHandler;

    #[async_trait]
    impl EventHandler<TestEvent> for TestEventHandler {
        async fn handle(&self, event: &TestEvent) -> Result<()> {
            tracing::info!("Handled test event: {}", event.message);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_event_bus() {
        let mut bus = EventBus::new();
        bus.subscribe(TestEventHandler).unwrap();

        let event = TestEvent::new("test message".to_string());
        bus.publish(event).await.unwrap();
    }

    #[tokio::test]
    async fn test_event_store() {
        let store = InMemoryEventStore::new();
        let event = TestEvent::new("test message".to_string());
        
        store.save_event(&event).await.unwrap();
        
        let events = store.get_events_by_aggregate_id("test-aggregate").await.unwrap();
        assert_eq!(events.len(), 1);
    }
}
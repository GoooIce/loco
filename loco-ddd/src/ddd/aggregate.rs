use crate::ddd::{entity::Entity, event::DomainEvent, Identifier, Version, DomainTimestamp};
use crate::Result;
use async_trait::async_trait;
use std::collections::VecDeque;

/// AggregateRoot trait - base for all domain aggregate roots
#[async_trait]
pub trait AggregateRoot: Entity + Send + Sync {
    type Event: DomainEvent;

    /// Get the list of uncommitted domain events
    fn get_uncommitted_events(&self) -> Vec<Self::Event>;

    /// Clear uncommitted events (typically after persisting)
    fn clear_uncommitted_events(&mut self);

    /// Mark all uncommitted events as committed
    fn mark_events_as_committed(&mut self);

    /// Get the aggregate version
    fn version(&self) -> &Version;

    /// Apply a domain event to this aggregate
    fn apply_event(&mut self, event: Self::Event) -> Result<()>;

    /// Get the aggregate's domain events (committed and uncommitted)
    fn get_domain_events(&self) -> Vec<Self::Event>;
}

/// BaseAggregate - provides common aggregate functionality
#[derive(Debug)]
pub struct BaseAggregate<T: Identifier, E: DomainEvent + Clone> {
    id: T,
    version: Version,
    created_at: DomainTimestamp,
    updated_at: DomainTimestamp,
    uncommitted_events: VecDeque<E>,
    committed_events: VecDeque<E>,
}

impl<T: Identifier, E: DomainEvent + Clone> BaseAggregate<T, E> {
    pub fn new(id: T) -> Self {
        let now = DomainTimestamp::new();
        Self {
            id,
            version: Version::new(),
            created_at: now.clone(),
            updated_at: now,
            uncommitted_events: VecDeque::new(),
            committed_events: VecDeque::new(),
        }
    }

    pub fn id(&self) -> &T {
        &self.id
    }

    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn created_at(&self) -> &DomainTimestamp {
        &self.created_at
    }

    pub fn updated_at(&self) -> &DomainTimestamp {
        &self.updated_at
    }

    pub fn increment_version(&mut self) {
        self.version.increment();
        self.updated_at = DomainTimestamp::new();
    }

    pub fn add_event(&mut self, event: E) {
        self.uncommitted_events.push_back(event);
    }

    pub fn get_uncommitted_events(&self) -> Vec<E> {
        self.uncommitted_events.iter().cloned().collect()
    }

    pub fn clear_uncommitted_events(&mut self) {
        self.uncommitted_events.clear();
    }

    pub fn mark_events_as_committed(&mut self) {
        while let Some(event) = self.uncommitted_events.pop_front() {
            self.committed_events.push_back(event);
        }
    }

    pub fn get_committed_events(&self) -> Vec<E> {
        self.committed_events.iter().cloned().collect()
    }

    pub fn get_all_events(&self) -> Vec<E> {
        let mut events = self.get_committed_events();
        events.extend(self.get_uncommitted_events());
        events
    }
}

/// AggregateRepository trait - specialized repository for aggregates
#[async_trait]
pub trait AggregateRepository<T: AggregateRoot>: Send + Sync {
    async fn save(&self, aggregate: &mut T) -> Result<()>;
    async fn find_by_id(&self, id: &T::Id) -> Result<Option<T>>;
    async fn delete(&self, id: &T::Id) -> Result<()>;
    async fn exists(&self, id: &T::Id) -> Result<bool>;
}

/// AggregateFactory trait - for creating aggregates
pub trait AggregateFactory<T: AggregateRoot> {
    fn create_aggregate(&self, id: T::Id) -> Result<T>;
    fn reconstitute_aggregate(&self, id: T::Id, events: Vec<T::Event>) -> Result<T>;
}

/// Aggregate lifecycle helper
pub struct AggregateLifecycle;

impl AggregateLifecycle {
    pub fn validate_aggregate_state<T: AggregateRoot>(aggregate: &T) -> Result<()> {
        // Validate basic aggregate state
        aggregate.validate()?;
        
        // Validate version consistency
        if AggregateRoot::version(aggregate).value() == 0 {
            return Err(crate::DddError::validation("Aggregate version cannot be zero"));
        }
        
        Ok(())
    }

    pub fn check_concurrency<T: AggregateRoot>(
        aggregate: &T,
        expected_version: &Version,
    ) -> Result<()> {
        if AggregateRoot::version(aggregate) != expected_version {
            return Err(crate::DddError::concurrency(
                "Aggregate version mismatch - concurrent modification detected",
            ));
        }
        Ok(())
    }

    pub fn ensure_events_consistency<T: AggregateRoot>(aggregate: &T) -> Result<()> {
        let events = aggregate.get_domain_events();
        
        // Check if event sequence is consistent
        for (i, event) in events.iter().enumerate() {
            if event.aggregate_id() != aggregate.id().as_string() {
                return Err(crate::DddError::validation(format!(
                    "Event {} has mismatched aggregate ID",
                    i
                )));
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ddd::event::{DomainEvent, EventData};

    #[derive(Debug, Clone)]
    struct TestEvent {
        event_id: String,
        aggregate_id: String,
        event_type: String,
        data: EventData,
        occurred_at: chrono::DateTime<chrono::Utc>,
    }

    impl DomainEvent for TestEvent {
        fn event_id(&self) -> &str {
            &self.event_id
        }

        fn aggregate_id(&self) -> &str {
            &self.aggregate_id
        }

        fn event_type(&self) -> &str {
            &self.event_type
        }

        fn occurred_at(&self) -> &chrono::DateTime<chrono::Utc> {
            &self.occurred_at
        }

        fn data(&self) -> &EventData {
            &self.data
        }

        fn version(&self) -> u32 {
            1
        }
    }

    #[derive(Debug)]
    struct TestAggregate {
        base: BaseAggregate<Uuid, TestEvent>,
        name: String,
    }

    impl Entity for TestAggregate {
        type Id = Uuid;

        fn id(&self) -> &Self::Id {
            self.base.id()
        }

        fn validate(&self) -> Result<()> {
            if self.name.is_empty() {
                return Err(crate::DddError::validation("Name cannot be empty"));
            }
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

    impl AggregateRoot for TestAggregate {
        type Event = TestEvent;

        fn get_uncommitted_events(&self) -> Vec<Self::Event> {
            self.base.get_uncommitted_events()
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
            self.base.add_event(event);
            self.base.increment_version();
            Ok(())
        }

        fn get_domain_events(&self) -> Vec<Self::Event> {
            self.base.get_all_events()
        }
    }

    #[test]
    fn test_aggregate_creation() {
        let id = Uuid::new_v4();
        let aggregate = TestAggregate {
            base: BaseAggregate::new(id),
            name: "Test Aggregate".to_string(),
        };

        assert_eq!(aggregate.id(), &id);
        assert_eq!(aggregate.version().value(), 0);
        assert!(aggregate.get_uncommitted_events().is_empty());
    }

    #[test]
    fn test_aggregate_event_management() {
        let id = Uuid::new_v4();
        let mut aggregate = TestAggregate {
            base: BaseAggregate::new(id),
            name: "Test Aggregate".to_string(),
        };

        let event = TestEvent {
            event_id: Uuid::new_v4().to_string(),
            aggregate_id: id.to_string(),
            event_type: "TestEvent".to_string(),
            data: EventData::new(serde_json::json!({"test": "data"})),
            occurred_at: chrono::Utc::now(),
        };

        aggregate.apply_event(event).unwrap();
        assert_eq!(aggregate.get_uncommitted_events().len(), 1);
        assert_eq!(aggregate.version().value(), 1);

        aggregate.mark_events_as_committed();
        assert!(aggregate.get_uncommitted_events().is_empty());
        assert_eq!(aggregate.get_committed_events().len(), 1);
    }

    #[test]
    fn test_aggregate_validation() {
        let id = Uuid::new_v4();
        let aggregate = TestAggregate {
            base: BaseAggregate::new(id),
            name: "".to_string(),
        };

        assert!(AggregateLifecycle::validate_aggregate_state(&aggregate).is_err());
    }
}
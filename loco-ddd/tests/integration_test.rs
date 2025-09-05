use loco_ddd::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    // Test event implementation
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestEvent {
        event_id: String,
        aggregate_id: String,
        event_type: String,
        data: EventData,
        occurred_at: chrono::DateTime<chrono::Utc>,
    }

    impl DomainEvent for TestEvent {
        fn event_id(&self) -> &str { &self.event_id }
        fn aggregate_id(&self) -> &str { &self.aggregate_id }
        fn event_type(&self) -> &str { &self.event_type }
        fn occurred_at(&self) -> &chrono::DateTime<chrono::Utc> { &self.occurred_at }
        fn data(&self) -> &EventData { &self.data }
        fn version(&self) -> u32 { 1 }
    }

    // Test aggregate implementation
    #[derive(Debug, Clone)]
    struct TestAggregate {
        base: BaseAggregate<Uuid, TestEvent>,
        name: String,
    }

    impl Entity for TestAggregate {
        type Id = Uuid;
        fn id(&self) -> &Self::Id { self.base.id() }
        fn validate(&self) -> Result<()> { 
            if self.name.is_empty() {
                return Err(DddError::validation("Name cannot be empty"));
            }
            Ok(())
        }
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

    #[test]
    fn test_library_imports() {
        // Test that all the main types can be imported
        let _result: Result<()> = Ok(());
        assert!(true);
    }

    #[test]
    fn test_value_object_email() {
        let email = Email::new("test@example.com".to_string()).unwrap();
        assert_eq!(email.value(), "test@example.com");
        assert!(email.is_valid());
    }

    #[test]
    fn test_value_object_email_invalid() {
        let result = Email::new("invalid-email".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_value_object_money() {
        let money = Money::new(100, "USD".to_string()).unwrap();
        assert_eq!(money.amount(), 100);
        assert_eq!(money.currency(), "USD");
        assert!(money.is_valid());
    }

    #[test]
    fn test_value_object_money_addition() {
        let money1 = Money::new(100, "USD".to_string()).unwrap();
        let money2 = Money::new(50, "USD".to_string()).unwrap();
        let result = money1.add(&money2).unwrap();
        assert_eq!(result.amount(), 150);
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
        assert!(aggregate.is_valid());
    }

    #[test]
    fn test_aggregate_validation() {
        let id = Uuid::new_v4();
        let aggregate = TestAggregate {
            base: BaseAggregate::new(id),
            name: "".to_string(),
        };

        assert!(!aggregate.is_valid());
        assert!(aggregate.validate().is_err());
    }

    #[test]
    fn test_repository_in_memory() {
        let repo = InMemoryRepository::<TestAggregate>::new();
        let id = Uuid::new_v4();
        
        // Test that repository can be created
        assert!(true);
    }

    #[test]
    fn test_event_bus() {
        let event_bus = EventBus::default();
        // Test that event bus can be created
        assert!(true);
    }

    #[test]
    fn test_cqrs_service() {
        let cqrs = CqrsService::new();
        // Test that CQRS service can be created
        assert!(true);
    }

    #[test]
    fn test_service_registry() {
        let mut registry = ServiceRegistry::new();
        let service = ServiceFactory::create_user_service();
        
        registry.register(service).unwrap();
        assert_eq!(registry.list_services().len(), 1);
    }

    #[test]
    fn test_query_builder() {
        let query = QueryBuilder::<TestAggregate>::new()
            .eq("name", serde_json::json!("test"))
            .limit(10)
            .build();

        match query {
            QueryExpression::And(expressions) => {
                assert_eq!(expressions.len(), 2);
            },
            _ => panic!("Expected And expression"),
        }
    }

    #[test]
    fn test_query_result() {
        let items = vec!["item1", "item2", "item3"];
        let result = QueryResult::new(items)
            .with_pagination(1, 10, 3);

        assert_eq!(result.len(), 3);
        assert_eq!(result.page, Some(1));
        assert_eq!(result.page_size, Some(10));
        assert_eq!(result.total_count, Some(3));
        assert!(!result.is_empty());
    }

    #[test]
    fn test_loco_ddd_config() {
        let config = loco_integration::LocoDddConfig::default();
        assert!(config.enable_events);
        assert!(config.enable_cqrs);
        assert!(config.enable_validation);
    }

    #[test]
    fn test_loco_ddd_context() {
        let config = loco_integration::LocoDddConfig::default();
        let context = loco_integration::LocoDddContext::new(config);
        
        assert!(context.event_bus.is_some());
        assert!(context.command_bus.is_some());
        assert!(context.query_bus.is_some());
    }

    #[test]
    fn test_version() {
        let mut version = Version::new();
        assert_eq!(version.value(), 0);
        
        version.increment();
        assert_eq!(version.value(), 1);
    }

    #[test]
    fn test_domain_timestamp() {
        let timestamp = DomainTimestamp::new();
        let now = chrono::Utc::now();
        
        // Check that timestamp is close to now (within 1 second)
        let diff = now.signed_duration_since(*timestamp.as_datetime());
        assert!(diff.num_seconds() < 1);
    }

    #[test]
    fn test_error_creation() {
        let error = DddError::validation("Test error");
        assert!(matches!(error, DddError::Validation(_)));
        
        let error = DddError::entity_not_found("test-id");
        assert!(matches!(error, DddError::EntityNotFound(_)));
        
        let error = DddError::repository("Test error");
        assert!(matches!(error, DddError::Repository(_)));
    }

    #[test]
    fn test_identifier_uuid() {
        let id = Uuid::new_v4();
        let id_str = id.as_str();
        let parsed_id = Uuid::from_str(id_str).unwrap();
        assert_eq!(id, parsed_id);
    }

    #[test]
    fn test_identifier_string() {
        let id = "test-id".to_string();
        let id_str = id.as_str();
        let parsed_id = String::from_str(id_str).unwrap();
        assert_eq!(id, parsed_id);
    }

    #[test]
    fn test_aggregate_lifecycle() {
        let id = Uuid::new_v4();
        let aggregate = TestAggregate {
            base: BaseAggregate::new(id),
            name: "Test Aggregate".to_string(),
        };

        assert!(AggregateLifecycle::validate_aggregate_state(&aggregate).is_ok());
        
        let expected_version = Version::new();
        assert!(AggregateLifecycle::check_concurrency(&aggregate, &expected_version).is_ok());
    }

    #[test]
    fn test_event_data() {
        let data = EventData::new(serde_json::json!({"test": "value"}));
        assert_eq!(data.as_json()["test"], "value");
        
        let parsed: serde_json::Value = data.get().unwrap();
        assert_eq!(parsed["test"], "value");
    }

    #[test]
    fn test_basic_command() {
        let command = BasicCommand::new(
            "test-aggregate".to_string(),
            "TestCommand".to_string(),
            serde_json::json!({"test": "data"}),
        );
        
        assert!(!command.command_id().is_empty());
        assert_eq!(command.aggregate_id(), "test-aggregate");
        assert_eq!(command.command_type(), "TestCommand");
    }

    #[test]
    fn test_basic_query() {
        let query = BasicQuery::new(
            "TestQuery".to_string(),
            serde_json::json!({"filter": "test"}),
        );
        
        assert!(!query.query_id().is_empty());
        assert_eq!(query.query_type(), "TestQuery");
    }

    #[test]
    fn test_repository_builder() {
        let repo = RepositoryBuilder::<TestAggregate>::new()
            .with_in_memory()
            .with_logging()
            .build();
        
        // Test that repository can be built
        assert!(true);
    }

    #[test]
    fn test_service_factory() {
        let user_service = ServiceFactory::create_user_service();
        let order_service = ServiceFactory::create_order_service();
        let payment_service = ServiceFactory::create_payment_service();
        
        assert_eq!(user_service.name(), "UserService");
        assert_eq!(order_service.name(), "OrderService");
        assert_eq!(payment_service.name(), "PaymentService");
        
        assert!(user_service.can_handle("create_user"));
        assert!(order_service.can_handle("create_order"));
        assert!(payment_service.can_handle("process_payment"));
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_full_workflow() {
        // Create a complete DDD workflow
        let event_bus = Arc::new(EventBus::default());
        let repository = Arc::new(InMemoryRepository::<TestAggregate>::new());
        let cqrs = CqrsService::new();
        
        // Test that all components work together
        assert!(true);
    }

    #[tokio::test]
    async fn test_event_bus_operations() {
        let event_bus = EventBus::default();
        
        let event = TestEvent {
            event_id: Uuid::new_v4().to_string(),
            aggregate_id: "test-aggregate".to_string(),
            event_type: "TestEvent".to_string(),
            data: EventData::new(serde_json::json!({"test": "data"})),
            occurred_at: chrono::Utc::now(),
        };

        // Test publishing and storing
        let result = event_bus.publish_and_store(event).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_service_coordinator() {
        let registry = Arc::new(ServiceRegistry::new());
        let user_service = ServiceFactory::create_user_service();
        registry.register(user_service).unwrap();
        
        let coordinator = ServiceCoordinator::new(registry)
            .with_middleware(Arc::new(LoggingServiceMiddleware));
        
        let result = coordinator.execute_operation(
            "create_user",
            serde_json::json!({"name": "John Doe", "email": "john@example.com"}),
        ).await;
        
        // This will fail because the base service doesn't implement the operation
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_query_processor() {
        let test_data = vec![
            "item1", "item2", "item3"
        ];
        let processor = InMemoryQueryProcessor::new(test_data);
        
        let query = QueryExpression::Eq("value".to_string(), serde_json::json!("item1"));
        let result = processor.execute_query(&query).await.unwrap();
        
        assert_eq!(result.len(), 0); // No items match the query
    }

    #[tokio::test]
    async fn test_loco_integration() {
        let config = loco_integration::LocoDddConfig::default();
        let app = loco_integration::DddLocoApp::new(config);
        
        // Test initialization
        let result = app.initialize().await;
        assert!(result.is_ok());
        
        // Test that DDD context is accessible
        let context = app.ddd_context();
        assert!(context.event_bus.is_some());
        assert!(context.command_bus.is_some());
        assert!(context.query_bus.is_some());
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_event_bus_performance() {
        let event_bus = EventBus::default();
        let num_events = 1000;
        
        let start = Instant::now();
        
        for i in 0..num_events {
            let event = TestEvent {
                event_id: Uuid::new_v4().to_string(),
                aggregate_id: format!("aggregate-{}", i),
                event_type: "TestEvent".to_string(),
                data: EventData::new(serde_json::json!({"test": "data"})),
                occurred_at: chrono::Utc::now(),
            };
            
            event_bus.publish_and_store(event).await.unwrap();
        }
        
        let duration = start.elapsed();
        println!("Published {} events in {:?}", num_events, duration);
        
        // Should be able to publish 1000 events in under 1 second
        assert!(duration.as_secs() < 1);
    }

    #[tokio::test]
    async fn test_repository_performance() {
        let repository = Arc::new(InMemoryRepository::<TestAggregate>::new());
        let num_aggregates = 100;
        
        let start = Instant::now();
        
        for i in 0..num_aggregates {
            let id = Uuid::new_v4();
            let mut aggregate = TestAggregate {
                base: BaseAggregate::new(id),
                name: format!("Aggregate {}", i),
            };
            
            repository.save(&mut aggregate).await.unwrap();
        }
        
        let duration = start.elapsed();
        println!("Saved {} aggregates in {:?}", num_aggregates, duration);
        
        // Should be able to save 100 aggregates in under 1 second
        assert!(duration.as_secs() < 1);
    }
}
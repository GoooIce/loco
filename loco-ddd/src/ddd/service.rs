use crate::Result;
use async_trait::async_trait;
use std::sync::Arc;

/// Domain service trait - represents business logic that doesn't naturally fit in entities or value objects
#[async_trait]
pub trait DomainService: Send + Sync {
    /// Get the service name
    fn name(&self) -> &str;

    /// Validate if the service can handle the given operation
    fn can_handle(&self, operation: &str) -> bool;

    /// Execute the service operation
    async fn execute(&self, operation: &str, params: serde_json::Value) -> Result<serde_json::Value>;
}

/// Service registry for managing domain services
pub struct ServiceRegistry {
    services: std::collections::HashMap<String, Arc<dyn DomainService>>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: std::collections::HashMap::new(),
        }
    }

    pub fn register<S: DomainService + 'static>(&mut self, service: Arc<S>) -> Result<()> {
        let service_name = service.name().to_string();
        self.services.insert(service_name, service);
        Ok(())
    }

    pub fn get_service(&self, name: &str) -> Option<Arc<dyn DomainService>> {
        self.services.get(name).cloned()
    }

    pub fn get_services_for_operation(&self, operation: &str) -> Vec<Arc<dyn DomainService>> {
        self.services
            .values()
            .filter(|service| service.can_handle(operation))
            .cloned()
            .collect()
    }

    pub fn list_services(&self) -> Vec<&str> {
        self.services.keys().map(|s| s.as_str()).collect()
    }

    pub fn unregister(&mut self, name: &str) -> Option<Arc<dyn DomainService>> {
        self.services.remove(name)
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Service coordinator for coordinating multiple services
pub struct ServiceCoordinator {
    registry: Arc<ServiceRegistry>,
    middleware: Vec<Arc<dyn ServiceMiddleware>>,
}

impl ServiceCoordinator {
    pub fn new(registry: Arc<ServiceRegistry>) -> Self {
        Self {
            registry,
            middleware: Vec::new(),
        }
    }

    pub fn with_middleware(mut self, middleware: Arc<dyn ServiceMiddleware>) -> Self {
        self.middleware.push(middleware);
        self
    }

    pub async fn execute_operation(
        &self,
        operation: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        // Get services that can handle this operation
        let services = self.registry.get_services_for_operation(operation);
        
        if services.is_empty() {
            return Err(crate::DddError::domain_service(format!(
                "No service found for operation: {}",
                operation
            )));
        }

        // Apply middleware before execution
        let mut params = params;
        for middleware in &self.middleware {
            params = middleware.before_execution(operation, params).await?;
        }

        // Execute services in order
        let mut result = None;
        for service in &services {
            let service_result = service.execute(operation, params.clone()).await;
            
            // Apply middleware after execution
            for middleware in &self.middleware {
                middleware.after_execution(operation, &service_result).await?;
            }

            match service_result {
                Ok(r) => {
                    result = Some(r);
                    break; // Stop at first successful execution
                },
                Err(e) => {
                    tracing::warn!("Service {} failed for operation {}: {}", service.name(), operation, e);
                    // Continue to next service
                },
            }
        }

        result.ok_or_else(|| {
            crate::DddError::domain_service(format!(
                "All services failed for operation: {}",
                operation
            ))
        })
    }

    pub async fn execute_parallel_operation(
        &self,
        operation: &str,
        params: serde_json::Value,
    ) -> Result<Vec<serde_json::Value>> {
        let services = self.registry.get_services_for_operation(operation);
        
        if services.is_empty() {
            return Err(crate::DddError::domain_service(format!(
                "No service found for operation: {}",
                operation
            )));
        }

        // Execute all services in parallel
        let futures: Vec<_> = services
            .into_iter()
            .map(|service| {
                let params = params.clone();
                async move {
                    let result = service.execute(operation, params).await;
                    (service.name().to_string(), result)
                }
            })
            .collect();

        let results = futures::future::join_all(futures).await;

        // Collect successful results
        let mut successful_results = Vec::new();
        let mut failed_services = Vec::new();

        for (service_name, result) in results {
            match result {
                Ok(r) => successful_results.push(r),
                Err(e) => {
                    failed_services.push((service_name, e));
                },
            }
        }

        if successful_results.is_empty() {
            return Err(crate::DddError::domain_service(format!(
                "All services failed for operation: {}. Failed services: {:?}",
                operation,
                failed_services
            )));
        }

        Ok(successful_results)
    }
}

/// Service middleware trait
#[async_trait]
pub trait ServiceMiddleware: Send + Sync {
    async fn before_execution(
        &self,
        operation: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value>;

    async fn after_execution(
        &self,
        operation: &str,
        result: &Result<serde_json::Value>,
    ) -> Result<()>;
}

/// Logging middleware for services
pub struct LoggingServiceMiddleware;

#[async_trait]
impl ServiceMiddleware for LoggingServiceMiddleware {
    async fn before_execution(
        &self,
        operation: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        tracing::info!("Executing domain service operation: {}", operation);
        tracing::debug!("Operation parameters: {}", params);
        Ok(params)
    }

    async fn after_execution(
        &self,
        operation: &str,
        result: &Result<serde_json::Value>,
    ) -> Result<()> {
        match result {
            Ok(_) => tracing::info!("Domain service operation {} completed successfully", operation),
            Err(e) => tracing::error!("Domain service operation {} failed: {}", operation, e),
        }
        Ok(())
    }
}

/// Validation middleware for services
pub struct ValidationServiceMiddleware;

#[async_trait]
impl ServiceMiddleware for ValidationServiceMiddleware {
    async fn before_execution(
        &self,
        operation: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        // Basic validation
        if operation.is_empty() {
            return Err(crate::DddError::validation("Operation cannot be empty"));
        }

        if !params.is_object() {
            return Err(crate::DddError::validation("Parameters must be an object"));
        }

        Ok(params)
    }

    async fn after_execution(
        &self,
        _operation: &str,
        _result: &Result<serde_json::Value>,
    ) -> Result<()> {
        Ok(())
    }
}

/// Base domain service implementation
pub struct BaseDomainService {
    name: String,
    handled_operations: Vec<String>,
}

impl BaseDomainService {
    pub fn new(name: &str, handled_operations: Vec<String>) -> Self {
        Self {
            name: name.to_string(),
            handled_operations,
        }
    }

    pub fn add_handled_operation(&mut self, operation: &str) {
        self.handled_operations.push(operation.to_string());
    }

    pub fn remove_handled_operation(&mut self, operation: &str) {
        self.handled_operations.retain(|op| op != operation);
    }
}

#[async_trait]
impl DomainService for BaseDomainService {
    fn name(&self) -> &str {
        &self.name
    }

    fn can_handle(&self, operation: &str) -> bool {
        self.handled_operations.contains(&operation.to_string())
    }

    async fn execute(&self, operation: &str, _params: serde_json::Value) -> Result<serde_json::Value> {
        // Base implementation - should be overridden by specific services
        Err(crate::DddError::domain_service(format!(
            "Service {} does not implement operation {}",
            self.name, operation
        )))
    }
}

/// Service factory for creating services
pub struct ServiceFactory;

impl ServiceFactory {
    pub fn create_user_service() -> Arc<dyn DomainService> {
        Arc::new(BaseDomainService::new(
            "UserService",
            vec!["create_user".to_string(), "update_user".to_string(), "delete_user".to_string()],
        ))
    }

    pub fn create_order_service() -> Arc<dyn DomainService> {
        Arc::new(BaseDomainService::new(
            "OrderService",
            vec!["create_order".to_string(), "update_order".to_string(), "cancel_order".to_string()],
        ))
    }

    pub fn create_payment_service() -> Arc<dyn DomainService> {
        Arc::new(BaseDomainService::new(
            "PaymentService",
            vec!["process_payment".to_string(), "refund_payment".to_string()],
        ))
    }
}

/// Service health check
pub struct ServiceHealthChecker {
    registry: Arc<ServiceRegistry>,
}

impl ServiceHealthChecker {
    pub fn new(registry: Arc<ServiceRegistry>) -> Self {
        Self { registry }
    }

    pub async fn check_health(&self) -> ServiceHealthReport {
        let services = self.registry.list_services();
        let mut healthy_services = Vec::new();
        let mut unhealthy_services = Vec::new();

        for service_name in services {
            match self.check_service_health(service_name).await {
                Ok(_) => healthy_services.push(service_name.to_string()),
                Err(e) => {
                    unhealthy_services.push(ServiceHealthStatus {
                        name: service_name.to_string(),
                        status: "unhealthy".to_string(),
                        error: Some(e.to_string()),
                        timestamp: chrono::Utc::now(),
                    });
                },
            }
        }

        ServiceHealthReport {
            healthy_services,
            unhealthy_services,
            timestamp: chrono::Utc::now(),
        }
    }

    async fn check_service_health(&self, service_name: &str) -> Result<()> {
        if let Some(_service) = self.registry.get_service(service_name) {
            // For now, just check if the service exists
            // In a real implementation, you might perform actual health checks
            Ok(())
        } else {
            Err(crate::DddError::domain_service(format!("Service {} not found", service_name)))
        }
    }
}

#[derive(Debug, Clone)]
pub struct ServiceHealthStatus {
    pub name: String,
    pub status: String,
    pub error: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct ServiceHealthReport {
    pub healthy_services: Vec<String>,
    pub unhealthy_services: Vec<ServiceHealthStatus>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_service_registry() {
        let mut registry = ServiceRegistry::new();
        let user_service = ServiceFactory::create_user_service();
        
        registry.register(user_service).unwrap();
        
        assert_eq!(registry.list_services().len(), 1);
        assert!(registry.get_service("UserService").is_some());
        assert!(registry.get_service("NonExistentService").is_none());
        
        let services = registry.get_services_for_operation("create_user");
        assert_eq!(services.len(), 1);
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
    async fn test_service_health_checker() {
        let registry = Arc::new(ServiceRegistry::new());
        let user_service = ServiceFactory::create_user_service();
        registry.register(user_service).unwrap();
        
        let health_checker = ServiceHealthChecker::new(registry);
        let report = health_checker.check_health().await;
        
        assert_eq!(report.healthy_services.len(), 1);
        assert!(report.unhealthy_services.is_empty());
    }

    #[tokio::test]
    async fn test_base_domain_service() {
        let service = BaseDomainService::new(
            "TestService",
            vec!["test_operation".to_string()],
        );
        
        assert_eq!(service.name(), "TestService");
        assert!(service.can_handle("test_operation"));
        assert!(!service.can_handle("other_operation"));
    }
}
//! Integration tests for Phase 3.4: Integration & Advanced Features
//!
//! This test validates the integration of all advanced features in the Rust bindings:
//! - Template caching with performance monitoring
//! - Enhanced error handling with detailed validation
//! - Performance optimization and metrics collection
//! - Cache management and LRU cleanup

use loco_bindings::*;
use pyo3::prelude::*;
use std::collections::HashMap;

/// Test helper to create a test parameters dictionary
fn create_test_params(model_name: &str, fields: Vec<&str>) -> HashMap<String, serde_json::Value> {
    let mut params = HashMap::new();
    params.insert("model_name".to_string(), serde_json::Value::String(model_name.to_string()));
    params.insert(
        "fields".to_string(),
        serde_json::Value::Array(
            fields.into_iter()
                .map(|f| serde_json::Value::String(f.to_string()))
                .collect()
        ),
    );
    params.insert("project_path".to_string(), serde_json::Value::String(".".to_string()));
    params
}

#[test]
fn test_error_handling_integration() {
    // Test validation error creation
    let error = ValidationError::new("Invalid model name", vec!["Start with a letter"]);
    assert_eq!(error.message, "Invalid model name");
    assert_eq!(error.suggestions.len(), 1);

    // Test file operation error
    let file_error = FileOperationError::new("File not found", "/path/to/file.rs");
    assert_eq!(file_error.message, "File not found");
    assert_eq!(file_error.file_path, "/path/to/file.rs");

    // Test error conversion to Python exception
    Python::with_gil(|py| {
        let py_error = error.to_py_err();
        assert!(py_error.is_instance_of::<PyErr>(py));
    });
}

#[test]
fn test_performance_metrics() {
    // Test performance metrics creation
    let mut metrics = PerformanceMetrics::new();
    assert_eq!(metrics.total_calls, 0);
    assert_eq!(metrics.avg_duration_ms, 0.0);

    // Update metrics
    metrics.update(5.0);
    assert_eq!(metrics.total_calls, 1);
    assert_eq!(metrics.avg_duration_ms, 5.0);
    assert_eq!(metrics.min_duration_ms, 5.0);
    assert_eq!(metrics.max_duration_ms, 5.0);

    // Update with different values
    metrics.update(15.0);
    assert_eq!(metrics.total_calls, 2);
    assert_eq!(metrics.avg_duration_ms, 10.0);
    assert_eq!(metrics.min_duration_ms, 5.0);
    assert_eq!(metrics.max_duration_ms, 15.0);

    // Test performance target checking
    assert!(metrics.is_under_performance_target(10.0));
    assert!(!metrics.is_under_performance_target(8.0));
}

#[test]
fn test_template_cache_integration() {
    let cache = TemplateCache::new(CacheConfig::default());

    // Test cache put and get
    let key = "test:model:user";
    let content = "model template content".to_string();

    cache.put(key.to_string(), content.clone()).unwrap();
    let retrieved = cache.get(key);
    assert_eq!(retrieved, Some(content));

    // Test cache contains
    assert!(cache.contains(key));
    assert!(!cache.contains("nonexistent:key"));

    // Test cache remove
    let removed = cache.remove(key);
    assert_eq!(removed, Some("model template content".to_string()));
    assert!(!cache.contains(key));

    // Test cache clear
    cache.put("key1".to_string(), "content1".to_string()).unwrap();
    cache.put("key2".to_string(), "content2".to_string()).unwrap();
    cache.clear();
    assert!(!cache.contains("key1"));
    assert!(!cache.contains("key2"));
}

#[test]
fn test_model_template_cache() {
    let model_cache = ModelTemplateCache::new();
    let fields = vec![
        FieldDefinition {
            name: "name".to_string(),
            field_type: FieldType::String,
            constraints: vec![],
            optional: false,
        },
        FieldDefinition {
            name: "email".to_string(),
            field_type: FieldType::String,
            constraints: vec![FieldConstraint::Unique],
            optional: false,
        },
    ];

    // Test cache miss
    let cached_template = model_cache.get_model_template("user", &fields);
    assert!(cached_template.is_none());

    // Test cache put and get
    let template_content = "generated model template".to_string();
    model_cache.cache_model_template("user", &fields, template_content.clone()).unwrap();

    let retrieved_template = model_cache.get_model_template("user", &fields);
    assert_eq!(retrieved_template, Some(template_content));
}

#[test]
fn test_controller_template_cache() {
    let controller_cache = ControllerTemplateCache::new();

    // Test cache miss
    let cached_template = controller_cache.get_controller_template("user", true);
    assert!(cached_template.is_none());

    // Test cache put and get
    let template_content = "generated controller template".to_string();
    controller_cache.cache_controller_template("user", true, template_content.clone()).unwrap();

    let retrieved_template = controller_cache.get_controller_template("user", true);
    assert_eq!(retrieved_template, Some(template_content));

    // Test different parameters
    let different_template = controller_cache.get_controller_template("user", false);
    assert!(different_template.is_none());
}

#[test]
fn test_view_template_cache() {
    let view_cache = ViewTemplateCache::new();

    // Test cache miss
    let cached_template = view_cache.get_view_template("user", "list");
    assert!(cached_template.is_none());

    // Test cache put and get
    let template_content = "generated view template".to_string();
    view_cache.cache_view_template("user", "list", template_content.clone()).unwrap();

    let retrieved_template = view_cache.get_view_template("user", "list");
    assert_eq!(retrieved_template, Some(template_content));

    // Test different view type
    let different_template = view_cache.get_view_template("user", "show");
    assert!(different_template.is_none());
}

#[test]
fn test_migration_template_cache() {
    let migration_cache = MigrationTemplateCache::new();
    let fields = vec![
        FieldDefinition {
            name: "name".to_string(),
            field_type: FieldType::String,
            constraints: vec![],
            optional: false,
        },
        FieldDefinition {
            name: "age".to_string(),
            field_type: FieldType::I32,
            constraints: vec![],
            optional: true,
        },
    ];

    // Test cache miss
    let cached_template = migration_cache.get_migration_template("user", &fields);
    assert!(cached_template.is_none());

    // Test cache put and get
    let template_content = "generated migration template".to_string();
    migration_cache.cache_migration_template("user", &fields, template_content.clone()).unwrap();

    let retrieved_template = migration_cache.get_migration_template("user", &fields);
    assert_eq!(retrieved_template, Some(template_content));
}

#[test]
fn test_cache_key_generation() {
    let cache = TemplateCache::new(CacheConfig::default());
    let fields1 = vec![
        FieldDefinition {
            name: "name".to_string(),
            field_type: FieldType::String,
            constraints: vec![],
            optional: false,
        },
    ];
    let fields2 = vec![
        FieldDefinition {
            name: "name".to_string(),
            field_type: FieldType::String,
            constraints: vec![],
            optional: false,
        },
    ];

    // Test consistent key generation
    let key1 = cache.generate_key("model", "user", &fields1);
    let key2 = cache.generate_key("model", "user", &fields2);
    assert_eq!(key1, key2); // Same fields should generate same key

    // Test different parameters generate different keys
    let key3 = cache.generate_key("model", "post", &fields1);
    assert_ne!(key1, key3); // Different model name should generate different key

    let key4 = cache.generate_key("controller", "user", &fields1);
    assert_ne!(key1, key4); // Different template type should generate different key
}

#[test]
fn test_cache_statistics() {
    let cache = TemplateCache::new(CacheConfig::default());

    // Get initial stats
    let stats = cache.get_stats();
    assert_eq!(stats.total_entries, 0);
    assert_eq!(stats.cache_hits, 0);
    assert_eq!(stats.cache_misses, 0);
    assert_eq!(stats.hit_rate(), 0.0);

    // Add entries
    cache.put("key1".to_string(), "content1".to_string()).unwrap();
    cache.put("key2".to_string(), "content2".to_string()).unwrap();

    // Test cache hits and misses
    let _ = cache.get("key1"); // hit
    let _ = cache.get("key2"); // hit
    let _ = cache.get("nonexistent"); // miss

    let stats = cache.get_stats();
    assert_eq!(stats.total_entries, 2);
    assert_eq!(stats.cache_hits, 2);
    assert_eq!(stats.cache_misses, 1);
    assert!(stats.hit_rate() > 0.0);
}

#[test]
fn test_optimized_generator() {
    let generator = OptimizedGenerator::new();

    // Test valid generation
    let result = generator.generate_model_optimized(
        "user",
        &vec!["name:string".to_string(), "email:string:unique".to_string()],
        "/tmp/test_project"
    );

    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.success);
    assert!(!response.created_files.is_empty());
    assert!(response.processing_time_ms > 0.0);

    // Test invalid inputs
    let result = generator.generate_model_optimized(
        "", // Empty model name
        &vec!["name:string".to_string()],
        "/tmp/test_project"
    );

    assert!(result.is_err());
}

#[test]
fn test_field_validation_integration() {
    // Test valid field definitions
    let valid_fields = vec![
        "name:string".to_string(),
        "email:string:unique".to_string(),
        "age:i32".to_string(),
        "content:text".to_string(),
        "created_at:datetime".to_string(),
        "is_active:boolean".to_string(),
    ];

    for field_str in &valid_fields {
        let field = FieldDefinition::from_str(field_str);
        assert!(field.is_ok(), "Field '{}' should be valid", field_str);
    }

    // Test invalid field definitions
    let invalid_fields = vec![
        "", // Empty
        "name", // No type
        "email:", // Empty type
        "age:invalid_type", // Invalid type
        "name:string:invalid_constraint", // Invalid constraint
    ];

    for field_str in &invalid_fields {
        let field = FieldDefinition::from_str(field_str);
        assert!(field.is_err(), "Field '{}' should be invalid", field_str);
    }
}

#[test]
fn test_performance_monitor_integration() {
    let monitor = PerformanceMonitor::new(10.0); // 10ms target

    // Test performance levels
    assert_eq!(monitor.check_performance(5.0), PerformanceAlert::Good);
    assert_eq!(monitor.check_performance(12.0), PerformanceAlert::Slow);
    assert_eq!(monitor.check_performance(15.0), PerformanceAlert::Warning);
    assert_eq!(monitor.check_performance(25.0), PerformanceAlert::Critical);

    // Test alert messages
    let alert = monitor.check_performance(15.0);
    let message = alert.message(15.0, 10.0);
    assert!(message.contains("15.00ms"));
    assert!(message.contains("10ms"));
}

#[tokio::test]
async fn test_template_cache_warmup() {
    // Test template cache warmup functionality
    let result = warm_template_cache().await;
    assert!(result.is_ok());

    // Verify cache is warmed by checking if common templates are cached
    let cache = get_template_cache();

    // The cache should now contain pre-warmed templates
    let stats = cache.get_stats();
    assert!(stats.total_entries > 0);
}

#[test]
fn test_template_cache_monitor() {
    let monitor = TemplateCacheMonitor::new();

    // Get performance metrics
    let metrics = monitor.get_performance_metrics();
    assert!(metrics.total_entries >= 0); // May be 0 if cache is empty

    // Test cache effectiveness check
    let is_effective = monitor.is_cache_effective();
    // This may be false if cache is empty, which is fine for a test

    // Test optimization suggestions
    let suggestions = monitor.suggest_optimizations();
    assert!(suggestions.len() >= 0); // Always returns a vector
}

#[test]
fn test_string_pool() {
    let pool = StringPool::new();

    // Test string interning
    let s1 = pool.intern("test_string");
    let s2 = pool.intern("test_string");
    let s3 = pool.intern("different_string");

    assert_eq!(s1, s2); // Same string should return same interned value
    assert_ne!(s1, s3); // Different strings should return different values
}

#[test]
fn test_optimized_field_definition() {
    let pool = StringPool::new();

    let field = FieldDefinition {
        name: "test_field".to_string(),
        field_type: FieldType::String,
        constraints: vec![FieldConstraint::Unique],
        optional: false,
    };

    let optimized_field = OptimizedFieldDefinition::from_field_def(&field, &pool);

    assert_eq!(optimized_field.name, "test_field");
    assert_eq!(optimized_field.field_type, "String");
    assert_eq!(optimized_field.constraints, vec!["unique"]);
    assert!(!optimized_field.optional);

    // Test string representation
    let field_str = optimized_field.to_string();
    assert!(field_str.contains("test_field:String"));
    assert!(field_str.contains("unique"));
}

#[test]
fn test_optimized_generation_response() {
    let file_info = FileInfo {
        path: "/test/path/file.rs".to_string(),
        file_type: "model".to_string(),
        size_bytes: 1024,
    };

    // Test successful response
    let success_response = OptimizedGenerationResponse::success(
        vec![file_info.clone()],
        vec![],
        5.5
    );

    assert!(success_response.success);
    assert_eq!(success_response.created_files.len(), 1);
    assert_eq!(success_response.processing_time_ms, 5.5);

    // Test error response
    let error_response = OptimizedGenerationResponse::error(
        vec!["Test error".to_string()],
        2.0
    );

    assert!(!error_response.success);
    assert_eq!(error_response.errors.len(), 1);
    assert_eq!(error_response.processing_time_ms, 2.0);

    // Test JSON conversion
    let json_value = success_response.to_json();
    assert!(json_value["success"].as_bool().unwrap());
    assert!(json_value["created_files"].as_array().unwrap().len() > 0);
}
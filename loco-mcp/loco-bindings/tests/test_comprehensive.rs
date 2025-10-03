//! Comprehensive unit tests for loco-bindings
//!
//! This test suite provides comprehensive coverage of all core functionality
//! including error handling, validation, template generation, and performance.

use loco_bindings::*;
use pyo3::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Test helper to create a temporary loco project structure
fn create_temp_loco_project() -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_path = temp_dir.path();

    // Create basic loco project structure
    fs::create_dir_all(project_path.join("src/models")).unwrap();
    fs::create_dir_all(project_path.join("src/controllers")).unwrap();
    fs::create_dir_all(project_path.join("src/views")).unwrap();
    fs::create_dir_all(project_path.join("src/routes")).unwrap();
    fs::create_dir_all(project_path.join("migration/src")).unwrap();

    // Create minimal Cargo.toml
    let cargo_toml = r#"[package]
name = "test-app"
version = "0.1.0"
edition = "2021"

[dependencies]
loco-rs = "0.3"
sea-orm = "0.12"
serde = { version = "1.0", features = ["derive"] }
"#;
    fs::write(project_path.join("Cargo.toml"), cargo_toml).unwrap();

    temp_dir
}

/// Test helper to create test parameters
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
    params
}

#[cfg(test)]
mod model_generation_tests {
    use super::*;

    #[test]
    fn test_generate_model_basic() {
        let temp_project = create_temp_loco_project();
        let mut params = create_test_params("user", vec!["name:string", "email:string"]);
        params.insert("project_path".to_string(), serde_json::Value::String(temp_project.path().to_string_lossy().to_string()));

        let result = generate_model(params);
        assert!(result["success"].as_bool().unwrap());

        let created_files = result["created_files"].as_array().unwrap();
        assert_eq!(created_files.len(), 2); // model + migration

        // Check that files were actually created
        assert!(temp_project.path().join("src/models/user.rs").exists());
        assert!(temp_project.path().join("migration/src").join("m_").join("_create_users.rs").exists());
    }

    #[test]
    fn test_generate_model_with_constraints() {
        let temp_project = create_temp_loco_project();
        let mut params = create_test_params("user", vec![
            "name:string",
            "email:string:unique",
            "age:i32:nullable",
            "status:string:default:active"
        ]);
        params.insert("project_path".to_string(), serde_json::Value::String(temp_project.path().to_string_lossy().to_string()));

        let result = generate_model(params);
        assert!(result["success"].as_bool().unwrap());

        let created_files = result["created_files"].as_array().unwrap();
        assert_eq!(created_files.len(), 2);
    }

    #[test]
    fn test_generate_model_invalid_name() {
        let params = create_test_params("123invalid", vec!["name:string"]);

        let result = generate_model(params);
        assert!(!result["success"].as_bool().unwrap());

        let errors = result["errors"].as_array().unwrap();
        assert!(!errors.is_empty());
        let error_msg = errors[0].as_str().unwrap();
        assert!(error_msg.contains("invalid") || error_msg.contains("Invalid"));
    }

    #[test]
    fn test_generate_model_duplicate_name() {
        let temp_project = create_temp_loco_project();
        let project_path = temp_project.path().to_string_lossy().to_string();

        // Create first model
        let mut params1 = create_test_params("user", vec!["name:string"]);
        params1.insert("project_path".to_string(), serde_json::Value::String(project_path.clone()));
        let result1 = generate_model(params1);
        assert!(result1["success"].as_bool().unwrap());

        // Try to create duplicate
        let mut params2 = create_test_params("user", vec!["email:string"]);
        params2.insert("project_path".to_string(), serde_json::Value::String(project_path));
        let result2 = generate_model(params2);
        assert!(!result2["success"].as_bool().unwrap());

        let errors = result2["errors"].as_array().unwrap();
        let error_msg = errors[0].as_str().unwrap();
        assert!(error_msg.contains("already exists"));
    }

    #[test]
    fn test_generate_model_invalid_project() {
        let mut params = create_test_params("user", vec!["name:string"]);
        params.insert("project_path".to_string(), serde_json::Value::String("/nonexistent/path".to_string()));

        let result = generate_model(params);
        assert!(!result["success"].as_bool().unwrap());

        let errors = result["errors"].as_array().unwrap();
        assert!(!errors.is_empty());
    }
}

#[cfg(test)]
mod scaffold_generation_tests {
    use super::*;

    #[test]
    fn test_generate_scaffold_full() {
        let temp_project = create_temp_loco_project();
        let mut params = create_test_params("blog_post", vec![
            "title:string",
            "content:text",
            "published:boolean"
        ]);
        params.insert("project_path".to_string(), serde_json::Value::String(temp_project.path().to_string_lossy().to_string()));
        params.insert("include_views".to_string(), serde_json::Value::Bool(true));
        params.insert("include_controllers".to_string(), serde_json::Value::Bool(true));
        params.insert("api_only".to_string(), serde_json::Value::Bool(false));

        let result = generate_scaffold(params);
        assert!(result["success"].as_bool().unwrap());

        let created_files = result["created_files"].as_array().unwrap();
        assert!(created_files.len() >= 4); // model, migration, controller, views

        // Check specific files
        assert!(temp_project.path().join("src/models/blog_post.rs").exists());
        assert!(temp_project.path().join("src/controllers/blog_posts.rs").exists());
        assert!(temp_project.path().join("src/views/blog_posts").exists());
    }

    #[test]
    fn test_generate_scaffold_api_only() {
        let temp_project = create_temp_loco_project();
        let mut params = create_test_params("api_key", vec![
            "key:string:unique",
            "name:string",
            "permissions:json"
        ]);
        params.insert("project_path".to_string(), serde_json::Value::String(temp_project.path().to_string_lossy().to_string()));
        params.insert("api_only".to_string(), serde_json::Value::Bool(true));

        let result = generate_scaffold(params);
        assert!(result["success"].as_bool().unwrap());

        let created_files = result["created_files"].as_array().unwrap();
        assert!(created_files.len() >= 3); // model, migration, controller

        // Check that views directory was not created
        assert!(!temp_project.path().join("src/views/api_key").exists());
    }

    #[test]
    fn test_generate_scaffold_controller_only() {
        let temp_project = create_temp_loco_project();
        let mut params = create_test_params("user", vec!["name:string"]);
        params.insert("project_path".to_string(), serde_json::Value::String(temp_project.path().to_string_lossy().to_string()));
        params.insert("include_views".to_string(), serde_json::Value::Bool(false));
        params.insert("include_controllers".to_string(), serde_json::Value::Bool(true));

        let result = generate_scaffold(params);
        assert!(result["success"].as_bool().unwrap());

        // Should have model, migration, controller but no views
        assert!(temp_project.path().join("src/models/user.rs").exists());
        assert!(temp_project.path().join("src/controllers/users.rs").exists());
        assert!(!temp_project.path().join("src/views/user").exists());
    }
}

#[cfg(test)]
mod controller_generation_tests {
    use super::*;

    #[test]
    fn test_generate_controller_for_existing_model() {
        let temp_project = create_temp_loco_project();
        let project_path = temp_project.path().to_string_lossy().to_string();

        // First create a model
        let mut model_params = create_test_params("user", vec!["name:string", "email:string"]);
        model_params.insert("project_path".to_string(), serde_json::Value::String(project_path.clone()));
        let model_result = generate_model(model_params);
        assert!(model_result["success"].as_bool().unwrap());

        // Then generate controller
        let mut controller_params = HashMap::new();
        controller_params.insert("model_name".to_string(), serde_json::Value::String("user".to_string()));
        controller_params.insert("actions".to_string(), serde_json::Value::Array(vec![
            serde_json::Value::String("index".to_string()),
            serde_json::Value::String("show".to_string())
        ]));
        controller_params.insert("project_path".to_string(), serde_json::Value::String(project_path));

        let result = generate_controller_view(controller_params);
        assert!(result["success"].as_bool().unwrap());

        assert!(temp_project.path().join("src/controllers/user.rs").exists());
        assert!(temp_project.path().join("src/views/user").exists());
    }

    #[test]
    fn test_generate_controller_nonexistent_model() {
        let temp_project = create_temp_loco_project();
        let mut params = HashMap::new();
        params.insert("model_name".to_string(), serde_json::Value::String("nonexistent".to_string()));
        params.insert("project_path".to_string(), serde_json::Value::String(temp_project.path().to_string_lossy().to_string()));

        let result = generate_controller_view(params);
        assert!(!result["success"].as_bool().unwrap());

        let errors = result["errors"].as_array().unwrap();
        let error_msg = errors[0].as_str().unwrap();
        assert!(error_msg.contains("not found") || error_msg.contains("doesn't exist"));
    }
}

#[cfg(test)]
mod field_validation_tests {
    use super::*;

    #[test]
    fn test_field_types() {
        let valid_fields = vec![
            "name:string",
            "content:text",
            "count:i32",
            "big_number:i64",
            "price:f32",
            "precise:f64",
            "active:boolean",
            "created_at:datetime",
            "id:uuid",
            "metadata:json"
        ];

        for field in &valid_fields {
            let field_def = FieldDefinition::from_str(field);
            assert!(field_def.is_ok(), "Field '{}' should be valid", field);
        }
    }

    #[test]
    fn test_field_constraints() {
        let valid_fields = vec![
            "email:string:unique",
            "description:text:nullable",
            "status:string:default:active",
            "user_id:i64:foreign_key:users",
            "name:string"
        ];

        for field in &valid_fields {
            let field_def = FieldDefinition::from_str(field);
            assert!(field_def.is_ok(), "Field '{}' should be valid", field);
        }
    }

    #[test]
    fn test_invalid_field_formats() {
        let invalid_fields = vec![
            "",                    // Empty
            "name",               // No type
            "email:",             // Empty type
            "age:invalid_type",   // Invalid type
            "name:string:invalid_constraint", // Invalid constraint
            "123name:string",     // Invalid name
            "name-with-dash:string", // Invalid name
        ];

        for field in &invalid_fields {
            let field_def = FieldDefinition::from_str(field);
            assert!(field_def.is_err(), "Field '{}' should be invalid", field);
        }
    }

    #[test]
    fn test_field_list_validation() {
        let valid_fields = vec![
            "name:string".to_string(),
            "email:string:unique".to_string(),
            "age:i32".to_string()
        ];

        let result = validate_field_list(&valid_fields);
        assert!(result.is_ok());
    }

    #[test]
    fn test_duplicate_field_names() {
        let duplicate_fields = vec![
            "name:string".to_string(),
            "name:string".to_string(),
        ];

        let result = validate_field_list(&duplicate_fields);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("duplicate") || error_msg.contains("Duplicate"));
    }

    #[test]
    fn test_multiple_primary_keys() {
        let multiple_pk_fields = vec![
            "id:i32:primary_key".to_string(),
            "uuid:uuid:primary_key".to_string(),
        ];

        let result = validate_field_list(&multiple_pk_fields);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("primary key"));
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_validation_error_creation() {
        let error = ValidationError::new(
            "Invalid model name".to_string(),
            vec!["Use snake_case naming".to_string()]
        );

        assert_eq!(error.message, "Invalid model name");
        assert_eq!(error.suggestions.len(), 1);
        assert_eq!(error.suggestions[0], "Use snake_case naming");
    }

    #[test]
    fn test_file_operation_error_creation() {
        let error = FileOperationError::new(
            "Permission denied".to_string(),
            "/path/to/file.rs".to_string()
        );

        assert_eq!(error.message, "Permission denied");
        assert_eq!(error.file_path, "/path/to/file.rs");
    }

    #[test]
    fn test_project_error_creation() {
        let error = ProjectError::new(
            "Not a loco project".to_string(),
            "/invalid/path".to_string(),
            vec!["src/models directory".to_string()]
        );

        assert_eq!(error.message, "Not a loco project");
        assert_eq!(error.project_path, "/invalid/path");
        assert_eq!(error.missing_elements.len(), 1);
    }

    #[test]
    fn test_error_to_json_conversion() {
        let error = ValidationError::new(
            "Test error".to_string(),
            vec!["Fix it".to_string()]
        );

        let json = error.to_json();
        assert_eq!(json["error_type"], "ValidationError");
        assert_eq!(json["message"], "Test error");
        assert_eq!(json["suggestions"].as_array().unwrap().len(), 1);
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_performance_model_generation() {
        let temp_project = create_temp_loco_project();
        let mut params = create_test_params("user", vec!["name:string", "email:string:unique"]);
        params.insert("project_path".to_string(), serde_json::Value::String(temp_project.path().to_string_lossy().to_string()));

        let start = Instant::now();
        let result = generate_model(params);
        let duration = start.elapsed();

        assert!(result["success"].as_bool().unwrap());

        // Should complete in under 100ms (very generous limit for tests)
        assert!(duration.as_millis() < 100, "Model generation took too long: {}ms", duration.as_millis());
    }

    #[test]
    fn test_performance_scaffold_generation() {
        let temp_project = create_temp_loco_project();
        let mut params = create_test_params("blog_post", vec![
            "title:string",
            "content:text",
            "published:boolean"
        ]);
        params.insert("project_path".to_string(), serde_json::Value::String(temp_project.path().to_string_lossy().to_string()));

        let start = Instant::now();
        let result = generate_scaffold(params);
        let duration = start.elapsed();

        assert!(result["success"].as_bool().unwrap());

        // Should complete in under 200ms for scaffold generation
        assert!(duration.as_millis() < 200, "Scaffold generation took too long: {}ms", duration.as_millis());
    }

    #[test]
    fn test_template_cache_effectiveness() {
        // This test would verify that template caching improves performance
        // Implementation would require access to cache statistics
        let temp_project = create_temp_loco_project();
        let project_path = temp_project.path().to_string_lossy().to_string();

        // Generate multiple models with same structure to test cache
        let models = vec!["user", "admin", "profile"];

        for model_name in models {
            let mut params = create_test_params(model_name, vec!["name:string", "email:string"]);
            params.insert("project_path".to_string(), serde_json::Value::String(project_path.clone()));

            let start = Instant::now();
            let result = generate_model(params);
            let duration = start.elapsed();

            assert!(result["success"].as_bool().unwrap());
            // Later generations should be faster due to caching
            assert!(duration.as_millis() < 50, "Cached generation took too long: {}ms", duration.as_millis());
        }
    }

    #[test]
    fn test_performance_metrics_collection() {
        // Generate some activity to collect metrics
        let temp_project = create_temp_loco_project();
        let mut params = create_test_params("user", vec!["name:string"]);
        params.insert("project_path".to_string(), serde_json::Value::String(temp_project.path().to_string_lossy().to_string()));

        // Generate a model to create some metrics
        let _result = generate_model(params);

        // Get performance metrics
        let metrics = get_performance_metrics();

        // Verify metrics structure
        assert!(metrics.contains_key("total_calls"));
        assert!(metrics.contains_key("avg_duration_ms"));
        assert!(metrics.contains_key("hit_rate"));
        assert!(metrics.contains_key("memory_usage_mb"));

        // Verify metrics are reasonable
        assert!(metrics["total_calls"].as_u64().unwrap() >= 1);
        assert!(metrics["avg_duration_ms"].as_f64().unwrap() >= 0.0);
    }
}

#[cfg(test)]
mod template_cache_tests {
    use super::*;

    #[test]
    fn test_template_cache_basic_operations() {
        let cache = TemplateCache::new(CacheConfig::default());

        // Test cache miss
        assert!(cache.get("nonexistent_key").is_none());

        // Test cache put and get
        let content = "test template content".to_string();
        cache.put("test_key".to_string(), content.clone()).unwrap();

        let retrieved = cache.get("test_key");
        assert_eq!(retrieved, Some(content));

        // Test cache contains
        assert!(cache.contains("test_key"));
        assert!(!cache.contains("nonexistent_key"));
    }

    #[test]
    fn test_template_cache_key_generation() {
        let cache = TemplateCache::new(CacheConfig::default());
        let fields = vec![
            FieldDefinition {
                name: "name".to_string(),
                field_type: FieldType::String,
                constraints: vec![],
                optional: false,
            }
        ];

        let key1 = cache.generate_key("model", "user", &fields);
        let key2 = cache.generate_key("model", "user", &fields);
        let key3 = cache.generate_key("model", "post", &fields);

        // Same inputs should generate same key
        assert_eq!(key1, key2);

        // Different inputs should generate different key
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_template_cache_statistics() {
        let cache = TemplateCache::new(CacheConfig::default());

        // Initial stats
        let stats = cache.get_stats();
        assert_eq!(stats.total_entries, 0);
        assert_eq!(stats.cache_hits, 0);
        assert_eq!(stats.cache_misses, 0);

        // Add entry
        cache.put("test".to_string(), "content".to_string()).unwrap();

        // Access entry (cache hit)
        let _ = cache.get("test");

        // Access non-existent entry (cache miss)
        let _ = cache.get("nonexistent");

        let final_stats = cache.get_stats();
        assert_eq!(final_stats.total_entries, 1);
        assert_eq!(final_stats.cache_hits, 1);
        assert_eq!(final_stats.cache_misses, 1);
        assert!(final_stats.hit_rate() > 0.0);
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
            }
        ];

        // Test cache miss
        assert!(model_cache.get_model_template("user", &fields).is_none());

        // Test cache put and get
        let template = "model template content".to_string();
        model_cache.cache_model_template("user", &fields, template.clone()).unwrap();

        let retrieved = model_cache.get_model_template("user", &fields);
        assert_eq!(retrieved, Some(template));
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_end_to_end_workflow() {
        let temp_project = create_temp_loco_project();
        let project_path = temp_project.path().to_string_lossy().to_string();

        // Step 1: Generate a user model
        let mut user_params = create_test_params("user", vec![
            "name:string",
            "email:string:unique",
            "created_at:datetime"
        ]);
        user_params.insert("project_path".to_string(), serde_json::Value::String(project_path.clone()));
        let user_result = generate_model(user_params);
        assert!(user_result["success"].as_bool().unwrap());

        // Step 2: Generate a blog post scaffold
        let mut blog_params = create_test_params("blog_post", vec![
            "title:string",
            "content:text",
            "published:boolean",
            "author_id:i64:foreign_key:users"
        ]);
        blog_params.insert("project_path".to_string(), serde_json::Value::String(project_path.clone()));
        blog_params.insert("include_views".to_string(), serde_json::Value::Bool(true));
        let blog_result = generate_scaffold(blog_params);
        assert!(blog_result["success"].as_bool().unwrap());

        // Step 3: Add controller to user model
        let mut controller_params = HashMap::new();
        controller_params.insert("model_name".to_string(), serde_json::Value::String("user".to_string()));
        controller_params.insert("project_path".to_string(), serde_json::Value::String(project_path));
        let controller_result = generate_controller_view(controller_params);
        assert!(controller_result["success"].as_bool().unwrap());

        // Verify all files were created
        assert!(temp_project.path().join("src/models/user.rs").exists());
        assert!(temp_project.path().join("src/models/blog_post.rs").exists());
        assert!(temp_project.path().join("src/controllers/blog_posts.rs").exists());
        assert!(temp_project.path().join("src/controllers/user.rs").exists());
        assert!(temp_project.path().join("src/views/blog_posts").exists());
        assert!(temp_project.path().join("src/views/user").exists());

        // Verify migration files exist
        let migration_dir = temp_project.path().join("migration/src");
        assert!(migration_dir.join("m_").join("_create_users.rs").exists());
        assert!(migration_dir.join("m_").join("_create_blog_posts.rs").exists());
    }

    #[test]
    fn test_error_recovery_workflow() {
        let temp_project = create_temp_loco_project();
        let project_path = temp_project.path().to_string_lossy().to_string();

        // Try to generate model with invalid name
        let mut invalid_params = create_test_params("123invalid", vec!["name:string"]);
        invalid_params.insert("project_path".to_string(), serde_json::Value::String(project_path.clone()));
        let invalid_result = generate_model(invalid_params);
        assert!(!invalid_result["success"].as_bool().unwrap());

        // Verify error details
        let errors = invalid_result["errors"].as_array().unwrap();
        assert!(!errors.is_empty());

        // Try again with valid name
        let mut valid_params = create_test_params("user", vec!["name:string"]);
        valid_params.insert("project_path".to_string(), serde_json::Value::String(project_path));
        let valid_result = generate_model(valid_params);
        assert!(valid_result["success"].as_bool().unwrap());

        // Verify recovery was successful
        assert!(temp_project.path().join("src/models/user.rs").exists());
    }

    #[test]
    fn test_concurrent_generation() {
        use std::thread;

        let temp_project = create_temp_loco_project();
        let project_path = temp_project.path().to_string_lossy().to_string();

        // Test concurrent model generation
        let handles: Vec<_> = (0..5).map(|i| {
            let project_path = project_path.clone();
            thread::spawn(move || {
                let mut params = create_test_params(&format!("model_{}", i), vec!["name:string"]);
                params.insert("project_path".to_string(), serde_json::Value::String(project_path));
                generate_model(params)
            })
        }).collect();

        // Wait for all threads to complete
        for handle in handles {
            let result = handle.join().unwrap();
            assert!(result["success"].as_bool().unwrap());
        }

        // Verify all models were created
        for i in 0..5 {
            let model_file = temp_project.path().join("src/models").join(format!("model_{}.rs", i));
            assert!(model_file.exists(), "Model {} was not created", i);
        }
    }
}
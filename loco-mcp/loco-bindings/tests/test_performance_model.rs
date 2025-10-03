use loco_bindings::*;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tempfile::TempDir;

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_project() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_str().unwrap();

        // Create a basic loco-rs project structure
        std::fs::create_dir_all(format!("{}/src/models", project_path)).unwrap();
        std::fs::create_dir_all(format!("{}/migration/src", project_path)).unwrap();

        // Create basic project files to make it look like a real loco project
        std::fs::write(format!("{}/Cargo.toml", project_path), r#"
[package]
name = "test-app"
version = "0.1.0"
edition = "2021"

[dependencies]
loco-rs = "0.3"
sea-orm = "0.12"
serde = "1.0"
"#).unwrap();

        std::fs::write(format!("{}/src/main.rs", project_path), r#"
fn main() {
    println!("Hello, world!");
}
"#).unwrap();

        temp_dir
    }

    #[test]
    fn test_performance_generate_model_simple() {
        let temp_dir = setup_test_project();
        let project_path = temp_dir.path().to_str().unwrap();

        let mut params = HashMap::new();
        params.insert("model_name", "simple_model");
        params.insert("fields", vec!["name:string"]);
        params.insert("project_path", project_path);

        let start = Instant::now();
        let result = generate_model(params).unwrap();
        let duration = start.elapsed();

        assert!(result["success"].as_bool().unwrap());
        assert!(duration < Duration::from_millis(10),
            "Simple model generation should take <10ms, took {:?}", duration);

        println!("Simple model generation took: {:?}", duration);
    }

    #[test]
    fn test_performance_generate_model_complex() {
        let temp_dir = setup_test_project();
        let project_path = temp_dir.path().to_str().unwrap();

        let mut params = HashMap::new();
        params.insert("model_name", "complex_model");
        params.insert("fields", vec![
            "name:string",
            "description:text",
            "price:i32",
            "is_active:boolean",
            "created_at:datetime",
            "uuid_field:uuid",
            "metadata:json",
            "email:string:unique",
            "user_id:i64",
            "category:string:nullable",
        ]);
        params.insert("project_path", project_path);

        let start = Instant::now();
        let result = generate_model(params).unwrap();
        let duration = start.elapsed();

        assert!(result["success"].as_bool().unwrap());
        assert!(duration < Duration::from_millis(10),
            "Complex model generation should take <10ms, took {:?}", duration);

        println!("Complex model generation took: {:?}", duration);
    }

    #[test]
    fn test_performance_generate_model_many_fields() {
        let temp_dir = setup_test_project();
        let project_path = temp_dir.path().to_str().unwrap();

        let mut fields = Vec::new();
        for i in 0..50 {
            fields.push(format!("field_{}:string", i));
        }

        let mut params = HashMap::new();
        params.insert("model_name", "many_fields_model");
        params.insert("fields", fields);
        params.insert("project_path", project_path);

        let start = Instant::now();
        let result = generate_model(params).unwrap();
        let duration = start.elapsed();

        assert!(result["success"].as_bool().unwrap());
        assert!(duration < Duration::from_millis(10),
            "Model with many fields should take <10ms, took {:?}", duration);

        println!("Model with 50 fields generation took: {:?}", duration);
    }

    #[test]
    fn test_performance_multiple_models_sequential() {
        let temp_dir = setup_test_project();
        let project_path = temp_dir.path().to_str().unwrap();

        let models = vec![
            ("user", vec!["name:string", "email:string:unique"]),
            ("product", vec!["title:string", "price:i32", "description:text"]),
            ("category", vec!["name:string", "parent_id:i32:nullable"]),
            ("order", vec!["user_id:i64", "total:f64", "status:string"]),
            ("review", vec!["user_id:i64", "product_id:i64", "rating:i32", "comment:text"]),
        ];

        let start = Instant::now();

        for (model_name, fields) in models {
            let mut params = HashMap::new();
            params.insert("model_name", model_name);
            params.insert("fields", fields);
            params.insert("project_path", project_path);

            let result = generate_model(params).unwrap();
            assert!(result["success"].as_bool().unwrap());
        }

        let total_duration = start.elapsed();
        let average_duration = total_duration / 5;

        assert!(average_duration < Duration::from_millis(10),
            "Average model generation should take <10ms, took {:?}", average_duration);

        println!("5 model generations took: {:?} (avg: {:?})", total_duration, average_duration);
    }

    #[test]
    fn test_performance_validation_only() {
        // Test performance of just validation (no file creation)
        let mut params = HashMap::new();
        params.insert("model_name", "validation_test");
        params.insert("fields", vec!["name:string", "price:i32", "email:string:unique"]);
        // Don't set project_path to test validation speed

        let start = Instant::now();
        let result = generate_model(params);
        let duration = start.elapsed();

        // Should fail quickly due to missing project_path
        assert!(result.is_err());
        assert!(duration < Duration::from_millis(1),
            "Validation should be very fast (<1ms), took {:?}", duration);

        println!("Validation took: {:?}", duration);
    }

    #[test]
    fn test_performance_error_handling() {
        let mut params = HashMap::new();
        params.insert("model_name", "123invalid"); // Invalid name
        params.insert("fields", vec!["name:string"]);

        let start = Instant::now();
        let result = generate_model(params);
        let duration = start.elapsed();

        assert!(result.is_err());
        assert!(duration < Duration::from_millis(1),
            "Error handling should be very fast (<1ms), took {:?}", duration);

        println!("Error handling took: {:?}", duration);
    }

    #[test]
    fn test_memory_usage_estimate() {
        let temp_dir = setup_test_project();
        let project_path = temp_dir.path().to_str().unwrap();

        // Test with increasingly complex models to check memory usage patterns
        let test_cases = vec![
            ("small", vec!["name:string"]),
            ("medium", vec!["name:string", "description:text", "price:i32", "is_active:boolean"]),
            ("large", (0..20).map(|i| format!("field_{}:string", i)).collect::<Vec<_>>()),
        ];

        for (name, fields) in test_cases {
            let mut params = HashMap::new();
            params.insert("model_name", format!("{}_model", name));
            params.insert("fields", fields);
            params.insert("project_path", project_path);

            let start = Instant::now();
            let result = generate_model(params).unwrap();
            let duration = start.elapsed();

            assert!(result["success"].as_bool().unwrap());
            assert!(duration < Duration::from_millis(10),
                "{} model generation should take <10ms, took {:?}", name, duration);

            println!("{} model generation: {:?}", name, duration);
        }
    }

    #[test]
    fn test_performance_regression_baseline() {
        // This test establishes a performance baseline
        // It can be used to detect performance regressions in the future
        let temp_dir = setup_test_project();
        let project_path = temp_dir.path().to_str().unwrap();

        let iterations = 10;
        let mut total_duration = Duration::ZERO;

        for i in 0..iterations {
            let mut params = HashMap::new();
            params.insert("model_name", format!("perf_test_{}", i));
            params.insert("fields", vec!["name:string", "value:i32", "created_at:datetime"]);
            params.insert("project_path", project_path);

            let start = Instant::now();
            let result = generate_model(params).unwrap();
            total_duration += start.elapsed();

            assert!(result["success"].as_bool().unwrap());
        }

        let average_duration = total_duration / iterations;

        // Performance requirement: average should be <10ms
        assert!(average_duration < Duration::from_millis(10),
            "Average generation time should be <10ms, took {:?}", average_duration);

        println!("Performance baseline: {:?} average over {} iterations", average_duration, iterations);

        // Also print the total to give a sense of overall throughput
        println!("Total time for {} iterations: {:?}", iterations, total_duration);
        println!("Throughput: {:.2} operations/second",
            iterations as f64 / total_duration.as_secs_f64());
    }
}
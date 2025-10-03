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

        // Create a complete loco-rs project structure
        std::fs::create_dir_all(format!("{}/src/models", project_path)).unwrap();
        std::fs::create_dir_all(format!("{}/src/controllers", project_path)).unwrap();
        std::fs::create_dir_all(format!("{}/src/views", project_path)).unwrap();
        std::fs::create_dir_all(format!("{}/src/routes", project_path)).unwrap();
        std::fs::create_dir_all(format!("{}/migration/src", project_path)).unwrap();

        // Create basic project files
        std::fs::write(format!("{}/Cargo.toml", project_path), r#"
[package]
name = "test-app"
version = "0.1.0"
edition = "2021"

[dependencies]
loco-rs = "0.3"
sea-orm = "0.12"
serde = "1.0"
tera = "1.0"
"#).unwrap();

        std::fs::write(format!("{}/src/main.rs", project_path), r#"
fn main() {
    println!("Hello, world!");
}
"#).unwrap();

        temp_dir
    }

    #[test]
    fn test_performance_generate_scaffold_simple() {
        let temp_dir = setup_test_project();
        let project_path = temp_dir.path().to_str().unwrap();

        let mut params = HashMap::new();
        params.insert("model_name", "simple_user");
        params.insert("fields", vec!["name:string", "email:string:unique"]);
        params.insert("include_views", true);
        params.insert("include_controllers", true);
        params.insert("api_only", false);
        params.insert("project_path", project_path);

        let start = Instant::now();
        let result = generate_scaffold(params).unwrap();
        let duration = start.elapsed();

        assert!(result["success"].as_bool().unwrap());
        assert!(duration < Duration::from_millis(10),
            "Simple scaffold generation should take <10ms, took {:?}", duration);

        println!("Simple scaffold generation took: {:?}", duration);
        println!("Generated {} files", result["created_files"].as_array().unwrap().len());
    }

    #[test]
    fn test_performance_generate_scaffold_api_only() {
        let temp_dir = setup_test_project();
        let project_path = temp_dir.path().to_str().unwrap();

        let mut params = HashMap::new();
        params.insert("model_name", "api_user");
        params.insert("fields", vec!["name:string", "token:string:unique", "active:boolean"]);
        params.insert("api_only", true);
        params.insert("project_path", project_path);

        let start = Instant::now();
        let result = generate_scaffold(params).unwrap();
        let duration = start.elapsed();

        assert!(result["success"].as_bool().unwrap());
        assert!(duration < Duration::from_millis(10),
            "API-only scaffold generation should take <10ms, took {:?}", duration);

        println!("API-only scaffold generation took: {:?}", duration);
        println!("Generated {} files", result["created_files"].as_array().unwrap().len());
    }

    #[test]
    fn test_performance_generate_scaffold_complex() {
        let temp_dir = setup_test_project();
        let project_path = temp_dir.path().to_str().unwrap();

        let mut params = HashMap::new();
        params.insert("model_name", "complex_product");
        params.insert("fields", vec![
            "name:string",
            "description:text",
            "price:i32",
            "category_id:i64",
            "is_active:boolean",
            "created_at:datetime",
            "updated_at:datetime",
            "metadata:json",
            "sku:string:unique",
            "stock_quantity:i32:default:0",
        ]);
        params.insert("include_views", true);
        params.insert("include_controllers", true);
        params.insert("api_only", false);
        params.insert("project_path", project_path);

        let start = Instant::now();
        let result = generate_scaffold(params).unwrap();
        let duration = start.elapsed();

        assert!(result["success"].as_bool().unwrap());
        assert!(duration < Duration::from_millis(10),
            "Complex scaffold generation should take <10ms, took {:?}", duration);

        println!("Complex scaffold generation took: {:?}", duration);
        println!("Generated {} files", result["created_files"].as_array().unwrap().len());
    }

    #[test]
    fn test_performance_multiple_scaffolds_sequential() {
        let temp_dir = setup_test_project();
        let project_path = temp_dir.path().to_str().unwrap();

        let scaffolds = vec![
            ("user", vec!["name:string", "email:string:unique"], false),
            ("product", vec!["title:string", "price:i32", "description:text"], false),
            ("order", vec!["user_id:i64", "total:f64", "status:string"], true),
            ("category", vec!["name:string", "parent_id:i32:nullable"], true),
        ];

        let start = Instant::now();

        for (model_name, fields, api_only) in scaffolds {
            let mut params = HashMap::new();
            params.insert("model_name", model_name);
            params.insert("fields", fields);
            params.insert("api_only", api_only);
            params.insert("project_path", project_path);

            let result = generate_scaffold(params).unwrap();
            assert!(result["success"].as_bool().unwrap());
        }

        let total_duration = start.elapsed();
        let average_duration = total_duration / 4;

        assert!(average_duration < Duration::from_millis(10),
            "Average scaffold generation should take <10ms, took {:?}", average_duration);

        println!("4 scaffold generations took: {:?} (avg: {:?})", total_duration, average_duration);
    }

    #[test]
    fn test_performance_scaffold_vs_individual_generation() {
        let temp_dir = setup_test_project();
        let project_path = temp_dir.path().to_str().unwrap();

        let fields = vec!["name:string", "email:string:unique", "active:boolean"];

        // Test scaffold generation
        let start = Instant::now();
        let mut params = HashMap::new();
        params.insert("model_name", "scaffold_test");
        params.insert("fields", fields.clone());
        params.insert("project_path", project_path);
        params.insert("include_views", true);
        params.insert("include_controllers", true);
        let scaffold_result = generate_scaffold(params).unwrap();
        let scaffold_duration = start.elapsed();

        // Test individual generation (model + controller_view)
        let start = Instant::now();

        // Generate model first
        let mut params = HashMap::new();
        params.insert("model_name", "individual_test");
        params.insert("fields", fields.clone());
        params.insert("project_path", project_path);
        let model_result = generate_model(params).unwrap();

        // Then generate controller and views
        let mut params = HashMap::new();
        params.insert("model_name", "individual_test");
        params.insert("project_path", project_path);
        let controller_result = generate_controller_view(params).unwrap();

        let individual_duration = start.elapsed();

        assert!(scaffold_duration < Duration::from_millis(10),
            "Scaffold generation should take <10ms, took {:?}", scaffold_duration);

        assert!(individual_duration < Duration::from_millis(10),
            "Individual generation should take <10ms, took {:?}", individual_duration);

        println!("Scaffold generation: {:?}", scaffold_duration);
        println!("Individual generation: {:?}", individual_duration);
        println!("Scaffold vs Individual ratio: {:.2}x",
            scaffold_duration.as_secs_f64() / individual_duration.as_secs_f64());

        // Scaffold should generally be faster due to consolidated operations
        // But both should meet the <10ms requirement
    }

    #[test]
    fn test_performance_scaffold_regression_baseline() {
        let temp_dir = setup_test_project();
        let project_path = temp_dir.path().to_str().unwrap();

        let iterations = 5; // Fewer iterations for scaffold as it generates more files
        let mut total_duration = Duration::ZERO;

        for i in 0..iterations {
            let mut params = HashMap::new();
            params.insert("model_name", format!("perf_scaffold_{}", i));
            params.insert("fields", vec!["name:string", "value:i32", "created_at:datetime"]);
            params.insert("project_path", project_path);

            let start = Instant::now();
            let result = generate_scaffold(params).unwrap();
            total_duration += start.elapsed();

            assert!(result["success"].as_bool().unwrap());
        }

        let average_duration = total_duration / iterations;

        // Performance requirement: average should be <10ms
        assert!(average_duration < Duration::from_millis(10),
            "Average scaffold generation time should be <10ms, took {:?}", average_duration);

        println!("Scaffold performance baseline: {:?} average over {} iterations", average_duration, iterations);
        println!("Total time for {} scaffold iterations: {:?}", iterations, total_duration);
        println!("Scaffold throughput: {:.2} operations/second",
            iterations as f64 / total_duration.as_secs_f64());
    }

    #[test]
    fn test_performance_large_model_scaffold() {
        let temp_dir = setup_test_project();
        let project_path = temp_dir.path().to_str().unwrap();

        let mut fields = Vec::new();
        for i in 0..30 {
            fields.push(format!("field_{}:string", i));
        }

        let mut params = HashMap::new();
        params.insert("model_name", "large_model");
        params.insert("fields", fields);
        params.insert("project_path", project_path);

        let start = Instant::now();
        let result = generate_scaffold(params).unwrap();
        let duration = start.elapsed();

        assert!(result["success"].as_bool().unwrap());
        assert!(duration < Duration::from_millis(10),
            "Large model scaffold should take <10ms, took {:?}", duration);

        println!("Large model scaffold (30 fields) took: {:?}", duration);
        println!("Generated {} files", result["created_files"].as_array().unwrap().len());
    }
}
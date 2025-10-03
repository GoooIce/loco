use loco_bindings::*;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use std::fs;

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_project_with_model(project_path: &str, model_name: &str) {
        // Create project structure
        fs::create_dir_all(format!("{}/src/models", project_path)).unwrap();
        fs::create_dir_all(format!("{}/src/controllers", project_path)).unwrap();
        fs::create_dir_all(format!("{}/src/views", project_path)).unwrap();
        fs::create_dir_all(format!("{}/src/routes", project_path)).unwrap();

        // Create basic project files
        fs::write(format!("{}/Cargo.toml", project_path), r#"
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

        // Create model file
        let model_content = format!(
            r#"use sea_orm::entity::prelude::*;
use serde::{{Deserialize, Serialize}};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "{}s")]
pub struct Model {{
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(column_type = "String(Some(255))")]
    pub name: String,
    #[sea_orm(column_type = "Text")]
    pub description: Option<String>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {{}}

impl ActiveModelBehavior for ActiveModel {{}}
"#,
            model_name
        );

        fs::write(format!("{}/src/models/{}.rs", project_path, model_name), model_content).unwrap();
    }

    #[test]
    fn test_performance_generate_controller_view_simple() {
        let temp_dir = tempfile::tempdir().unwrap();
        let project_path = temp_dir.path().to_str().unwrap();

        setup_test_project_with_model(project_path, "product");

        let mut params = HashMap::new();
        params.insert("model_name", "product");
        params.insert("actions", vec!["index", "show", "create", "update", "delete"]);
        params.insert("view_types", vec!["list", "show", "form", "edit"]);
        params.insert("project_path", project_path);

        let start = Instant::now();
        let result = generate_controller_view(params).unwrap();
        let duration = start.elapsed();

        assert!(result["success"].as_bool().unwrap());
        assert!(duration < Duration::from_millis(10),
            "Simple controller/view generation should take <10ms, took {:?}", duration);

        println!("Simple controller/view generation took: {:?}", duration);
        println!("Generated {} files", result["created_files"].as_array().unwrap().len());
    }

    #[test]
    fn test_performance_generate_controller_view_only() {
        let temp_dir = tempfile::tempdir().unwrap();
        let project_path = temp_dir.path().to_str().unwrap();

        setup_test_project_with_model(project_path, "user");

        let mut params = HashMap::new();
        params.insert("model_name", "user");
        params.insert("actions", vec!["index", "show", "create"]);
        params.insert("view_types", vec![]); // No views
        params.insert("project_path", project_path);

        let start = Instant::now();
        let result = generate_controller_view(params).unwrap();
        let duration = start.elapsed();

        assert!(result["success"].as_bool().unwrap());
        assert!(duration < Duration::from_millis(10),
            "Controller-only generation should take <10ms, took {:?}", duration);

        println!("Controller-only generation took: {:?}", duration);
    }

    #[test]
    fn test_performance_generate_controller_view_readonly() {
        let temp_dir = tempfile::tempdir().unwrap();
        let project_path = temp_dir.path().to_str().unwrap();

        setup_test_project_with_model(project_path, "post");

        let mut params = HashMap::new();
        params.insert("model_name", "post");
        params.insert("actions", vec!["index", "show"]); // Read-only actions
        params.insert("view_types", vec!["list", "show"]);
        params.insert("project_path", project_path);

        let start = Instant::now();
        let result = generate_controller_view(params).unwrap();
        let duration = start.elapsed();

        assert!(result["success"].as_bool().unwrap());
        assert!(duration < Duration::from_millis(10),
            "Read-only controller generation should take <10ms, took {:?}", duration);

        println!("Read-only controller generation took: {:?}", duration);
    }

    #[test]
    fn test_performance_generate_multiple_controller_views_sequential() {
        let temp_dir = tempfile::tempdir().unwrap();
        let project_path = temp_dir.path().to_str().unwrap();

        let models = vec!["user", "product", "category", "order"];

        // Set up all models first
        for model_name in &models {
            setup_test_project_with_model(project_path, model_name);
        }

        let start = Instant::now();

        for model_name in &models {
            let mut params = HashMap::new();
            params.insert("model_name", *model_name);
            params.insert("actions", vec!["index", "show", "create", "update", "delete"]);
            params.insert("view_types", vec!["list", "show", "form"]);
            params.insert("project_path", project_path);

            let result = generate_controller_view(params).unwrap();
            assert!(result["success"].as_bool().unwrap());
        }

        let total_duration = start.elapsed();
        let average_duration = total_duration / 4;

        assert!(average_duration < Duration::from_millis(10),
            "Average controller/view generation should take <10ms, took {:?}", average_duration);

        println!("4 controller/view generations took: {:?} (avg: {:?})", total_duration, average_duration);
    }

    #[test]
    fn test_performance_controller_view_error_cases() {
        let test_cases = vec![
            ("Invalid model name", HashMap::from([
                ("model_name", "123invalid"),
                ("actions", vec!["index", "show"]),
                ("view_types", vec!["list", "show"])
            ])),
            ("Invalid actions", HashMap::from([
                ("model_name", "test"),
                ("actions", vec!["invalid_action"]),
                ("view_types", vec!["list"])
            ])),
            ("Invalid view types", HashMap::from([
                ("model_name", "test"),
                ("actions", vec!["index", "show"]),
                ("view_types", vec!["invalid_view_type"])
            ])),
        ];

        for (case_name, mut params) in test_cases {
            let start = Instant::now();
            let result = generate_controller_view(params);
            let duration = start.elapsed();

            assert!(result.is_err(), "Case '{}' should fail", case_name);
            assert!(duration < Duration::from_millis(1),
                "Error case '{}' should be very fast (<1ms), took {:?}", case_name, duration);

            println!("Error case '{}' took: {:?}", case_name, duration);
        }
    }

    #[test]
    fn test_performance_controller_view_vs_full_scaffold() {
        let temp_dir = tempfile::tempdir().unwrap();
        let project_path = temp_dir.path().to_str().unwrap();

        setup_test_project_with_model(project_path, "comparison");

        let fields = vec!["name:string", "description:text", "active:boolean"];

        // Test full scaffold
        let start = Instant::now();
        let mut params = HashMap::new();
        params.insert("model_name", "full_scaffold");
        params.insert("fields", fields.clone());
        params.insert("project_path", project_path);
        let scaffold_result = generate_scaffold(params).unwrap();
        let scaffold_duration = start.elapsed();

        // Test model + controller_view (equivalent to scaffold)
        let start = Instant::now();

        // Generate model
        let mut params = HashMap::new();
        params.insert("model_name", "step_by_step");
        params.insert("fields", fields.clone());
        params.insert("project_path", project_path);
        let model_result = generate_model(params).unwrap();

        // Generate controller and views
        let mut params = HashMap::new();
        params.insert("model_name", "step_by_step");
        params.insert("actions", vec!["index", "show", "create", "update", "delete"]);
        params.insert("view_types", vec!["list", "show", "form", "edit"]);
        params.insert("project_path", project_path);
        let controller_result = generate_controller_view(params).unwrap();

        let step_by_step_duration = start.elapsed();

        assert!(scaffold_duration < Duration::from_millis(10));
        assert!(step_by_step_duration < Duration::from_millis(10));

        println!("Full scaffold: {:?}", scaffold_duration);
        println!("Step by step: {:?}", step_by_step_duration);
        println!("Files generated - scaffold: {}, step-by-step: {}",
            scaffold_result["created_files"].as_array().unwrap().len(),
            model_result["created_files"].as_array().unwrap().len() + controller_result["created_files"].as_array().unwrap().len()
        );
    }

    #[test]
    fn test_performance_controller_view_regression_baseline() {
        let temp_dir = tempfile::tempdir().unwrap();
        let project_path = temp_dir.path().to_str().unwrap();

        let iterations = 8;
        let mut total_duration = Duration::ZERO;

        for i in 0..iterations {
            let model_name = format!("perf_model_{}", i);
            setup_test_project_with_model(project_path, &model_name);

            let mut params = HashMap::new();
            params.insert("model_name", model_name);
            params.insert("actions", vec!["index", "show", "create"]);
            params.insert("view_types", vec!["list", "show", "form"]);
            params.insert("project_path", project_path);

            let start = Instant::now();
            let result = generate_controller_view(params).unwrap();
            total_duration += start.elapsed();

            assert!(result["success"].as_bool().unwrap());
        }

        let average_duration = total_duration / iterations;

        assert!(average_duration < Duration::from_millis(10),
            "Average controller/view generation should be <10ms, took {:?}", average_duration);

        println!("Controller/view performance baseline: {:?} average over {} iterations", average_duration, iterations);
        println!("Controller/view throughput: {:.2} operations/second",
            iterations as f64 / total_duration.as_secs_f64());
    }

    #[test]
    fn test_performance_complex_controller_view() {
        let temp_dir = tempfile::tempdir().unwrap();
        let project_path = temp_dir.path().to_str().unwrap();

        setup_test_project_with_model(project_path, "complex_model");

        let mut params = HashMap::new();
        params.insert("model_name", "complex_model");
        params.insert("actions", vec!["index", "show", "create", "update", "delete", "batch_update", "export"]);
        params.insert("view_types", vec!["list", "show", "form", "edit", "batch_form"]);
        params.insert("project_path", project_path);

        let start = Instant::now();
        let result = generate_controller_view(params).unwrap();
        let duration = start.elapsed();

        assert!(result["success"].as_bool().unwrap());
        assert!(duration < Duration::from_millis(10),
            "Complex controller/view generation should take <10ms, took {:?}", duration);

        println!("Complex controller/view generation took: {:?}", duration);
        println!("Generated {} files", result["created_files"].as_array().unwrap().len());
    }
}
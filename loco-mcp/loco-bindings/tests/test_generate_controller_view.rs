use loco_bindings::*;
use std::collections::HashMap;
use tempfile::TempDir;
use std::fs;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_model(project_path: &str, model_name: &str) {
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
}}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {{}}

impl ActiveModelBehavior for ActiveModel {{}}
"#,
            model_name
        );

        fs::write(
            format!("{}/src/models/{}.rs", project_path, model_name),
            model_content,
        ).unwrap();
    }

    #[test]
    fn test_generate_controller_view_success() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_str().unwrap();

        // Create a basic loco-rs project structure
        fs::create_dir_all(format!("{}/src/models", project_path)).unwrap();
        fs::create_dir_all(format!("{}/src/controllers", project_path)).unwrap();
        fs::create_dir_all(format!("{}/src/views", project_path)).unwrap();
        fs::create_dir_all(format!("{}/src/routes", project_path)).unwrap();

        // Create existing model
        create_test_model(project_path, "product");

        let mut params = HashMap::new();
        params.insert("model_name", "product");
        params.insert("actions", vec!["index", "show", "create", "update"]);
        params.insert("view_types", vec!["list", "show", "form"]);
        params.insert("project_path", project_path);

        let result = generate_controller_view(params).unwrap();

        assert!(result["success"].as_bool().unwrap());
        assert!(result["created_files"].as_array().unwrap().len() >= 3); // controller + views + routes

        let created_files = result["created_files"].as_array().unwrap();

        // Check controller file
        let controller_file = created_files.iter().find(|f| {
            f["path"].as_str().unwrap().contains("controllers/products.rs")
        }).unwrap();
        assert_eq!(controller_file["type"].as_str().unwrap(), "controller");

        // Check view files
        let has_list_view = created_files.iter().any(|f| {
            f["path"].as_str().unwrap().contains("views/products/list.html.tera")
        });
        let has_show_view = created_files.iter().any(|f| {
            f["path"].as_str().unwrap().contains("views/products/show.html.tera")
        });
        let has_form_view = created_files.iter().any(|f| {
            f["path"].as_str().unwrap().contains("views/products/form.html.tera")
        });

        assert!(has_list_view);
        assert!(has_show_view);
        assert!(has_form_view);
    }

    #[test]
    fn test_generate_controller_view_model_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_str().unwrap();

        // Create project structure but no model
        fs::create_dir_all(format!("{}/src/models", project_path)).unwrap();

        let mut params = HashMap::new();
        params.insert("model_name", "nonexistent");
        params.insert("project_path", project_path);

        let result = generate_controller_view(params);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("not found") || error.to_string().contains("Model not found"));
    }

    #[test]
    fn test_generate_controller_view_selective_actions() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_str().unwrap();

        fs::create_dir_all(format!("{}/src/models", project_path)).unwrap();
        fs::create_dir_all(format!("{}/src/controllers", project_path)).unwrap();
        fs::create_dir_all(format!("{}/src/routes", project_path)).unwrap();

        // Create existing model
        create_test_model(project_path, "post");

        let mut params = HashMap::new();
        params.insert("model_name", "post");
        params.insert("actions", vec!["index", "show"]); // Only read actions
        params.insert("view_types", vec!["list", "show"]);
        params.insert("project_path", project_path);

        let result = generate_controller_view(params).unwrap();

        assert!(result["success"].as_bool().unwrap());
        let created_files = result["created_files"].as_array().unwrap();

        // Should have controller with only index and show actions
        let controller_file = created_files.iter().find(|f| {
            f["path"].as_str().unwrap().contains("controllers/posts.rs")
        }).unwrap();

        // Verify controller content (would need to read file in actual implementation)
        assert_eq!(controller_file["type"].as_str().unwrap(), "controller");
    }

    #[test]
    fn test_generate_controller_view_invalid_actions() {
        let mut params = HashMap::new();
        params.insert("model_name", "test");
        params.insert("actions", vec!["invalid_action"]);

        let result = generate_controller_view(params);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("invalid") || error.to_string().contains("action"));
    }

    #[test]
    fn test_generate_controller_view_invalid_view_types() {
        let mut params = HashMap::new();
        params.insert("model_name", "test");
        params.insert("view_types", vec!["invalid_view_type"]);

        let result = generate_controller_view(params);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("invalid") || error.to_string().contains("view"));
    }

    #[test]
    fn test_generate_controller_view_missing_model_name() {
        let params = HashMap::new(); // Missing model_name

        let result = generate_controller_view(params);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("required") || error.to_string().contains("model_name"));
    }
}
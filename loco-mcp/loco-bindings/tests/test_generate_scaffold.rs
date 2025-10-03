use loco_bindings::*;
use std::collections::HashMap;
use tempfile::TempDir;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_scaffold_success() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_str().unwrap();

        // Create a basic loco-rs project structure
        std::fs::create_dir_all(format!("{}/src/models", project_path)).unwrap();
        std::fs::create_dir_all(format!("{}/src/controllers", project_path)).unwrap();
        std::fs::create_dir_all(format!("{}/src/views", project_path)).unwrap();
        std::fs::create_dir_all(format!("{}/src/routes", project_path)).unwrap();
        std::fs::create_dir_all(format!("{}/migration/src", project_path)).unwrap();

        let mut params = HashMap::new();
        params.insert("model_name", "user");
        params.insert("fields", vec!["email:string:unique", "name:string", "active:boolean"]);
        params.insert("include_views", true);
        params.insert("include_controllers", true);
        params.insert("api_only", false);
        params.insert("project_path", project_path);

        let result = generate_scaffold(params).unwrap();

        assert!(result["success"].as_bool().unwrap());
        assert!(result["created_files"].as_array().unwrap().len() >= 5); // model, migration, controller, views, routes

        let created_files = result["created_files"].as_array().unwrap();

        // Check model file
        let model_file = created_files.iter().find(|f| {
            f["path"].as_str().unwrap().contains("models/user.rs")
        }).unwrap();
        assert_eq!(model_file["type"].as_str().unwrap(), "model");

        // Check controller file
        let controller_file = created_files.iter().find(|f| {
            f["path"].as_str().unwrap().contains("controllers/users.rs")
        }).unwrap();
        assert_eq!(controller_file["type"].as_str().unwrap(), "controller");

        // Check migration file
        let migration_file = created_files.iter().find(|f| {
            f["path"].as_str().unwrap().contains("migration/src/m_")
        }).unwrap();
        assert_eq!(migration_file["type"].as_str().unwrap(), "migration");
    }

    #[test]
    fn test_generate_scaffold_api_only() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_str().unwrap();

        // Create a basic loco-rs project structure
        std::fs::create_dir_all(format!("{}/src/models", project_path)).unwrap();
        std::fs::create_dir_all(format!("{}/src/controllers", project_path)).unwrap();
        std::fs::create_dir_all(format!("{}/src/routes", project_path)).unwrap();
        std::fs::create_dir_all(format!("{}/migration/src", project_path)).unwrap();

        let mut params = HashMap::new();
        params.insert("model_name", "api_user");
        params.insert("fields", vec!["name:string", "token:string:unique"]);
        params.insert("api_only", true);
        params.insert("project_path", project_path);

        let result = generate_scaffold(params).unwrap();

        assert!(result["success"].as_bool().unwrap());
        let created_files = result["created_files"].as_array().unwrap();

        // Should have model, migration, controller but no views
        let has_model = created_files.iter().any(|f| f["type"].as_str().unwrap() == "model");
        let has_migration = created_files.iter().any(|f| f["type"].as_str().unwrap() == "migration");
        let has_controller = created_files.iter().any(|f| f["type"].as_str().unwrap() == "controller");
        let has_view = created_files.iter().any(|f| f["type"].as_str().unwrap() == "view");

        assert!(has_model);
        assert!(has_migration);
        assert!(has_controller);
        assert!(!has_view); // No views for API-only
    }

    #[test]
    fn test_generate_scaffold_conflicting_options() {
        let mut params = HashMap::new();
        params.insert("model_name", "test");
        params.insert("fields", vec!["name:string"]);
        params.insert("include_views", true);
        params.insert("api_only", true); // Conflict with include_views

        let result = generate_scaffold(params);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("conflict") || error.to_string().contains("invalid"));
    }

    #[test]
    fn test_generate_scaffold_without_controllers() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_str().unwrap();

        // Create a basic loco-rs project structure
        std::fs::create_dir_all(format!("{}/src/models", project_path)).unwrap();
        std::fs::create_dir_all(format!("{}/migration/src", project_path)).unwrap();

        let mut params = HashMap::new();
        params.insert("model_name", "simple");
        params.insert("fields", vec!["name:string"]);
        params.insert("include_controllers", false);
        params.insert("include_views", false);
        params.insert("project_path", project_path);

        let result = generate_scaffold(params).unwrap();

        assert!(result["success"].as_bool().unwrap());
        let created_files = result["created_files"].as_array().unwrap();

        // Should only have model and migration
        let has_model = created_files.iter().any(|f| f["type"].as_str().unwrap() == "model");
        let has_migration = created_files.iter().any(|f| f["type"].as_str().unwrap() == "migration");
        let has_controller = created_files.iter().any(|f| f["type"].as_str().unwrap() == "controller");

        assert!(has_model);
        assert!(has_migration);
        assert!(!has_controller);
    }
}
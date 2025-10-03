use loco_bindings::*;
use std::collections::HashMap;
use tempfile::TempDir;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_validation_model_name() {
        let mut params = HashMap::new();
        params.insert("model_name", "123invalid");
        params.insert("fields", vec!["name:string"]);

        let result = generate_model(params);
        assert!(result.is_err());

        let error = result.unwrap_err();
        let error_msg = error.to_string();

        assert!(error_msg.contains("Invalid model name") ||
               error_msg.contains("model name") ||
               error_msg.contains("validation"));
    }

    #[test]
    fn test_error_validation_field_type() {
        let mut params = HashMap::new();
        params.insert("model_name", "test");
        params.insert("fields", vec!["name:unsupported_type"]);

        let result = generate_model(params);
        assert!(result.is_err());

        let error = result.unwrap_err();
        let error_msg = error.to_string();

        assert!(error_msg.contains("Unsupported field type") ||
               error_msg.contains("field type") ||
               error_msg.contains("supported"));
    }

    #[test]
    fn test_error_missing_required_fields() {
        // Test missing model_name
        let mut params = HashMap::new();
        params.insert("fields", vec!["name:string"]);

        let result = generate_model(params);
        assert!(result.is_err());

        let error = result.unwrap_err();
        let error_msg = error.to_string();
        assert!(error_msg.contains("required") || error_msg.contains("model_name"));

        // Test missing fields
        let mut params = HashMap::new();
        params.insert("model_name", "test");

        let result = generate_model(params);
        assert!(result.is_err());

        let error = result.unwrap_err();
        let error_msg = error.to_string();
        assert!(error_msg.contains("required") || error_msg.contains("fields"));
    }

    #[test]
    fn test_error_empty_fields_list() {
        let mut params = HashMap::new();
        params.insert("model_name", "test");
        params.insert("fields", Vec::<String>::new());

        let result = generate_model(params);
        assert!(result.is_err());

        let error = result.unwrap_err();
        let error_msg = error.to_string();
        assert!(error_msg.contains("field") || error_msg.contains("empty"));
    }

    #[test]
    fn test_error_invalid_project_directory() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_str().unwrap();

        // Don't create project structure, so it should fail

        let mut params = HashMap::new();
        params.insert("model_name", "test");
        params.insert("fields", vec!["name:string"]);
        params.insert("project_path", project_path);

        let result = generate_model(params);
        assert!(result.is_err());

        let error = result.unwrap_err();
        let error_msg = error.to_string();

        assert!(error_msg.contains("project") ||
               error_msg.contains("directory") ||
               error_msg.contains("loco") ||
               error_msg.contains("Not a loco project"));
    }

    #[test]
    fn test_error_permission_denied() {
        // This test would require setting up a directory without write permissions
        // For now, we'll simulate by using an invalid path
        let invalid_path = "/root/invalid/path/that/should/not/exist";

        let mut params = HashMap::new();
        params.insert("model_name", "test");
        params.insert("fields", vec!["name:string"]);
        params.insert("project_path", invalid_path);

        let result = generate_model(params);
        assert!(result.is_err());

        let error = result.unwrap_err();
        let error_msg = error.to_string();

        assert!(error_msg.contains("permission") ||
               error_msg.contains("Permission") ||
               error_msg.contains("access") ||
               error_msg.contains("exist"));
    }

    #[test]
    fn test_error_model_already_exists() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_str().unwrap();

        // Create project structure
        std::fs::create_dir_all(format!("{}/src/models", project_path)).unwrap();
        std::fs::create_dir_all(format!("{}/migration/src", project_path)).unwrap();

        // Create existing model file
        let model_content = r#"
use sea_orm::entity::prelude::*;
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
}
"#;
        std::fs::write(format!("{}/src/models/user.rs", project_path), model_content).unwrap();

        let mut params = HashMap::new();
        params.insert("model_name", "user");
        params.insert("fields", vec!["name:string"]);
        params.insert("project_path", project_path);

        let result = generate_model(params);
        assert!(result.is_err());

        let error = result.unwrap_err();
        let error_msg = error.to_string();

        assert!(error_msg.contains("exists") ||
               error_msg.contains("already") ||
               error_msg.contains("duplicate"));
    }

    #[test]
    fn test_error_scaffold_conflicting_options() {
        let mut params = HashMap::new();
        params.insert("model_name", "test");
        params.insert("fields", vec!["name:string"]);
        params.insert("include_views", true);
        params.insert("api_only", true); // These conflict

        let result = generate_scaffold(params);
        assert!(result.is_err());

        let error = result.unwrap_err();
        let error_msg = error.to_string();

        assert!(error_msg.contains("conflict") ||
               error_msg.contains("invalid") ||
               error_msg.contains("cannot"));
    }

    #[test]
    fn test_error_controller_view_model_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_str().unwrap();

        // Create project structure but no model
        std::fs::create_dir_all(format!("{}/src/models", project_path)).unwrap();

        let mut params = HashMap::new();
        params.insert("model_name", "nonexistent");
        params.insert("project_path", project_path);

        let result = generate_controller_view(params);
        assert!(result.is_err());

        let error = result.unwrap_err();
        let error_msg = error.to_string();

        assert!(error_msg.contains("not found") ||
               error_msg.contains("exist") ||
               error_msg.contains("Model"));
    }

    #[test]
    fn test_error_invalid_actions() {
        let mut params = HashMap::new();
        params.insert("model_name", "test");
        params.insert("actions", vec!["invalid_action", "another_invalid"]);

        let result = generate_controller_view(params);
        assert!(result.is_err());

        let error = result.unwrap_err();
        let error_msg = error.to_string();

        assert!(error_msg.contains("invalid") ||
               error_msg.contains("action") ||
               error_msg.contains("supported"));
    }

    #[test]
    fn test_error_invalid_view_types() {
        let mut params = HashMap::new();
        params.insert("model_name", "test");
        params.insert("view_types", vec!["invalid_view_type"]);

        let result = generate_controller_view(params);
        assert!(result.is_err());

        let error = result.unwrap_err();
        let error_msg = error.to_string();

        assert!(error_msg.contains("invalid") ||
               error_msg.contains("view") ||
               error_msg.contains("type"));
    }

    #[test]
    fn test_error_field_format() {
        let invalid_fields = vec![
            "invalid_format",
            "name", // missing type
            ":", // missing name and type
            "name:type:too:many:colons", // malformed
        ];

        for field in invalid_fields {
            let mut params = HashMap::new();
            params.insert("model_name", "test");
            params.insert("fields", vec![field.to_string()]);

            let result = generate_model(params);
            assert!(result.is_err(), "Field '{}' should cause error", field);

            let error = result.unwrap_err();
            let error_msg = error.to_string();

            assert!(error_msg.contains("format") ||
                   error_msg.contains("Invalid field") ||
                   error_msg.contains("malformed"),
                "Error should mention format issue for '{}', got: {}", field, error_msg);
        }
    }

    #[test]
    fn test_error_duplicate_fields() {
        let mut params = HashMap::new();
        params.insert("model_name", "test");
        params.insert("fields", vec!["name:string", "name:string"]);

        let result = generate_model(params);
        assert!(result.is_err());

        let error = result.unwrap_err();
        let error_msg = error.to_string();

        assert!(error_msg.contains("duplicate") ||
               error_msg.contains("Duplicate") ||
               error_msg.contains("already exists"));
    }

    #[test]
    fn test_error_messages_are_helpful() {
        let test_cases = vec![
            (
                HashMap::from([("model_name", "123invalid"), ("fields", vec!["name:string"])]),
                "generate_model",
                vec!["model name", "start with letter", "invalid"]
            ),
            (
                HashMap::from([("model_name", "test"), ("fields", vec!["name:invalid_type"])]),
                "generate_model",
                vec!["field type", "supported", "string", "i32"]
            ),
            (
                HashMap::from([("model_name", "test")]),
                "generate_model",
                vec!["required", "fields"]
            ),
        ];

        for (mut params, function_name, expected_keywords) in test_cases {
            let result = if function_name == "generate_model" {
                generate_model(params)
            } else {
                panic!("Unknown function: {}", function_name);
            };

            if let Err(error) = result {
                let error_msg = error.to_string().to_lowercase();

                for keyword in expected_keywords {
                    assert!(error_msg.contains(&keyword.to_lowercase()),
                        "Error message should contain '{}' for case '{}', got: {}",
                        keyword, function_name, error_msg);
                }
            } else {
                panic!("Expected error for case '{}', but got success", function_name);
            }
        }
    }
}
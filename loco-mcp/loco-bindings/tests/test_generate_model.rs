use loco_bindings::*;
use std::collections::HashMap;
use tempfile::TempDir;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_model_success() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_str().unwrap();

        // Create a basic loco-rs project structure
        std::fs::create_dir_all(format!("{}/src/models", project_path)).unwrap();
        std::fs::create_dir_all(format!("{}/migration/src", project_path)).unwrap();

        let mut params = HashMap::new();
        params.insert("model_name", "product");
        params.insert("fields", vec!["name:string", "price:i32", "sku:string:unique"]);
        params.insert("project_path", project_path);

        let result = generate_model(params).unwrap();

        assert!(result["success"].as_bool().unwrap());
        assert_eq!(result["created_files"].as_array().unwrap().len(), 2); // model + migration

        let created_files = result["created_files"].as_array().unwrap();
        let model_file = created_files.iter().find(|f| {
            f["path"].as_str().unwrap().contains("models/product.rs")
        }).unwrap();
        assert_eq!(model_file["type"].as_str().unwrap(), "model");
        assert!(model_file["size_bytes"].as_u64().unwrap() > 0);
    }

    #[test]
    fn test_generate_model_invalid_name() {
        let mut params = HashMap::new();
        params.insert("model_name", "123invalid");
        params.insert("fields", vec!["name:string"]);

        let result = generate_model(params);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Invalid model name"));
    }

    #[test]
    fn test_generate_model_duplicate_fields() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_str().unwrap();

        let mut params = HashMap::new();
        params.insert("model_name", "test");
        params.insert("fields", vec!["name:string", "name:string"]);
        params.insert("project_path", project_path);

        let result = generate_model(params);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Duplicate field"));
    }

    #[test]
    fn test_generate_model_unsupported_field_type() {
        let mut params = HashMap::new();
        params.insert("model_name", "test");
        params.insert("fields", vec!["name:invalid_type"]);

        let result = generate_model(params);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Unsupported field type"));
    }

    #[test]
    fn test_generate_model_missing_required_params() {
        let mut params = HashMap::new();
        params.insert("model_name", "test");
        // Missing fields

        let result = generate_model(params);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("required"));
    }

    #[test]
    fn test_generate_model_empty_fields() {
        let mut params = HashMap::new();
        params.insert("model_name", "test");
        params.insert("fields", Vec::<String>::new());

        let result = generate_model(params);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("at least one field"));
    }

    #[test]
    fn test_generate_model_invalid_field_format() {
        let mut params = HashMap::new();
        params.insert("model_name", "test");
        params.insert("fields", vec!["invalid_format"]);

        let result = generate_model(params);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Invalid field format"));
    }
}
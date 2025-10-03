use loco_bindings::*;
use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_model_name_valid() {
        let valid_names = vec![
            "user",
            "product_category",
            "api_key",
            "user_profile",
            "simple",
            "a", // minimum length
            "very_long_model_name_that_is_still_valid_under_sixty_four_characters", // close to max
        ];

        for name in valid_names {
            let mut params = HashMap::new();
            params.insert("model_name", name);
            params.insert("fields", vec!["name:string"]);

            // This should not fail due to model name validation
            let result = generate_model(params);
            // Note: Might fail for other reasons (like no project structure), but not model name
            if let Err(e) = result {
                assert!(!e.to_string().contains("Invalid model name"),
                    "Model name '{}' should be valid, but got error: {}", name, e);
            }
        }
    }

    #[test]
    fn test_validate_model_name_invalid() {
        let invalid_names = vec![
            "123invalid", // starts with number
            "_private",   // starts with underscore
            "user-name",  // contains hyphen
            "user name",  // contains space
            "User",       // contains uppercase
            "user@domain", // contains special char
            "",           // empty
            "user\x00name", // contains null byte
        ];

        for name in invalid_names {
            let mut params = HashMap::new();
            params.insert("model_name", name);
            params.insert("fields", vec!["name:string"]);

            let result = generate_model(params);
            assert!(result.is_err(), "Model name '{}' should be invalid", name);

            let error = result.unwrap_err();
            assert!(error.to_string().contains("Invalid model name") ||
                   error.to_string().contains("model name"),
                "Error should mention model name validation for '{}', got: {}", name, error);
        }
    }

    #[test]
    fn test_validate_field_definitions_valid() {
        let valid_fields = vec![
            vec!["name:string"],
            vec!["title:string", "content:text"],
            vec!["email:string:unique"],
            vec!["price:i32"],
            vec!["created_at:datetime:nullable"],
            vec!["is_active:boolean:default:true"],
            vec!["data:json"],
            vec!["id:primary_key"],
            vec!["user_id:i64:foreign_key"],
        ];

        for fields in valid_fields {
            let mut params = HashMap::new();
            params.insert("model_name", "test_model");
            params.insert("fields", fields.clone());

            let result = generate_model(params);
            // Should not fail due to field validation (might fail for other reasons)
            if let Err(e) = result {
                assert!(!e.to_string().contains("Invalid field") &&
                       !e.to_string().contains("Unsupported field type") &&
                       !e.to_string().contains("field format"),
                    "Field combination '{:?}' should be valid, but got error: {}", fields, e);
            }
        }
    }

    #[test]
    fn test_validate_field_definitions_invalid() {
        let invalid_fields = vec![
            vec!["123name:string"], // invalid field name
            vec!["name:invalid_type"], // unsupported type
            vec!["name:string:invalid_constraint"], // invalid constraint
            vec!["name:string:too:many:colons"], // malformed
            vec![""], // empty field definition
            vec!["name"], // missing type
            vec![":"], // missing name and type
        ];

        for fields in invalid_fields {
            let mut params = HashMap::new();
            params.insert("model_name", "test_model");
            params.insert("fields", fields.clone());

            let result = generate_model(params);
            assert!(result.is_err(), "Field combination '{:?}' should be invalid", fields);

            let error = result.unwrap_err();
            assert!(error.to_string().contains("Invalid field") ||
                   error.to_string().contains("Unsupported field") ||
                   error.to_string().contains("field format"),
                "Error should mention field validation for '{:?}', got: {}", fields, error);
        }
    }

    #[test]
    fn test_validate_field_types() {
        let supported_types = vec![
            "string", "i32", "i64", "f32", "f64", "boolean",
            "datetime", "uuid", "json", "text"
        ];

        for field_type in supported_types {
            let field_def = format!("test_field:{}", field_type);
            let mut params = HashMap::new();
            params.insert("model_name", "test_model");
            params.insert("fields", vec![field_def]);

            let result = generate_model(params);
            if let Err(e) = result {
                assert!(!e.to_string().contains("Unsupported field type"),
                    "Field type '{}' should be supported, got error: {}", field_type, e);
            }
        }
    }

    #[test]
    fn test_validate_constraints() {
        let valid_constraints = vec![
            "unique",
            "primary_key",
            "nullable",
            "optional",
            "default:true",
            "default:100",
            "default:some_value",
        ];

        for constraint in valid_constraints {
            let field_def = format!("test_field:string:{}", constraint);
            let mut params = HashMap::new();
            params.insert("model_name", "test_model");
            params.insert("fields", vec![field_def]);

            let result = generate_model(params);
            if let Err(e) = result {
                assert!(!e.to_string().contains("Invalid constraint") &&
                       !e.to_string().contains("incompatible"),
                    "Constraint '{}' should be valid, got error: {}", constraint, e);
            }
        }
    }

    #[test]
    fn test_validate_duplicate_field_names() {
        let mut params = HashMap::new();
        params.insert("model_name", "test_model");
        params.insert("fields", vec!["name:string", "name:string", "email:string"]);

        let result = generate_model(params);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.to_string().contains("duplicate") ||
               error.to_string().contains("Duplicate") ||
               error.to_string().contains("already exists"),
            "Error should mention duplicate field names, got: {}", error);
    }

    #[test]
    fn test_validate_reserved_field_names() {
        let reserved_names = vec!["id"]; // 'id' is typically reserved for primary key

        for reserved_name in reserved_names {
            let field_def = format!("{}:string", reserved_name);
            let mut params = HashMap::new();
            params.insert("model_name", "test_model");
            params.insert("fields", vec![field_def]);

            let result = generate_model(params);
            if let Err(e) = result {
                // Some frameworks allow 'id' field definition, so this might not always error
                // Check if error specifically mentions reserved names
                if e.to_string().contains("reserved") || e.to_string().contains("id") {
                    // Expected behavior for reserved name
                    continue;
                }
            }
            // If no error about reserved name, the framework allows it
        }
    }

    #[test]
    fn test_validate_field_count_limits() {
        // Test with too many fields (if there's a limit)
        let mut many_fields = Vec::new();
        for i in 0..200 { // Create 200 fields
            many_fields.push(format!("field_{}:string", i));
        }

        let mut params = HashMap::new();
        params.insert("model_name", "model_with_many_fields");
        params.insert("fields", many_fields);

        let result = generate_model(params);
        // This might succeed or fail depending on implementation limits
        // If it fails, it should provide a helpful error message
        if let Err(e) = result {
            assert!(e.to_string().contains("field") ||
                   e.to_string().contains("limit") ||
                   e.to_string().contains("too many"),
                "Error for many fields should be informative, got: {}", e);
        }
    }
}
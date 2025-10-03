//! Field parsing and validation for model definitions
//!
//! This module handles parsing and validation of field definitions from string
//! representations like "name:string:unique" into structured field information.

use crate::error::{BindingError, BindingResult};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Supported field types in loco-rs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FieldType {
    String,
    I32,
    I64,
    F32,
    F64,
    Boolean,
    DateTime,
    Uuid,
    Json,
    Text,
}

impl FieldType {
    /// Parse field type from string
    pub fn from_str(s: &str) -> BindingResult<Self> {
        match s.to_lowercase().as_str() {
            "string" => Ok(FieldType::String),
            "i32" => Ok(FieldType::I32),
            "i64" => Ok(FieldType::I64),
            "f32" => Ok(FieldType::F32),
            "f64" => Ok(FieldType::F64),
            "boolean" => Ok(FieldType::Boolean),
            "datetime" => Ok(FieldType::DateTime),
            "uuid" => Ok(FieldType::Uuid),
            "json" => Ok(FieldType::Json),
            "text" => Ok(FieldType::Text),
            _ => Err(BindingError::validation(format!(
                "Unsupported field type: '{}'. Supported types: string, i32, i64, f32, f64, boolean, datetime, uuid, json, text",
                s
            ))),
        }
    }

    /// Convert to string representation
    pub fn to_string(&self) -> &'static str {
        match self {
            FieldType::String => "string",
            FieldType::I32 => "i32",
            FieldType::I64 => "i64",
            FieldType::F32 => "f32",
            FieldType::F64 => "f64",
            FieldType::Boolean => "boolean",
            FieldType::DateTime => "datetime",
            FieldType::Uuid => "uuid",
            FieldType::Json => "json",
            FieldType::Text => "text",
        }
    }
}

/// Field constraints
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FieldConstraint {
    Unique,
    PrimaryKey,
    Nullable,
    Optional,
    Default(String),
    ForeignKey(String),
}

impl FieldConstraint {
    /// Parse constraint from string
    pub fn from_str(s: &str) -> BindingResult<Self> {
        if s == "unique" {
            Ok(FieldConstraint::Unique)
        } else if s == "primary_key" {
            Ok(FieldConstraint::PrimaryKey)
        } else if s == "nullable" || s == "optional" {
            Ok(FieldConstraint::Nullable)
        } else if s.starts_with("default:") {
            let value = s.strip_prefix("default:")
                .ok_or_else(|| BindingError::validation("Invalid default constraint format"))?;
            Ok(FieldConstraint::Default(value.to_string()))
        } else if s.starts_with("foreign_key:") {
            let value = s.strip_prefix("foreign_key:")
                .ok_or_else(|| BindingError::validation("Invalid foreign_key constraint format"))?;
            Ok(FieldConstraint::ForeignKey(value.to_string()))
        } else {
            Err(BindingError::validation(format!(
                "Unsupported constraint: '{}'. Supported constraints: unique, primary_key, nullable, optional, default:<value>, foreign_key:<table>",
                s
            )))
        }
    }

    /// Convert to string representation
    pub fn to_string(&self) -> String {
        match self {
            FieldConstraint::Unique => "unique".to_string(),
            FieldConstraint::PrimaryKey => "primary_key".to_string(),
            FieldConstraint::Nullable => "nullable".to_string(),
            FieldConstraint::Optional => "optional".to_string(),
            FieldConstraint::Default(value) => format!("default:{}", value),
            FieldConstraint::ForeignKey(value) => format!("foreign_key:{}", value),
        }
    }
}

/// Field definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldDefinition {
    pub name: String,
    pub field_type: FieldType,
    pub constraints: Vec<FieldConstraint>,
    pub optional: bool,
}

impl FieldDefinition {
    /// Parse field definition from string format "name:type[:constraint]*"
    pub fn from_str(s: &str) -> BindingResult<Self> {
        let parts: Vec<&str> = s.split(':').collect();

        if parts.len() < 2 {
            return Err(BindingError::validation(format!(
                "Invalid field format: '{}'. Expected format: 'name:type[:constraint]*'",
                s
            )));
        }

        let name = parts[0].trim();
        let type_str = parts[1].trim();

        // Validate field name
        Self::validate_field_name(name)?;

        // Parse field type
        let field_type = FieldType::from_str(type_str)?;

        // Parse constraints
        let mut constraints = Vec::new();
        let mut optional = false;

        for constraint_str in parts.iter().skip(2) {
            let constraint_str = constraint_str.trim();
            if constraint_str == "optional" || constraint_str == "nullable" {
                optional = true;
            }

            constraints.push(FieldConstraint::from_str(constraint_str)?);
        }

        Ok(FieldDefinition {
            name: name.to_string(),
            field_type,
            constraints,
            optional,
        })
    }

    /// Validate field name according to Rust naming conventions
    fn validate_field_name(name: &str) -> BindingResult<()> {
        if name.is_empty() {
            return Err(BindingError::validation("Field name cannot be empty"));
        }

        // Check if starts with letter
        if !name.chars().next().unwrap().is_ascii_alphabetic() {
            return Err(BindingError::validation(format!(
                "Invalid field name: '{}' must start with a letter",
                name
            )));
        }

        // Check if contains only valid characters
        if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err(BindingError::validation(format!(
                "Invalid field name: '{}' contains invalid characters. Use only letters, numbers, and underscores",
                name
            )));
        }

        // Check reserved keywords
        let reserved = ["id", "type", "struct", "enum", "impl", "fn", "let", "mut"];
        if reserved.contains(&name) {
            return Err(BindingError::validation(format!(
                "Field name '{}' is a reserved keyword. Choose a different name",
                name
            )));
        }

        Ok(())
    }

    /// Convert to string representation
    pub fn to_string(&self) -> String {
        let mut result = format!("{}:{}", self.name, self.field_type.to_string());

        for constraint in &self.constraints {
            result.push(':');
            result.push_str(&constraint.to_string());
        }

        result
    }

    /// Check if field has a specific constraint
    pub fn has_constraint(&self, constraint: &FieldConstraint) -> bool {
        self.constraints.contains(constraint)
    }

    /// Check if field is primary key
    pub fn is_primary_key(&self) -> bool {
        self.has_constraint(&FieldConstraint::PrimaryKey)
    }

    /// Check if field is unique
    pub fn is_unique(&self) -> bool {
        self.has_constraint(&FieldConstraint::Unique)
    }

    /// Check if field is nullable
    pub fn is_nullable(&self) -> bool {
        self.optional || self.has_constraint(&FieldConstraint::Nullable)
    }
}

/// Validate a list of field definitions
pub fn validate_field_list(fields: &[String]) -> BindingResult<Vec<FieldDefinition>> {
    if fields.is_empty() {
        return Err(BindingError::validation("At least one field must be specified"));
    }

    let mut field_names = HashSet::new();
    let mut parsed_fields = Vec::new();

    for field_str in fields {
        let field = FieldDefinition::from_str(field_str)?;

        // Check for duplicate field names
        if field_names.contains(&field.name) {
            return Err(BindingError::validation(format!(
                "Duplicate field name: '{}'. Each field name must be unique",
                field.name
            )));
        }

        field_names.insert(field.name.clone());
        parsed_fields.push(field);
    }

    // Ensure at most one primary key
    let primary_key_count = parsed_fields.iter().filter(|f| f.is_primary_key()).count();
    if primary_key_count > 1 {
        return Err(BindingError::validation(
            "Only one field can be marked as primary key"
        ));
    }

    Ok(parsed_fields)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_parsing() {
        let field = FieldDefinition::from_str("name:string").unwrap();
        assert_eq!(field.name, "name");
        assert_eq!(field.field_type, FieldType::String);
        assert!(field.constraints.is_empty());
        assert!(!field.optional);

        let field = FieldDefinition::from_str("email:string:unique").unwrap();
        assert_eq!(field.name, "email");
        assert_eq!(field.field_type, FieldType::String);
        assert_eq!(field.constraints.len(), 1);
        assert!(field.is_unique());

        let field = FieldDefinition::from_str("price:i32:nullable").unwrap();
        assert_eq!(field.name, "price");
        assert_eq!(field.field_type, FieldType::I32);
        assert!(field.optional);
    }

    #[test]
    fn test_invalid_field_names() {
        let invalid_names = ["123name", "name-with-dash", "name with space", "Name", "id"];

        for name in invalid_names {
            let result = FieldDefinition::from_str(&format!("{}:string", name));
            assert!(result.is_err(), "Field name '{}' should be invalid", name);
        }
    }

    #[test]
    fn test_duplicate_field_validation() {
        let fields = vec![
            "name:string".to_string(),
            "name:string".to_string(),
        ];

        let result = validate_field_list(&fields);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duplicate field name"));
    }
}
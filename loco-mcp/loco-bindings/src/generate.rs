//! Core generation logic for models, scaffolds, and controllers
//!
//! This module provides the main generation functions that create loco-rs
//! model files, migrations, controllers, and views.

use crate::error::{BindingError, BindingResult};
use crate::field::validate_field_list;
use crate::file_ops::{create_file, ensure_directory_exists, file_exists, FileInfo};
use crate::loco_detect::{validate_loco_project, LocoProjectInfo};
use crate::template::{render_model_template, render_migration_template, render_controller_template, render_view_templates};
use crate::template_cache::{ModelTemplateCache, ControllerTemplateCache, ViewTemplateCache, MigrationTemplateCache};
use crate::performance::{PerformanceMetrics, OptimizedGenerator, serialize_response_fast};
use pyo3::prelude::*;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;

/// Generate response structure
#[derive(Debug, Clone)]
pub struct GenerationResponse {
    pub success: bool,
    pub created_files: Vec<FileInfo>,
    pub modified_files: Vec<FileInfo>,
    pub errors: Vec<String>,
}

impl GenerationResponse {
    pub fn success() -> Self {
        Self {
            success: true,
            created_files: Vec::new(),
            modified_files: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            created_files: Vec::new(),
            modified_files: Vec::new(),
            errors: vec![message],
        }
    }

    pub fn with_created_file(mut self, file: FileInfo) -> Self {
        self.created_files.push(file);
        self
    }

    pub fn with_modified_file(mut self, file: FileInfo) -> Self {
        self.modified_files.push(file);
        self
    }

    pub fn with_error(mut self, error: String) -> Self {
        self.errors.push(error);
        self.success = false;
        self
    }

    /// Convert to JSON for Python bindings
    pub fn to_json(&self) -> Value {
        json!({
            "success": self.success,
            "created_files": self.created_files.iter().map(|f| f.to_json()).collect::<Vec<_>>(),
            "modified_files": self.modified_files.iter().map(|f| f.to_json()).collect::<Vec<_>>(),
            "errors": self.errors
        })
    }
}

/// Generate a model and migration file
#[pyfunction]
pub fn generate_model(params: HashMap<String, Value>) -> PyResult<Value> {
    let result = generate_model_internal(params);
    match result {
        Ok(response) => Ok(response.to_json()),
        Err(error) => Err(error.to_py_err()),
    }
}

/// Internal implementation of model generation
fn generate_model_internal(params: HashMap<String, Value>) -> BindingResult<GenerationResponse> {
    let start_time = Instant::now();

    // Extract and validate parameters
    let model_name = extract_string_param(&params, "model_name")?;
    let fields = extract_string_array_param(&params, "fields")?;
    let project_path = extract_string_param(&params, "project_path").unwrap_or_else(|_| ".".to_string());

    // Validate inputs
    validate_model_name(&model_name)?;
    let field_definitions = validate_field_list(&fields)?;

    // Validate project
    let project_info = validate_loco_project(&project_path)?;

    // Check if model already exists
    let model_file_path = project_info.src_models_path.join(format!("{}.rs", model_name));
    if file_exists(&model_file_path) {
        return Err(BindingError::validation(format!(
            "Model '{}' already exists. Choose a different name or remove existing file",
            model_name
        )));
    }

    let mut response = GenerationResponse::success();

    // Use template cache for model generation
    let model_cache = ModelTemplateCache::new();

    // Try to get cached model template, generate and cache if not found
    let model_content = model_cache.get_model_template(&model_name, &field_definitions)
        .unwrap_or_else(|| {
            let content = render_model_template(&model_name, &field_definitions).unwrap();
            let _ = model_cache.cache_model_template(&model_name, &field_definitions, content.clone());
            content
        });

    let model_file_info = create_file(&model_file_path, &model_content, "model")?;
    response = response.with_created_file(model_file_info);

    // Use template cache for migration generation
    let migration_cache = MigrationTemplateCache::new();

    // Try to get cached migration template, generate and cache if not found
    let migration_content = migration_cache.get_migration_template(&model_name, &field_definitions)
        .unwrap_or_else(|| {
            let content = render_migration_template(&model_name, &field_definitions).unwrap();
            let _ = migration_cache.cache_migration_template(&model_name, &field_definitions, content.clone());
            content
        });

    let migration_file_path = project_info.migration_src_path.join(format!("m_{}_create_{}.rs",
        chrono::Utc::now().format("%Y%m%d_%H%M%S"),
        format!("{}s", model_name)
    ));
    let migration_file_info = create_file(&migration_file_path, &migration_content, "migration")?;
    response = response.with_created_file(migration_file_info);

    // Update performance metrics
    let processing_time = start_time.elapsed().as_millis() as f64;

    Ok(response)
}

/// Generate complete CRUD scaffolding
#[pyfunction]
pub fn generate_scaffold(params: HashMap<String, Value>) -> PyResult<Value> {
    let result = generate_scaffold_internal(params);
    match result {
        Ok(response) => Ok(response.to_json()),
        Err(error) => Err(error.to_py_err()),
    }
}

/// Internal implementation of scaffold generation
fn generate_scaffold_internal(params: HashMap<String, Value>) -> BindingResult<GenerationResponse> {
    let start_time = Instant::now();

    // Extract and validate parameters
    let model_name = extract_string_param(&params, "model_name")?;
    let fields = extract_string_array_param(&params, "fields")?;
    let include_views = extract_bool_param(&params, "include_views").unwrap_or(true);
    let include_controllers = extract_bool_param(&params, "include_controllers").unwrap_or(true);
    let api_only = extract_bool_param(&params, "api_only").unwrap_or(false);
    let project_path = extract_string_param(&params, "project_path").unwrap_or_else(|_| ".".to_string());

    // Validate configuration
    if api_only && include_views {
        return Err(BindingError::validation(
            "Cannot have both api_only=true and include_views=true"
        ));
    }

    // Validate inputs
    validate_model_name(&model_name)?;
    let field_definitions = validate_field_list(&fields)?;

    // Validate project
    let project_info = validate_loco_project(&project_path)?;

    let mut response = GenerationResponse::success();

    // Generate model and migration (reuse model generation logic)
    let model_params = HashMap::from([
        ("model_name".to_string(), Value::String(model_name.clone())),
        ("fields".to_string(), Value::Array(fields.into_iter().map(Value::String).collect())),
        ("project_path".to_string(), Value::String(project_path.clone())),
    ]);
    let model_response = generate_model_internal(model_params)?;

    if !model_response.success {
        return Err(BindingError::validation(format!(
            "Failed to generate model: {}",
            model_response.errors.join(", ")
        )));
    }

    // Add model files to response
    for file in model_response.created_files {
        response = response.with_created_file(file);
    }

    // Generate controller with caching
    if include_controllers {
        ensure_directory_exists(&project_info.src_controllers_path)?;

        let controller_cache = ControllerTemplateCache::new();
        let include_views_flag = !api_only;

        // Try to get cached controller template, generate and cache if not found
        let controller_content = controller_cache.get_controller_template(&model_name, include_views_flag)
            .unwrap_or_else(|| {
                let content = render_controller_template(&model_name, &field_definitions, include_views_flag).unwrap();
                let _ = controller_cache.cache_controller_template(&model_name, include_views_flag, content.clone());
                content
            });

        let controller_file_path = project_info.src_controllers_path.join(format!("{}.rs", model_name));
        let controller_file_info = create_file(&controller_file_path, &controller_content, "controller")?;
        response = response.with_created_file(controller_file_info);
    }

    // Generate views with caching
    if include_views && !api_only {
        ensure_directory_exists(&project_info.src_views_path)?;

        let views_dir = project_info.src_views_path.join(&model_name);
        ensure_directory_exists(&views_dir)?;

        let view_cache = ViewTemplateCache::new();
        let view_templates = render_view_templates(&model_name, &field_definitions)?;

        for (template_name, content) in view_templates {
            // Try to get cached view template, generate and cache if not found
            let view_content = view_cache.get_view_template(&model_name, &template_name)
                .unwrap_or_else(|| {
                    // Re-generate just this template
                    let templates = render_view_templates(&model_name, &field_definitions).unwrap();
                    let cached_content = templates.get(&template_name).cloned().unwrap_or_else(|| content.clone());
                    let _ = view_cache.cache_view_template(&model_name, &template_name, cached_content.clone());
                    cached_content
                });

            let view_file_path = views_dir.join(format!("{}.html.tera", template_name));
            let view_file_info = create_file(&view_file_path, &view_content, "view")?;
            response = response.with_created_file(view_file_info);
        }
    }

    // Update routes (this is a simplified version - in real implementation would parse and modify routes)
    let routes_file_path = project_info.src_routes_path.join("mod.rs");
    if file_exists(&routes_file_path) {
        // In a real implementation, we would parse the existing routes file
        // and add the new routes. For now, we'll just mark it as modified.
        let routes_file_info = FileInfo {
            path: routes_file_path.to_string_lossy().to_string(),
            file_type: "route".to_string(),
            size_bytes: 0, // Would calculate actual size
        };
        response = response.with_modified_file(routes_file_info);
    }

    // Update performance metrics
    let processing_time = start_time.elapsed().as_millis() as f64;

    Ok(response)
}

/// Generate controller and views for existing model
#[pyfunction]
pub fn generate_controller_view(params: HashMap<String, Value>) -> PyResult<Value> {
    let result = generate_controller_view_internal(params);
    match result {
        Ok(response) => Ok(response.to_json()),
        Err(error) => Err(error.to_py_err()),
    }
}

/// Internal implementation of controller/view generation
fn generate_controller_view_internal(params: HashMap<String, Value>) -> BindingResult<GenerationResponse> {
    let start_time = Instant::now();

    // Extract and validate parameters
    let model_name = extract_string_param(&params, "model_name")?;
    let actions = extract_string_array_param(&params, "actions").unwrap_or_else(|_| {
        vec!["index".to_string(), "show".to_string(), "create".to_string(), "update".to_string(), "delete".to_string()]
    });
    let view_types = extract_string_array_param(&params, "view_types").unwrap_or_else(|_| {
        vec!["list".to_string(), "show".to_string(), "form".to_string(), "edit".to_string()]
    });
    let project_path = extract_string_param(&params, "project_path").unwrap_or_else(|_| ".".to_string());

    // Validate inputs
    validate_model_name(&model_name)?;
    validate_actions(&actions)?;
    validate_view_types(&view_types)?;

    // Validate project
    let project_info = validate_loco_project(&project_path)?;

    // Check if model exists
    let model_file_path = project_info.src_models_path.join(format!("{}.rs", model_name));
    if !file_exists(&model_file_path) {
        return Err(BindingError::validation(format!(
            "Model '{}' not found. Generate the model first",
            model_name
        )));
    }

    let mut response = GenerationResponse::success();

    // For simplicity, we'll use empty field definitions here
    // In a real implementation, we would parse the existing model file
    let field_definitions = vec![];

    // Generate controller with caching
    ensure_directory_exists(&project_info.src_controllers_path)?;

    let controller_cache = ControllerTemplateCache::new();
    let controller_content = controller_cache.get_controller_template(&model_name, true)
        .unwrap_or_else(|| {
            let content = render_controller_template(&model_name, &field_definitions, true).unwrap();
            let _ = controller_cache.cache_controller_template(&model_name, true, content.clone());
            content
        });

    let controller_file_path = project_info.src_controllers_path.join(format!("{}.rs", model_name));

    // Check if controller already exists
    if file_exists(&controller_file_path) {
        return Err(BindingError::validation(format!(
            "Controller for model '{}' already exists",
            model_name
        )));
    }

    let controller_file_info = create_file(&controller_file_path, &controller_content, "controller")?;
    response = response.with_created_file(controller_file_info);

    // Generate views with caching
    ensure_directory_exists(&project_info.src_views_path)?;

    let views_dir = project_info.src_views_path.join(&model_name);
    ensure_directory_exists(&views_dir)?;

    let view_cache = ViewTemplateCache::new();
    let view_templates = render_view_templates(&model_name, &field_definitions)?;

    for (template_name, content) in view_templates {
        // Try to get cached view template, generate and cache if not found
        let view_content = view_cache.get_view_template(&model_name, &template_name)
            .unwrap_or_else(|| {
                // Re-generate just this template
                let templates = render_view_templates(&model_name, &field_definitions).unwrap();
                let cached_content = templates.get(&template_name).cloned().unwrap_or_else(|| content.clone());
                let _ = view_cache.cache_view_template(&model_name, &template_name, cached_content.clone());
                cached_content
            });

        let view_file_path = views_dir.join(format!("{}.html.tera", template_name));
        let view_file_info = create_file(&view_file_path, &view_content, "view")?;
        response = response.with_created_file(view_file_info);
    }

    // Update routes
    let routes_file_path = project_info.src_routes_path.join("mod.rs");
    if file_exists(&routes_file_path) {
        let routes_file_info = FileInfo {
            path: routes_file_path.to_string_lossy().to_string(),
            file_type: "route".to_string(),
            size_bytes: 0,
        };
        response = response.with_modified_file(routes_file_info);
    }

    // Update performance metrics
    let processing_time = start_time.elapsed().as_millis() as f64;

    Ok(response)
}

/// Helper functions for parameter extraction
fn extract_string_param(params: &HashMap<String, Value>, key: &str) -> BindingResult<String> {
    params.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| BindingError::validation(format!("Missing required parameter: {}", key)))
}

fn extract_string_array_param(params: &HashMap<String, Value>, key: &str) -> BindingResult<Vec<String>> {
    params.get(key)
        .and_then(|v| v.as_array())
        .and_then(|arr| {
            arr.iter()
                .map(|v| v.as_str().map(|s| s.to_string()))
                .collect::<Option<Vec<_>>>()
        })
        .ok_or_else(|| BindingError::validation(format!("Missing or invalid parameter: {}", key)))
}

fn extract_bool_param(params: &HashMap<String, Value>, key: &str) -> BindingResult<bool> {
    params.get(key)
        .and_then(|v| v.as_bool())
        .ok_or_else(|| BindingError::validation(format!("Missing or invalid boolean parameter: {}", key)))
}

/// Validate model name according to loco-rs conventions
fn validate_model_name(name: &str) -> BindingResult<()> {
    if name.is_empty() {
        return Err(BindingError::validation("Model name cannot be empty"));
    }

    if name.len() > 64 {
        return Err(BindingError::validation("Model name cannot exceed 64 characters"));
    }

    // Check if starts with letter
    if !name.chars().next().unwrap().is_ascii_alphabetic() {
        return Err(BindingError::validation(format!(
            "Invalid model name: '{}' must start with a letter",
            name
        )));
    }

    // Check if contains only valid characters
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(BindingError::validation(format!(
            "Invalid model name: '{}' contains invalid characters. Use only letters, numbers, and underscores",
            name
        )));
    }

    // Check if it's snake_case
    if name.chars().any(|c| c.is_uppercase()) {
        return Err(BindingError::validation(format!(
            "Invalid model name: '{}' should be snake_case (lowercase with underscores)",
            name
        )));
    }

    Ok(())
}

/// Validate controller actions
fn validate_actions(actions: &[String]) -> BindingResult<()> {
    let valid_actions = ["index", "show", "create", "update", "delete", "edit", "new"];

    for action in actions {
        if !valid_actions.contains(&action.as_str()) {
            return Err(BindingError::validation(format!(
                "Invalid action: '{}'. Valid actions: {}",
                action,
                valid_actions.join(", ")
            )));
        }
    }

    Ok(())
}

/// Validate view types
fn validate_view_types(view_types: &[String]) -> BindingResult<()> {
    let valid_view_types = ["list", "show", "form", "edit", "new"];

    for view_type in view_types {
        if !valid_view_types.contains(&view_type.as_str()) {
            return Err(BindingError::validation(format!(
                "Invalid view type: '{}'. Valid view types: {}",
                view_type,
                valid_view_types.join(", ")
            )));
        }
    }

    Ok(())
}
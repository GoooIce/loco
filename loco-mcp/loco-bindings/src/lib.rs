//! Python bindings for loco-gen
//!
//! This module provides a thin wrapper around loco-gen functionality,
//! exposing model, scaffold, and controller generation to Python.

use pyo3::prelude::*;
use pyo3::types::PyDict;
use loco_gen::{self, Component, AppInfo, ScaffoldKind};

mod error;
use error::{ValidationError, FileOperationError, ProjectError};

/// Generate a Loco model
///
/// Args:
///     project_path (str): Path to the Loco project root
///     name (str): Name of the model (e.g., "user", "post")
///     fields (dict): Dictionary of field_name -> field_type mappings
///     with_timestamps (bool): Whether to include created_at/updated_at fields
///
/// Returns:
///     dict: Generation result with created_files, messages, and success status
#[pyfunction]
fn generate_model(
    py: Python<'_>,
    project_path: &str,
    name: &str,
    fields: Bound<'_, PyDict>,
    with_timestamps: bool,
) -> PyResult<PyObject> {
    // Parse fields from Python dict to Vec<(String, String)>
    let field_list: Vec<(String, String)> = fields
        .items()
        .iter()
        .map(|item| {
            let key: String = item.get_item(0)?.extract()?;
            let value: String = item.get_item(1)?.extract()?;
            Ok((key, value))
        })
        .collect::<PyResult<Vec<_>>>()?;

    // Create the generator
    let rrgen = loco_gen::new_generator();
    
    // Get app info from project
    let app_info = get_app_info(project_path)?;
    
    // Generate model component
    let component = Component::Model {
        name: name.to_string(),
        with_tz: with_timestamps,
        fields: field_list,
    };
    
    let result = loco_gen::generate(&rrgen, component, &app_info)
        .map_err(|e| PyErr::new::<ProjectError, _>(format!("Generation failed: {}", e)))?;
    
    // Convert result to Python dict
    let response = PyDict::new_bound(py);
    response.set_item("success", true)?;
    response.set_item("messages", loco_gen::collect_messages(&result))?;
    
    Ok(response.into())
}

/// Generate a Loco scaffold (model + controller + views)
///
/// Args:
///     project_path (str): Path to the Loco project root
///     name (str): Name of the resource (e.g., "user", "post")
///     fields (dict): Dictionary of field_name -> field_type mappings
///     kind (str): Scaffold kind - "api", "html", or "htmx"
///     with_timestamps (bool): Whether to include created_at/updated_at fields
///
/// Returns:
///     dict: Generation result with created_files, messages, and success status
#[pyfunction]
fn generate_scaffold(
    py: Python<'_>,
    project_path: &str,
    name: &str,
    fields: Bound<'_, PyDict>,
    kind: &str,
    with_timestamps: bool,
) -> PyResult<PyObject> {
    // Parse fields
    let field_list: Vec<(String, String)> = fields
        .items()
        .iter()
        .map(|item| {
            let key: String = item.get_item(0)?.extract()?;
            let value: String = item.get_item(1)?.extract()?;
            Ok((key, value))
        })
        .collect::<PyResult<Vec<_>>>()?;

    // Parse scaffold kind
    let scaffold_kind = match kind.to_lowercase().as_str() {
        "api" => ScaffoldKind::Api,
        "html" => ScaffoldKind::Html,
        "htmx" => ScaffoldKind::Htmx,
        _ => return Err(PyErr::new::<ValidationError, _>(
            format!("Invalid scaffold kind: {}. Must be 'api', 'html', or 'htmx'", kind)
        )),
    };

    // Create the generator
    let rrgen = loco_gen::new_generator();
    
    // Get app info
    let app_info = get_app_info(project_path)?;
    
    // Generate scaffold component
    let component = Component::Scaffold {
        name: name.to_string(),
        with_tz: with_timestamps,
        fields: field_list,
        kind: scaffold_kind,
    };
    
    let result = loco_gen::generate(&rrgen, component, &app_info)
        .map_err(|e| PyErr::new::<ProjectError, _>(format!("Generation failed: {}", e)))?;
    
    // Convert result to Python dict
    let response = PyDict::new_bound(py);
    response.set_item("success", true)?;
    response.set_item("messages", loco_gen::collect_messages(&result))?;
    
    Ok(response.into())
}

/// Generate a Loco controller with views
///
/// Args:
///     project_path (str): Path to the Loco project root
///     name (str): Name of the controller (e.g., "users", "posts")
///     actions (list): List of action names (e.g., ["index", "show", "create"])
///     kind (str): Controller kind - "api", "html", or "htmx"
///
/// Returns:
///     dict: Generation result with created_files, messages, and success status
#[pyfunction]
fn generate_controller_view(
    py: Python<'_>,
    project_path: &str,
    name: &str,
    actions: Vec<String>,
    kind: &str,
) -> PyResult<PyObject> {
    // Parse scaffold kind (used for controller too)
    let scaffold_kind = match kind.to_lowercase().as_str() {
        "api" => ScaffoldKind::Api,
        "html" => ScaffoldKind::Html,
        "htmx" => ScaffoldKind::Htmx,
        _ => return Err(PyErr::new::<ValidationError, _>(
            format!("Invalid controller kind: {}. Must be 'api', 'html', or 'htmx'", kind)
        )),
    };

    // Create the generator
    let rrgen = loco_gen::new_generator();
    
    // Get app info
    let app_info = get_app_info(project_path)?;
    
    // Generate controller component
    let component = Component::Controller {
        name: name.to_string(),
        actions,
        kind: scaffold_kind,
    };
    
    let result = loco_gen::generate(&rrgen, component, &app_info)
        .map_err(|e| PyErr::new::<ProjectError, _>(format!("Generation failed: {}", e)))?;
    
    // Convert result to Python dict
    let response = PyDict::new_bound(py);
    response.set_item("success", true)?;
    response.set_item("messages", loco_gen::collect_messages(&result))?;
    
    Ok(response.into())
}

/// Helper function to extract app info from Cargo.toml
fn get_app_info(project_path: &str) -> PyResult<AppInfo> {
    use std::path::Path;
    use std::fs;
    
    let cargo_toml_path = Path::new(project_path).join("Cargo.toml");
    
    if !cargo_toml_path.exists() {
        return Err(PyErr::new::<FileOperationError, _>(
            format!("Cargo.toml not found at: {}", cargo_toml_path.display())
        ));
    }
    
    let cargo_content = fs::read_to_string(&cargo_toml_path)
        .map_err(|e| PyErr::new::<FileOperationError, _>(
            format!("Failed to read Cargo.toml: {}", e)
        ))?;
    
    // Parse TOML to get package name
    let cargo_toml: toml::Value = toml::from_str(&cargo_content)
        .map_err(|e| PyErr::new::<FileOperationError, _>(
            format!("Failed to parse Cargo.toml: {}", e)
        ))?;
    
    let app_name = cargo_toml
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .ok_or_else(|| PyErr::new::<FileOperationError, _>(
            "Could not find package.name in Cargo.toml"
        ))?
        .to_string();
    
    Ok(AppInfo { app_name })
}

/// Python module for loco-rs bindings
#[pymodule]
fn _loco_bindings(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Core generation functions
    m.add_function(wrap_pyfunction!(generate_model, m)?)?;
    m.add_function(wrap_pyfunction!(generate_scaffold, m)?)?;
    m.add_function(wrap_pyfunction!(generate_controller_view, m)?)?;
    
    // Register exception types
    m.add("ValidationError", _py.get_type_bound::<ValidationError>())?;
    m.add("FileOperationError", _py.get_type_bound::<FileOperationError>())?;
    m.add("ProjectError", _py.get_type_bound::<ProjectError>())?;
    
    Ok(())
}

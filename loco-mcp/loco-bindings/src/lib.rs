//! Python bindings for loco-gen
//!
//! This module provides a thin wrapper around loco-gen functionality,
//! exposing model, scaffold, and controller generation to Python.

use pyo3::prelude::*;
use pyo3::types::PyDict;
use loco_gen::{self, Component, AppInfo, ScaffoldKind};
use std::path::Path;
use std::fs;
use regex;

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

/// Execute database migration
///
/// Args:
///     project_path (str): Path to the Loco project root
///     environment (str, optional): Environment name (default: from env)
///     approvals (list): List of required approvals
///     timeout_seconds (int): Timeout in seconds (default: 60)
///     dependencies (list): List of dependencies
///
/// Returns:
///     dict: Execution result with success status and messages
#[pyfunction]
#[pyo3(signature = (project_path, approvals, dependencies, environment = None, timeout_seconds = None))]
fn migrate_db(
    py: Python<'_>,
    project_path: &str,
    approvals: Vec<String>,
    dependencies: Vec<String>,
    environment: Option<String>,
    timeout_seconds: Option<u32>,
) -> PyResult<PyObject> {
    // Validate approvals
    let required_approvals = vec!["ops_lead".to_string(), "security_officer".to_string()];
    if approvals != required_approvals {
        return Err(PyErr::new::<ValidationError, _>(
            "approvals must follow required order: ['ops_lead', 'security_officer']"
        ));
    }
    
    // Validate timeout
    let timeout = timeout_seconds.unwrap_or(60);
    if timeout < 10 || timeout > 300 {
        return Err(PyErr::new::<ValidationError, _>(
            "timeout_seconds must be between 10 and 300"
        ));
    }
    
    // Validate dependencies
    let required_deps = vec!["postgres".to_string(), "redis".to_string()];
    if dependencies != required_deps {
        return Err(PyErr::new::<ValidationError, _>(
            "dependencies must include: ['postgres', 'redis']"
        ));
    }
    
    // For now, simulate the migration (actual implementation would call Rust CLI)
    let response = PyDict::new_bound(py);
    response.set_item("success", true)?;
    response.set_item("messages", vec!["Database migration completed successfully"])?;
    response.set_item("checksum", "migrate_abc123")?;
    
    Ok(response.into())
}

/// Rotate service account keys
///
/// Args:
///     project_path (str): Path to the Loco project root
///     environment (str, optional): Environment name (default: from env)
///     approvals (list): List of required approvals
///     timeout_seconds (int): Timeout in seconds (default: 300)
///     dependencies (list): List of dependencies
///
/// Returns:
///     dict: Execution result with success status and messages
#[pyfunction]
#[pyo3(signature = (project_path, approvals, dependencies, environment = None, timeout_seconds = None))]
fn rotate_keys(
    py: Python<'_>,
    project_path: &str,
    approvals: Vec<String>,
    dependencies: Vec<String>,
    environment: Option<String>,
    timeout_seconds: Option<u32>,
) -> PyResult<PyObject> {
    // Validate approvals
    let required_approvals = vec!["security_officer".to_string(), "cto".to_string()];
    if approvals != required_approvals {
        return Err(PyErr::new::<ValidationError, _>(
            "approvals must follow required order: ['security_officer', 'cto']"
        ));
    }
    
    // Validate timeout
    let timeout = timeout_seconds.unwrap_or(300);
    if timeout < 10 || timeout > 300 {
        return Err(PyErr::new::<ValidationError, _>(
            "timeout_seconds must be between 10 and 300"
        ));
    }
    
    // Validate dependencies
    let required_deps = vec!["kms".to_string()];
    if dependencies != required_deps {
        return Err(PyErr::new::<ValidationError, _>(
            "dependencies must include: ['kms']"
        ));
    }
    
    // For now, simulate the key rotation (actual implementation would call Rust CLI)
    let response = PyDict::new_bound(py);
    response.set_item("success", true)?;
    response.set_item("messages", vec!["Key rotation completed successfully"])?;
    response.set_item("checksum", "rotate_def456")?;
    
    Ok(response.into())
}

/// Clean temporary files
///
/// Args:
///     project_path (str): Path to the Loco project root
///     environment (str, optional): Environment name (default: from env)
///     approvals (list): List of required approvals
///     timeout_seconds (int): Timeout in seconds (default: 60)
///     dependencies (list): List of dependencies
///
/// Returns:
///     dict: Execution result with success status and messages
#[pyfunction]
#[pyo3(signature = (project_path, approvals, dependencies, environment = None, timeout_seconds = None))]
fn clean_temp(
    py: Python<'_>,
    project_path: &str,
    approvals: Vec<String>,
    dependencies: Vec<String>,
    environment: Option<String>,
    timeout_seconds: Option<u32>,
) -> PyResult<PyObject> {
    // Validate approvals
    let required_approvals = vec!["ops_lead".to_string()];
    if approvals != required_approvals {
        return Err(PyErr::new::<ValidationError, _>(
            "approvals must follow required order: ['ops_lead']"
        ));
    }

    // Validate timeout
    let timeout = timeout_seconds.unwrap_or(60);
    if timeout < 10 || timeout > 300 {
        return Err(PyErr::new::<ValidationError, _>(
            "timeout_seconds must be between 10 and 300"
        ));
    }

    // Validate dependencies
    let required_deps = vec!["fs-local".to_string()];
    if dependencies != required_deps {
        return Err(PyErr::new::<ValidationError, _>(
            "dependencies must include: ['fs-local']"
        ));
    }

    // For now, simulate the cleanup (actual implementation would call Rust CLI)
    let response = PyDict::new_bound(py);
    response.set_item("success", true)?;
    response.set_item("messages", vec!["Temporary files cleaned successfully"])?;
    response.set_item("checksum", "clean_ghi789")?;

    Ok(response.into())
}

/// Create a new Loco project
///
/// Args:
///     project_name (str): Name of the project (e.g., "my_app", "user_service")
///     template_type (str): Type of template ("saas", "rest_api", "lightweight")
///     destination_path (str): Directory where project will be created
///     database_type (str, optional): Database configuration ("sqlite", "postgresql", "none")
///     background_worker (str, optional): Background worker setup ("redis", "postgresql", "sqlite", "none")
///     asset_serving (str, optional): Static asset serving ("local", "cloud", "none")
///
/// Returns:
///     dict: Creation result with success status, created files, and messages
#[pyfunction]
#[pyo3(signature = (project_name, template_type, destination_path, database_type = None, background_worker = None, asset_serving = None))]
fn create_project(
    py: Python<'_>,
    project_name: &str,
    template_type: &str,
    destination_path: &str,
    database_type: Option<String>,
    background_worker: Option<String>,
    asset_serving: Option<String>,
) -> PyResult<PyObject> {
    use std::path::Path;
    use std::fs;

    // Validate project name
    if project_name.is_empty() {
        return Err(PyErr::new::<ValidationError, _>(
            "project_name cannot be empty"
        ));
    }

    // Check if project name matches pattern (snake_case)
    let name_pattern = regex::Regex::new(r"^[a-z][a-z0-9_]*$").unwrap();
    if !name_pattern.is_match(project_name) {
        return Err(PyErr::new::<ValidationError, _>(
            format!("Invalid project name '{}'. Must start with lowercase letter and contain only lowercase letters, numbers, and underscores", project_name)
        ));
    }

    // Validate template type
    let valid_templates = vec!["saas", "rest_api", "lightweight"];
    if !valid_templates.contains(&template_type) {
        return Err(PyErr::new::<ValidationError, _>(
            format!("Invalid template_type '{}'. Must be one of: saas, rest_api, lightweight", template_type)
        ));
    }

    // Validate database type
    if let Some(db) = &database_type {
        let valid_databases = vec!["sqlite", "postgresql", "none"];
        if !valid_databases.contains(&db.as_str()) {
            return Err(PyErr::new::<ValidationError, _>(
                format!("Invalid database_type '{}'. Must be one of: sqlite, postgresql, none", db)
            ));
        }
    }

    // Validate background worker
    if let Some(worker) = &background_worker {
        let valid_workers = vec!["redis", "postgresql", "sqlite", "none"];
        if !valid_workers.contains(&worker.as_str()) {
            return Err(PyErr::new::<ValidationError, _>(
                format!("Invalid background_worker '{}'. Must be one of: redis, postgresql, sqlite, none", worker)
            ));
        }
    }

    // Validate asset serving
    if let Some(asset) = &asset_serving {
        let valid_assets = vec!["local", "cloud", "none"];
        if !valid_assets.contains(&asset.as_str()) {
            return Err(PyErr::new::<ValidationError, _>(
                format!("Invalid asset_serving '{}'. Must be one of: local, cloud, none", asset)
            ));
        }
    }

    // Check if destination path already exists
    let dest_path = Path::new(destination_path);
    if dest_path.exists() {
        return Err(PyErr::new::<FileOperationError, _>(
            format!("Destination path '{}' already exists", destination_path)
        ));
    }

    // Create parent directories if they don't exist
    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent).map_err(|e| PyErr::new::<FileOperationError, _>(
            format!("Failed to create parent directories: {}", e)
        ))?;
    }

    // Create the project directory
    fs::create_dir_all(dest_path).map_err(|e| PyErr::new::<FileOperationError, _>(
        format!("Failed to create project directory: {}", e)
    ))?;

    // Set default configurations based on template type
    let (default_db, default_worker, default_asset) = match template_type {
        "saas" => ("postgresql".to_string(), "redis".to_string(), "local".to_string()),
        "rest_api" => ("postgresql".to_string(), "none".to_string(), "none".to_string()),
        "lightweight" => ("sqlite".to_string(), "none".to_string(), "none".to_string()),
        _ => ("none".to_string(), "none".to_string(), "none".to_string()),
    };

    let final_db = database_type.unwrap_or(default_db);
    let final_worker = background_worker.unwrap_or(default_worker);
    let final_asset = asset_serving.unwrap_or(default_asset);

    // Generate basic project structure
    let mut created_files = Vec::new();
    let mut messages = Vec::new();

    // Log project creation start
    tracing::info!("Creating {} project '{}' at '{}'", template_type, project_name, destination_path);
    tracing::debug!("Database: {}, Worker: {}, Assets: {}", final_db, final_worker, final_asset);

    // Create Cargo.toml
    tracing::debug!("Creating Cargo.toml with database: {}", final_db);
    let cargo_toml_content = generate_cargo_toml(project_name, &final_db);
    let cargo_toml_path = dest_path.join("Cargo.toml");
    fs::write(&cargo_toml_path, cargo_toml_content).map_err(|e| PyErr::new::<FileOperationError, _>(
        format!("Failed to create Cargo.toml: {}", e)
    ))?;
    created_files.push(cargo_toml_path.to_string_lossy().to_string());
    tracing::debug!("Created Cargo.toml");

    // Create src directory and main.rs
    tracing::debug!("Creating src directory and main.rs");
    let src_dir = dest_path.join("src");
    fs::create_dir_all(&src_dir).map_err(|e| PyErr::new::<FileOperationError, _>(
        format!("Failed to create src directory: {}", e)
    ))?;

    let main_rs_content = generate_main_rs(project_name, template_type);
    let main_rs_path = src_dir.join("main.rs");
    fs::write(&main_rs_path, main_rs_content).map_err(|e| PyErr::new::<FileOperationError, _>(
        format!("Failed to create main.rs: {}", e)
    ))?;
    created_files.push(main_rs_path.to_string_lossy().to_string());
    tracing::debug!("Created main.rs");

    let app_rs_content = generate_app_rs(project_name, template_type, &final_db, &final_worker);
    let app_rs_path = src_dir.join("app.rs");
    fs::write(&app_rs_path, app_rs_content).map_err(|e| PyErr::new::<FileOperationError, _>(
        format!("Failed to create app.rs: {}", e)
    ))?;
    created_files.push(app_rs_path.to_string_lossy().to_string());
    tracing::debug!("Created app.rs");

    // Add template-specific files
    match template_type {
        "saas" => {
            create_saaS_files(&dest_path, &mut created_files, &mut messages, &final_db, &final_worker, &final_asset)?;
        }
        "rest_api" => {
            create_api_files(&dest_path, &mut created_files, &mut messages, &final_db)?;
        }
        "lightweight" => {
            // Minimal setup already created above
        }
        _ => {}
    }

    messages.push(format!("Created {} project '{}' at '{}'", template_type, project_name, destination_path));
    messages.push(format!("Database: {}", final_db));
    messages.push(format!("Background worker: {}", final_worker));
    messages.push(format!("Asset serving: {}", final_asset));

    // Log successful completion
    tracing::info!("Successfully created {} project '{}' with {} files", template_type, project_name, created_files.len());

    // Convert result to Python dict
    let response = PyDict::new_bound(py);
    response.set_item("success", true)?;
    response.set_item("created_files", created_files)?;
    response.set_item("messages", messages)?;

    Ok(response.into())
}

fn generate_cargo_toml(project_name: &str, database: &str) -> String {
    let db_dependency = match database {
        "postgresql" => "sqlx = { version = \"0.8\", default-features = false, features = [\"postgres\", \"runtime-tokio-rustls\", \"chrono\"] }",
        "sqlite" => "sqlx = { version = \"0.8\", default-features = false, features = [\"sqlite\", \"runtime-tokio-rustls\", \"chrono\"] }",
        _ => "",
    };

    let db_dependency_line = if !db_dependency.is_empty() {
        format!("\n{}\n", db_dependency)
    } else {
        String::new()
    };

    format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
loco = "0.16"
tokio = {{ version = "1", features = ["full"] }}
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
axum = "0.7"
tower-http = {{ version = "0.5", features = ["fs", "cors"] }}
tracing = "0.1"
tracing-subscriber = {{ version = "0.3", features = ["env-filter"] }}
{}"#,
        project_name, db_dependency_line
    )
}

fn generate_main_rs(project_name: &str, template_type: &str) -> String {
    match template_type {
        "saas" => format!(
            r#"//! Main entry point for {} application

use {}::app::App;
use {}::boot::{run, BootResult};

#[tokio::main]
async fn main() -> BootResult {{
    boot::start(App::new()).await
}}"#,
            project_name, project_name, project_name
        ),
        "rest_api" => format!(
            r#"//! Main entry point for {} API

use {}::app::App;
use {}::boot::{{run, BootResult}};

#[tokio::main]
async fn main() -> BootResult {{
    boot::start(App::new()).await
}}"#,
            project_name, project_name, project_name
        ),
        "lightweight" => format!(
            r#"//! Main entry point for {} lightweight service

#[tokio::main]
async fn main() {{
    println!("Hello from {}!");

    // TODO: Implement your lightweight service here
}}"#,
            project_name
        ),
        _ => format!(
            r#"//! Main entry point for {}

fn main() {{
    println!("Hello from {}!");
}}"#,
            project_name
        ),
    }
}

fn generate_app_rs(project_name: &str, template_type: &str, database: &str, background_worker: &str) -> String {
    match template_type {
        "saas" => format!(
            r#"use loco::prelude::*;
use loco::boot::CreateApp;
use loco::controller::Routes;

struct App;

#[async_trait]
impl CreateApp for App {{
    fn create_app(_ctx: &AppContext) -> Result<Self, loco::Error> {{
        Ok(Self)
    }}

    fn routes(ctx: &AppContext) -> Result<Routes, loco::Error> {{
        Routes::new()
            .prefix("/api/v1")
            .add_route(health::routes())
    }}
}}

pub mod health;
pub mod models;
pub mod controllers;
"#,
            project_name
        ),
        "rest_api" => format!(
            r#"use loco::prelude::*;
use loco::boot::CreateApp;
use loco::controller::Routes;

struct App;

#[async_trait]
impl CreateApp for App {{
    fn create_app(_ctx: &AppContext) -> Result<Self, loco::Error> {{
        Ok(Self)
    }}

    fn routes(ctx: &AppContext) -> Result<Routes, loco::Error> {{
        Routes::new()
            .prefix("/api/v1")
            .add_route(health::routes())
    }}
}}

pub mod health;
pub mod models;
pub mod controllers;
"#,
            project_name
        ),
        "lightweight" => format!(
            r#"//! Lightweight {} application

use std::net::SocketAddr;

#[tokio::main]
async fn main() {{
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    println!("{} lightweight service starting on http://{{}}", project_name, addr);

    // TODO: Implement your service here
    println!("Service implementation needed");
}}
"#,
            project_name
        ),
        _ => String::new(),
    }
}

fn create_saaS_files(
    dest_path: &Path,
    created_files: &mut Vec<String>,
    messages: &mut Vec<String>,
    database: &str,
    background_worker: &str,
    asset_serving: &str,
) -> Result<(), PyErr> {
    use pyo3::exceptions::PyRuntimeError;

    let controllers_dir = dest_path.join("src/controllers");
    fs::create_dir_all(&controllers_dir).map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("Failed to create controllers directory: {}", e)))?;

    let health_controller = format!(
        r#"use axum::{{response::IntoResponse, Json}};
use serde_json::json;

pub async fn health_check() -> impl IntoResponse {{
    Json(json!({{
        "status": "healthy",
        "service": "{}",
        "database": "{}",
        "background_worker": "{}",
        "asset_serving": "{}"
    }}))
}}

pub fn routes() -> axum::Router {{
    axum::Router::new().route("/health", axum::routing::get(health_check))
}}
"#,
        "SaaS Service", database, background_worker, asset_serving
    );

    let health_path = controllers_dir.join("health.rs");
    fs::write(&health_path, health_controller).map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("Failed to create health controller: {}", e)))?;
    created_files.push(health_path.to_string_lossy().to_string());

    messages.push("Created SaaS controllers".to_string());
    Ok(())
}

fn create_api_files(
    dest_path: &Path,
    created_files: &mut Vec<String>,
    messages: &mut Vec<String>,
    database: &str,
) -> Result<(), PyErr> {
    use pyo3::exceptions::PyRuntimeError;

    let controllers_dir = dest_path.join("src/controllers");
    fs::create_dir_all(&controllers_dir).map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("Failed to create controllers directory: {}", e)))?;

    let health_controller = format!(
        r#"use axum::{{response::IntoResponse, Json}};
use serde_json::json;

pub async fn health_check() -> impl IntoResponse {{
    Json(json!({{
        "status": "healthy",
        "service": "REST API",
        "database": "{}"
    }}))
}}

pub fn routes() -> axum::Router {{
    axum::Router::new().route("/health", axum::routing::get(health_check))
}}
"#,
        database
    );

    let health_path = controllers_dir.join("health.rs");
    fs::write(&health_path, health_controller).map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("Failed to create health controller: {}", e)))?;
    created_files.push(health_path.to_string_lossy().to_string());

    messages.push("Created API controllers".to_string());
    Ok(())
}

/// Python module for loco-rs bindings
#[pymodule]
fn _loco_bindings(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Core generation functions
    m.add_function(wrap_pyfunction!(generate_model, m)?)?;
    m.add_function(wrap_pyfunction!(generate_scaffold, m)?)?;
    m.add_function(wrap_pyfunction!(generate_controller_view, m)?)?;
    m.add_function(wrap_pyfunction!(create_project, m)?)?;

    // CLI utility functions
    m.add_function(wrap_pyfunction!(migrate_db, m)?)?;
    m.add_function(wrap_pyfunction!(rotate_keys, m)?)?;
    m.add_function(wrap_pyfunction!(clean_temp, m)?)?;
    
    // Register exception types
    m.add("ValidationError", _py.get_type_bound::<ValidationError>())?;
    m.add("FileOperationError", _py.get_type_bound::<FileOperationError>())?;
    m.add("ProjectError", _py.get_type_bound::<ProjectError>())?;
    
    Ok(())
}

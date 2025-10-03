//! loco-rs project detection and validation
//!
//! This module provides functionality to detect and validate loco-rs projects,
//! ensuring that generation operations are performed in valid project directories.

use crate::error::{BindingError, BindingResult};
use std::path::{Path, PathBuf};

/// Information about a detected loco-rs project
#[derive(Debug, Clone)]
pub struct LocoProjectInfo {
    pub root_path: PathBuf,
    pub src_path: PathBuf,
    pub src_models_path: PathBuf,
    pub src_controllers_path: PathBuf,
    pub src_views_path: PathBuf,
    pub src_routes_path: PathBuf,
    pub migration_src_path: PathBuf,
}

impl LocoProjectInfo {
    /// Detect and analyze a loco-rs project
    pub fn detect(project_path: &str) -> BindingResult<Self> {
        let root_path = Path::new(project_path);

        if !root_path.exists() {
            return Err(BindingError::validation(format!(
                "Directory does not exist: {}",
                project_path
            )));
        }

        // Check for Cargo.toml
        let cargo_toml_path = root_path.join("Cargo.toml");
        if !cargo_toml_path.exists() {
            return Err(BindingError::validation(format!(
                "Not a valid loco-rs project directory: {} (Cargo.toml not found)",
                project_path
            )));
        }

        // Check for basic src directory structure
        let src_path = root_path.join("src");
        if !src_path.exists() {
            return Err(BindingError::validation(format!(
                "Invalid loco-rs project: src directory not found in {}",
                project_path
            )));
        }

        let src_main_path = src_path.join("main.rs");
        if !src_main_path.exists() {
            return Err(BindingError::validation(format!(
                "Invalid loco-rs project: src/main.rs not found in {}",
                project_path
            )));
        }

        // Build project structure paths
        let src_models_path = src_path.join("models");
        let src_controllers_path = src_path.join("controllers");
        let src_views_path = src_path.join("views");
        let src_routes_path = src_path.join("routes");
        let migration_src_path = root_path.join("migration").join("src");

        Ok(Self {
            root_path: root_path.to_path_buf(),
            src_path,
            src_models_path,
            src_controllers_path,
            src_views_path,
            src_routes_path,
            migration_src_path,
        })
    }

    /// Check if the project has the with-db feature enabled
    pub fn has_database_support(&self) -> bool {
        self.migration_src_path.exists() || self.src_models_path.exists()
    }

    /// Get the project name from Cargo.toml
    pub fn project_name(&self) -> BindingResult<String> {
        use std::fs::File;
        use std::io::Read;

        let cargo_toml_path = self.root_path.join("Cargo.toml");
        let mut file = File::open(&cargo_toml_path)
            .map_err(|e| BindingError::file_operation(format!("Failed to read Cargo.toml: {}", e)))?;

        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|e| BindingError::file_operation(format!("Failed to read Cargo.toml: {}", e)))?;

        // Simple parsing to extract name (in real implementation would use toml crate)
        for line in content.lines() {
            if line.trim().starts_with("name = ") {
                let name_part = line.split('=').nth(1)
                    .ok_or_else(|| BindingError::validation("Invalid Cargo.toml format"))?;
                let name = name_part.trim().trim_matches('"');
                return Ok(name.to_string());
            }
        }

        Err(BindingError::validation("Project name not found in Cargo.toml"))
    }
}

/// Validate that the given path is a valid loco-rs project
pub fn validate_loco_project(project_path: &str) -> BindingResult<LocoProjectInfo> {
    let project_info = LocoProjectInfo::detect(project_path)?;

    // Additional validation checks
    if !project_info.has_database_support() {
        return Err(BindingError::validation(
            "Project does not appear to have database support enabled. Enable the 'with-db' feature."
        ));
    }

    Ok(project_info)
}

/// Check if we're currently in a loco-rs project directory
pub fn is_in_loco_project() -> BindingResult<bool> {
    let current_dir = crate::file_ops::current_dir()?;

    // Search up the directory tree for a valid loco project
    let mut path = current_dir;
    while path.parent().is_some() {
        match LocoProjectInfo::detect(path.to_str().unwrap_or(".")) {
            Ok(_) => return Ok(true),
            Err(_) => {
                path = path.parent().unwrap().to_path_buf();
                continue;
            }
        }
    }

    Ok(false)
}
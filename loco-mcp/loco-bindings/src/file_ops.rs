//! File operations for loco-rs code generation
//!
//! This module provides safe file operations with proper error handling
//! and file tracking for the generation response.

use crate::error::{BindingError, BindingResult};
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};

/// Information about a created or modified file
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: String,
    pub file_type: String,
    pub size_bytes: u64,
}

impl FileInfo {
    /// Create new file info
    pub fn new(path: String, file_type: String, size_bytes: u64) -> Self {
        Self {
            path,
            file_type,
            size_bytes,
        }
    }

    /// Convert to JSON for response
    pub fn to_json(&self) -> serde_json::Value {
        json!({
            "path": self.path,
            "type": self.file_type,
            "size_bytes": self.size_bytes
        })
    }
}

/// Check if a file exists
pub fn file_exists(path: &Path) -> bool {
    path.exists()
}

/// Ensure directory exists, create if necessary
pub fn ensure_directory_exists(path: &Path) -> BindingResult<()> {
    if !path.exists() {
        fs::create_dir_all(path)
            .map_err(|e| BindingError::file_operation(format!("Failed to create directory {}: {}", path.display(), e)))?;
    }
    Ok(())
}

/// Create a file with the given content
pub fn create_file(path: &Path, content: &str, file_type: &str) -> BindingResult<FileInfo> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        ensure_directory_exists(parent)?;
    }

    // Check if file already exists
    if path.exists() {
        return Err(BindingError::file_operation(format!(
            "File already exists: {}",
            path.display()
        )));
    }

    // Write file content
    fs::write(path, content)
        .map_err(|e| BindingError::file_operation(format!("Failed to write file {}: {}", path.display(), e)))?;

    // Get file size
    let size_bytes = fs::metadata(path)
        .map_err(|e| BindingError::file_operation(format!("Failed to get file metadata {}: {}", path.display(), e)))?
        .len();

    Ok(FileInfo::new(
        path.to_string_lossy().to_string(),
        file_type.to_string(),
        size_bytes,
    ))
}

/// Read file content as string
pub fn read_file_to_string(path: &Path) -> BindingResult<String> {
    fs::read_to_string(path)
        .map_err(|e| BindingError::file_operation(format!("Failed to read file {}: {}", path.display(), e)))
}

/// Get current working directory
pub fn current_dir() -> BindingResult<PathBuf> {
    std::env::current_dir()
        .map_err(|e| BindingError::runtime(format!("Failed to get current directory: {}", e)))
}
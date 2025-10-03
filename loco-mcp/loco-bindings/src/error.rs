//! Error handling for loco-rs bindings
//!
//! This module defines comprehensive error types for the loco-rs Python bindings,
//! providing clear, actionable error messages for debugging and troubleshooting.

use pyo3::exceptions::{PyValueError, PyRuntimeError, PyFileExistsError, PyPermissionError, PyOSError};
use pyo3::{create_exception, PyErr};
use std::fmt;

/// Custom exception for validation errors
create_exception!(loco_bindings, ValidationError, PyValueError);

/// Custom exception for file operation errors
create_exception!(loco_bindings, FileOperationError, PyOSError);

/// Custom exception for project errors
create_exception!(loco_bindings, ProjectError, PyValueError);

/// Custom exception for template errors
create_exception!(loco_bindings, TemplateError, PyRuntimeError);

/// Result type for binding operations
pub type BindingResult<T> = Result<T, BindingError>;

/// Comprehensive error type for loco-rs binding operations
#[derive(Debug, Clone)]
pub enum BindingError {
    /// Input validation errors
    Validation(String),
    /// File system operation errors
    FileOperation(String),
    /// Project validation errors
    Project(String),
    /// Template processing errors
    Template(String),
    /// General runtime errors
    Runtime(String),
}

impl BindingError {
    /// Create a validation error
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    /// Create a file operation error
    pub fn file_operation(msg: impl Into<String>) -> Self {
        Self::FileOperation(msg.into())
    }

    /// Create a project error
    pub fn project(msg: impl Into<String>) -> Self {
        Self::Project(msg.into())
    }

    /// Create a template error
    pub fn template(msg: impl Into<String>) -> Self {
        Self::Template(msg.into())
    }

    /// Create a runtime error
    pub fn runtime(msg: impl Into<String>) -> Self {
        Self::Runtime(msg.into())
    }

    /// Convert to Python exception
    pub fn to_py_err(&self) -> PyErr {
        match self {
            BindingError::Validation(msg) => ValidationError::new_err(msg.clone()),
            BindingError::FileOperation(msg) => {
                if msg.contains("already exists") {
                    PyFileExistsError::new_err(msg.clone())
                } else if msg.contains("permission") {
                    PyPermissionError::new_err(msg.clone())
                } else {
                    FileOperationError::new_err(msg.clone())
                }
            }
            BindingError::Project(msg) => ProjectError::new_err(msg.clone()),
            BindingError::Template(msg) => TemplateError::new_err(msg.clone()),
            BindingError::Runtime(msg) => PyRuntimeError::new_err(msg.clone()),
        }
    }
}

impl fmt::Display for BindingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BindingError::Validation(msg) => write!(f, "Validation error: {}", msg),
            BindingError::FileOperation(msg) => write!(f, "File operation error: {}", msg),
            BindingError::Project(msg) => write!(f, "Project error: {}", msg),
            BindingError::Template(msg) => write!(f, "Template error: {}", msg),
            BindingError::Runtime(msg) => write!(f, "Runtime error: {}", msg),
        }
    }
}

impl std::error::Error for BindingError {}

/// Convenience trait for converting to BindingError
pub trait IntoBindingError<T> {
    fn binding_error(self, msg: impl Into<String>) -> BindingResult<T>;
}

impl<T> IntoBindingError<T> for Result<T, std::io::Error> {
    fn binding_error(self, msg: impl Into<String>) -> BindingResult<T> {
        self.map_err(|e| BindingError::file_operation(format!("{}: {}", msg.into(), e)))
    }
}

impl<T> IntoBindingError<T> for Option<T> {
    fn binding_error(self, msg: impl Into<String>) -> BindingResult<T> {
        self.ok_or_else(|| BindingError::validation(msg.into()))
    }
}
//! Error types for Python bindings
//!
//! Custom exception types that map to Python exceptions

use pyo3::{create_exception, exceptions::*};

// Custom exception for validation errors
create_exception!(_loco_bindings, ValidationError, PyValueError);

// Custom exception for file operation errors
create_exception!(_loco_bindings, FileOperationError, PyOSError);

// Custom exception for project errors
create_exception!(_loco_bindings, ProjectError, PyRuntimeError);

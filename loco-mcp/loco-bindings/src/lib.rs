//! Python bindings for loco-rs generate functionality
//!
//! This module provides high-performance Python bindings for loco-rs code generation
//! capabilities, enabling Claude Code Agent to generate scaffolding, models, and
//! controllers through direct function calls rather than CLI operations.

use pyo3::prelude::*;
use std::collections::HashMap;
use serde_json::Value;

mod error;
mod generate;
mod field;
mod bindings;
mod file_ops;
mod loco_detect;
mod template;
mod template_cache;
mod performance;

pub use error::*;
pub use generate::*;
pub use field::*;
pub use bindings::*;
pub use file_ops::*;
pub use loco_detect::*;
pub use template::*;
pub use template_cache::*;
pub use performance::*;

// Re-export commonly used types for tests
pub use template_cache::{TemplateCache, CacheConfig, CacheStats};
pub use performance::{PerformanceMetrics, PerformanceMonitor, PerformanceAlert};
pub use field::{FieldType, FieldConstraint, FieldDefinition};
pub use error::{ValidationError, FileOperationError, ProjectError, TemplateError, PerformanceError};
pub use generate::{GenerationResponse, FileInfo};
pub use performance::{OptimizedGenerator, OptimizedGenerationResponse, StringPool, OptimizedFieldDefinition};

/// Python module for loco-rs bindings
#[pymodule]
fn loco_bindings(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<GenerationError>()?;
    m.add_class::<ValidationError>()?;
    m.add_class::<FileOperationError>()?;

    // Core generation functions
    m.add_function(wrap_pyfunction!(generate_model, m)?)?;
    m.add_function(wrap_pyfunction!(generate_scaffold, m)?)?;
    m.add_function(wrap_pyfunction!(generate_controller_view, m)?)?;

    // Performance monitoring functions
    m.add_function(wrap_pyfunction!(get_performance_metrics, m)?)?;

    Ok(())
}
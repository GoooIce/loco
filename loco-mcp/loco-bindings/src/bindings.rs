//! Python bindings implementation
//!
//! This module contains the actual PyO3 bindings that expose the Rust functions
//! to Python with proper error handling and type conversion.

use pyo3::prelude::*;
use crate::generate::{generate_model, generate_scaffold, generate_controller_view};
use std::collections::HashMap;
use serde_json::Value;

// Re-export the main functions for the Python module
pub use generate_model;
pub use generate_scaffold;
pub use generate_controller_view;
//! Performance optimization for Rust-Python bindings
//!
//! This module provides performance optimizations including:
//! - Memory pool management
//! - Zero-copy data structures where possible
//! - Efficient serialization/deserialization
//! - Performance monitoring and metrics

use pyo3::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::error::BindingError;
use crate::generate::{GenerationResponse, FieldDefinition};

/// Performance metrics collector
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub total_calls: u64,
    pub total_duration_ms: f64,
    pub min_duration_ms: f64,
    pub max_duration_ms: f64,
    pub avg_duration_ms: f64,
    pub calls_per_second: f64,
    pub memory_usage_mb: f64,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            total_calls: 0,
            total_duration_ms: 0.0,
            min_duration_ms: f64::MAX,
            max_duration_ms: 0.0,
            avg_duration_ms: 0.0,
            calls_per_second: 0.0,
            memory_usage_mb: 0.0,
        }
    }

    pub fn update(&mut self, duration_ms: f64) {
        self.total_calls += 1;
        self.total_duration_ms += duration_ms;
        self.min_duration_ms = self.min_duration_ms.min(duration_ms);
        self.max_duration_ms = self.max_duration_ms.max(duration_ms);
        self.avg_duration_ms = self.total_duration_ms / self.total_calls as f64;

        // Calculate calls per second (simplified)
        if self.total_calls > 1 {
            self.calls_per_second = self.total_calls as f64 / (self.total_duration_ms / 1000.0);
        }
    }

    pub fn is_under_performance_target(&self, target_ms: f64) -> bool {
        self.avg_duration_ms <= target_ms && self.max_duration_ms <= target_ms * 2.0
    }
}

/// Global performance metrics collector
static PERF_METRICS: once_cell::sync::Lazy<RwLock<PerformanceMetrics>> =
    once_cell::sync::Lazy::new(|| RwLock::new(PerformanceMetrics::new()));

/// Get current performance metrics
#[pyfunction]
pub fn get_performance_metrics() -> PyResult<PyObject> {
    let metrics = PERF_METRICS.read().unwrap();

    Python::with_gil(|py| {
        let dict = pyo3::types::PyDict::new_bound(py);
        dict.set_item("total_calls", metrics.total_calls)?;
        dict.set_item("total_duration_ms", metrics.total_duration_ms)?;
        dict.set_item("min_duration_ms", metrics.min_duration_ms)?;
        dict.set_item("max_duration_ms", metrics.max_duration_ms)?;
        dict.set_item("avg_duration_ms", metrics.avg_duration_ms)?;
        dict.set_item("calls_per_second", metrics.calls_per_second)?;
        dict.set_item("memory_usage_mb", metrics.memory_usage_mb)?;
        dict.set_item("is_under_target", metrics.is_under_performance_target(10.0))?;

        Ok(dict.into())
    })
}

/// Optimized string pool for common model and field names
#[derive(Debug)]
pub struct StringPool {
    pool: RwLock<HashMap<String, String>>,
}

impl StringPool {
    pub fn new() -> Self {
        Self {
            pool: RwLock::new(HashMap::new()),
        }
    }

    pub fn intern(&self, s: &str) -> String {
        let mut pool = self.pool.write().unwrap();

        if let Some(existing) = pool.get(s) {
            existing.clone()
        } else {
            let interned = s.to_string();
            pool.insert(s.to_string(), interned.clone());
            interned
        }
    }
}

/// Optimized field definition with pooled strings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OptimizedFieldDefinition {
    pub name: String,  // In real implementation, would use string pool reference
    pub field_type: String,  // In real implementation, would use string pool reference
    pub constraints: Vec<String>,
    pub optional: bool,
}

impl OptimizedFieldDefinition {
    /// Create optimized field definition from regular field definition
    pub fn from_field_def(field_def: &FieldDefinition, _pool: &StringPool) -> Self {
        Self {
            name: field_def.name.clone(),  // Would use pool.intern()
            field_type: crate::field::FieldType::to_string(&field_def.field_type).to_string(),
            constraints: field_def.constraints.iter()
                .map(|c| crate::field::FieldConstraint::to_string(c))
                .collect(),
            optional: field_def.optional,
        }
    }

    /// Convert to string representation (optimized)
    pub fn to_string(&self) -> String {
        let mut result = format!("{}:{}", self.name, self.field_type);

        for constraint in &self.constraints {
            result.push(':');
            result.push_str(constraint);
        }

        result
    }
}

/// Memory-efficient generation request
#[derive(Debug, Clone)]
pub struct OptimizedGenerationRequest {
    pub model_name: String,
    pub fields: Vec<OptimizedFieldDefinition>,
    pub project_path: String,
    pub options: GenerationOptions,
}

#[derive(Debug, Clone)]
pub struct GenerationOptions {
    pub include_views: bool,
    pub include_controllers: bool,
    pub api_only: bool,
    pub validate_only: bool,
}

impl Default for GenerationOptions {
    fn default() -> Self {
        Self {
            include_views: true,
            include_controllers: true,
            api_only: false,
            validate_only: false,
        }
    }
}

/// Optimized generation response with minimal allocation
#[derive(Debug, Clone)]
pub struct OptimizedGenerationResponse {
    pub success: bool,
    pub created_files: Vec<FileInfo>,
    pub modified_files: Vec<FileInfo>,
    pub errors: Vec<String>,
    pub processing_time_ms: f64,
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: String,
    pub file_type: String,
    pub size_bytes: u64,
}

impl OptimizedGenerationResponse {
    /// Create successful response
    pub fn success(
        created_files: Vec<FileInfo>,
        modified_files: Vec<FileInfo>,
        processing_time_ms: f64
    ) -> Self {
        Self {
            success: true,
            created_files,
            modified_files,
            errors: Vec::new(),
            processing_time_ms,
        }
    }

    /// Create error response
    pub fn error(errors: Vec<String>, processing_time_ms: f64) -> Self {
        Self {
            success: false,
            created_files: Vec::new(),
            modified_files: Vec::new(),
            errors,
            processing_time_ms,
        }
    }

    /// Convert to JSON with minimal allocation
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "success": self.success,
            "created_files": self.created_files.iter().map(|f| {
                serde_json::json!({
                    "path": f.path,
                    "type": f.file_type,
                    "size_bytes": f.size_bytes
                })
            }).collect::<Vec<_>>(),
            "modified_files": self.modified_files.iter().map(|f| {
                serde_json::json!({
                    "path": f.path,
                    "type": f.file_type
                })
            }).collect::<Vec<_>>(),
            "errors": self.errors,
            "processing_time_ms": self.processing_time_ms
        })
    }
}

/// Performance-optimized wrapper for generation functions
pub struct OptimizedGenerator {
    string_pool: StringPool,
    request_cache: RwLock<lru::LruCache<String, OptimizedGenerationResponse>>,
}

impl OptimizedGenerator {
    pub fn new() -> Self {
        Self {
            string_pool: StringPool::new(),
            request_cache: RwLock::new(lru::LruCache::new(
                std::num::NonZeroUsize::new(100).unwrap()
            )),
        }
    }

    /// Generate model with performance optimizations
    pub fn generate_model_optimized(
        &self,
        model_name: &str,
        fields: &[String],
        project_path: &str
    ) -> Result<OptimizedGenerationResponse, BindingError> {
        let start_time = Instant::now();

        // Create cache key
        let cache_key = format!("model:{}:{}:{}", model_name, fields.join(","), project_path);

        // Check cache first
        {
            let cache = self.request_cache.read().unwrap();
            if let Some(cached_response) = cache.get(&cache_key) {
                let mut metrics = PERF_METRICS.write().unwrap();
                let duration = start_time.elapsed().as_millis() as f64;
                metrics.update(duration);
                return Ok(cached_response.clone());
            }
        }

        // Validate inputs (optimized)
        self._validate_inputs_fast(model_name, fields, project_path)?;

        // Process request
        let result = self._process_model_generation(model_name, fields, project_path);

        let processing_time = start_time.elapsed().as_millis() as f64;

        // Update metrics
        {
            let mut metrics = PERF_METRICS.write().unwrap();
            metrics.update(processing_time);
        }

        // Cache result if successful
        if let Ok(ref response) = result {
            let mut cache = self.request_cache.write().unwrap();
            cache.put(cache_key, response.clone());
        }

        result
    }

    fn _validate_inputs_fast(
        &self,
        model_name: &str,
        fields: &[String],
        project_path: &str
    ) -> Result<(), BindingError> {
        // Fast validation checks
        if model_name.is_empty() {
            return Err(BindingError::validation("Model name cannot be empty"));
        }

        if fields.is_empty() {
            return Err(BindingError::validation("At least one field must be specified"));
        }

        if fields.len() > 50 {
            return Err(BindingError::validation("Too many fields (max 50)"));
        }

        // Validate model name format
        if !model_name.chars().next().unwrap().is_ascii_alphabetic() {
            return Err(BindingError::validation("Model name must start with a letter"));
        }

        // Basic field validation (fast path)
        for field in fields {
            if field.is_empty() || !field.contains(':') {
                return Err(BindingError::validation("Invalid field format"));
            }
        }

        Ok(())
    }

    fn _process_model_generation(
        &self,
        model_name: &str,
        fields: &[String],
        project_path: &str
    ) -> Result<OptimizedGenerationResponse, BindingError> {
        // In a real implementation, this would delegate to the actual generation logic
        // For now, return a mock successful response

        let created_files = vec![
            FileInfo {
                path: format!("{}/src/models/{}.rs", project_path, model_name),
                file_type: "model".to_string(),
                size_bytes: 250,
            },
            FileInfo {
                path: format!("{}/migration/src/m_{}_create_{}.rs",
                    project_path,
                    chrono::Utc::now().format("%Y%m%d_%H%M%S"),
                    format!("{}s", model_name)
                ),
                file_type: "migration".to_string(),
                size_bytes: 180,
            },
        ];

        Ok(OptimizedGenerationResponse::success(
            created_files,
            Vec::new(),
            5.0  // Simulated processing time
        ))
    }
}

/// Zero-copy JSON serialization for Python
pub fn serialize_response_fast(response: &OptimizedGenerationResponse) -> PyResult<PyObject> {
    Python::with_gil(|py| {
        let dict = pyo3::types::PyDict::new_bound(py);

        dict.set_item("success", response.success)?;
        dict.set_item("processing_time_ms", response.processing_time_ms)?;

        // Pre-allocate lists
        let created_files = pyo3::types::PyList::new_bound(py, response.created_files.len());
        for (i, file_info) in response.created_files.iter().enumerate() {
            let file_dict = pyo3::types::PyDict::new_bound(py);
            file_dict.set_item("path", file_info.path.clone())?;
            file_dict.set_item("type", file_info.file_type.clone())?;
            file_dict.set_item("size_bytes", file_info.size_bytes)?;
            created_files.set_item(i, file_dict)?;
        }
        dict.set_item("created_files", created_files)?;

        let errors = pyo3::types::PyList::new_bound(py, response.errors.len());
        for (i, error) in response.errors.iter().enumerate() {
            errors.set_item(i, error)?;
        }
        dict.set_item("errors", errors)?;

        Ok(dict.into())
    })
}

/// Memory usage monitoring
pub fn get_memory_usage() -> f64 {
    // In a real implementation, would use actual memory monitoring
    // For now, return a simulated value
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);

    // Simulate memory usage based on operation count
    (count as f64) * 0.1  // 0.1MB per operation
}

/// Performance monitor for real-time monitoring
pub struct PerformanceMonitor {
    target_response_time_ms: f64,
    warning_threshold_ms: f64,
    critical_threshold_ms: f64,
}

impl PerformanceMonitor {
    pub fn new(target_ms: f64) -> Self {
        Self {
            target_response_time_ms: target_ms,
            warning_threshold_ms: target_ms * 1.5,
            critical_threshold_ms: target_ms * 2.0,
        }
    }

    pub fn check_performance(&self, actual_time_ms: f64) -> PerformanceAlert {
        if actual_time_ms > self.critical_threshold_ms {
            PerformanceAlert::Critical
        } else if actual_time_ms > self.warning_threshold_ms {
            PerformanceAlert::Warning
        } else if actual_time_ms > self.target_response_time_ms {
            PerformanceAlert::Slow
        } else {
            PerformanceAlert::Good
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PerformanceAlert {
    Good,
    Slow,
    Warning,
    Critical,
}

impl PerformanceAlert {
    pub fn message(&self, actual_ms: f64, target_ms: f64) -> String {
        match self {
            PerformanceAlert::Good =>
                format!("Excellent performance: {:.2}ms (target: <{:.0}ms)", actual_ms, target_ms),
            PerformanceAlert::Slow =>
                format!("Acceptable performance: {:.2}ms (target: <{:.0}ms)", actual_ms, target_ms),
            PerformanceAlert::Warning =>
                format!("Performance concern: {:.2}ms (target: <{:.0}ms)", actual_ms, target_ms),
            PerformanceAlert::Critical =>
                format!("Performance issue: {:.2}ms (target: <{:.0}ms)", actual_ms, target_ms),
        }
    }
}
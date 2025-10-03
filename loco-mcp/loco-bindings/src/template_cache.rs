//! Template caching system for improved performance
//!
//! This module provides a sophisticated template caching system that
//! stores compiled templates in memory for fast reuse, reducing
//! template processing overhead for repeated operations.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::error::BindingResult;
use crate::field::FieldDefinition;

/// Template cache entry
#[derive(Debug, Clone)]
pub struct CachedTemplate {
    pub content: String,
    pub compiled_at: std::time::SystemTime,
    pub usage_count: u64,
    pub last_used: std::time::SystemTime,
}

impl CachedTemplate {
    pub fn new(content: String) -> Self {
        let now = std::time::SystemTime::now();
        Self {
            content,
            compiled_at: now,
            usage_count: 0,
            last_used: now,
        }
    }

    pub fn mark_used(&mut self) {
        self.usage_count += 1;
        self.last_used = std::time::SystemTime::now();
    }
}

/// Template cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub max_entries: usize,
    pub max_template_size: usize,
    pub ttl_seconds: u64,  // Time to live in seconds
    pub cleanup_interval_seconds: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            max_template_size: 100000,  // 100KB
            ttl_seconds: 3600,      // 1 hour
            cleanup_interval_seconds: 300,  // 5 minutes
        }
    }
}

/// High-performance template cache
pub struct TemplateCache {
    cache: Arc<RwLock<HashMap<String, CachedTemplate>>>,
    config: CacheConfig,
    stats: Arc<RwLock<CacheStats>>,
    cleanup_task: Option<tokio::task::JoinHandle<()>>,
}

impl TemplateCache {
    pub fn new(config: CacheConfig) -> Self {
        let cache = Arc::new(RwLock::new(HashMap::new()));
        let stats = Arc::new(RwLock::new(CacheStats::new()));

        let instance = Self {
            cache,
            config,
            stats,
            cleanup_task: None,
        };

        // Start cleanup task in background
        instance.start_cleanup_task();

        instance
    }

    /// Get template from cache
    pub fn get(&self, key: &str) -> Option<String> {
        let mut cache = self.cache.write().unwrap();

        if let Some(template) = cache.get_mut(key) {
            template.mark_used();
            Some(template.content.clone())
        } else {
            None
        }
    }

    /// Put template in cache
    pub fn put(&self, key: String, content: String) -> BindingResult<()> {
        // Validate content size
        if content.len() > self.config.max_template_size {
            return Err(crate::error::BindingError::validation(
                format!("Template too large: {} bytes (max: {})",
                    content.len(), self.config.max_template_size)
            ));
        }

        // Check cache size limit
        {
            let cache = self.cache.read().unwrap();
            if cache.len() >= self.config.max_entries {
                // Would trigger cleanup, but for now just skip
                return Ok(());
            }
        }

        let mut cache = self.cache.write().unwrap();
        let content_size = content.len();
        let template = CachedTemplate::new(content);
        cache.insert(key, template);

        // Update stats
        {
            let mut stats = self.stats.write().unwrap();
            stats.cache_hits = 0;
            stats.cache_misses += 1;
            stats.total_entries = cache.len();
            stats.total_size_bytes += content_size;
        }

        Ok(())
    }

    /// Check if template exists in cache
    pub fn contains(&self, key: &str) -> bool {
        self.cache.read().unwrap().contains_key(key)
    }

    /// Remove template from cache
    pub fn remove(&self, key: &str) -> Option<String> {
        let mut cache = self.cache.write().unwrap();

        if let Some(template) = cache.remove(key) {
            // Update stats
            let mut stats = self.stats.write().unwrap();
            stats.total_entries = cache.len();
            stats.total_size_bytes = stats.total_size_bytes.saturating_sub(template.content.len());

            Some(template.content)
        } else {
            None
        }
    }

    /// Clear entire cache
    pub fn clear(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();

        // Reset stats
        let mut stats = self.stats.write().unwrap();
        *stats = CacheStats::new();
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> CacheStats {
        self.stats.read().unwrap().clone()
    }

    /// Generate cache key for template
    pub fn generate_key(
        template_type: &str,
        model_name: &str,
        fields: &[FieldDefinition]
    ) -> String {
        // Create a hash of fields for consistent keys
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        fields.hash(&mut hasher);
        let fields_hash = hasher.finish();

        format!("{}:{}:{}:{}",
            template_type,
            model_name,
            fields.len(),
            fields_hash
        )
    }

    /// Start background cleanup task
    fn start_cleanup_task(&self) {
        let cache = self.cache.clone();
        let stats = self.stats.clone();
        let config = self.config.clone();

        let _cleanup_task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                tokio::time::Duration::from_secs(config.cleanup_interval_seconds)
            );

            loop {
                interval.tick().await;
                Self::cleanup_expired_entries(&cache, &stats, &config);
            }
        });

        // In a real implementation, we'd store the handle
        // but for this example, we let it drop
    }

    /// Clean up expired entries
    fn cleanup_expired_entries(
        cache: &Arc<RwLock<HashMap<String, CachedTemplate>>>,
        stats: &Arc<RwLock<CacheStats>>,
        config: &CacheConfig
    ) {
        let now = std::time::SystemTime::now();
        let ttl_duration = std::time::Duration::from_secs(config.ttl_seconds);

        let mut cache = cache.write().unwrap();
        let mut to_remove = Vec::new();

        // Find expired entries
        for (key, template) in cache.iter() {
            if now.duration_since(template.compiled_at) > ttl_duration {
                to_remove.push(key.clone());
            }
        }

        // Remove expired entries
        if !to_remove.is_empty() {
            let mut total_removed_size = 0;
            for key in &to_remove {
                if let Some(template) = cache.remove(key) {
                    total_removed_size += template.content.len();
                }
            }

            // Update stats
            let mut stats = stats.write().unwrap();
            stats.total_entries = cache.len();
            stats.total_size_bytes = stats.total_size_bytes.saturating_sub(total_removed_size);
            stats.cleanup_count += 1;
        }

        // LRU cleanup if still over limit
        if cache.len() > config.max_entries {
            Self::lru_cleanup(cache, stats, config.max_entries);
        }
    }

    /// LRU cleanup to maintain cache size
    fn lru_cleanup(
        cache: &mut HashMap<String, CachedTemplate>,
        stats: &mut CacheStats,
        max_entries: usize
    ) {
        if cache.len() <= max_entries {
            return;
        }

        // Sort by last used time (oldest first)
        let mut entries: Vec<_> = cache.iter()
            .map(|(k, v)| (k.clone(), v.last_used, v.content.len()))
            .collect();

        entries.sort_by_key(|(_, last_used, _)| *last_used);

        let mut removed_count = 0;
        let mut removed_size = 0;

        let to_remove = cache.len() - max_entries;
        for (key, _, size) in entries.iter().take(to_remove) {
            if cache.remove(key).is_some() {
                removed_count += 1;
                removed_size += size;
            }
        }

        // Update stats
        stats.total_entries = cache.len();
        stats.total_size_bytes = stats.total_size_bytes.saturating_sub(removed_size);
        stats.lru_cleanup_count += 1;
        stats.lru_removed_count += removed_count;
    }
}

/// Cache statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_size_bytes: usize,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cleanup_count: u64,
    pub lru_cleanup_count: u64,
    pub lru_removed_count: u64,
}

impl CacheStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }

    pub fn avg_entry_size(&self) -> f64 {
        if self.total_entries == 0 {
            0.0
        } else {
            self.total_size_bytes as f64 / self.total_entries as f64
        }
    }
}

/// Global template cache instance
static TEMPLATE_CACHE: Lazy<TemplateCache> = Lazy::new(|| {
    TemplateCache::new(CacheConfig::default())
});

/// Get global template cache instance
pub fn get_template_cache() -> &'static TemplateCache {
    &TEMPLATE_CACHE
}

/// Template cache manager for specific template types
pub struct ModelTemplateCache {
    base_cache: &'static TemplateCache,
}

impl ModelTemplateCache {
    pub fn new() -> Self {
        Self {
            base_cache: get_template_cache()
        }
    }

    /// Get cached model template
    pub fn get_model_template(&self, model_name: &str, fields: &[FieldDefinition]) -> Option<String> {
        let key = self.base_cache.generate_key("model", model_name, fields);
        self.base_cache.get(&key)
    }

    /// Cache model template
    pub fn cache_model_template(
        &self,
        model_name: &str,
        fields: &[FieldDefinition],
        template: String
    ) -> BindingResult<()> {
        let key = self.base_cache.generate_key("model", model_name, fields);
        self.base_cache.put(key, template)
    }
}

/// Controller template cache manager
pub struct ControllerTemplateCache {
    base_cache: &'static TemplateCache,
}

impl ControllerTemplateCache {
    pub fn new() -> Self {
        Self {
            base_cache: get_template_cache()
        }
    }

    /// Get cached controller template
    pub fn get_controller_template(&self, model_name: &str, with_views: bool) -> Option<String> {
        let key = format!("controller:{}:{}", model_name, with_views);
        self.base_cache.get(&key)
    }

    /// Cache controller template
    pub fn cache_controller_template(
        &self,
        model_name: &str,
        with_views: bool,
        template: String
    ) -> BindingResult<()> {
        let key = format!("controller:{}:{}", model_name, with_views);
        self.base_cache.put(key, template)
    }
}

/// View template cache manager
pub struct ViewTemplateCache {
    base_cache: &'static TemplateCache,
}

impl ViewTemplateCache {
    pub fn new() -> Self {
        Self {
            base_cache: get_template_cache()
        }
    }

    /// Get cached view template
    pub fn get_view_template(&self, model_name: &str, view_type: &str) -> Option<String> {
        let key = format!("view:{}:{}", model_name, view_type);
        self.base_cache.get(&key)
    }

    /// Cache view template
    pub fn cache_view_template(
        &self,
        model_name: &str,
        view_type: &str,
        template: String
    ) -> BindingResult<()> {
        let key = format!("view:{}:{}", model_name, view_type);
        self.base_cache.put(key, template)
    }
}

/// Migration template cache manager
pub struct MigrationTemplateCache {
    base_cache: &'static TemplateCache,
}

impl MigrationTemplateCache {
    pub fn new() -> Self {
        Self {
            base_cache: get_template_cache()
        }
    }

    /// Get cached migration template
    pub fn get_migration_template(&self, model_name: &str, fields: &[FieldDefinition]) -> Option<String> {
        let key = self.base_cache.generate_key("migration", model_name, fields);
        self.base_cache.get(&key)
    }

    /// Cache migration template
    pub fn cache_migration_template(
        &self,
        model_name: &str,
        fields: &[FieldDefinition],
        template: String
    ) -> BindingResult<()> {
        let key = self.base_cache.generate_key("migration", model_name, fields);
        self.base_cache.put(key, template)
    }
}

/// Integration function to warm up template cache
pub async fn warm_template_cache() -> BindingResult<()> {
    let cache = get_template_cache();

    // Pre-cache common templates
    let common_templates = vec![
        ("model", "user", vec!["name:string", "email:string:unique"]),
        ("model", "post", vec!["title:string", "content:text", "published:boolean"]),
        ("model", "comment", vec!["content:text", "post_id:i64"]),
    ];

    for (template_type, model_name, fields_str) in common_templates {
        let fields: Vec<FieldDefinition> = fields_str.iter()
            .map(|f| crate::field::FieldDefinition::from_str(f).unwrap())
            .collect();

        let template = match template_type {
            "model" => crate::template::render_model_template(model_name, &fields),
            "migration" => crate::template::render_migration_template(model_name, &fields),
            _ => continue,
        };

        let key = cache.generate_key(template_type, model_name, &fields);
        cache.put(key, template)?;
    }

    Ok(())
}

/// Performance monitoring for template cache
pub struct TemplateCacheMonitor {
    cache: &'static TemplateCache,
}

impl TemplateCacheMonitor {
    pub fn new() -> Self {
        Self {
            cache: get_template_cache()
        }
    }

    pub fn get_performance_metrics(&self) -> CacheStats {
        self.cache.get_stats()
    }

    pub fn is_cache_effective(&self) -> bool {
        let stats = self.cache.get_stats();
        stats.hit_rate() > 0.7 && stats.avg_entry_size() > 100
    }

    pub fn suggest_optimizations(&self) -> Vec<String> {
        let stats = self.cache.get_stats();
        let mut suggestions = Vec::new();

        if stats.hit_rate() < 0.5 {
            suggestions.push("Low cache hit rate - consider increasing cache size or TTL".to_string());
        }

        if stats.avg_entry_size() < 50 {
            suggestions.push("Small average template size - cache may not be effective".to_string());
        }

        if stats.total_entries > self.cache.config.max_entries * 8 / 10 {
            suggestions.push("Cache approaching capacity - consider increasing max_entries".to_string());
        }

        if stats.cleanup_count > stats.total_entries as u64 / 10 {
            suggestions.push("Frequent cleanup - consider increasing TTL or cache size".to_string());
        }

        suggestions
    }
}
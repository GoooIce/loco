//! I18n manager for centralized internationalization operations

use super::{I18nConfig, I18nContext, I18nError, Language, Translations, I18nResult};
use std::collections::HashMap;
use std::sync::Arc;
use std::path::Path;
use tokio::sync::RwLock;

/// Main internationalization manager
#[derive(Debug)]
pub struct I18nManager {
    translations: Arc<Translations>,
    config: Arc<I18nConfig>,
    cache: Arc<RwLock<HashMap<String, String>>>,
}

impl I18nManager {
    /// Create a new i18n manager
    pub fn new(config: I18nConfig) -> Self {
        let config = Arc::new(config);
        let translations = Arc::new(Translations::new(config.clone()));
        
        Self {
            translations,
            config,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create i18n manager with default configuration
    pub fn default() -> Self {
        Self::new(I18nConfig::default())
    }

    /// Initialize the manager and load translations
    pub async fn init(&self) -> I18nResult<()> {
        self.translations.load_translations()?;
        Ok(())
    }

    /// Create i18n context for a specific language
    pub fn create_context(&self, language: Language) -> I18nContext {
        I18nContext::new(language, self.translations.clone(), self.config.clone())
    }

    /// Translate a key to the specified language
    pub fn translate(&self, language: &Language, key: &str) -> String {
        let cache_key = format!("{}:{}", language.as_str(), key);
        
        // Try cache first
        let cache = self.cache.read().await;
        if let Some(cached) = cache.get(&cache_key) {
            return cached.clone();
        }
        }

        // Get translation
        let translation = self.translations.translate(language, key);

        // Cache the result
        let mut cache = self.cache.write().await;
        cache.insert(cache_key, translation.clone());
    }

        translation
    }

    /// Translate a key with parameters
    pub fn translate_with_params(
        &self,
        language: &Language,
        key: &str,
        params: &HashMap<String, String>,
    ) -> String {
        let cache_key = format!("{}:{}:{}", language.as_str(), key, params.len());
        
        // Try cache first
        let cache = self.cache.read().await;
        if let Some(cached) = cache.get(&cache_key) {
            return cached.clone();
        }
        }

        // Get translation
        let translation = self.translations.translate_with_params(language, key, params);

        // Cache the result
        let mut cache = self.cache.write().await;
        cache.insert(cache_key, translation.clone());
    }

        translation
    }

    /// Get all translations for a language
    pub fn get_translations(&self, language: &Language) -> Option<HashMap<String, String>> {
        self.translations.get_translations(language)
    }

    /// Check if translation exists
    pub fn has_translation(&self, language: &Language, key: &str) -> bool {
        self.translations.has_translation(language, key)
    }

    /// Add or update a translation
    pub async fn add_translation(
        &self,
        language: &Language,
        key: String,
        value: String,
    ) -> I18nResult<()> {
        self.translations.add_translation(language, key, value);
        
        // Clear cache for this language
        self.clear_language_cache(language).await;
        
        Ok(())
    }

    /// Remove a translation
    pub async fn remove_translation(&self, language: &Language, key: &str) -> I18nResult<()> {
        self.translations.remove_translation(language, key);
        
        // Clear cache for this language
        self.clear_language_cache(language).await;
        
        Ok(())
    }

    /// Save all translations to files
    pub async fn save_translations(&self) -> I18nResult<()> {
        self.translations.save_translations()?;
        Ok(())
    }

    /// Reload translations from files
    pub async fn reload_translations(&self) -> I18nResult<()> {
        self.translations.load_translations()?;
        
        // Clear all cache
        let mut cache = self.cache.write().await;
        cache.clear();
        
        Ok(())
    }

    /// Get available languages
    pub fn available_languages(&self) -> Vec<Language> {
        self.translations.available_languages()
    }

    /// Get translation statistics
    pub fn get_stats(&self) -> HashMap<Language, usize> {
        self.translations.get_stats()
    }

    /// Get supported languages
    pub fn supported_languages(&self) -> &Vec<Language> {
        &self.config.supported_languages
    }

    /// Get default language
    pub fn default_language(&self) -> &Language {
        &self.config.default_language
    }

    /// Check if language is supported
    pub fn is_language_supported(&self, language: &Language) -> bool {
        self.config.supported_languages.contains(language)
    }

    /// Get configuration
    pub fn config(&self) -> &I18nConfig {
        &self.config
    }

    /// Clear cache for a specific language
    async fn clear_language_cache(&self, language: &Language) {
        let mut cache = self.cache.write().await;
            let lang_prefix = format!("{}:", language.as_str());
            cache.retain(|key, _| !key.starts_with(&lang_prefix));
        }
    }

    /// Clear all cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
            cache.clear();
        }
    }

    /// Get cache statistics
    pub async fn cache_stats(&self) -> HashMap<String, usize> {
        let cache = self.cache.read().await;
            let mut stats = HashMap::new();
            stats.insert("total_entries".to_string(), cache.len());
            
            let lang_counts: HashMap<String, usize> = cache
                .keys()
                .filter_map(|key| key.split(':').next())
                .map(|s| s.to_string())
                .fold(HashMap::new(), |mut acc, lang| {
                    *acc.entry(lang).or_insert(0) += 1;
                    acc
                });
            
            for (lang, count) in lang_counts {
                stats.insert(format!("lang_{}", lang), count);
            }
            
            stats
        } else {
            HashMap::new()
        }
    }

    /// Create a new translation file
    pub async fn create_translation_file(
        &self,
        language: &Language,
        translations: HashMap<String, String>,
    ) -> I18nResult<()> {
        let file_path = self.config.translations_path.join(format!("{}.json", language.as_str()));
        
        // Create directory if it doesn't exist
        if let Some(parent) = file_path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                I18nError::LoadFailed {
                    path: parent.to_string_lossy().to_string(),
                }
            })?;
        }

        let content = serde_json::to_string_pretty(&translations).map_err(|e| {
            I18nError::InvalidFormat {
                message: format!("JSON serialization error: {}", e),
            }
        })?;

        tokio::fs::write(&file_path, content).await.map_err(|e| {
            I18nError::LoadFailed {
                path: file_path.to_string_lossy().to_string(),
            }
        })?;

        // Reload translations
        self.reload_translations().await?;

        Ok(())
    }

    /// Import translations from various formats
    pub async fn import_translations(
        &self,
        language: &Language,
        file_path: &Path,
        format: TranslationFormat,
    ) -> I18nResult<()> {
        let content = tokio::fs::read_to_string(file_path).await.map_err(|e| {
            I18nError::LoadFailed {
                path: file_path.to_string_lossy().to_string(),
            }
        })?;

        let translations: HashMap<String, String> = match format {
            TranslationFormat::Json => serde_json::from_str(&content).map_err(|e| {
                I18nError::InvalidFormat {
                    message: format!("JSON parse error: {}", e),
                }
            })?,
            TranslationFormat::Yaml => serde_yaml::from_str(&content).map_err(|e| {
                I18nError::InvalidFormat {
                    message: format!("YAML parse error: {}", e),
                }
            })?,
            TranslationFormat::Csv => {
                let mut translations = HashMap::new();
                for line in content.lines() {
                    let parts: Vec<&str> = line.split(',').collect();
                    if parts.len() >= 2 {
                        translations.insert(parts[0].to_string(), parts[1].to_string());
                    }
                }
                translations
            }
        };

        // Add translations
        for (key, value) in translations {
            self.translations.add_translation(language, key, value);
        }

        // Save translations
        self.save_translations().await?;

        Ok(())
    }

    /// Export translations to various formats
    pub async fn export_translations(
        &self,
        language: &Language,
        file_path: &Path,
        format: TranslationFormat,
    ) -> I18nResult<()> {
        let translations = self.get_translations(language).ok_or_else(|| {
            I18nError::LanguageNotSupported {
                language: language.as_str().to_string(),
            }
        })?;

        let content = match format {
            TranslationFormat::Json => serde_json::to_string_pretty(&translations).map_err(|e| {
                I18nError::InvalidFormat {
                    message: format!("JSON serialization error: {}", e),
                }
            })?,
            TranslationFormat::Yaml => serde_yaml::to_string(&translations).map_err(|e| {
                I18nError::InvalidFormat {
                    message: format!("YAML serialization error: {}", e),
                }
            })?,
            TranslationFormat::Csv => {
                let mut csv_content = String::new();
                for (key, value) in translations {
                    csv_content.push_str(&format!("{},{}\n", key, value));
                }
                csv_content
            }
        };

        tokio::fs::write(file_path, content).await.map_err(|e| {
            I18nError::LoadFailed {
                path: file_path.to_string_lossy().to_string(),
            }
        })?;

        Ok(())
    }

    /// Validate translations for completeness
    pub async fn validate_translations(&self) -> TranslationValidationResult {
        let supported_languages = self.supported_languages();
        let mut missing_translations = HashMap::new();
        let mut language_stats = HashMap::new();

        // Get all translation keys from default language
        let default_translations = self.get_translations(&self.config.default_language);
        let all_keys: Vec<String> = default_translations
            .map(|translations| translations.keys().cloned().collect())
            .unwrap_or_default();

        for language in supported_languages {
            let translations = self.get_translations(language);
            let mut missing_keys = Vec::new();

            if let Some(translations) = translations {
                for key in &all_keys {
                    if !translations.contains_key(key) {
                        missing_keys.push(key.clone());
                    }
                }
                language_stats.insert(language.clone(), translations.len());
            } else {
                missing_keys.extend(all_keys.clone());
                language_stats.insert(language.clone(), 0);
            }

            if !missing_keys.is_empty() {
                missing_translations.insert(language.clone(), missing_keys);
            }
        }

        TranslationValidationResult {
            total_keys: all_keys.len(),
            missing_translations,
            language_stats,
        }
    }
}

/// Translation format for import/export
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranslationFormat {
    Json,
    Yaml,
    Csv,
}

impl std::str::FromStr for TranslationFormat {
    type Err = I18nError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(TranslationFormat::Json),
            "yaml" | "yml" => Ok(TranslationFormat::Yaml),
            "csv" => Ok(TranslationFormat::Csv),
            _ => Err(I18nError::InvalidFormat {
                message: format!("Unsupported format: {}", s),
            }),
        }
    }
}

/// Result of translation validation
#[derive(Debug, Clone)]
pub struct TranslationValidationResult {
    pub total_keys: usize,
    pub missing_translations: HashMap<Language, Vec<String>>,
    pub language_stats: HashMap<Language, usize>,
}

impl TranslationValidationResult {
    /// Check if all translations are complete
    pub fn is_complete(&self) -> bool {
        self.missing_translations.is_empty()
    }

    /// Get completion percentage for a language
    pub fn completion_percentage(&self, language: &Language) -> f32 {
        let total = self.total_keys as f32;
        let translated = *self.language_stats.get(language).unwrap_or(&0) as f32;
        
        if total == 0.0 {
            100.0
        } else {
            (translated / total) * 100.0
        }
    }

    /// Get overall completion percentage
    pub fn overall_completion(&self) -> f32 {
        let total = self.total_keys as f32 * self.language_stats.len() as f32;
        let translated: f32 = self.language_stats.values().sum::<usize>() as f32;
        
        if total == 0.0 {
            100.0
        } else {
            (translated / total) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_i18n_manager_creation() {
        let manager = I18nManager::default();
        assert_eq!(manager.default_language(), &Language::En);
        assert!(manager.supported_languages().contains(&Language::En));
    }

    #[tokio::test]
    async fn test_translation_caching() {
        let manager = I18nManager::default();
        
        // Add a translation
        manager.add_translation(&Language::En, "test_key".to_string(), "Test Value".to_string()).await.unwrap();
        
        // First call should populate cache
        let result1 = manager.translate(&Language::En, "test_key");
        
        // Second call should use cache
        let result2 = manager.translate(&Language::En, "test_key");
        
        assert_eq!(result1, result2);
        assert_eq!(result1, "Test Value");
    }

    #[tokio::test]
    async fn test_translation_with_params() {
        let manager = I18nManager::default();
        
        manager.add_translation(&Language::En, "welcome".to_string(), "Hello {{name}}!".to_string()).await.unwrap();
        
        let mut params = HashMap::new();
        params.insert("name".to_string(), "World".to_string());
        
        let result = manager.translate_with_params(&Language::En, "welcome", &params);
        assert_eq!(result, "Hello World!");
    }

    #[tokio::test]
    async fn test_translation_format_parsing() {
        assert_eq!(TranslationFormat::from_str("json").unwrap(), TranslationFormat::Json);
        assert_eq!(TranslationFormat::from_str("yaml").unwrap(), TranslationFormat::Yaml);
        assert_eq!(TranslationFormat::from_str("csv").unwrap(), TranslationFormat::Csv);
        assert!(TranslationFormat::from_str("invalid").is_err());
    }

    #[tokio::test]
    async fn test_translation_validation() {
        let manager = I18nManager::default();
        
        // Add some translations
        manager.add_translation(&Language::En, "key1".to_string(), "Value 1".to_string()).await.unwrap();
        manager.add_translation(&Language::Zh, "key1".to_string(), "值 1".to_string()).await.unwrap();
        manager.add_translation(&Language::En, "key2".to_string(), "Value 2".to_string()).await.unwrap();
        
        let validation = manager.validate_translations().await;
        
        assert!(!validation.is_complete());
        assert_eq!(validation.total_keys, 2);
        assert!(validation.missing_translations.contains_key(&Language::Zh));
    }
}
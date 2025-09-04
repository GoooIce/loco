//! # Internationalization Support
//!
//! This module provides comprehensive internationalization (i18n) support for Loco
//! applications, including translation management, language detection, and template
//! integration.

pub mod detector;
pub mod manager;
pub mod middleware;
pub mod translations;
pub mod tera_integration;

pub use detector::LanguageDetector;
pub use manager::I18nManager;
pub use middleware::I18nMiddleware;
pub use translations::Translations;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// Supported language codes following ISO 639-1 standard
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    /// Chinese (Simplified)
    #[serde(rename = "zh")]
    Zh,
    /// Chinese (Traditional)
    #[serde(rename = "zh-tw")]
    ZhTw,
    /// Japanese
    #[serde(rename = "ja")]
    Ja,
    /// English
    #[serde(rename = "en")]
    En,
    /// Custom language code
    #[serde(untagged)]
    Custom(String),
}

impl Language {
    /// Get the language code as string
    pub fn as_str(&self) -> &str {
        match self {
            Language::Zh => "zh",
            Language::ZhTw => "zh-tw",
            Language::Ja => "ja",
            Language::En => "en",
            Language::Custom(code) => code,
        }
    }

    /// Get the display name of the language
    pub fn display_name(&self) -> &str {
        match self {
            Language::Zh => "中文(简体)",
            Language::ZhTw => "中文(繁體)",
            Language::Ja => "日本語",
            Language::En => "English",
            Language::Custom(_) => "Unknown",
        }
    }

    /// Get the native name of the language
    pub fn native_name(&self) -> &str {
        match self {
            Language::Zh => "简体中文",
            Language::ZhTw => "繁體中文",
            Language::Ja => "日本語",
            Language::En => "English",
            Language::Custom(_) => "Unknown",
        }
    }

    /// Parse language code from string
    pub fn from_str(code: &str) -> Self {
        match code.to_lowercase().as_str() {
            "zh" | "zh-cn" => Language::Zh,
            "zh-tw" | "zh-hk" => Language::ZhTw,
            "ja" | "jp" => Language::Ja,
            "en" | "en-us" => Language::En,
            custom => Language::Custom(custom.to_string()),
        }
    }

    /// Check if this is an East Asian language (requires special handling)
    pub fn is_east_asian(&self) -> bool {
        matches!(self, Language::Zh | Language::ZhTw | Language::Ja)
    }
}

impl Default for Language {
    fn default() -> Self {
        Language::En
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Configuration for internationalization
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct I18nConfig {
    /// Default language for the application
    #[serde(default)]
    pub default_language: Language,
    
    /// Supported languages
    #[serde(default = "default_supported_languages")]
    pub supported_languages: Vec<Language>,
    
    /// Path to translation files directory
    #[serde(default = "default_translations_path")]
    pub translations_path: PathBuf,
    
    /// Enable automatic language detection from request headers
    #[serde(default = "default_true")]
    pub auto_detect: bool,
    
    /// Enable language switching via URL parameter
    #[serde(default = "default_true")]
    pub allow_url_param: bool,
    
    /// URL parameter name for language switching
    #[serde(default = "default_lang_param")]
    pub lang_param: String,
    
    /// Enable language switching via cookie
    #[serde(default = "default_true")]
    pub allow_cookie: bool,
    
    /// Cookie name for language preference
    #[serde(default = "default_cookie_name")]
    pub cookie_name: String,
    
    /// Cookie expiration time in seconds
    #[serde(default = "default_cookie_max_age")]
    pub cookie_max_age: i64,
}

impl Default for I18nConfig {
    fn default() -> Self {
        Self {
            default_language: Language::En,
            supported_languages: default_supported_languages(),
            translations_path: default_translations_path(),
            auto_detect: true,
            allow_url_param: true,
            lang_param: default_lang_param(),
            allow_cookie: true,
            cookie_name: default_cookie_name(),
            cookie_max_age: default_cookie_max_age(),
        }
    }
}

fn default_supported_languages() -> Vec<Language> {
    vec![Language::Zh, Language::Ja, Language::En]
}

fn default_translations_path() -> PathBuf {
    PathBuf::from("assets").join("translations")
}

fn default_true() -> bool {
    true
}

fn default_lang_param() -> String {
    "lang".to_string()
}

fn default_cookie_name() -> String {
    "app_language".to_string()
}

fn default_cookie_max_age() -> i64 {
    365 * 24 * 60 * 60 // 1 year in seconds
}

/// Translation key-value pairs
pub type TranslationMap = HashMap<String, String>;

/// Context containing internationalization data for request handling
#[derive(Debug, Clone)]
pub struct I18nContext {
    /// Current language
    pub language: Language,
    /// Translation manager
    pub translations: Arc<Translations>,
    /// Configuration
    pub config: Arc<I18nConfig>,
}

impl I18nContext {
    /// Create a new i18n context
    pub fn new(language: Language, translations: Arc<Translations>, config: Arc<I18nConfig>) -> Self {
        Self {
            language,
            translations,
            config,
        }
    }

    /// Translate a key to the current language
    pub fn t(&self, key: &str) -> String {
        self.translations.translate(&self.language, key)
    }

    /// Translate a key with parameters
    pub fn t_with_params(&self, key: &str, params: &HashMap<String, String>) -> String {
        self.translations.translate_with_params(&self.language, key, params)
    }

    /// Check if a translation exists for the current language
    pub fn has_translation(&self, key: &str) -> bool {
        self.translations.has_translation(&self.language, key)
    }

    /// Get the fallback language for the current language
    pub fn fallback_language(&self) -> Option<&Language> {
        self.translations.fallback_language(&self.language)
    }
}

/// Result type for i18n operations
pub type I18nResult<T> = Result<T, I18nError>;

/// Internationalization errors
#[derive(Debug, thiserror::Error)]
pub enum I18nError {
    #[error("Translation not found for key: {key} in language: {language}")]
    TranslationNotFound { key: String, language: String },
    
    #[error("Language not supported: {language}")]
    LanguageNotSupported { language: String },
    
    #[error("Failed to load translation file: {path}")]
    LoadFailed { path: String },
    
    #[error("Invalid translation format: {message}")]
    InvalidFormat { message: String },
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_from_str() {
        assert_eq!(Language::from_str("zh"), Language::Zh);
        assert_eq!(Language::from_str("zh-cn"), Language::Zh);
        assert_eq!(Language::from_str("ja"), Language::Ja);
        assert_eq!(Language::from_str("en"), Language::En);
        assert_eq!(Language::from_str("fr"), Language::Custom("fr".to_string()));
    }

    #[test]
    fn test_language_display() {
        assert_eq!(Language::Zh.as_str(), "zh");
        assert_eq!(Language::Ja.as_str(), "ja");
        assert_eq!(Language::En.as_str(), "en");
    }

    #[test]
    fn test_language_native_name() {
        assert_eq!(Language::Zh.native_name(), "简体中文");
        assert_eq!(Language::Ja.native_name(), "日本語");
        assert_eq!(Language::En.native_name(), "English");
    }

    #[test]
    fn test_language_is_east_asian() {
        assert!(Language::Zh.is_east_asian());
        assert!(Language::ZhTw.is_east_asian());
        assert!(Language::Ja.is_east_asian());
        assert!(!Language::En.is_east_asian());
    }

    #[test]
    fn test_i18n_config_default() {
        let config = I18nConfig::default();
        assert_eq!(config.default_language, Language::En);
        assert!(config.auto_detect);
        assert_eq!(config.lang_param, "lang");
        assert_eq!(config.cookie_name, "app_language");
    }
}
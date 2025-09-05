//! Translation management system

use super::{I18nConfig, I18nError, Language, TranslationMap};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use std::sync::RwLock;

/// Translation storage and management
#[derive(Debug)]
pub struct Translations {
    /// Translations organized by language and key
    translations: RwLock<HashMap<Language, TranslationMap>>,
    /// Fallback languages for each language
    fallback_languages: HashMap<Language, Language>,
    /// Configuration
    config: Arc<I18nConfig>,
}

impl Translations {
    /// Create a new translations instance
    pub fn new(config: Arc<I18nConfig>) -> Self {
        let mut fallback_languages = HashMap::new();
        
        // Set up fallback languages
        fallback_languages.insert(Language::Zh, Language::En);
        fallback_languages.insert(Language::ZhTw, Language::Zh);
        fallback_languages.insert(Language::Ja, Language::En);
        
        Self {
            translations: RwLock::new(HashMap::new()),
            fallback_languages,
            config,
        }
    }

    /// Load translations from the configured directory
    pub fn load_translations(&self) -> super::I18nResult<()> {
        let translations_dir = &self.config.translations_path;
        
        if !translations_dir.exists() {
            return Err(I18nError::LoadFailed {
                path: translations_dir.to_string_lossy().to_string(),
            });
        }

        let mut translations = self.translations.write().unwrap();
        translations.clear();

        for language in &self.config.supported_languages {
            let translation_file = translations_dir.join(format!("{}.json", language.as_str()));
            
            if translation_file.exists() {
                self.load_language_translations(&mut translations, language, &translation_file)?;
            } else {
                // Try YAML format as fallback
                let yaml_file = translations_dir.join(format!("{}.yaml", language.as_str()));
                if yaml_file.exists() {
                    self.load_language_translations_yaml(&mut translations, language, &yaml_file)?;
                }
            }
        }

        Ok(())
    }

    /// Load translations for a specific language from JSON file
    fn load_language_translations(
        &self,
        translations: &mut HashMap<Language, TranslationMap>,
        language: &Language,
        file_path: &Path,
    ) -> super::I18nResult<()> {
        let content = fs::read_to_string(file_path)
            .map_err(|_e| I18nError::LoadFailed {
                path: file_path.to_string_lossy().to_string(),
            })?;

        let translation_map: TranslationMap = serde_json::from_str(&content)
            .map_err(|e| I18nError::InvalidFormat {
                message: format!("JSON parse error: {}", e),
            })?;

        translations.insert(language.clone(), translation_map);
        Ok(())
    }

    /// Load translations for a specific language from YAML file
    fn load_language_translations_yaml(
        &self,
        translations: &mut HashMap<Language, TranslationMap>,
        language: &Language,
        file_path: &Path,
    ) -> super::I18nResult<()> {
        let content = fs::read_to_string(file_path)
            .map_err(|_e| I18nError::LoadFailed {
                path: file_path.to_string_lossy().to_string(),
            })?;

        let translation_map: TranslationMap = serde_yaml::from_str(&content)
            .map_err(|e| I18nError::InvalidFormat {
                message: format!("YAML parse error: {}", e),
            })?;

        translations.insert(language.clone(), translation_map);
        Ok(())
    }

    /// Get translation for a key in the specified language
    pub fn translate(&self, language: &Language, key: &str) -> String {
        let translations = self.translations.read().unwrap();
        
        // Try direct translation
        if let Some(lang_translations) = translations.get(language) {
            if let Some(translation) = lang_translations.get(key) {
                return translation.clone();
            }
        }

        // Try fallback language
        if let Some(fallback_lang) = self.fallback_languages.get(language) {
            if let Some(fallback_translations) = translations.get(fallback_lang) {
                if let Some(translation) = fallback_translations.get(key) {
                    return translation.clone();
                }
            }
        }

        // Try default language
        if language != &self.config.default_language {
            if let Some(default_translations) = translations.get(&self.config.default_language) {
                if let Some(translation) = default_translations.get(key) {
                    return translation.clone();
                }
            }
        }

        // Return key as fallback
        key.to_string()
    }

    /// Get translation with parameters
    pub fn translate_with_params(
        &self,
        language: &Language,
        key: &str,
        params: &HashMap<String, String>,
    ) -> String {
        let template = self.translate(language, key);
        
        // Simple parameter substitution: {{param}} -> value
        let mut result = template;
        for (param, value) in params {
            let placeholder = format!("{{{{{}}}}}", param);
            result = result.replace(&placeholder, value);
        }
        
        result
    }

    /// Check if translation exists for a key in the specified language
    pub fn has_translation(&self, language: &Language, key: &str) -> bool {
        let translations = self.translations.read().unwrap();
        
        if let Some(lang_translations) = translations.get(language) {
            lang_translations.contains_key(key)
        } else {
            false
        }
    }

    /// Get fallback language for the specified language
    pub fn fallback_language(&self, language: &Language) -> Option<&Language> {
        self.fallback_languages.get(language)
    }

    /// Get all available translations for a language
    pub fn get_translations(&self, language: &Language) -> Option<TranslationMap> {
        let translations = self.translations.read().unwrap();
        translations.get(language).cloned()
    }

    /// Add or update a translation
    pub fn add_translation(&self, language: &Language, key: String, value: String) {
        let mut translations = self.translations.write().unwrap();
        translations
            .entry(language.clone())
            .or_insert_with(HashMap::new)
            .insert(key, value);
    }

    /// Remove a translation
    pub fn remove_translation(&self, language: &Language, key: &str) {
        let mut translations = self.translations.write().unwrap();
        if let Some(lang_translations) = translations.get_mut(language) {
            lang_translations.remove(key);
        }
    }

    /// Save translations to files
    pub fn save_translations(&self) -> super::I18nResult<()> {
        let translations = self.translations.read().unwrap();
        let translations_dir = &self.config.translations_path;

        for (language, translation_map) in translations.iter() {
            let file_path = translations_dir.join(format!("{}.json", language.as_str()));
            
            let content = serde_json::to_string_pretty(translation_map)
                .map_err(|e| I18nError::InvalidFormat {
                    message: format!("JSON serialization error: {}", e),
                })?;

            fs::write(&file_path, content)
                .map_err(|_e| I18nError::LoadFailed {
                    path: file_path.to_string_lossy().to_string(),
                })?;
        }

        Ok(())
    }

    /// Get all supported languages that have translations
    pub fn available_languages(&self) -> Vec<Language> {
        let translations = self.translations.read().unwrap();
        translations.keys().cloned().collect()
    }

    /// Get translation statistics
    pub fn get_stats(&self) -> HashMap<Language, usize> {
        let translations = self.translations.read().unwrap();
        translations
            .iter()
            .map(|(lang, map)| (lang.clone(), map.len()))
            .collect()
    }
}

/// Translation file structure for organized translations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationFile {
    /// Language code
    pub language: String,
    /// Translation categories
    pub categories: HashMap<String, TranslationMap>,
    /// Flat translations (for backward compatibility)
    #[serde(flatten)]
    pub translations: Option<TranslationMap>,
}

impl TranslationFile {
    /// Create a new translation file
    pub fn new(language: String) -> Self {
        Self {
            language,
            categories: HashMap::new(),
            translations: None,
        }
    }

    /// Add a translation to a category
    pub fn add_to_category(&mut self, category: &str, key: String, value: String) {
        self.categories
            .entry(category.to_string())
            .or_insert_with(HashMap::new)
            .insert(key, value);
    }

    /// Get translation from category
    pub fn get_from_category(&self, category: &str, key: &str) -> Option<&String> {
        self.categories.get(category).and_then(|cat| cat.get(key))
    }

    /// Convert to flat translation map
    pub fn to_flat_map(&self) -> TranslationMap {
        let mut flat_map = if let Some(translations) = &self.translations {
            translations.clone()
        } else {
            HashMap::new()
        };

        for (category, translations) in &self.categories {
            for (key, value) in translations {
                let flat_key = format!("{}.{}", category, key);
                flat_map.insert(flat_key, value.clone());
            }
        }

        flat_map
    }

    /// Load from flat translation map
    pub fn from_flat_map(language: String, flat_map: TranslationMap) -> Self {
        let mut categories = HashMap::new();
        let mut remaining = HashMap::new();

        for (key, value) in flat_map {
            if key.contains('.') {
                let parts: Vec<&str> = key.split('.').collect();
                if parts.len() >= 2 {
                    let category = parts[0];
                    let sub_key = parts[1..].join(".");
                    categories
                        .entry(category.to_string())
                        .or_insert_with(HashMap::new)
                        .insert(sub_key, value);
                    continue;
                }
            }
            remaining.insert(key, value);
        }

        Self {
            language,
            categories,
            translations: if remaining.is_empty() {
                None
            } else {
                Some(remaining)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_translation_file() {
        let mut file = TranslationFile::new("zh".to_string());
        file.add_to_category("nav", "home".to_string(), "首页".to_string());
        file.add_to_category("nav", "about".to_string(), "关于".to_string());
        file.add_to_category("form", "submit".to_string(), "提交".to_string());
        
        assert_eq!(file.get_from_category("nav", "home"), Some(&"首页".to_string()));
        
        let flat_map = file.to_flat_map();
        assert_eq!(flat_map.get("nav.home"), Some(&"首页".to_string()));
        assert_eq!(flat_map.get("form.submit"), Some(&"提交".to_string()));
    }

    #[test]
    fn test_translation_file_from_flat() {
        let mut flat_map = HashMap::new();
        flat_map.insert("nav.home".to_string(), "首页".to_string());
        flat_map.insert("nav.about".to_string(), "关于".to_string());
        flat_map.insert("submit".to_string(), "提交".to_string());
        
        let file = TranslationFile::from_flat_map("zh".to_string(), flat_map);
        assert_eq!(file.get_from_category("nav", "home"), Some(&"首页".to_string()));
        assert_eq!(file.categories.get("nav").unwrap().len(), 2);
    }
}
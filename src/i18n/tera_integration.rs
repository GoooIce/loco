//! Tera template integration for i18n

use super::{I18nContext, Language};
use serde_json::Value;
use std::collections::HashMap;
use tera::{Tera, Filter, Function, Result as TeraResult};

/// Register i18n filters and functions with Tera
pub fn register_i18n(tera: &mut Tera) {
    // Register translation filter
    tera.register_filter("t", TranslationFilter);
    
    // Register translation function
    tera.register_function("t", TranslationFunction);
    
    // Register language function
    tera.register_function("lang", LanguageFunction);
    
    // Register translation exists function
    tera.register_function("has_translation", HasTranslationFunction);
}

/// Translation filter for Tera templates
#[derive(Debug)]
pub struct TranslationFilter;

impl Filter for TranslationFilter {
    fn filter(&self, value: &Value, args: &HashMap<String, Value>) -> TeraResult<Value> {
        let key = value.as_str().ok_or_else(|| {
            tera::Error::msg("Translation filter requires a string key")
        })?;
        
        // Try to get i18n context from args
        let _i18n_context: Option<&serde_json::Value> = args.get("i18n_context");
        
        // If no context, return the key as fallback
        if _i18n_context.is_none() {
            return Ok(Value::String(format!("t({})", key)));
        }
        
        // For now, just return the key - in a real implementation,
        // you'd access the i18n context and perform the translation
        Ok(Value::String(key.to_string()))
    }
}

/// Translation function for Tera templates
#[derive(Debug)]
pub struct TranslationFunction;

impl Function for TranslationFunction {
    fn call(&self, args: &HashMap<String, Value>) -> TeraResult<Value> {
        let key = args.get("key").and_then(|v| v.as_str()).ok_or_else(|| {
            tera::Error::msg("t() function requires a 'key' argument")
        })?;
        
        let params = args.get("params")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect::<HashMap<String, String>>()
            })
            .unwrap_or_default();
        
        // Try to get i18n context
        let _i18n_context: Option<&serde_json::Value> = args.get("i18n_context");
        
        if _i18n_context.is_some() {
            // Perform translation with context
            // For now, return a placeholder
            let translated = if params.is_empty() {
                format!("Translated: {}", key)
            } else {
                format!("Translated: {} with params {:?}", key, params)
            };
            Ok(Value::String(translated))
        } else {
            // Fallback to key
            Ok(Value::String(key.to_string()))
        }
    }
}

/// Language function for Tera templates
#[derive(Debug)]
pub struct LanguageFunction;

impl Function for LanguageFunction {
    fn call(&self, args: &HashMap<String, Value>) -> TeraResult<Value> {
        let action = args.get("action").and_then(|v| v.as_str()).unwrap_or("current");
        
        // Try to get i18n context
        let _i18n_context: Option<&serde_json::Value> = args.get("i18n_context");
        
        match action {
            "current" => {
                if _i18n_context.is_some() {
                    // Return current language
                    Ok(Value::String("en".to_string())) // Placeholder
                } else {
                    Ok(Value::String("en".to_string()))
                }
            }
            "native_name" => {
                let lang_code = args.get("lang").and_then(|v| v.as_str()).unwrap_or("en");
                let lang = Language::from_str(lang_code);
                Ok(Value::String(lang.native_name().to_string()))
            }
            "display_name" => {
                let lang_code = args.get("lang").and_then(|v| v.as_str()).unwrap_or("en");
                let lang = Language::from_str(lang_code);
                Ok(Value::String(lang.display_name().to_string()))
            }
            "is_east_asian" => {
                let lang_code = args.get("lang").and_then(|v| v.as_str()).unwrap_or("en");
                let lang = Language::from_str(lang_code);
                Ok(Value::Bool(lang.is_east_asian()))
            }
            _ => Err(tera::Error::msg(format!("Unknown action: {}", action))),
        }
    }
}

/// Has translation function for Tera templates
#[derive(Debug)]
pub struct HasTranslationFunction;

impl Function for HasTranslationFunction {
    fn call(&self, args: &HashMap<String, Value>) -> TeraResult<Value> {
        let _key = args.get("key").and_then(|v| v.as_str()).ok_or_else(|| {
            tera::Error::msg("has_translation() function requires a 'key' argument")
        })?;
        
        // Try to get i18n context
        let _i18n_context: Option<&serde_json::Value> = args.get("i18n_context");
        
        if _i18n_context.is_some() {
            // Check if translation exists
            Ok(Value::Bool(true)) // Placeholder
        } else {
            Ok(Value::Bool(false))
        }
    }
}

/// Helper trait to extend Tera with i18n functionality
pub trait TeraI18nExt {
    /// Register i18n filters and functions
    fn register_i18n(&mut self);
    /// Add i18n context to template rendering
    fn render_with_i18n<S: serde::Serialize>(
        &self,
        template_name: &str,
        data: S,
        i18n_context: &I18nContext,
    ) -> TeraResult<String>;
}

impl TeraI18nExt for Tera {
    fn register_i18n(&mut self) {
        register_i18n(self);
    }
    
    fn render_with_i18n<S: serde::Serialize>(
        &self,
        template_name: &str,
        data: S,
        i18n_context: &I18nContext,
    ) -> TeraResult<String> {
        let mut context = tera::Context::from_serialize(data)?;
        
        // Add i18n context data
        context.insert("i18n", &I18nTemplateContext {
            current_language: i18n_context.language.clone(),
            current_language_code: i18n_context.language.as_str().to_string(),
            current_language_display: i18n_context.language.display_name().to_string(),
            current_language_native: i18n_context.language.native_name().to_string(),
            is_east_asian: i18n_context.language.is_east_asian(),
            supported_languages: i18n_context.config.supported_languages.clone(),
        });
        
        // Add translation function placeholder
        context.insert("t_function", &"translation_function_placeholder");
        
        self.render(template_name, &context)
    }
}

/// I18n context for templates
#[derive(Debug, Clone, serde::Serialize)]
pub struct I18nTemplateContext {
    pub current_language: Language,
    pub current_language_code: String,
    pub current_language_display: String,
    pub current_language_native: String,
    pub is_east_asian: bool,
    pub supported_languages: Vec<Language>,
}

impl I18nTemplateContext {
    pub fn new(language: Language, supported_languages: Vec<Language>) -> Self {
        Self {
            current_language: language.clone(),
            current_language_code: language.as_str().to_string(),
            current_language_display: language.display_name().to_string(),
            current_language_native: language.native_name().to_string(),
            is_east_asian: language.is_east_asian(),
            supported_languages,
        }
    }
}


/// Helper macros for template usage
#[macro_export]
macro_rules! t {
    ($key:expr) => {
        $key
    };
    ($key:expr, $($param:expr => $value:expr),*) => {
        format!($key, $($param = $value),*)
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use tera::Context;

    #[test]
    fn test_tera_i18n_ext() {
        let mut tera = Tera::default();
        tera.register_i18n();
        
        // Test that filters and functions are registered
        // This is a basic test - in practice, you'd test template rendering
        assert!(true); // If we reach here, registration didn't panic
    }

    #[test]
    fn test_i18n_template_context() {
        let context = I18nTemplateContext::new(
            Language::Zh,
            vec![Language::Zh, Language::En],
        );
        
        assert_eq!(context.current_language_code, "zh");
        assert_eq!(context.current_language_display, "中文(简体)");
        assert_eq!(context.current_language_native, "简体中文");
        assert!(context.is_east_asian);
    }

    #[test]
    fn test_i18n_template_function() {
        // This would require a real I18nContext to test properly
        // For now, we just test the struct creation
        let context = I18nContext::new(
            Language::En,
            std::sync::Arc::new(crate::i18n::Translations::new(
                std::sync::Arc::new(crate::i18n::I18nConfig::default()),
            )),
            std::sync::Arc::new(crate::i18n::I18nConfig::default()),
        );
        
        let template_func = I18nTemplateFunction { context };
        assert!(true); // If we reach here, creation didn't panic
    }
}
//! Language detection from HTTP requests

use super::{Language, I18nConfig};
use axum::extract::Request;
use axum::http::header::{ACCEPT_LANGUAGE, COOKIE};
use axum::http::HeaderMap;
use std::str::FromStr;

/// Language detector for HTTP requests
#[derive(Debug, Clone)]
pub struct LanguageDetector {
    config: std::sync::Arc<I18nConfig>,
}

impl LanguageDetector {
    /// Create a new language detector
    pub fn new(config: std::sync::Arc<I18nConfig>) -> Self {
        Self { config }
    }

    /// Detect language from request
    pub fn detect_from_request(&self, request: &Request) -> Language {
        // Try URL parameter first
        if self.config.allow_url_param {
            if let Some(lang) = self.detect_from_url(request) {
                return lang;
            }
        }

        // Try cookie
        if self.config.allow_cookie {
            if let Some(lang) = self.detect_from_cookie(request) {
                return lang;
            }
        }

        // Try Accept-Language header
        if self.config.auto_detect {
            if let Some(lang) = self.detect_from_headers(request) {
                return lang;
            }
        }

        // Return default language
        self.config.default_language.clone()
    }

    /// Detect language from URL parameter
    fn detect_from_url(&self, request: &Request) -> Option<Language> {
        let uri = request.uri();
        let query = uri.query()?;
        
        for param in query.split('&') {
            let mut parts = param.splitn(2, '=');
            let key = parts.next()?;
            let value = parts.next()?;
            
            if key == self.config.lang_param {
                let lang = Language::from_str(value);
                if self.is_language_supported(&lang) {
                    return Some(lang);
                }
            }
        }
        
        None
    }

    /// Detect language from cookie
    fn detect_from_cookie(&self, request: &Request) -> Option<Language> {
        let headers = request.headers();
        let cookie_header = headers.get(COOKIE)?;
        let cookie_str = cookie_header.to_str().ok()?;
        
        for cookie in cookie_str.split(';') {
            let cookie = cookie.trim();
            let mut parts = cookie.splitn(2, '=');
            let name = parts.next()?.trim();
            let value = parts.next()?.trim();
            
            if name == self.config.cookie_name {
                let lang = Language::from_str(value);
                if self.is_language_supported(&lang) {
                    return Some(lang);
                }
            }
        }
        
        None
    }

    /// Detect language from Accept-Language header
    fn detect_from_headers(&self, request: &Request) -> Option<Language> {
        let headers = request.headers();
        let accept_lang = headers.get(ACCEPT_LANGUAGE)?;
        let accept_lang_str = accept_lang.to_str().ok()?;
        
        self.parse_accept_language(accept_lang_str)
    }

    /// Parse Accept-Language header and find best match
    fn parse_accept_language(&self, accept_lang: &str) -> Option<Language> {
        let mut languages = Vec::new();
        
        for part in accept_lang.split(',') {
            let part = part.trim();
            let mut lang_part = part.split(';');
            let lang_code = lang_part.next()?;
            let q_value = lang_part
                .next()
                .and_then(|s| s.strip_prefix("q="))
                .and_then(|s| f32::from_str(s).ok())
                .unwrap_or(1.0);
            
            languages.push((lang_code.trim(), q_value));
        }
        
        // Sort by q-value (descending)
        languages.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Find first supported language
        for (lang_code, _) in languages {
            let lang = Language::from_str(lang_code);
            if self.is_language_supported(&lang) {
                return Some(lang);
            }
        }
        
        None
    }

    /// Check if language is supported
    fn is_language_supported(&self, language: &Language) -> bool {
        self.config.supported_languages.contains(language)
    }

    /// Detect language from headers only
    pub fn detect_from_headers_only(&self, headers: &HeaderMap) -> Option<Language> {
        if !self.config.auto_detect {
            return None;
        }

        let accept_lang = headers.get(ACCEPT_LANGUAGE)?;
        let accept_lang_str = accept_lang.to_str().ok()?;
        
        self.parse_accept_language(accept_lang_str)
    }

    /// Detect language from cookie only
    pub fn detect_from_cookie_only(&self, headers: &HeaderMap) -> Option<Language> {
        if !self.config.allow_cookie {
            return None;
        }

        let cookie_header = headers.get(COOKIE)?;
        let cookie_str = cookie_header.to_str().ok()?;
        
        for cookie in cookie_str.split(';') {
            let cookie = cookie.trim();
            let mut parts = cookie.splitn(2, '=');
            let name = parts.next()?.trim();
            let value = parts.next()?.trim();
            
            if name == self.config.cookie_name {
                let lang = Language::from_str(value);
                if self.is_language_supported(&lang) {
                    return Some(lang);
                }
            }
        }
        
        None
    }

    /// Create language preference map from Accept-Language header
    pub fn parse_language_preferences(&self, accept_lang: &str) -> Vec<(Language, f32)> {
        let mut preferences = Vec::new();
        
        for part in accept_lang.split(',') {
            let part = part.trim();
            let mut lang_part = part.split(';');
            let lang_code = lang_part.next().unwrap_or("");
            let q_value = lang_part
                .next()
                .and_then(|s| s.strip_prefix("q="))
                .and_then(|s| f32::from_str(s).ok())
                .unwrap_or(1.0);
            
            let lang = Language::from_str(lang_code);
            if self.is_language_supported(&lang) {
                preferences.push((lang, q_value));
            }
        }
        
        // Sort by q-value (descending)
        preferences.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        preferences
    }

    /// Get all supported languages with their display names
    pub fn get_supported_languages(&self) -> Vec<(Language, String, String)> {
        self.config
            .supported_languages
            .iter()
            .map(|lang| {
                (
                    lang.clone(),
                    lang.display_name().to_string(),
                    lang.native_name().to_string(),
                )
            })
            .collect()
    }

    /// Generate URL for language switching
    pub fn generate_lang_switch_url(&self, current_url: &str, target_lang: &Language) -> String {
        let mut url = current_url.to_string();
        
        // Remove existing lang parameter
        if let Some(query_start) = url.find('?') {
            let query = &url[query_start + 1..];
            let new_query: Vec<&str> = query
                .split('&')
                .filter(|param| !param.starts_with(&format!("{}=", self.config.lang_param)))
                .collect();
            
            url = format!(
                "{}?{}&{}={}",
                &url[..query_start],
                new_query.join("&"),
                self.config.lang_param,
                target_lang.as_str()
            );
        } else {
            url = format!("{}?{}={}", url, self.config.lang_param, target_lang.as_str());
        }
        
        url
    }

    /// Generate cookie header for language preference
    pub fn generate_lang_cookie(&self, language: &Language) -> String {
        format!(
            "{}={}; Max-Age={}; Path=/; HttpOnly; SameSite=Lax",
            self.config.cookie_name,
            language.as_str(),
            self.config.cookie_max_age
        )
    }
}

/// Language quality value parser
#[derive(Debug, Clone)]
pub struct LanguageQuality {
    pub language: Language,
    pub quality: f32,
    pub subtags: Vec<String>,
}

impl LanguageQuality {
    /// Parse language tag with quality value
    pub fn parse(tag: &str) -> Option<Self> {
        let mut parts = tag.split(';');
        let lang_part = parts.next()?;
        let quality = parts
            .next()
            .and_then(|s| s.strip_prefix("q="))
            .and_then(|s| f32::from_str(s).ok())
            .unwrap_or(1.0);

        let mut subtags: Vec<String> = lang_part.split('-').map(|s| s.to_lowercase()).collect();
        let primary = subtags.remove(0);
        
        let language = match primary.as_str() {
            "zh" => {
                if subtags.contains(&"tw".to_string()) || subtags.contains(&"hk".to_string()) {
                    Language::ZhTw
                } else {
                    Language::Zh
                }
            }
            "ja" => Language::Ja,
            "en" => Language::En,
            _ => Language::Custom(primary),
        };

        Some(LanguageQuality {
            language,
            quality,
            subtags,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn test_language_from_str() {
        assert_eq!(Language::from_str("zh"), Language::Zh);
        assert_eq!(Language::from_str("zh-CN"), Language::Zh);
        assert_eq!(Language::from_str("zh-TW"), Language::ZhTw);
        assert_eq!(Language::from_str("ja"), Language::Ja);
        assert_eq!(Language::from_str("en-US"), Language::En);
        assert_eq!(Language::from_str("fr"), Language::Custom("fr".to_string()));
    }

    #[test]
    fn test_language_detector_basic() {
        let config = I18nConfig::default();
        let detector = LanguageDetector::new(std::sync::Arc::new(config));
        
        let supported = detector.get_supported_languages();
        assert!(!supported.is_empty());
    }

    #[test]
    fn test_parse_accept_language() {
        let config = I18nConfig::default();
        let detector = LanguageDetector::new(std::sync::Arc::new(config));
        
        let accept_lang = "zh-CN,zh;q=0.9,en;q=0.8";
        let lang = detector.parse_accept_language(accept_lang);
        assert_eq!(lang, Some(Language::Zh));
    }

    #[test]
    fn test_language_quality_parse() {
        let quality = LanguageQuality::parse("zh-CN;q=0.9").unwrap();
        assert_eq!(quality.language, Language::Zh);
        assert_eq!(quality.quality, 0.9);
        assert_eq!(quality.subtags, vec!["cn".to_string()]);
    }

    #[test]
    fn test_generate_lang_switch_url() {
        let config = I18nConfig::default();
        let detector = LanguageDetector::new(std::sync::Arc::new(config));
        
        let url = "https://example.com/page";
        let new_url = detector.generate_lang_switch_url(url, &Language::Ja);
        assert!(new_url.contains("lang=ja"));
        
        let url_with_params = "https://example.com/page?foo=bar";
        let new_url = detector.generate_lang_switch_url(url_with_params, &Language::Ja);
        assert!(new_url.contains("foo=bar"));
        assert!(new_url.contains("lang=ja"));
    }

    #[test]
    fn test_generate_lang_cookie() {
        let config = I18nConfig::default();
        let detector = LanguageDetector::new(std::sync::Arc::new(config));
        
        let cookie = detector.generate_lang_cookie(&Language::Zh);
        assert!(cookie.contains("app_language=zh"));
        assert!(cookie.contains("Max-Age="));
    }
}
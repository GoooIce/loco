//! I18n middleware for Axum

use super::{I18nConfig, I18nContext, Language, LanguageDetector, Translations};
use axum::{
    extract::Request,
    http::header::SET_COOKIE,
    response::Response,
    middleware::Next,
};
use std::sync::Arc;
use async_trait::async_trait;

/// I18n middleware for automatic language detection and context injection
#[derive(Debug, Clone)]
pub struct I18nMiddleware {
    translations: Arc<Translations>,
    detector: LanguageDetector,
    config: Arc<I18nConfig>,
}

impl I18nMiddleware {
    /// Create a new I18n middleware
    pub fn new(translations: Arc<Translations>, config: Arc<I18nConfig>) -> Self {
        let detector = LanguageDetector::new(config.clone());
        Self {
            translations,
            detector,
            config,
        }
    }

    /// Create middleware with default configuration
    pub fn with_default_config(translations: Arc<Translations>) -> Self {
        let config = Arc::new(I18nConfig::default());
        Self::new(translations, config)
    }
}

// I18n middleware implementation as a tower service
impl tower::Service<Request> for I18nMiddleware {
    type Response = Response;
    type Error = axum::Error;
    type Future = std::future::Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, mut request: Request) -> Self::Future {
        // Detect language from request
        let language = self.detector.detect_from_request(&request);

        // Create i18n context
        let i18n_context = I18nContext::new(
            language.clone(),
            self.translations.clone(),
            self.config.clone(),
        );

        // Inject i18n context into request extensions
        request.extensions_mut().insert(i18n_context.clone());

        // For now, return a simple response
        // In a real implementation, this would be part of a middleware chain
        let response = Response::new(axum::body::Body::empty());
        
        std::future::ready(Ok(response))
    }
}

/// Builder for I18n middleware
pub struct I18nMiddlewareBuilder {
    config: I18nConfig,
}

impl I18nMiddlewareBuilder {
    /// Create a new builder with default configuration
    pub fn new() -> Self {
        Self {
            config: I18nConfig::default(),
        }
    }

    /// Set the default language
    pub fn default_language(mut self, language: Language) -> Self {
        self.config.default_language = language;
        self
    }

    /// Set supported languages
    pub fn supported_languages(mut self, languages: Vec<Language>) -> Self {
        self.config.supported_languages = languages;
        self
    }

    /// Set translations path
    pub fn translations_path<P: AsRef<std::path::Path>>(mut self, path: P) -> Self {
        self.config.translations_path = path.as_ref().to_path_buf();
        self
    }

    /// Enable or disable automatic language detection
    pub fn auto_detect(mut self, enabled: bool) -> Self {
        self.config.auto_detect = enabled;
        self
    }

    /// Enable or disable URL parameter for language switching
    pub fn allow_url_param(mut self, enabled: bool) -> Self {
        self.config.allow_url_param = enabled;
        self
    }

    /// Set URL parameter name for language switching
    pub fn lang_param(mut self, param: String) -> Self {
        self.config.lang_param = param;
        self
    }

    /// Enable or disable cookie for language preference
    pub fn allow_cookie(mut self, enabled: bool) -> Self {
        self.config.allow_cookie = enabled;
        self
    }

    /// Set cookie name for language preference
    pub fn cookie_name(mut self, name: String) -> Self {
        self.config.cookie_name = name;
        self
    }

    /// Set cookie max age in seconds
    pub fn cookie_max_age(mut self, max_age: i64) -> Self {
        self.config.cookie_max_age = max_age;
        self
    }

    /// Build the middleware
    pub fn build(self) -> I18nMiddleware {
        let config = Arc::new(self.config);
        let translations = Arc::new(Translations::new(config.clone()));
        let detector = LanguageDetector::new(config.clone());
        
        I18nMiddleware {
            translations,
            detector,
            config,
        }
    }
}

impl Default for I18nMiddlewareBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension trait for Axum Router to easily add i18n support
pub trait I18nRouterExt {
    /// Add i18n middleware to the router with default configuration
    fn with_i18n(self) -> Self;
    
    /// Add i18n middleware with custom configuration
    fn with_i18n_config(self, config: I18nConfig) -> Self;
}

#[cfg(feature = "axum")]
impl I18nRouterExt for axum::Router {
    fn with_i18n(self) -> Self {
        let config = I18nConfig::default();
        self.with_i18n_config(config)
    }
    
    fn with_i18n_config(self, config: I18nConfig) -> Self {
        let config = Arc::new(config);
        let translations = Arc::new(Translations::new(config.clone()));
        let middleware = I18nMiddleware::new(translations, config);
        self.layer(axum::middleware::from_fn(middleware))
    }
}

/// Extract i18n context from request
pub struct I18nContextExtractor(pub I18nContext);

#[axum::async_trait]
impl<S> axum::extract::FromRequestParts<S> for I18nContextExtractor
where
    S: Send + Sync,
{
    type Rejection = axum::http::StatusCode;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<I18nContext>()
            .cloned()
            .map(I18nContextExtractor)
            .ok_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl std::ops::Deref for I18nContextExtractor {
    type Target = I18nContext;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for I18nContextExtractor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Helper function to get current language from request
pub fn get_current_language(request: &axum::extract::Request) -> Option<Language> {
    request.extensions().get::<I18nContext>().map(|ctx| ctx.language.clone())
}

/// Helper function to get translation function from request
pub fn get_translator(request: &axum::extract::Request) -> Option<impl Fn(&str) -> String + Clone> {
    request.extensions().get::<I18nContext>().map(|ctx| {
        let ctx = ctx.clone();
        move |key: &str| ctx.t(key)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{extract::Request, middleware::Next, body::Body, http::StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_middleware_builder() {
        let middleware = I18nMiddlewareBuilder::new()
            .default_language(Language::Zh)
            .supported_languages(vec![Language::Zh, Language::Ja])
            .lang_param("locale".to_string())
            .build();

        assert_eq!(middleware.config.default_language, Language::Zh);
        assert_eq!(middleware.config.lang_param, "locale");
    }

    #[tokio::test]
    async fn test_middleware_request_processing() {
        let config = I18nConfig::default();
        let config = Arc::new(config);
        let translations = Arc::new(Translations::new(config.clone()));
        let middleware = I18nMiddleware::new(translations, config);

        let request = Request::builder()
            .uri("/test")
            .header("Accept-Language", "zh-CN")
            .body(Body::empty())
            .unwrap();

        let next = Next::new(|_| async {
            Ok(Response::builder().body(Body::empty()).unwrap())
        });

        let response = middleware.run(request, next).await.unwrap();
        
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers().get("Content-Language").unwrap(), "zh");
    }

    #[test]
    fn test_i18n_context_extractor() {
        let config = I18nConfig::default();
        let config = Arc::new(config);
        let translations = Arc::new(Translations::new(config.clone()));
        let context = I18nContext::new(Language::Zh, translations, config);

        let mut request = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();
        
        request.extensions_mut().insert(context);

        // In a real test, this would be used in an Axum handler
        // For now, we just verify the context is properly set
        assert!(request.extensions().get::<I18nContext>().is_some());
    }
}
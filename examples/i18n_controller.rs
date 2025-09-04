//! Example controller demonstrating i18n usage

use loco_rs::controller::{View, Views};
use loco_rs::prelude::*;
use serde_json::json;

/// Home page with i18n support
pub async fn home(ctx: &AppContext) -> Result<View> {
    let i18n_context = &ctx.i18n_context;
    
    let data = json!({
        "title": i18n_context.t("nav.home"),
        "welcome_message": i18n_context.t("common.welcome"),
        "current_lang": i18n_context.language.as_str(),
        "lang_display": i18n_context.language.display_name(),
        "lang_native": i18n_context.language.native_name(),
        "is_east_asian": i18n_context.language.is_east_asian(),
        "supported_languages": i18n_context.config.supported_languages.iter()
            .map(|lang| json!({
                "code": lang.as_str(),
                "display": lang.display_name(),
                "native": lang.native_name()
            }))
            .collect::<Vec<_>>()
    });

    Views::render("home.html", data)
}

/// About page with i18n support
pub async fn about(ctx: &AppContext) -> Result<View> {
    let i18n_context = &ctx.i18n_context;
    
    let data = json!({
        "title": i18n_context.t("nav.about"),
        "content": i18n_context.t("common.about"),
        "contact_info": i18n_context.t("nav.contact")
    });

    Views::render("about.html", data)
}

/// Language switch endpoint
pub async fn switch_language(
    ctx: &AppContext,
    req: &Request,
) -> Result<Redirect> {
    let query = req.uri().query().unwrap_or("");
    let lang_code = query
        .split('&')
        .find(|p| p.starts_with("lang="))
        .and_then(|p| p.split('=').nth(1))
        .unwrap_or("en");

    let language = loco_rs::i18n::Language::from_str(lang_code);
    
    // In a real implementation, you would set the language in a cookie
    // and redirect back to the previous page
    
    Ok(Redirect::to("/"))
}

/// API endpoint for getting translations
pub async fn api_translations(ctx: &AppContext) -> Result<Json<serde_json::Value>> {
    let i18n_context = &ctx.i18n_context;
    
    let translations = json!({
        "current_language": i18n_context.language.as_str(),
        "translations": i18n_context.translations.get_translations(&i18n_context.language)
    });

    Ok(Json(translations))
}

/// API endpoint for language info
pub async fn api_language_info(ctx: &AppContext) -> Result<Json<serde_json::Value>> {
    let i18n_context = &ctx.i18n_context;
    
    let info = json!({
        "current": {
            "code": i18n_context.language.as_str(),
            "display": i18n_context.language.display_name(),
            "native": i18n_context.language.native_name(),
            "is_east_asian": i18n_context.language.is_east_asian()
        },
        "supported": i18n_context.config.supported_languages.iter()
            .map(|lang| json!({
                "code": lang.as_str(),
                "display": lang.display_name(),
                "native": lang.native_name()
            }))
            .collect::<Vec<_>>()
    });

    Ok(Json(info))
}

/// Form submission with i18n validation messages
pub async fn submit_form(
    ctx: &AppContext,
    params: Params,
) -> Result<Json<serde_json::Value>> {
    let i18n_context = &ctx.i18n_context;
    
    // Simulate form validation
    let mut errors = Vec::new();
    
    if params.get("email").map_or(true, |e| !e.contains('@')) {
        errors.push(i18n_context.t("validation.invalid_email"));
    }
    
    if params.get("password").map_or(true, |p| p.len() < 8) {
        errors.push(i18n_context.t("validation.password_too_short"));
    }
    
    let response = json!({
        "success": errors.is_empty(),
        "errors": errors,
        "message": if errors.is_empty() {
            i18n_context.t("messages.save_success")
        } else {
            i18n_context.t("messages.save_failed")
        }
    });

    Ok(Json(response))
}

/// Example of using translation with parameters
pub async fn profile_page(ctx: &AppContext) -> Result<View> {
    let i18n_context = &ctx.i18n_context;
    
    // Example user data
    let username = "张三";
    let message_count = 5;
    
    let mut params = std::collections::HashMap::new();
    params.insert("username".to_string(), username.to_string());
    params.insert("count".to_string(), message_count.to_string());
    
    let data = json!({
        "title": i18n_context.t("nav.profile"),
        "greeting": i18n_context.t_with_params("messages.welcome_user", &params),
        "message_count": i18n_context.t_with_params("messages.message_count", &params),
        "last_login": i18n_context.t("date.today")
    });

    Views::render("profile.html", data)
}

/// Language detection demo
pub async fn lang_demo(ctx: &AppContext, req: &Request) -> Result<View> {
    let i18n_context = &ctx.i18n_context;
    
    // Detect language from various sources
    let detected_from = vec![
        ("URL Parameter", "lang"),
        ("Cookie", "app_language"),
        ("Accept-Language", "browser preference"),
        ("Default", "application setting")
    ];
    
    let data = json!({
        "title": i18n_context.t("common.welcome"),
        "current_language": i18n_context.language.as_str(),
        "detection_sources": detected_from,
        "browser_info": {
            "user_agent": req.headers().get("user-agent")
                .and_then(|h| h.to_str().ok())
                .unwrap_or("Unknown"),
            "accept_language": req.headers().get("accept-language")
                .and_then(|h| h.to_str().ok())
                .unwrap_or("Not specified")
        }
    });

    Views::render("lang_demo.html", data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use loco_rs::testing::prelude::*;

    #[tokio::test]
    async fn test_home_page() {
        let ctx = AppContext::new().await;
        let result = home(&ctx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_api_language_info() {
        let ctx = AppContext::new().await;
        let result = api_language_info(&ctx).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert!(response.as_object().unwrap().contains_key("current"));
        assert!(response.as_object().unwrap().contains_key("supported"));
    }
}
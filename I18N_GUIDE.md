# Loco Internationalization (i18n) Guide

This guide explains how to enable and use internationalization features in Loco applications.

## Features

- 🌍 Multi-language support (Chinese, Japanese, English prioritized)
- 🔍 Smart language detection (HTTP headers, cookies, URL parameters)
- 📝 Tera template integration
- 💾 Translation file management (JSON/YAML)
- 🚀 High-performance caching
- 🔧 Configurable middleware

## Quick Start

### 1. Enable i18n Feature

Add i18n feature to `Cargo.toml`:

```toml
[dependencies]
loco-rs = { version = "0.16", features = ["i18n"] }
```

### 2. Configure i18n

Add i18n configuration to `config/development.yaml`:

```yaml
i18n:
  default_language: zh
  supported_languages:
    - zh
    - ja
    - en
  translations_path: assets/translations
  auto_detect: true
  allow_url_param: true
  lang_param: lang
  allow_cookie: true
  cookie_name: app_language
  cookie_max_age: 31536000
```

### 3. Add Translation Files

Create translation files in `assets/translations/` directory:

- `zh.json` - Chinese translations
- `ja.json` - Japanese translations  
- `en.json` - English translations

### 4. Initialize i18n Manager

```rust
use loco_rs::i18n::{I18nManager, I18nConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = I18nConfig::default();
    let manager = I18nManager::new(config);
    manager.init().await?;
    
    // Use translations
    let translation = manager.translate(&Language::Zh, "welcome");
    println!("{}", translation); // Output: 欢迎
    
    Ok(())
}
```

## Usage in Controllers

### Basic Usage

```rust
use loco_rs::i18n::{I18nContext, Language};
use loco_rs::controller::{View, Views};

async fn home_page(ctx: &AppContext) -> Result<View> {
    let i18n_context = ctx.i18n_context.clone();
    
    let data = serde_json::json!({
        "title": i18n_context.t("welcome"),
        "content": i18n_context.t_with_params("greeting", &std::collections::HashMap::from([
            ("name".to_string(), "User".to_string())
        ]))
    });
    
    Views::render("home.html", data)
}
```

### Get Current Language

```rust
async fn get_current_language(ctx: &AppContext) -> Result<String> {
    let current_lang = &ctx.i18n_context.language;
    Ok(format!("Current language: {}", current_lang.native_name()))
}
```

## Usage in Templates

### Basic Translation

```html
<h1>{{ i18n.current_language_display }}</h1>
<p>{{ t(key="welcome") }}</p>
<p>{{ t(key="greeting", params={name="World"}) }}</p>
```

### Conditional Translation

```html
{% if i18n.is_east_asian %}
    <p>East Asian language special handling</p>
{% endif %}
```

### Language Switching

```html
<div class="language-switcher">
    <a href="?lang=zh">中文</a>
    <a href="?lang=ja">日本語</a>
    <a href="?lang=en">English</a>
</div>
```

### Display All Supported Languages

```html
<ul>
{% for lang in i18n.supported_languages %}
    <li>
        <a href="?lang={{ lang.current_language_code }}">
            {{ lang.current_language_native }}
        </a>
    </li>
{% endfor %}
</ul>
```

## Middleware Integration

### Add i18n Middleware

```rust
use loco_rs::i18n::{I18nMiddleware, I18nConfig};
use loco_rs::i18n::middleware::I18nRouterExt;

let app = axum::Router::new()
    .route("/", get(home))
    .with_i18n_config(I18nConfig::default());
```

### Custom Middleware Configuration

```rust
let config = I18nConfig {
    default_language: Language::Zh,
    supported_languages: vec![Language::Zh, Language::Ja, Language::En],
    translations_path: "assets/translations".into(),
    auto_detect: true,
    allow_url_param: true,
    lang_param: "locale".to_string(),
    allow_cookie: true,
    cookie_name: "user_lang".to_string(),
    cookie_max_age: 86400, // 1 day
};

let app = axum::Router::new()
    .route("/", get(home))
    .with_i18n_config(config);
```

## Translation Management

### Add Translation

```rust
use std::collections::HashMap;

async fn add_translation(manager: &I18nManager) -> Result<(), loco_rs::i18n::I18nError> {
    manager.add_translation(
        &Language::Zh,
        "new_feature".to_string(),
        "新功能".to_string(),
    ).await
}
```

### Batch Import Translations

```rust
use loco_rs::i18n::TranslationFormat;
use std::path::Path;

async fn import_translations(
    manager: &I18nManager,
    file_path: &Path,
) -> Result<(), loco_rs::i18n::I18nError> {
    manager.import_translations(
        &Language::Zh,
        file_path,
        TranslationFormat::Json,
    ).await
}
```

### Export Translations

```rust
async fn export_translations(
    manager: &I18nManager,
    file_path: &Path,
) -> Result<(), loco_rs::i18n::I18nError> {
    manager.export_translations(
        &Language::Zh,
        file_path,
        TranslationFormat::Csv,
    ).await
}
```

### Validate Translation Completeness

```rust
async fn validate_translations(manager: &I18nManager) {
    let validation = manager.validate_translations().await;
    
    println!("Total keys: {}", validation.total_keys);
    println!("Completion: {:.1}%", validation.overall_completion());
    
    for (lang, missing) in &validation.missing_translations {
        println!("{} missing {} translations", lang.as_str(), missing.len());
    }
}
```

## Language Detection Priority

Language detection follows this priority order:

1. **URL Parameter** (default: `?lang=zh`)
2. **Cookie** (default: `app_language`)
3. **Accept-Language HTTP Header**
4. **Default Language**

## Translation File Formats

### JSON Format

```json
{
  "nav": {
    "home": "Home",
    "about": "About"
  },
  "messages": {
    "welcome": "Welcome",
    "goodbye": "Goodbye"
  }
}
```

### YAML Format

```yaml
nav:
  home: Home
  about: About
messages:
  welcome: Welcome
  goodbye: Goodbye
```

### Nested Access

In templates, you can use dot notation to access nested translation keys:

```html
{{ t(key="nav.home") }}  <!-- Output: Home -->
{{ t(key="messages.welcome") }}  <!-- Output: Welcome -->
```

## Advanced Features

### Translation Caching

```rust
// Clear cache
manager.clear_cache().await;

// Get cache statistics
let stats = manager.cache_stats().await;
println!("Cache entries: {}", stats.get("total_entries").unwrap_or(&0));
```

### Custom Languages

```rust
let custom_lang = Language::Custom("ko".to_string()); // Korean
let translation = manager.translate(&custom_lang, "hello");
```

### Translation Parameter Replacement

```rust
let mut params = HashMap::new();
params.insert("name".to_string(), "John".to_string());
params.insert("count".to_string(), "5".to_string());

let translated = manager.translate_with_params(
    &Language::En,
    "welcome_message",
    &params
);
// If translation is "Welcome {{name}}, you have {{count}} messages"
// Result will be: "Welcome John, you have 5 messages"
```

## Best Practices

### 1. Translation Key Naming

- Use dot-separated hierarchical structure: `nav.home`, `form.submit`
- Use descriptive key names: `user.profile.settings` instead of `user.prof.set`
- Maintain consistency: use lowercase and underscores throughout

### 2. Translation File Organization

```
assets/translations/
├── zh.json      # Chinese
├── ja.json      # Japanese
├── en.json      # English
└── ko.json      # Korean
```

### 3. Performance Optimization

- Enable translation caching
- Disable hot-reloading in production
- Use CDN for distributing translation files

### 4. Testing Strategy

```rust
#[tokio::test]
async fn test_translation_loading() {
    let manager = I18nManager::default();
    manager.init().await.unwrap();
    
    assert_eq!(manager.translate(&Language::Zh, "welcome"), "欢迎");
}
```

## Troubleshooting

### Common Issues

1. **Translations Not Showing**
   - Check translation file paths
   - Verify file format is valid
   - Confirm language codes are correct

2. **Language Detection Not Working**
   - Check middleware configuration
   - Verify cookie and URL parameter settings
   - Confirm Accept-Language header format

3. **Performance Issues**
   - Enable caching
   - Check translation file sizes
   - Optimize translation key structure

### Debug Mode

```rust
// Enable verbose logging
env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

// Check translation statistics
let stats = manager.get_stats();
println!("Translation stats: {:?}", stats);
```

## Summary

Loco's internationalization system provides comprehensive multi-language support including:

- Smart language detection
- Flexible translation management
- Template integration
- High-performance caching
- Easy-to-use API

By properly configuring and using these features, you can easily build multilingual web applications.
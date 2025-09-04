# Loco 国际化 (i18n) 使用指南

本指南介绍如何在 Loco 应用中启用和使用国际化功能。

## 功能特性

- 🌍 多语言支持（优先中文简体、日语、英语）
- 🔍 智能语言检测（HTTP 头、Cookie、URL 参数）
- 📝 Tera 模板集成
- 💾 翻译文件管理（JSON/YAML）
- 🚀 高性能缓存机制
- 🔧 可配置的中间件

## 快速开始

### 1. 启用 i18n 功能

在 `Cargo.toml` 中添加 i18n feature：

```toml
[dependencies]
loco-rs = { version = "0.16", features = ["i18n"] }
```

### 2. 配置 i18n

在 `config/development.yaml` 中添加 i18n 配置：

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

### 3. 添加翻译文件

在 `assets/translations/` 目录下创建翻译文件：

- `zh.json` - 中文翻译
- `ja.json` - 日语翻译  
- `en.json` - 英语翻译

### 4. 初始化 i18n 管理器

```rust
use loco_rs::i18n::{I18nManager, I18nConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = I18nConfig::default();
    let manager = I18nManager::new(config);
    manager.init().await?;
    
    // 使用翻译
    let translation = manager.translate(&Language::Zh, "welcome");
    println!("{}", translation); // 输出: 欢迎
    
    Ok(())
}
```

## 在控制器中使用

### 基本用法

```rust
use loco_rs::i18n::{I18nContext, Language};
use loco_rs::controller::{View, Views};

async fn home_page(ctx: &AppContext) -> Result<View> {
    let i18n_context = ctx.i18n_context.clone();
    
    let data = serde_json::json!({
        "title": i18n_context.t("welcome"),
        "content": i18n_context.t_with_params("greeting", &std::collections::HashMap::from([
            ("name".to_string(), "用户".to_string())
        ]))
    });
    
    Views::render("home.html", data)
}
```

### 获取当前语言

```rust
async fn get_current_language(ctx: &AppContext) -> Result<String> {
    let current_lang = &ctx.i18n_context.language;
    Ok(format!("当前语言: {}", current_lang.native_name()))
}
```

## 在模板中使用

### 基本翻译

```html
<h1>{{ i18n.current_language_display }}</h1>
<p>{{ t(key="welcome") }}</p>
<p>{{ t(key="greeting", params={name="World"}) }}</p>
```

### 条件翻译

```html
{% if i18n.is_east_asian %}
    <p>东亚语言特殊处理</p>
{% endif %}
```

### 语言切换

```html
<div class="language-switcher">
    <a href="?lang=zh">中文</a>
    <a href="?lang=ja">日本語</a>
    <a href="?lang=en">English</a>
</div>
```

### 显示所有支持的语言

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

## 中间件集成

### 添加 i18n 中间件

```rust
use loco_rs::i18n::{I18nMiddleware, I18nConfig};
use loco_rs::i18n::middleware::I18nRouterExt;

let app = axum::Router::new()
    .route("/", get(home))
    .with_i18n_config(I18nConfig::default());
```

### 自定义中间件配置

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

## 翻译管理

### 添加翻译

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

### 批量导入翻译

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

### 导出翻译

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

### 验证翻译完整性

```rust
async fn validate_translations(manager: &I18nManager) {
    let validation = manager.validate_translations().await;
    
    println!("总键数: {}", validation.total_keys);
    println!("完成度: {:.1}%", validation.overall_completion());
    
    for (lang, missing) in &validation.missing_translations {
        println!("{} 缺少 {} 个翻译", lang.as_str(), missing.len());
    }
}
```

## 语言检测优先级

语言检测按照以下优先级进行：

1. **URL 参数** (默认: `?lang=zh`)
2. **Cookie** (默认: `app_language`)
3. **Accept-Language HTTP 头**
4. **默认语言**

## 翻译文件格式

### JSON 格式

```json
{
  "nav": {
    "home": "首页",
    "about": "关于"
  },
  "messages": {
    "welcome": "欢迎",
    "goodbye": "再见"
  }
}
```

### YAML 格式

```yaml
nav:
  home: 首页
  about: 关于
messages:
  welcome: 欢迎
  goodbye: 再见
```

### 嵌套访问

在模板中可以使用点号访问嵌套的翻译键：

```html
{{ t(key="nav.home") }}  <!-- 输出: 首页 -->
{{ t(key="messages.welcome") }}  <!-- 输出: 欢迎 -->
```

## 高级功能

### 翻译缓存

```rust
// 清除缓存
manager.clear_cache().await;

// 获取缓存统计
let stats = manager.cache_stats().await;
println!("缓存条目数: {}", stats.get("total_entries").unwrap_or(&0));
```

### 自定义语言

```rust
let custom_lang = Language::Custom("ko".to_string()); // 韩语
let translation = manager.translate(&custom_lang, "hello");
```

### 翻译参数替换

```rust
let mut params = HashMap::new();
params.insert("name".to_string(), "张三".to_string());
params.insert("count".to_string(), "5".to_string());

let translated = manager.translate_with_params(
    &Language::Zh,
    "welcome_message",
    &params
);
// 如果翻译是 "欢迎 {{name}}，您有 {{count}} 条消息"
// 结果将是: "欢迎 张三，您有 5 条消息"
```

## 最佳实践

### 1. 翻译键命名

- 使用点号分隔的层次结构：`nav.home`, `form.submit`
- 使用描述性的键名：`user.profile.settings` 而不是 `user.prof.set`
- 保持一致性：全部使用小写和下划线

### 2. 翻译文件组织

```
assets/translations/
├── zh.json      # 中文
├── ja.json      # 日语
├── en.json      # 英语
└── ko.json      # 韩语
```

### 3. 性能优化

- 启用翻译缓存
- 在生产环境中禁用热重载
- 使用 CDN 分发翻译文件

### 4. 测试策略

```rust
#[tokio::test]
async fn test_translation_loading() {
    let manager = I18nManager::default();
    manager.init().await.unwrap();
    
    assert_eq!(manager.translate(&Language::Zh, "welcome"), "欢迎");
}
```

## 故障排除

### 常见问题

1. **翻译未显示**
   - 检查翻译文件路径是否正确
   - 确认文件格式是否有效
   - 验证语言代码是否正确

2. **语言检测不工作**
   - 检查中间件是否正确配置
   - 确认 Cookie 和 URL 参数设置
   - 验证 Accept-Language 头格式

3. **性能问题**
   - 启用缓存
   - 检查翻译文件大小
   - 优化翻译键结构

### 调试模式

```rust
// 启用详细日志
env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

// 检查翻译统计
let stats = manager.get_stats();
println!("翻译统计: {:?}", stats);
```

## 总结

Loco 的国际化系统提供了完整的多语言支持，包括：

- 智能语言检测
- 灵活的翻译管理
- 模板集成
- 高性能缓存
- 易于使用的 API

通过正确配置和使用这些功能，你可以轻松构建支持多语言的 Web 应用程序。
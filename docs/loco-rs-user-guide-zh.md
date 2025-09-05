# Loco-rs 用户开发指南

## 🎯 面向 Loco-rs 框架使用者

这份文档专门为使用 Loco-rs 框架构建 Web 应用的开发者准备，涵盖快速上手、项目开发、部署和最佳实践。

## 🚀 快速开始

### 1. 环境准备

#### 系统要求
- **Rust**: 1.70+
- **数据库**: PostgreSQL 12+ 或 SQLite 3+
- **Redis**: 6+ (可选，用于后台任务)
- **Node.js**: 16+ (可选，用于前端构建)

#### 安装 Loco CLI
```bash
# 安装 Loco CLI
cargo install loco-cli

# 验证安装
loco --version
```

### 2. 创建项目

#### 交互式创建
```bash
# 使用交互式向导创建项目
loco new

# 按照提示选择：
# - 应用名称: myapp
# - 模板类型: SaaS App (Server Side Rendering)
# - 数据库: PostgreSQL
# - 后台任务: Queue (Redis)
# - 资产配置: ServerSide
```

#### 快速创建
```bash
# 直接使用模板创建
loco new --template saas-ssr myapp
loco new --template saas-csr myapp
loco new --template rest-api myapp
loco new --template lightweight myapp
```

#### 项目结构
```
myapp/
├── src/
│   ├── app.rs              # 应用配置
│   ├── controllers/        # 控制器
│   ├── models/            # 数据模型
│   ├── views/             # 视图模板
│   ├── workers/           # 后台任务
│   └── mailers/           # 邮件发送
├── config/
│   ├── development.yaml   # 开发配置
│   ├── test.yaml          # 测试配置
│   └── production.yaml    # 生产配置
├── migrations/            # 数据库迁移
├── tests/                # 测试文件
└── Cargo.toml           # 项目配置
```

### 3. 开发环境设置

#### 数据库设置
```bash
# 创建数据库
createdb myapp_development
createdb myapp_test

# 运行迁移
cargo loco db reset

# 生成模型
cargo loco db entities
```

#### 启动开发服务器
```bash
# 启动 Web 服务器
cargo loco start

# 启动后台任务
cargo loco start --worker

# 同时启动服务器和任务
cargo loco start --server-and-worker
```

## 🏗️ 项目开发

### 1. 数据模型开发

#### 创建模型
```bash
# 生成用户模型
cargo loco generate model user \
    name:string \
    email:string^ \
    password:string \
    age:int? \
    is_active:bool \
    created_at:timestamp \
    updated_at:timestamp

# 生成文章模型
cargo loco generate model post \
    title:string \
    content:text \
    user_id:references:users \
    published_at:timestamp?
```

#### 模型关系
```rust
// src/models/user.rs
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub email: String,
    pub password: String,
    pub age: Option<i32>,
    pub is_active: bool,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::post::Entity")]
    Post,
}

impl Related<super::post::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Post.def()
    }
}
```

#### 模型方法
```rust
// src/models/user.rs
impl Model {
    pub fn new(name: &str, email: &str, password: &str) -> Self {
        Self {
            id: 0,
            name: name.to_string(),
            email: email.to_string(),
            password: hash_password(password),
            age: None,
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }
    
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if !self.email.contains('@') {
            return Err("Invalid email format".to_string());
        }
        Ok(())
    }
}

// 查询方法
impl ActiveModel {
    pub async fn find_by_email(db: &DatabaseConnection, email: &str) -> Result<Option<Model>> {
        Entity::find()
            .filter(Column::Email.eq(email))
            .one(db)
            .await
            .map_err(|e| Error::DbErr(e))
    }
}
```

### 2. 控制器开发

#### 生成控制器
```bash
# 生成用户控制器 (API)
cargo loco generate controller api user \
    index show create update delete

# 生成文章控制器 (HTML)
cargo loco generate controller html post \
    index show create edit update delete

# 生成完整脚手架
cargo loco generate scaffold post \
    title:string \
    content:text \
    user_id:references:users
```

#### 控制器实现
```rust
// src/controllers/user.rs
use loco_rs::prelude::*;
use crate::models::users;

pub struct UserController;

impl UserController {
    /// 获取用户列表
    pub async fn index(
        Path(page): Path<u32>,
        State(ctx): State<AppContext>,
    ) -> Result<Format<Json<Vec<users::Model>>>> {
        let users = users::Entity::find()
            .paginate(ctx, &PaginationParams::new(page, 20))
            .await?;
        
        format::json(users)
    }
    
    /// 获取用户详情
    pub async fn show(
        Path(id): Path<i32>,
        State(ctx): State<AppContext>,
    ) -> Result<Format<Json<users::Model>>> {
        let user = users::Entity::find_by_id(id)
            .one(&ctx.db)
            .await?
            .ok_or(Error::NotFound("User not found".to_string()))?;
        
        format::json(user)
    }
    
    /// 创建用户
    pub async fn create(
        State(ctx): State<AppContext>,
        Json(params): Json<CreateUserParams>,
    ) -> Result<Format<Json<users::Model>>> {
        let user = users::ActiveModel {
            name: Set(params.name),
            email: Set(params.email),
            password: Set(hash_password(&params.password)),
            ..Default::default()
        };
        
        let user = user.insert(&ctx.db).await?;
        
        format::json(user)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserParams {
    pub name: String,
    pub email: String,
    pub password: String,
}
```

#### 路由注册
```rust
// src/app.rs
impl Hooks for App {
    async fn before_routes(&self, app: &mut AppRoutes) -> Result<()> {
        // API 路由
        app.add_route(
            Routes::new()
                .prefix("/api")
                .add_route(
                    Routes::new()
                        .prefix("/users")
                        .add_route(users::routes::Routes::new())
                )
        );
        
        // Web 路由
        app.add_route(
            Routes::new()
                .prefix("/web")
                .add_route(
                    Routes::new()
                        .prefix("/posts")
                        .add_route(posts::routes::Routes::new())
                )
        );
        
        Ok(())
    }
}
```

### 3. 视图开发

#### 视图模板
```html
<!-- src/views/posts/index.html.tera -->
{% extends "layouts/app.html.tera" %}

{% block content %}
<div class="container">
    <h1>文章列表</h1>
    
    <a href="/posts/new" class="btn btn-primary">新建文章</a>
    
    <table class="table">
        <thead>
            <tr>
                <th>ID</th>
                <th>标题</th>
                <th>作者</th>
                <th>创建时间</th>
                <th>操作</th>
            </tr>
        </thead>
        <tbody>
            {% for post in posts %}
            <tr>
                <td>{{ post.id }}</td>
                <td>{{ post.title }}</td>
                <td>{{ post.user.name }}</td>
                <td>{{ post.created_at }}</td>
                <td>
                    <a href="/posts/{{ post.id }}/edit">编辑</a>
                    <a href="/posts/{{ post.id }}">查看</a>
                </td>
            </tr>
            {% endfor %}
        </tbody>
    </table>
</div>
{% endblock %}
```

#### 控制器视图方法
```rust
// src/controllers/post.rs
impl PostController {
    pub async fn index(
        State(ctx): State<AppContext>,
    ) -> Result<impl IntoResponse> {
        let posts = posts::Entity::find()
            .find_also_related(users::Entity)
            .all(&ctx.db)
            .await?;
        
        let data = context! {
            "posts" => posts
        };
        
        render_view("posts/index.html.tera", data)
    }
    
    pub async fn show(
        Path(id): Path<i32>,
        State(ctx): State<AppContext>,
    ) -> Result<impl IntoResponse> {
        let post = posts::Entity::find_by_id(id)
            .find_also_related(users::Entity)
            .one(&ctx.db)
            .await?
            .ok_or(Error::NotFound("Post not found".to_string()))?;
        
        let data = context! {
            "post" => post
        };
        
        render_view("posts/show.html.tera", data)
    }
}
```

### 4. 后台任务开发

#### 创建任务
```bash
# 生成邮件发送任务
cargo loco generate worker email

# 生成数据清理任务
cargo loco generate task cleanup
```

#### 任务实现
```rust
// src/workers/email.rs
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailJob {
    pub to: String,
    pub subject: String,
    pub body: String,
}

pub struct EmailWorker;

#[async_trait]
impl BackgroundWorker<EmailJob> for EmailWorker {
    fn queue() -> Option<String> {
        Some("email".to_string())
    }
    
    fn build(_ctx: &AppContext) -> Self {
        Self
    }
    
    async fn perform(&self, job: EmailJob) -> crate::Result<()> {
        info!("Sending email to: {}", job.to);
        
        // 发送邮件逻辑
        let email = Email::new()
            .to(job.to)
            .subject(job.subject)
            .body(job.body);
        
        // 使用邮件发送器
        // ctx.mailer.as_ref().unwrap().send(&email).await?;
        
        Ok(())
    }
}

// 任务调用
pub async fn send_welcome_email(user_email: &str, ctx: &AppContext) -> Result<()> {
    let job = EmailJob {
        to: user_email.to_string(),
        subject: "Welcome to our platform!".to_string(),
        body: "Thank you for joining us!".to_string(),
    };
    
    ctx.queue_provider.as_ref()
        .unwrap()
        .push(job)
        .await?;
    
    Ok(())
}
```

### 5. 认证和授权

#### JWT 认证
```rust
// src/controllers/auth.rs
use loco_rs::prelude::*;
use crate::models::users;

pub struct AuthController;

impl AuthController {
    /// 用户登录
    pub async fn login(
        State(ctx): State<AppContext>,
        Json(params): Json<LoginParams>,
    ) -> Result<Format<Json<AuthResponse>>> {
        let user = users::Entity::find()
            .filter(users::Column::Email.eq(&params.email))
            .one(&ctx.db)
            .await?
            .ok_or(Error::Unauthorized("Invalid credentials".to_string()))?;
        
        if !verify_password(&params.password, &user.password) {
            return Err(Error::Unauthorized("Invalid credentials".to_string()));
        }
        
        let jwt = ctx.jwt.as_ref().unwrap();
        let token = jwt.generate_token(
            3600, // 1 hour
            user.id.to_string(),
            std::collections::HashMap::new(),
        )?;
        
        Ok(format::json(AuthResponse {
            token,
            user: user.into(),
        }))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginParams {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}

// 认证中间件
pub async fn auth_middleware(
    req: Request,
    next: Next,
) -> Result<impl IntoResponse> {
    let auth_header = req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());
    
    let token = auth_header
        .and_then(|h| h.strip_prefix("Bearer ").ok())
        .ok_or(Error::Unauthorized("Missing token".to_string()))?;
    
    // 验证 token
    // let claims = validate_token(token)?;
    
    Ok(next.run(req).await)
}
```

### 6. 表单验证

#### 验证规则
```rust
// src/controllers/user.rs
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 2, max = 50))]
    pub name: String,
    
    #[validate(email)]
    pub email: String,
    
    #[validate(length(min = 8))]
    pub password: String,
    
    #[validate(range(min = 18, max = 120))]
    pub age: Option<i32>,
}

impl UserController {
    pub async fn create(
        State(ctx): State<AppContext>,
        Json(params): Json<CreateUserRequest>,
    ) -> Result<Format<Json<users::Model>>> {
        // 验证请求数据
        params.validate()
            .map_err(|e| Error::Validation(e.to_string()))?;
        
        // 检查邮箱是否已存在
        if users::Entity::find()
            .filter(users::Column::Email.eq(&params.email))
            .one(&ctx.db)
            .await?
            .is_some()
        {
            return Err(Error::Validation("Email already exists".to_string()));
        }
        
        // 创建用户
        let user = users::ActiveModel {
            name: Set(params.name),
            email: Set(params.email),
            password: Set(hash_password(&params.password)),
            age: Set(params.age),
            ..Default::default()
        };
        
        let user = user.insert(&ctx.db).await?;
        
        Ok(format::json(user))
    }
}
```

## 🧪 测试

### 1. 单元测试
```rust
// tests/models/user_test.rs
use crate::models::users;
use loco_rs::testing;

#[tokio::test]
async fn test_user_creation() {
    let db = testing::mock_db().await;
    
    let user = users::ActiveModel {
        name: Set("Test User".to_string()),
        email: Set("test@example.com".to_string()),
        password: Set("hashed_password".to_string()),
        ..Default::default()
    };
    
    let user = user.insert(&db).await.unwrap();
    assert!(user.id > 0);
    assert_eq!(user.name, "Test User");
}

#[tokio::test]
async fn test_user_validation() {
    let user = users::Model::new("", "invalid-email", "password");
    assert!(user.validate().is_err());
}
```

### 2. 集成测试
```rust
// tests/controllers/user_controller_test.rs
use crate::controllers::user::*;
use axum_test::TestServer;
use loco_rs::testing;

#[tokio::test]
async fn test_user_index() {
    let app = testing::create_app().await;
    let server = TestServer::new(app).unwrap();
    
    let response = server
        .get("/api/users/1")
        .expect_json::<users::Model>();
    
    assert_eq!(response.id, 1);
}

#[tokio::test]
async fn test_user_create() {
    let app = testing::create_app().await;
    let server = TestServer::new(app).unwrap();
    
    let new_user = serde_json::json!({
        "name": "New User",
        "email": "newuser@example.com",
        "password": "password123"
    });
    
    let response = server
        .post("/api/users")
        .json(&new_user)
        .expect_json::<users::Model>();
    
    assert_eq!(response.name, "New User");
}
```

### 3. 端到端测试
```rust
// tests/e2e/auth_test.rs
use reqwest::Client;

#[tokio::test]
async fn test_login_flow() {
    let client = Client::new();
    let app_url = "http://localhost:3000";
    
    // 注册用户
    let register_response = client
        .post(&format!("{}/api/auth/register", app_url))
        .json(&serde_json::json!({
            "name": "Test User",
            "email": "test@example.com",
            "password": "password123"
        }))
        .send()
        .await
        .unwrap();
    
    assert!(register_response.status().is_success());
    
    // 登录
    let login_response = client
        .post(&format!("{}/api/auth/login", app_url))
        .json(&serde_json::json!({
            "email": "test@example.com",
            "password": "password123"
        }))
        .send()
        .await
        .unwrap();
    
    assert!(login_response.status().is_success());
    
    let auth_response: AuthResponse = login_response.json().await.unwrap();
    assert!(!auth_response.token.is_empty());
}
```

## 📊 监控和日志

### 1. 日志配置
```yaml
# config/development.yaml
logger:
  level: debug
  format: pretty

# config/production.yaml
logger:
  level: info
  format: json
```

### 2. 结构化日志
```rust
use tracing::{info, warn, error};

pub async fn process_order(order: Order, ctx: &AppContext) -> Result<()> {
    info!(
        order_id = %order.id,
        customer_id = %order.customer_id,
        amount = %order.amount,
        "Processing order"
    );
    
    match order_processor::process(&order, ctx).await {
        Ok(_) => {
            info!(
                order_id = %order.id,
                "Order processed successfully"
            );
        }
        Err(e) => {
            error!(
                order_id = %order.id,
                error = %e,
                "Failed to process order"
            );
            return Err(e);
        }
    }
    
    Ok(())
}
```

### 3. 健康检查
```rust
// src/controllers/health.rs
pub struct HealthController;

impl HealthController {
    pub async fn check(State(ctx): State<AppContext>) -> Result<Format<Json<HealthResponse>>> {
        let mut checks = HashMap::new();
        
        // 数据库检查
        checks.insert("database".to_string(), 
            users::Entity::find()
                .limit(1)
                .one(&ctx.db)
                .await
                .map(|_| "ok".to_string())
                .unwrap_or_else(|_| "error".to_string())
        );
        
        // 缓存检查
        checks.insert("cache".to_string(), 
            ctx.cache.set("health_check", "ok", None).await
                .map(|_| "ok".to_string())
                .unwrap_or_else(|_| "error".to_string())
        );
        
        Ok(format::json(HealthResponse {
            status: "ok".to_string(),
            checks,
            timestamp: chrono::Utc::now(),
        }))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub checks: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
}
```

## 🚀 部署

### 1. Docker 部署
```bash
# 生成 Docker 配置
cargo loco generate deployment docker

# 构建镜像
docker build -t myapp .

# 运行容器
docker run -p 3000:3000 myapp
```

### 2. 生产配置
```yaml
# config/production.yaml
application:
  host: 0.0.0.0
  port: 3000
  workers: 4

database:
  uri: "postgres://user:password@localhost:5432/myapp_production"
  min_connections: 5
  max_connections: 20

cache:
  driver: "redis"
  uri: "redis://localhost:6379/0"

mailer:
  smtp:
    host: "smtp.gmail.com"
    port: 587
    username: "your-email@gmail.com"
    password: "your-password"
    from: "noreply@yourapp.com"

logger:
  level: info
  format: json
```

### 3. 环境变量
```bash
# .env.production
DATABASE_URL=postgres://user:password@localhost:5432/myapp_production
REDIS_URL=redis://localhost:6379/0
JWT_SECRET=your-jwt-secret-key
RUST_LOG=info
RUST_ENV=production
```

## 🔧 性能优化

### 1. 数据库优化
```rust
// 使用索引
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(indexed)]
    pub email: String,
    // 其他字段...
}

// 查询优化
pub async fn find_active_users(
    db: &DatabaseConnection,
    limit: u64,
) -> Result<Vec<users::Model>> {
    users::Entity::find()
        .filter(users::Column::Active.eq(true))
        .order_by_asc(users::Column::CreatedAt)
        .limit(limit)
        .all(db)
        .await
        .map_err(|e| Error::DbErr(e))
}
```

### 2. 缓存优化
```rust
// 缓存用户查询
pub async fn get_user_with_cache(
    ctx: &AppContext,
    user_id: i32,
) -> Result<Option<users::Model>> {
    let cache_key = format!("user:{}", user_id);
    
    if let Some(cached) = ctx.cache.get(&cache_key).await? {
        let user: users::Model = serde_json::from_slice(&cached)?;
        return Ok(Some(user));
    }
    
    let user = users::Entity::find_by_id(user_id)
        .one(&ctx.db)
        .await?;
    
    if let Some(ref user) = user {
        let serialized = serde_json::to_vec(user)?;
        ctx.cache.set(&cache_key, serialized, Some(Duration::hours(1))).await?;
    }
    
    Ok(user)
}
```

### 3. 异步优化
```rust
// 并发处理
pub async fn process_multiple_orders(
    orders: Vec<Order>,
    ctx: &AppContext,
) -> Result<Vec<OrderResult>> {
    let futures = orders.into_iter().map(|order| {
        let ctx = ctx.clone();
        async move {
            process_single_order(order, &ctx).await
        }
    });
    
    try_join_all(futures).await
}
```

## 🛡️ 安全最佳实践

### 1. 输入验证
```rust
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 2, max = 50))]
    pub name: String,
    
    #[validate(email)]
    pub email: String,
    
    #[validate(length(min = 8, regex = "password_regex"))]
    pub password: String,
}

// 自定义验证器
fn password_regex(password: &str) -> Result<(), validator::ValidationError> {
    if password.len() < 8 {
        return Err(validator::ValidationError::new("password_too_short"));
    }
    if !password.chars().any(|c| c.is_ascii_uppercase()) {
        return Err(validator::ValidationError::new("password_missing_uppercase"));
    }
    if !password.chars().any(|c| c.is_ascii_digit()) {
        return Err(validator::ValidationError::new("password_missing_digit"));
    }
    Ok(())
}
```

### 2. SQL 注入防护
```rust
// 使用 SeaORM 参数化查询
pub async fn find_users_by_email_pattern(
    db: &DatabaseConnection,
    email_pattern: &str,
) -> Result<Vec<users::Model>> {
    users::Entity::find()
        .filter(users::Column::Email.like(format!("%{}%", email_pattern)))
        .all(db)
        .await
        .map_err(|e| Error::DbErr(e))
}
```

### 3. XSS 防护
```html
<!-- 自动 HTML 转义 -->
<div>{{ user.name }}</div>

<!-- 手动转义 -->
<div>{{ user.name | escape }}</div>

<!-- 原始 HTML (谨慎使用) -->
<div>{{ user.bio | safe }}</div>
```

## 📚 常见问题

### 1. 数据库连接问题
```rust
// 连接池配置
pub fn create_db_pool(uri: &str) -> Result<DatabaseConnection> {
    let mut opt = ConnectOptions::new(uri.to_string());
    opt.max_connections(20)
       .min_connections(5)
       .connect_timeout(Duration::from_secs(30))
       .idle_timeout(Duration::from_secs(600));
    
    Database::connect(opt).await
        .map_err(|e| Error::Database(e.to_string()))
}
```

### 2. 内存泄漏问题
```rust
// 使用 Arc 共享数据
pub struct AppState {
    pub db: Arc<DatabaseConnection>,
    pub cache: Arc<Cache>,
}

// 避免不必要的克隆
pub async fn process_data(data: &[u8]) -> Result<Vec<u8>> {
    let processed = data.iter()
        .map(|&b| b * 2)
        .collect::<Vec<_>>();
    
    Ok(processed)
}
```

### 3. 并发安全问题
```rust
use tokio::sync::Mutex;

pub struct Counter {
    value: Mutex<i32>,
}

impl Counter {
    pub async fn increment(&self) -> i32 {
        let mut value = self.value.lock().await;
        *value += 1;
        *value
    }
}
```

## 🎯 最佳实践总结

### 1. 项目结构
- 使用清晰的模块结构
- 遵循约定优于配置原则
- 保持代码的可读性和可维护性

### 2. 错误处理
- 使用 Result 类型进行错误处理
- 提供清晰的错误信息
- 实现适当的错误恢复机制

### 3. 性能优化
- 使用异步编程提高并发性能
- 实现缓存策略减少数据库查询
- 优化数据库查询和索引

### 4. 安全考虑
- 实现输入验证和输出编码
- 使用安全的密码哈希算法
- 定期更新依赖项

### 5. 测试策略
- 编写单元测试和集成测试
- 使用测试覆盖率工具
- 实现端到端测试

---

*这份用户指南为使用 Loco-rs 框架的开发者提供了全面的开发指导，涵盖从项目创建到部署的完整开发流程。*
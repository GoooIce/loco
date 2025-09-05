# Loco-rs 框架完整速记文档

## 📋 项目概览

Loco-rs 是一个受 Ruby on Rails 启发的 Rust Web 框架，构建在 Axum 之上，遵循"约定优于配置"原则，提供完整的 Web 应用开发体验。

### 🎯 设计哲学
- **约定优于配置**: 减少决策疲劳，提高开发效率
- **类型安全**: 利用 Rust 的类型系统确保编译时安全
- **异步优先**: 基于 Tokio 的全异步架构
- **模块化设计**: 清晰的模块边界和职责分离
- **开发体验**: 提供丰富的工具和脚手架

### 🏗️ 核心架构
```
应用层 (Application Layer)
├── 用户应用 (User Application)
└── Loco 应用框架 (Loco App Framework)

框架层 (Framework Layer)
├── Web 服务器 (Axum-based)
├── 控制器系统 (Controller System)
├── 模型系统 (Model System)
└── 后台任务系统 (Background Workers)

核心层 (Core Layer)
├── 应用生命周期 (App Lifecycle)
├── 配置管理 (Configuration)
├── 错误处理 (Error Handling)
└── 中间件系统 (Middleware System)

基础设施层 (Infrastructure Layer)
├── 数据库连接 (Database)
├── 缓存系统 (Cache)
├── 存储系统 (Storage)
└── 邮件系统 (Mailer)
```

## 📦 核心组件

### 1. **loco-rs** (主框架库)
**版本**: 0.16.3 | **位置**: `/src`

#### 核心模块
- **应用生命周期**: `app.rs`, `boot.rs`
- **控制器系统**: `controller/`
- **数据库集成**: `model/`, `db/`
- **后台任务**: `bgworker/`
- **认证系统**: `auth/`
- **缓存系统**: `cache/`
- **邮件系统**: `mailer/`
- **存储系统**: `storage/`
- **配置管理**: `config.rs`
- **错误处理**: `errors.rs`

#### 关键特性
- **Hooks trait**: 应用生命周期钩子
- **SharedStore**: 类型安全的异构数据存储
- **多启动模式**: ServerOnly, ServerAndWorker, WorkerOnly, All
- **中间件集成**: Tower 中间件支持
- **多数据库支持**: PostgreSQL, SQLite, MySQL

### 2. **loco-gen** (代码生成器)
**版本**: 0.16.3 | **位置**: `/loco-gen`

#### 核心功能
- **组件生成**: 模型、控制器、迁移、脚手架
- **模板系统**: 基于 Tera 的代码模板
- **类型映射**: 智能类型推断和转换
- **部署配置**: Docker, Shuttle 等部署模板

#### 支持的生成命令
```bash
# 模型生成
cargo loco generate model user name:string email:string^

# 控制器生成
cargo loco generate controller api user index show create

# 脚手架生成
cargo loco generate scaffold post title:string content:text

# 部署配置
cargo loco generate deployment docker
```

### 3. **loco-new** (项目生成器)
**版本**: 0.16.2 | **位置**: `/loco-new`

#### 应用模板类型
1. **SaaS App (Server Side Rendering)**: 完整企业级应用
2. **SaaS App (Client Side Rendering)**: 前后端分离
3. **REST API**: 纯 API 服务
4. **Lightweight Service**: 最小化服务
5. **Advanced**: 完全自定义配置

#### 交互式配置
- **应用名称**: Unicode XID 命名验证
- **数据库选择**: SQLite, PostgreSQL, None
- **后台任务**: Async, Queue, Blocking, None
- **资产配置**: ServerSide, ClientSide, None

### 4. **xtask** (开发任务管理)
**版本**: 0.2.0 | **位置**: `/xtask`

#### 核心命令
```bash
# 运行完整测试套件
cargo xtask test

# 快速测试（仅核心库）
cargo xtask test --quick

# 版本管理
cargo xtask bump <version>
```

#### 开发流程集成
- **代码质量**: `cargo fmt`, `cargo clippy`
- **测试执行**: 多项目并行测试
- **版本同步**: 统一版本管理
- **CI/CD**: 标准化检查流程

## 🚀 快速开始

### 1. 环境准备
```bash
# 安装 Loco CLI
cargo install loco-cli

# 创建项目
loco new myapp

# 选择模板和配置
# - SaaS App (Server Side Rendering)
# - PostgreSQL 数据库
# - Redis 后台任务
# - ServerSide 资产配置
```

### 2. 开发环境设置
```bash
# 进入项目目录
cd myapp

# 设置数据库
createdb myapp_development
createdb myapp_test

# 运行迁移
cargo loco db reset

# 启动开发服务器
cargo loco start

# 启动后台任务
cargo loco start --worker
```

### 3. 项目结构
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

## 🏗️ 核心开发模式

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
```

#### 模型实现
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
```

### 2. 控制器开发

#### 生成控制器
```bash
# 生成用户控制器
cargo loco generate controller api user index show create update delete
```

#### 控制器实现
```rust
// src/controllers/user.rs
pub struct UserController;

impl UserController {
    pub async fn index(
        State(ctx): State<AppContext>,
    ) -> Result<Format<Json<Vec<users::Model>>>> {
        let users = users::Entity::find()
            .all(&ctx.db)
            .await?;
        
        format::json(users)
    }
    
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

### 3. 路由注册
```rust
// src/app.rs
impl Hooks for App {
    async fn before_routes(&self, app: &mut AppRoutes) -> Result<()> {
        app.add_route(
            Routes::new()
                .prefix("/api")
                .add_route(
                    Routes::new()
                        .prefix("/users")
                        .add_route(users::routes::Routes::new())
                )
        );
        
        Ok(())
    }
}
```

### 4. 后台任务开发

#### 创建任务
```bash
# 生成邮件发送任务
cargo loco generate worker email
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
        
        // ctx.mailer.as_ref().unwrap().send(&email).await?;
        
        Ok(())
    }
}
```

### 5. 认证系统

#### JWT 认证
```rust
// src/controllers/auth.rs
pub struct AuthController;

impl AuthController {
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
```

## 🧪 测试

### 1. 单元测试
```rust
// tests/models/user_test.rs
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
```

### 2. 集成测试
```rust
// tests/controllers/user_controller_test.rs
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
    
    #[validate(length(min = 8))]
    pub password: String,
}
```

### 2. 密码安全
```rust
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

pub fn hash_password(password: &str) -> String {
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    
    argon2
        .hash_password(password.as_bytes(), &salt)
        .expect("Unable to hash password")
        .to_string()
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    let argon2 = Argon2::default();
    let parsed_hash = PasswordHash::new(hash).unwrap();
    
    argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}
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
    
    // 处理逻辑
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

## 📚 最佳实践

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

## 🎯 开发者指南

### 面向 Loco-rs 框架开发者
- 参与框架开发和维护
- 扩展框架功能
- 修复框架问题
- 贡献代码和文档

### 面向 Loco-rs 框架使用者
- 使用框架构建应用
- 解决开发问题
- 遵循最佳实践
- 优化应用性能

## 🌐 学习资源

### 官方资源
- **文档**: https://docs.rs/loco-rs
- **GitHub**: https://github.com/loco-rs/loco
- **示例**: https://github.com/loco-rs/loco-examples

### 相关技术
- **Axum**: https://docs.rs/axum
- **SeaORM**: https://www.sea-ql.org/SeaORM/
- **Tokio**: https://tokio.rs/
- **Tera**: https://tera.netlify.app/

---

*这份完整的速记文档提供了 Loco-rs 框架的全面指南，适合框架开发者和使用者快速上手和深入理解框架的各个方面。*
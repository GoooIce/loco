# Loco-rs 框架开发指南

## 🎯 面向 Loco-rs 框架开发者

这份文档专门为参与 Loco-rs 框架开发和维护的开发者准备，涵盖内部架构、开发流程、代码规范和最佳实践。

## 🏗️ 框架内部架构

### 核心设计原则

#### 1. **约定优于配置**
- 减少框架使用者的决策疲劳
- 提供合理的默认值和约定
- 保持配置的可扩展性

#### 2. **类型安全优先**
- 利用 Rust 类型系统确保编译时安全
- 减少运行时错误和异常
- 提供清晰的 API 接口

#### 3. **异步原生**
- 基于 Tokio 的全异步架构
- 非阻塞 I/O 操作
- 高并发处理能力

#### 4. **模块化设计**
- 清晰的模块边界和职责分离
- 可选的特性标志
- 渐进式功能启用

### 架构层次

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

## 📦 核心模块开发指南

### 1. 应用生命周期管理 (`app.rs`, `boot.rs`)

#### AppContext 设计
```rust
// 核心应用上下文 - 所有服务的容器
pub struct AppContext {
    pub environment: Environment,
    pub db: DatabaseConnection,
    pub queue_provider: Option<Arc<bgworker::Queue>>,
    pub config: Config,
    pub mailer: Option<EmailSender>,
    pub storage: Arc<Storage>,
    pub cache: Arc<cache::Cache>,
    pub shared_store: Arc<SharedStore>,
}
```

**开发原则**:
- 保持 AppContext 的简洁性和可扩展性
- 使用 Arc 包装需要共享的服务
- 提供优雅的服务初始化和关闭

#### Hooks trait 设计
```rust
#[async_trait]
pub trait Hooks {
    // 路由注册前
    async fn before_routes(&self, app: &mut AppRoutes) -> Result<()>;
    
    // 路由注册后
    async fn after_routes(&self, app: &mut AppRoutes) -> Result<()>;
    
    // 初始化器
    async fn initializers(&self, ctx: &AppContext) -> Result<()>;
    
    // 服务器启动前
    async fn before_server_start(&self, ctx: &AppContext) -> Result<()>;
    
    // 服务器启动后
    async fn after_server_start(&self, ctx: &AppContext) -> Result<()>;
}
```

**最佳实践**:
- 提供合理的默认实现
- 保持 Hook 方法的一致性
- 清晰的错误处理和传播

### 2. 控制器系统 (`controller/`)

#### 路由设计模式
```rust
// 路由组管理
pub struct Routes {
    pub prefix: Option<String>,
    pub handlers: Vec<Handler>,
}

// 路由合并和嵌套
impl Routes {
    pub fn merge(&mut self, other: Routes) {
        // 实现路由合并逻辑
    }
    
    pub fn nest(mut self, prefix: &str) -> Self {
        // 实现路由嵌套
    }
}
```

**设计考虑**:
- 支持路由分组和嵌套
- 提供灵活的前缀管理
- 集成中间件系统

#### 控制器实现模式
```rust
// 标准控制器实现
pub struct UserController;

impl UserController {
    pub async fn index(ctx: AppContext) -> Result<Format<Json<Vec<User>>>> {
        // 实现列表逻辑
    }
    
    pub async fn show(Path(id): Path<i32>, ctx: AppContext) -> Result<Format<Json<User>>> {
        // 实现详情逻辑
    }
}
```

### 3. 数据库集成 (`model/`, `db/`)

#### 模型设计原则
```rust
// 使用 SeaORM 实体
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub email: String,
    pub password: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::post::Entity")]
    Post,
}
```

**最佳实践**:
- 保持模型的简洁性
- 使用 SeaORM 的关系系统
- 提供合理的默认值

#### 数据库连接管理
```rust
// 连接池配置
pub struct DatabaseConfig {
    pub uri: String,
    pub min_connections: u32,
    pub max_connections: u32,
    pub connect_timeout: Duration,
    pub idle_timeout: Duration,
}

// 连接管理
impl Database {
    pub async fn connect(config: &DatabaseConfig) -> Result<DatabaseConnection> {
        // 实现连接逻辑
    }
}
```

### 4. 后台任务系统 (`bgworker/`)

#### 任务队列设计
```rust
// 统一的任务 trait
#[async_trait]
pub trait BackgroundWorker<T: Job + Send + Sync> {
    fn queue() -> Option<String>;
    fn build(ctx: &AppContext) -> Self;
    async fn perform(&self, job: T) -> crate::Result<()>;
}

// 任务实现示例
pub struct EmailWorker;

#[async_trait]
impl BackgroundWorker<EmailJob> for EmailWorker {
    fn queue() -> Option<String> {
        Some("email".to_string())
    }
    
    fn build(ctx: &AppContext) -> Self {
        Self
    }
    
    async fn perform(&self, job: EmailJob) -> crate::Result<()> {
        // 实现邮件发送逻辑
    }
}
```

**多后端支持策略**:
- Redis: 生产环境推荐
- PostgreSQL: 使用 SKIP LOCKED
- SQLite: 应用级锁

### 5. 认证系统 (`auth/`)

#### JWT 实现
```rust
pub struct JWT {
    secret: String,
    algorithm: Algorithm,
}

impl JWT {
    pub fn new(secret: &str, algorithm: Algorithm) -> Self {
        Self {
            secret: secret.to_string(),
            algorithm,
        }
    }
    
    pub fn generate_token(&self, expiration: u64, pid: String, claims: Map<String, Value>) -> JWTResult<String> {
        // 实现令牌生成
    }
    
    pub fn validate(&self, token: &str) -> JWTResult<TokenData<UserClaims>> {
        // 实现令牌验证
    }
}
```

**安全考虑**:
- 使用强加密算法 (HS512)
- 合理的过期时间设置
- 安全的密钥管理

### 6. 缓存系统 (`cache/`)

#### 驱动抽象设计
```rust
#[async_trait]
pub trait CacheDriver: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    async fn set(&self, key: &str, value: Vec<u8>, ttl: Option<Duration>) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn exists(&self, key: &str) -> Result<bool>;
}

// 缓存包装器
pub struct Cache {
    pub driver: Box<dyn CacheDriver>,
}
```

**性能优化**:
- 实现连接池
- 批量操作支持
- 智能过期策略

## 🔧 开发工作流

### 1. 代码生成器开发 (`loco-gen`)

#### 模板系统架构
```rust
// 模板引擎包装
pub struct TemplateEngine {
    tera: Tera,
    context: Context,
}

impl TemplateEngine {
    pub fn new() -> Result<Self> {
        // 初始化模板引擎
    }
    
    pub fn render(&self, template_name: &str, context: &Context) -> Result<String> {
        // 渲染模板
    }
}
```

#### 类型映射系统
```rust
// 字段类型定义
pub struct FieldType {
    pub name: String,      // 用户输入类型名
    pub rust: RustType,    // Rust 类型
    pub schema: String,    // Schema 类型
    pub col_type: String,  // 数据库列类型
    pub arity: usize,      // 参数数量
}

// 类型映射管理
pub struct TypeMapper {
    mappings: HashMap<String, FieldType>,
}
```

**开发原则**:
- 保持模板的简洁性和可维护性
- 提供清晰的错误信息
- 支持自定义扩展

### 2. 项目生成器开发 (`loco-new`)

#### 交互式向导设计
```rust
pub struct Wizard {
    app_name: String,
    template_type: TemplateType,
    db_option: DBOption,
    bg_option: BackgroundOption,
    assets_option: AssetsOption,
}

impl Wizard {
    pub async fn run(&mut self) -> Result<Settings> {
        // 实现交互式向导
    }
}
```

**用户体验考虑**:
- 提供清晰的选项说明
- 智能默认值
- 输入验证和错误提示

### 3. 开发任务管理 (`xtask`)

#### 任务执行架构
```rust
// CI 任务管理
pub struct CI {
    project_dir: PathBuf,
}

impl CI {
    pub fn run_all_tests(&self) -> Result<Vec<RunResults>> {
        // 运行所有测试
    }
    
    pub fn run_quick_tests(&self) -> Result<Vec<RunResults>> {
        // 运行快速测试
    }
}
```

**自动化考虑**:
- 支持并行执行
- 提供详细的错误报告
- 集成 CI/CD 流程

## 📋 代码规范和最佳实践

### 1. 代码风格

#### Rust 代码规范
```rust
// 使用标准 Rust 格式
// 遵循 Rust API 指南
// 使用 clippy 进行代码检查

// 示例：函数命名
pub fn get_user_by_id(id: i32) -> Result<Option<User>> {
    // 实现逻辑
}

// 示例：错误处理
pub fn process_user(user: User) -> Result<ProcessedUser> {
    user.validate()
        .map_err(|e| Error::Validation(e.to_string()))?
        .process()
        .map_err(|e| Error::Processing(e.to_string()))
}
```

#### 文档规范
```rust
/// 用户控制器
/// 
/// 提供用户相关的 HTTP 端点
/// 
/// # 端点
/// - `GET /users` - 获取用户列表
/// - `GET /users/{id}` - 获取用户详情
/// - `POST /users` - 创建用户
pub struct UserController;

impl UserController {
    /// 获取用户列表
    /// 
    /// # 参数
    /// - `ctx`: 应用上下文
    /// 
    /// # 返回
    /// 返回用户列表的 JSON 响应
    /// 
    /// # 错误
    /// 返回数据库查询错误
    pub async fn index(ctx: AppContext) -> Result<Format<Json<Vec<User>>>> {
        // 实现逻辑
    }
}
```

### 2. 测试策略

#### 单元测试
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_user_creation() {
        let user = User::new("test@example.com", "password");
        assert!(user.is_valid());
    }
    
    #[tokio::test]
    async fn test_user_repository() {
        let repo = UserRepository::new(&mock_db());
        let user = repo.create("test@example.com", "password").await.unwrap();
        assert!(user.id > 0);
    }
}
```

#### 集成测试
```rust
#[tokio::test]
async fn test_user_controller_integration() {
    // 设置测试应用
    let app = create_test_app().await;
    
    // 发送测试请求
    let response = app
        .post("/api/users")
        .json(&json!({"email": "test@example.com", "password": "password"}))
        .await;
    
    // 验证响应
    response.assert_status(201);
    response.assert_json::<User>();
}
```

### 3. 错误处理

#### 错误类型设计
```rust
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("配置错误: {0}")]
    Config(String),
    
    #[error("数据库错误: {0}")]
    Database(#[from] sea_orm::DbErr),
    
    #[error("验证错误: {0}")]
    Validation(String),
    
    #[error("未找到: {0}")]
    NotFound(String),
    
    #[error("内部服务器错误")]
    Internal,
}

// HTTP 错误响应
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            Error::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            Error::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error".to_string()),
        };
        
        (status, Json(json!({ "error": error_message }))).into_response()
    }
}
```

### 4. 性能优化

#### 异步编程最佳实践
```rust
// 使用正确的异步 trait
#[async_trait]
pub trait AsyncService {
    async fn process(&self, request: Request) -> Result<Response>;
}

// 避免阻塞操作
pub async fn handle_request(request: Request) -> Result<Response> {
    // 使用 spawn_blocking 处理 CPU 密集型任务
    let result = tokio::task::spawn_blocking(|| {
        // CPU 密集型操作
        heavy_computation(request.data)
    }).await?;
    
    Ok(Response::new(result))
}
```

#### 资源管理
```rust
// 连接池配置
pub fn create_database_pool(config: &DatabaseConfig) -> Result<DatabaseConnection> {
    let mut opt = ConnectOptions::new(config.uri.clone());
    opt.max_connections(config.max_connections)
       .min_connections(config.min_connections)
       .connect_timeout(Duration::from_secs(config.connect_timeout))
       .idle_timeout(Duration::from_secs(config.idle_timeout));
    
    Database::connect(opt).await.map_err(|e| Error::Database(e))
}
```

## 🚀 发布流程

### 1. 版本管理

#### 语义化版本控制
```bash
# 版本格式: MAJOR.MINOR.PATCH
# MAJOR: 不兼容的 API 更改
# MINOR: 向后兼容的功能添加
# PATCH: 向后兼容的错误修复

# 使用 xtask 进行版本管理
cargo xtask bump 0.16.4
```

#### 发布检查清单
- [ ] 所有测试通过
- [ ] 代码格式化检查
- [ ] Clippy 检查通过
- [ ] 文档更新
- [ ] 变更日志更新
- [ ] 版本号更新
- [ ] 依赖项检查

### 2. CI/CD 集成

#### GitHub Actions 工作流
```yaml
name: CI
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
      - name: Run tests
        run: |
          cargo xtask test
          cargo xtask test --quick
```

### 3. 文档生成

#### API 文档
```bash
# 生成 API 文档
cargo doc --no-deps --features "with-db auth_jwt cli testing"

# 打开文档
cargo doc --open
```

#### 用户指南
- 使用 Markdown 格式
- 包含代码示例
- 提供最佳实践指导

## 🔍 调试和故障排除

### 1. 日志系统

#### 结构化日志
```rust
use tracing::{info, warn, error, debug};

pub async fn process_user(user: User) -> Result<()> {
    info!(user_id = %user.id, "Processing user");
    
    match user.validate() {
        Ok(_) => {
            debug!("User validation passed");
            // 继续处理
        }
        Err(e) => {
            warn!(error = %e, "User validation failed");
            return Err(e.into());
        }
    }
}
```

### 2. 性能分析

#### 性能监控
```rust
use tracing::span;

pub async fn handle_request(request: Request) -> Result<Response> {
    let span = span!(Level::INFO, "request", 
        method = %request.method(),
        path = %request.uri().path()
    );
    
    let _enter = span.enter();
    
    // 处理请求
    let response = process_request(request).await;
    
    info!(duration_ms = %start.elapsed().as_millis(), "Request completed");
    
    response
}
```

### 3. 内存管理

#### 内存优化
```rust
// 使用 Arc 共享数据
pub struct SharedState {
    config: Arc<Config>,
    cache: Arc<Cache>,
}

// 避免不必要的克隆
pub fn process_data(data: &[u8]) -> Result<Vec<u8>> {
    // 使用引用而不是克隆
    let processed = data.iter()
        .map(|&b| b * 2)
        .collect::<Vec<_>>();
    
    Ok(processed)
}
```

## 🌐 社区和贡献

### 1. 贡献指南

#### 代码贡献流程
1. Fork 项目仓库
2. 创建功能分支
3. 编写代码和测试
4. 提交 Pull Request
5. 等待代码审查
6. 合并到主分支

#### 代码审查标准
- 代码风格符合规范
- 测试覆盖率充足
- 文档完整准确
- 性能影响评估
- 安全性考虑

### 2. 问题报告

#### Bug 报告模板
```markdown
## Bug 描述
简要描述 bug 的情况

## 复现步骤
1. 执行步骤 A
2. 执行步骤 B
3. 观察结果

## 期望行为
描述期望的结果

## 实际行为
描述实际的结果

## 环境信息
- 操作系统: 
- Rust 版本:
- Loco 版本:
```

### 3. 功能请求

#### 功能请求模板
```markdown
## 功能描述
描述新功能的用途和价值

## 使用场景
描述具体的使用场景

## 建议实现
建议的实现方式

## 替代方案
考虑的替代方案
```

## 📚 学习资源

### 1. 内部文档
- 架构设计文档
- API 参考文档
- 代码规范文档
- 测试指南

### 2. 外部资源
- Rust 官方文档
- Axum 框架文档
- SeaORM 文档
- Tokio 异步编程文档

### 3. 最佳实践
- Rust 编程最佳实践
- Web 应用安全最佳实践
- 性能优化最佳实践
- 测试驱动开发最佳实践

---

*这份开发指南为 Loco-rs 框架开发者提供了全面的开发指导，涵盖架构设计、开发流程、代码规范和最佳实践。*
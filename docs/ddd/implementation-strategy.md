# DDD + TDD 在 Loco 中的实施策略

## 🎯 实施策略概述

基于对 DDD 和 TDD 概念的分析以及对 Loco 框架的评估，我们制定以下实施策略：

### 核心原则
1. **渐进式实施**: 分阶段逐步引入 DDD 和 TDD
2. **实用主义**: 根据项目实际情况调整实施策略
3. **测试优先**: 始终遵循 TDD 的 Red-Green-Refactor 循环
4. **业务驱动**: 以业务价值为导向进行架构设计

### 实施路径
```
阶段 1: 基础设施搭建 → 阶段 2: 核心领域开发 → 阶段 3: 集成测试 → 阶段 4: 部署运维
```

## 🏗️ 架构设计策略

### 1. 分层架构实施

#### 依赖方向控制
```
用户界面层 → 应用服务层 → 领域层 ← 基础设施层
```

#### 接口隔离原则
```rust
// 定义清晰的接口，实现依赖倒置
pub trait UserRepository: Send + Sync {
    async fn save(&self, user: &UserAggregate) -> Result<()>;
    async fn find_by_id(&self, id: &UserId) -> Result<Option<UserAggregate>>;
    async fn find_by_email(&self, email: &Email) -> Result<Option<UserAggregate>>;
    async fn delete(&self, id: &UserId) -> Result<()>;
}

// 领域层只依赖接口，不依赖具体实现
pub struct UserService {
    repository: Arc<dyn UserRepository>,
    event_publisher: Arc<dyn EventPublisher>,
}
```

#### 模块组织策略
```rust
// 使用 Rust 的模块系统组织代码
pub mod domains {
    pub mod user {
        pub mod entities;
        pub mod value_objects;
        pub mod aggregates;
        pub mod services;
        pub mod events;
        pub mod repositories;
    }
    
    pub mod order {
        pub mod entities;
        pub mod value_objects;
        pub mod aggregates;
        pub mod services;
        pub mod events;
        pub mod repositories;
    }
}

pub mod applications {
    pub mod services;
    pub mod commands;
    pub mod queries;
    pub mod dtos;
}

pub mod infrastructure {
    pub mod persistence;
    pub mod events;
    pub mod external;
    pub mod cache;
}

pub mod interfaces {
    pub mod controllers;
    pub mod routes;
    pub mod middleware;
}
```

### 2. 领域建模策略

#### 通用语言建立
```rust
// 使用业务术语定义类型和操作
pub type UserId = String;
pub type OrderId = String;
pub type Email = String; // 将被值对象包装

// 业务操作命名
pub fn register_user(command: RegisterUserCommand) -> Result<UserRegisteredEvent> {
    // 实现用户注册逻辑
}

pub fn place_order(command: PlaceOrderCommand) -> Result<OrderPlacedEvent> {
    // 实现下单逻辑
}
```

#### 限界上下文识别
```rust
// 每个限界上下文一个模块
pub mod user_context {
    pub use super::user::*;
    
    // 上下文特定的类型和操作
    pub fn user_login(credentials: LoginCredentials) -> Result<UserLoggedInEvent> {
        // 用户登录逻辑
    }
}

pub mod order_context {
    pub use super::order::*;
    
    // 上下文特定的类型和操作
    pub fn order_payment(order_id: OrderId, payment: Payment) -> Result<OrderPaidEvent> {
        // 订单支付逻辑
    }
}
```

#### 聚合设计
```rust
// 聚合根实现
#[derive(Debug, Clone)]
pub struct UserAggregate {
    id: UserId,
    email: Email,
    name: String,
    password_hash: String,
    version: u32,
    events: Vec<UserEvent>,
}

impl AggregateRoot for UserAggregate {
    type Id = UserId;
    type Event = UserEvent;
    
    fn id(&self) -> &UserId {
        &self.id
    }
    
    fn version(&self) -> u32 {
        self.version
    }
    
    fn events(&self) -> Vec<UserEvent> {
        self.events.clone()
    }
}

impl UserAggregate {
    pub fn new(id: UserId, email: Email, name: String, password: String) -> Self {
        let password_hash = hash_password(&password);
        let mut aggregate = Self {
            id,
            email,
            name,
            password_hash,
            version: 1,
            events: Vec::new(),
        };
        
        aggregate.events.push(UserEvent::Registered(UserRegisteredEvent {
            user_id: id.clone(),
            email: email.value().clone(),
            registered_at: Utc::now(),
        }));
        
        aggregate
    }
    
    pub fn change_email(&mut self, new_email: Email) -> Result<()> {
        if self.email == new_email {
            return Err(Error::EmailAlreadyUsed);
        }
        
        self.email = new_email.clone();
        self.version += 1;
        
        self.events.push(UserEvent::EmailChanged(EmailChangedEvent {
            user_id: self.id.clone(),
            old_email: self.email.value().clone(),
            new_email: new_email.value().clone(),
            changed_at: Utc::now(),
        }));
        
        Ok(())
    }
}
```

## 🧪 TDD 实施策略

### 1. 测试金字塔策略

#### 测试层次划分
```
     ┌─────────────────┐
     │   端到端测试   │  ← 少量，关键业务流程
     ├─────────────────┤
     │   集成测试     │  ← 中等数量，组件交互
     ├─────────────────┤
     │   单元测试     │  ← 大量，核心业务逻辑
     └─────────────────┘
```

#### 测试优先级
1. **单元测试**: 测试领域逻辑和业务规则
2. **集成测试**: 测试组件间交互和外部依赖
3. **端到端测试**: 测试完整的用户流程

### 2. 测试驱动开发流程

#### Red-Green-Refactor 循环
```rust
// 1. Red: 编写失败的测试
#[tokio::test]
async fn test_user_registration() {
    // 准备测试数据
    let command = RegisterUserCommand {
        email: "test@example.com".to_string(),
        name: "Test User".to_string(),
        password: "password123".to_string(),
    };
    
    let service = UserService::new(/* 依赖 */);
    
    // 执行测试
    let result = service.register_user(command).await;
    
    // 验证结果
    assert!(result.is_ok());
    let event = result.unwrap();
    assert_eq!(event.user_id, "user_id");
}

// 2. Green: 编写最少的代码使测试通过
impl UserService {
    pub async fn register_user(&self, command: RegisterUserCommand) -> Result<UserRegisteredEvent> {
        // 最简单的实现
        Ok(UserRegisteredEvent {
            user_id: "user_id".to_string(),
            email: command.email,
            registered_at: Utc::now(),
        })
    }
}

// 3. Refactor: 重构代码，保持测试通过
impl UserService {
    pub async fn register_user(&self, command: RegisterUserCommand) -> Result<UserRegisteredEvent> {
        // 验证邮箱格式
        let email = Email::new(command.email)?;
        
        // 检查邮箱是否已存在
        if self.repository.find_by_email(&email).await?.is_some() {
            return Err(Error::EmailAlreadyExists);
        }
        
        // 创建用户聚合
        let user_id = UserId::new();
        let user = UserAggregate::new(
            user_id.clone(),
            email,
            command.name,
            command.password,
        );
        
        // 保存用户
        self.repository.save(&user).await?;
        
        // 发布事件
        let event = UserRegisteredEvent {
            user_id: user_id.value().clone(),
            email: user.email.value().clone(),
            registered_at: Utc::now(),
        };
        
        self.event_publisher.publish(event.clone()).await?;
        
        Ok(event)
    }
}
```

#### 测试数据管理
```rust
// 测试工厂模式
pub struct UserFactory;

impl UserFactory {
    pub fn create_user() -> UserAggregate {
        UserAggregate::new(
            UserId::new(),
            Email::new("test@example.com").unwrap(),
            "Test User".to_string(),
            "password123".to_string(),
        )
    }
    
    pub fn create_command() -> RegisterUserCommand {
        RegisterUserCommand {
            email: "test@example.com".to_string(),
            name: "Test User".to_string(),
            password: "password123".to_string(),
        }
    }
}

// Mock 依赖
pub struct MockUserRepository {
    users: Arc<Mutex<HashMap<UserId, UserAggregate>>>,
}

#[async_trait]
impl UserRepository for MockUserRepository {
    async fn save(&self, user: &UserAggregate) -> Result<()> {
        let mut users = self.users.lock().unwrap();
        users.insert(user.id().clone(), user.clone());
        Ok(())
    }
    
    async fn find_by_id(&self, id: &UserId) -> Result<Option<UserAggregate>> {
        let users = self.users.lock().unwrap();
        Ok(users.get(id).cloned())
    }
    
    async fn find_by_email(&self, email: &Email) -> Result<Option<UserAggregate>> {
        let users = self.users.lock().unwrap();
        Ok(users.values().find(|u| u.email() == email).cloned())
    }
    
    async fn delete(&self, id: &UserId) -> Result<()> {
        let mut users = self.users.lock().unwrap();
        users.remove(id);
        Ok(())
    }
}
```

### 3. 集成测试策略

#### 仓库集成测试
```rust
#[tokio::test]
async fn test_user_repository_integration() {
    // 使用测试数据库
    let db = create_test_database().await;
    let repository = PostgresUserRepository::new(db);
    
    // 创建用户
    let user = UserFactory::create_user();
    repository.save(&user).await.unwrap();
    
    // 查询用户
    let found_user = repository.find_by_id(user.id()).await.unwrap();
    assert!(found_user.is_some());
    assert_eq!(found_user.unwrap().email(), user.email());
    
    // 删除用户
    repository.delete(user.id()).await.unwrap();
    let deleted_user = repository.find_by_id(user.id()).await.unwrap();
    assert!(deleted_user.is_none());
}
```

#### 控制器集成测试
```rust
#[tokio::test]
async fn test_user_controller_integration() {
    // 创建测试应用
    let app = create_test_app().await;
    let server = TestServer::new(app).unwrap();
    
    // 测试用户注册
    let response = server
        .post("/api/users/register")
        .json(&json!({
            "email": "test@example.com",
            "name": "Test User",
            "password": "password123"
        }))
        .await;
    
    response.assert_status(201);
    response.assert_json::<UserDto>();
}
```

## 🔄 事件驱动策略

### 1. 领域事件实现

#### 事件定义
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserEvent {
    Registered(UserRegisteredEvent),
    EmailChanged(EmailChangedEvent),
    PasswordChanged(PasswordChangedEvent),
    Deleted(UserDeletedEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRegisteredEvent {
    pub user_id: String,
    pub email: String,
    pub registered_at: DateTime<Utc>,
}

impl DomainEvent for UserEvent {
    fn event_type(&self) -> &str {
        match self {
            UserEvent::Registered(_) => "user.registered",
            UserEvent::EmailChanged(_) => "user.email_changed",
            UserEvent::PasswordChanged(_) => "user.password_changed",
            UserEvent::Deleted(_) => "user.deleted",
        }
    }
    
    fn occurred_at(&self) -> DateTime<Utc> {
        match self {
            UserEvent::Registered(e) => e.registered_at,
            UserEvent::EmailChanged(e) => e.changed_at,
            UserEvent::PasswordChanged(e) => e.changed_at,
            UserEvent::Deleted(e) => e.deleted_at,
        }
    }
    
    fn aggregate_id(&self) -> &str {
        match self {
            UserEvent::Registered(e) => &e.user_id,
            UserEvent::EmailChanged(e) => &e.user_id,
            UserEvent::PasswordChanged(e) => &e.user_id,
            UserEvent::Deleted(e) => &e.user_id,
        }
    }
}
```

#### 事件处理器
```rust
pub struct UserEventHandler {
    email_service: Arc<dyn EmailService>,
    notification_service: Arc<dyn NotificationService>,
}

#[async_trait]
impl EventHandler for UserEventHandler {
    type Event = UserEvent;
    
    async fn handle(&self, event: &UserEvent) -> Result<()> {
        match event {
            UserEvent::Registered(event) => {
                // 发送欢迎邮件
                self.email_service.send_welcome_email(&event.email).await?;
                
                // 发送通知
                self.notification_service.send_notification(
                    &event.user_id,
                    "Welcome to our platform!",
                ).await?;
            }
            UserEvent::EmailChanged(event) => {
                // 发送邮件变更确认
                self.email_service.send_email_change_confirmation(
                    &event.new_email,
                    &event.old_email,
                ).await?;
            }
            _ => {}
        }
        Ok(())
    }
}
```

### 2. 事件存储实现

#### 事件存储接口
```rust
#[async_trait]
pub trait EventStore: Send + Sync {
    async fn save_events(&self, aggregate_id: &str, events: &[DomainEvent]) -> Result<()>;
    async fn get_events(&self, aggregate_id: &str) -> Result<Vec<Box<dyn DomainEvent>>>;
    async fn get_events_by_type(&self, event_type: &str) -> Result<Vec<Box<dyn DomainEvent>>>;
}

pub struct PostgresEventStore {
    pool: Arc<Pool<Postgres>>,
}

#[async_trait]
impl EventStore for PostgresEventStore {
    async fn save_events(&self, aggregate_id: &str, events: &[DomainEvent]) -> Result<()> {
        let mut transaction = self.pool.begin().await?;
        
        for event in events {
            let event_data = serde_json::to_value(event)?;
            
            sqlx::query!(
                r#"
                INSERT INTO domain_events (aggregate_id, event_type, event_data, occurred_at)
                VALUES ($1, $2, $3, $4)
                "#,
                aggregate_id,
                event.event_type(),
                event_data,
                event.occurred_at()
            )
            .execute(&mut transaction)
            .await?;
        }
        
        transaction.commit().await?;
        Ok(())
    }
    
    async fn get_events(&self, aggregate_id: &str) -> Result<Vec<Box<dyn DomainEvent>>> {
        let rows = sqlx::query!(
            r#"
            SELECT event_type, event_data FROM domain_events
            WHERE aggregate_id = $1
            ORDER BY occurred_at ASC
            "#,
            aggregate_id
        )
        .fetch_all(&self.pool)
        .await?;
        
        let mut events = Vec::new();
        for row in rows {
            let event: Box<dyn DomainEvent> = match row.event_type.as_str() {
                "user.registered" => {
                    let event: UserRegisteredEvent = serde_json::from_value(row.event_data)?;
                    Box::new(UserEvent::Registered(event))
                }
                // 其他事件类型...
                _ => return Err(Error::UnknownEventType),
            };
            events.push(event);
        }
        
        Ok(events)
    }
}
```

## 🚀 CQRS 实施策略

### 1. 命令查询分离

#### 命令定义
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterUserCommand {
    pub email: String,
    pub name: String,
    pub password: String,
}

impl Command for RegisterUserCommand {
    type Result = UserRegisteredEvent;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetUserQuery {
    pub user_id: String,
}

impl Query for GetUserQuery {
    type Result = UserDto;
}
```

#### 命令处理器
```rust
pub struct RegisterUserCommandHandler {
    user_repository: Arc<dyn UserRepository>,
    event_store: Arc<dyn EventStore>,
    event_publisher: Arc<dyn EventPublisher>,
}

#[async_trait]
impl CommandHandler<RegisterUserCommand> for RegisterUserCommandHandler {
    async fn handle(&self, command: RegisterUserCommand) -> Result<UserRegisteredEvent> {
        // 验证邮箱
        let email = Email::new(command.email)?;
        
        // 检查邮箱是否已存在
        if self.user_repository.find_by_email(&email).await?.is_some() {
            return Err(Error::EmailAlreadyExists);
        }
        
        // 创建用户聚合
        let user_id = UserId::new();
        let user = UserAggregate::new(
            user_id.clone(),
            email,
            command.name,
            command.password,
        );
        
        // 保存事件
        let events = user.events();
        self.event_store.save_events(&user_id.to_string(), &events).await?;
        
        // 保存聚合
        self.user_repository.save(&user).await?;
        
        // 发布事件
        for event in events {
            self.event_publisher.publish(event).await?;
        }
        
        // 返回结果
        match events.first() {
            Some(UserEvent::Registered(event)) => Ok(event.clone()),
            _ => Err(Error::EventNotFound),
        }
    }
}
```

#### 查询处理器
```rust
pub struct GetUserQueryHandler {
    user_repository: Arc<dyn UserRepository>,
}

#[async_trait]
impl QueryHandler<GetUserQuery> for GetUserQueryHandler {
    async fn handle(&self, query: GetUserQuery) -> Result<UserDto> {
        let user_id = UserId::from(query.user_id);
        let user = self.user_repository.find_by_id(&user_id).await?
            .ok_or(Error::UserNotFound)?;
        
        Ok(UserDto {
            id: user.id().to_string(),
            email: user.email().to_string(),
            name: user.name().to_string(),
            created_at: user.created_at(),
        })
    }
}
```

### 2. 读模型实现

#### 读模型定义
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct UserReadModel {
    pub id: String,
    pub email: String,
    pub name: String,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserDto {
    pub id: String,
    pub email: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
}
```

#### 事件处理器更新读模型
```rust
pub struct UserReadModelUpdater {
    pool: Arc<Pool<Postgres>>,
}

#[async_trait]
impl EventHandler for UserReadModelUpdater {
    type Event = UserEvent;
    
    async fn handle(&self, event: &UserEvent) -> Result<()> {
        match event {
            UserEvent::Registered(event) => {
                sqlx::query!(
                    r#"
                    INSERT INTO user_read_models (id, email, name, email_verified, created_at, updated_at)
                    VALUES ($1, $2, $3, $4, $5, $6)
                    ON CONFLICT (id) DO UPDATE SET
                    email = $2,
                    name = $3,
                    updated_at = $6
                    "#,
                    event.user_id,
                    event.email,
                    "Test User", // 从命令中获取
                    false, // 初始未验证
                    event.registered_at,
                    event.registered_at,
                )
                .execute(&self.pool)
                .await?;
            }
            UserEvent::EmailChanged(event) => {
                sqlx::query!(
                    r#"
                    UPDATE user_read_models
                    SET email = $1, updated_at = $2
                    WHERE id = $3
                    "#,
                    event.new_email,
                    event.changed_at,
                    event.user_id,
                )
                .execute(&self.pool)
                .await?;
            }
            _ => {}
        }
        Ok(())
    }
}
```

## 🛠️ 实施工具和模板

### 1. 代码生成模板

#### DDD 组件生成器
```rust
// 生成限界上下文
pub fn generate_bounded_context(name: &str) -> Result<String> {
    let template = r#"
// src/domains/{name}/mod.rs
pub mod entities;
pub mod value_objects;
pub mod aggregates;
pub mod services;
pub mod events;
pub mod repositories;

pub use self::aggregates::*;
pub use self::entities::*;
pub use self::value_objects::*;
pub use self::services::*;
pub use self::events::*;
pub use self::repositories::*;
"#;
    
    Ok(template.replace("{name}", name))
}

// 生成聚合
pub fn generate_aggregate(bounded_context: &str, name: &str) -> Result<String> {
    let template = r#"
// src/domains/{bounded_context}/aggregates/{name}.rs
use crate::ddd::*;
use crate::domains::{bounded_context}::*;

#[derive(Debug, Clone)]
pub struct {Name}Aggregate {{
    id: {Name}Id,
    // 聚合属性
    version: u32,
    events: Vec<{Name}Event>,
}}

impl AggregateRoot for {Name}Aggregate {{
    type Id = {Name}Id;
    type Event = {Name}Event;
    
    fn id(&self) -> &Self::Id {{
        &self.id
    }}
    
    fn version(&self) -> u32 {{
        self.version
    }}
    
    fn events(&self) -> Vec<Self::Event> {{
        self.events.clone()
    }}
}}

impl {Name}Aggregate {{
    pub fn new(id: {Name}Id) -> Self {{
        Self {{
            id,
            // 初始化属性
            version: 1,
            events: Vec::new(),
        }}
    }}
}}
"#;
    
    Ok(template
        .replace("{bounded_context}", bounded_context)
        .replace("{name}", name)
        .replace("{Name}", &name.to_title_case()))
}
```

### 2. 测试模板

#### 单元测试模板
```rust
// 测试模板
#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::*;

    #[tokio::test]
    async fn test_{test_name}() {
        // Arrange
        let {arrangement} = {setup}();
        
        // Act
        let result = {action}().await;
        
        // Assert
        assert!(result.is_ok());
        // 更多断言...
    }
}
```

#### 集成测试模板
```rust
// 集成测试模板
#[tokio::test]
async fn test_{integration_name}() {
    // 创建测试应用
    let app = create_test_app().await;
    let server = TestServer::new(app).unwrap();
    
    // 准备测试数据
    let test_data = serde_json::json!({{
        // 测试数据
    }});
    
    // 执行请求
    let response = server
        .post("{endpoint}")
        .json(&test_data)
        .await;
    
    // 验证结果
    response.assert_status({expected_status});
    response.assert_json::<{ResponseType}>();
}
```

## 📊 监控和度量

### 1. 业务指标监控

#### 领域事件监控
```rust
pub struct DomainEventMonitor {
    metrics: Arc<Metrics>,
}

impl DomainEventMonitor {
    pub async fn track_event(&self, event: &dyn DomainEvent) {
        let event_type = event.event_type();
        let aggregate_id = event.aggregate_id();
        
        // 记录事件计数
        self.metrics
            .counter("domain_events_total")
            .with_label("type", event_type)
            .increment();
        
        // 记录事件处理时间
        let start_time = std::time::Instant::now();
        // 处理事件...
        let duration = start_time.elapsed();
        
        self.metrics
            .histogram("domain_event_duration_seconds")
            .with_label("type", event_type)
            .record(duration.as_secs_f64());
    }
}
```

### 2. 技术指标监控

#### 数据库查询监控
```rust
pub struct DatabaseMonitor {
    metrics: Arc<Metrics>,
}

impl DatabaseMonitor {
    pub async fn monitor_query<F, T>(&self, query_name: &str, query: F) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>>,
    {
        let start_time = std::time::Instant::now();
        
        let result = query.await;
        
        let duration = start_time.elapsed();
        
        // 记录查询时间
        self.metrics
            .histogram("database_query_duration_seconds")
            .with_label("query", query_name)
            .record(duration.as_secs_f64());
        
        // 记录查询计数
        self.metrics
            .counter("database_queries_total")
            .with_label("query", query_name)
            .increment();
        
        result
    }
}
```

## 🔄 持续集成策略

### 1. CI/CD 流程

#### GitHub Actions 工作流
```yaml
name: DDD + TDD CI/CD

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    
    services:
      postgres:
        image: postgres:13
        env:
          POSTGRES_PASSWORD: postgres
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
      redis:
        image: redis:6
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        override: true
    
    - name: Cache cargo registry
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Run tests
      run: |
        cargo xtask test
        cargo xtask test --quick
    
    - name: Check code formatting
      run: |
        cargo fmt --all -- --check
    
    - name: Run clippy
      run: |
        cargo clippy -- -W clippy::pedantic
    
    - name: Generate documentation
      run: |
        cargo doc --no-deps --features "with-db auth_jwt cli testing"
```

### 2. 质量门控

#### 测试覆盖率要求
```yaml
# .github/workflows/coverage.yml
- name: Run coverage
  run: |
    cargo install cargo-tarpaulin
    cargo tarpaulin --out Xml --target-dir target/coverage

- name: Upload coverage
  uses: codecov/codecov-action@v3
  with:
    file: ./target/coverage/cobertura.xml
    flags: unittests
    name: codecov-umbrella
    fail_ci_if_error: true
```

#### 性能测试
```yaml
# .github/workflows/performance.yml
- name: Run performance tests
  run: |
    cargo install cargo-criterion
    cargo criterion
```

---

*这份实施策略提供了在 Loco 框架中实施 DDD + TDD 的详细方案，包括架构设计、测试策略、事件驱动、CQRS 实现等关键方面。*
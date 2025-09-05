# DDD + TDD + Loco 完整实施指南

## 📋 概述

本指南提供了在 Loco 框架中实施领域驱动设计（DDD）和测试驱动开发（TDD）的完整方案。基于深度分析，我们提供了一个实用的、可操作的实施方案。

## 🎯 实施原则

### 核心原则
1. **领域优先**: 业务逻辑驱动技术决策
2. **测试驱动**: 测试作为设计和文档的工具
3. **持续重构**: 代码质量是持续改进的结果
4. **分层架构**: 清晰的职责分离和依赖管理
5. **事件驱动**: 松耦合的领域事件通信

### 实施优先级
1. **高优先级**: 核心领域、聚合根、领域服务
2. **中优先级**: 应用服务、基础设施、测试覆盖
3. **低优先级**: 性能优化、监控、高级特性

## 🏗️ 项目结构

### 目录结构
```
myapp/
├── src/
│   ├── ddd/                           # DDD 基础设施
│   │   ├── lib.rs                     # DDD 核心特质
│   │   ├── aggregate.rs               # 聚合基类
│   │   ├── entity.rs                  # 实体基类
│   │   ├── value_object.rs            # 值对象基类
│   │   ├── repository.rs              # 仓库接口
│   │   ├── service.rs                 # 服务基类
│   │   ├── event.rs                   # 事件系统
│   │   ├── command.rs                 # 命令系统
│   │   └── query.rs                   # 查询系统
│   ├── domains/                       # 领域层
│   │   ├── user/                      # 用户限界上下文
│   │   │   ├── mod.rs                 # 限界上下文入口
│   │   │   ├── entities/              # 实体
│   │   │   │   ├── mod.rs
│   │   │   │   └── user.rs
│   │   │   ├── value_objects/         # 值对象
│   │   │   │   ├── mod.rs
│   │   │   │   └── email.rs
│   │   │   ├── aggregates/             # 聚合
│   │   │   │   ├── mod.rs
│   │   │   │   └── user_aggregate.rs
│   │   │   ├── services/              # 领域服务
│   │   │   │   ├── mod.rs
│   │   │   │   └── user_service.rs
│   │   │   ├── events/                # 领域事件
│   │   │   │   ├── mod.rs
│   │   │   │   └── user_events.rs
│   │   │   └── repositories/          # 仓库接口
│   │   │       ├── mod.rs
│   │   │       └── user_repository.rs
│   │   └── order/                     # 订单限界上下文
│   │       ├── entities/
│   │       ├── value_objects/
│   │       ├── aggregates/
│   │       ├── services/
│   │       ├── events/
│   │       └── repositories/
│   ├── applications/                  # 应用层
│   │   ├── mod.rs                     # 应用层入口
│   │   ├── commands/                  # 命令
│   │   │   ├── mod.rs
│   │   │   ├── user_commands.rs
│   │   │   └── order_commands.rs
│   │   ├── queries/                   # 查询
│   │   │   ├── mod.rs
│   │   │   ├── user_queries.rs
│   │   │   └── order_queries.rs
│   │   ├── services/                  # 应用服务
│   │   │   ├── mod.rs
│   │   │   ├── user_application_service.rs
│   │   │   └── order_application_service.rs
│   │   └── dtos/                      # 数据传输对象
│   │       ├── mod.rs
│   │       ├── user_dto.rs
│   │       └── order_dto.rs
│   ├── infrastructure/                # 基础设施层
│   │   ├── mod.rs                     # 基础设施入口
│   │   ├── persistence/                # 持久化
│   │   │   ├── mod.rs
│   │   │   ├── user_repository_impl.rs
│   │   │   └── order_repository_impl.rs
│   │   ├── events/                    # 事件处理
│   │   │   ├── mod.rs
│   │   │   ├── event_store.rs
│   │   │   └── event_handlers.rs
│   │   ├── external/                  # 外部服务
│   │   │   ├── mod.rs
│   │   │   ├── email_service.rs
│   │   │   └── payment_service.rs
│   │   └── cache/                     # 缓存
│   │       ├── mod.rs
│   │       └── redis_cache.rs
│   └── interfaces/                    # 接口层
│       ├── mod.rs                     # 接口层入口
│       ├── controllers/               # 控制器
│       │   ├── mod.rs
│       │   ├── user_controller.rs
│       │   └── order_controller.rs
│       ├── routes/                    # 路由
│       │   ├── mod.rs
│       │   ├── user_routes.rs
│       │   └── order_routes.rs
│       └── middleware/                # 中间件
│           ├── mod.rs
│           ├── auth_middleware.rs
│           └── logging_middleware.rs
├── tests/                             # 测试
│   ├── unit/                          # 单元测试
│   │   ├── domains/                   # 领域层测试
│   │   ├── applications/              # 应用层测试
│   │   └── infrastructure/            # 基础设施层测试
│   ├── integration/                   # 集成测试
│   │   ├── controllers/               # 控制器测试
│   │   ├── repositories/              # 仓库测试
│   │   └── services/                  # 服务测试
│   └── e2e/                           # 端到端测试
│       ├── user_flows/                # 用户流程测试
│       └── business_scenarios/        # 业务场景测试
├── config/                            # 配置
│   ├── development.yaml               # 开发配置
│   ├── test.yaml                      # 测试配置
│   └── production.yaml                # 生产配置
├── migrations/                        # 数据库迁移
├── Cargo.toml                         # 项目配置
└── README.md                          # 项目说明
```

## 🚀 实施步骤

### 阶段 1：基础设施搭建

#### 1.1 创建 DDD 基础设施

**创建 DDD 核心特质**：
```rust
// src/ddd/lib.rs
pub mod aggregate;
pub mod entity;
pub mod value_object;
pub mod repository;
pub mod service;
pub mod event;
pub mod command;
pub mod query;

pub use aggregate::*;
pub use entity::*;
pub use value_object::*;
pub use repository::*;
pub use service::*;
pub use event::*;
pub use command::*;
pub use query::*;

// DDD 核心特质
pub trait AggregateRoot: Entity + Send + Sync {
    type Id: Send + Sync;
    type Event: DomainEvent;
    
    fn id(&self) -> &Self::Id;
    fn version(&self) -> u32;
    fn events(&self) -> Vec<Self::Event>;
    fn clear_events(&mut self);
}

pub trait Entity: Send + Sync {
    type Id: Send + Sync;
    
    fn id(&self) -> &Self::Id;
    fn equals(&self, other: &Self) -> bool;
}

pub trait ValueObject: Send + Sync + Clone {
    fn equals(&self, other: &Self) -> bool;
}

pub trait Repository<T: AggregateRoot>: Send + Sync {
    async fn save(&self, aggregate: &T) -> Result<()>;
    async fn find_by_id(&self, id: &T::Id) -> Result<Option<T>>;
    async fn delete(&self, id: &T::Id) -> Result<()>;
}

pub trait DomainEvent: Send + Sync + Clone {
    fn event_type(&self) -> &str;
    fn occurred_at(&self) -> DateTime<Utc>;
    fn aggregate_id(&self) -> &str;
}

pub trait EventHandler: Send + Sync {
    type Event: DomainEvent;
    
    async fn handle(&self, event: &Self::Event) -> Result<()>;
}
```

**创建事件系统**：
```rust
// src/ddd/event/mod.rs
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;

pub struct EventPublisher {
    subscribers: Vec<Box<dyn EventHandler<Event = dyn DomainEvent>>>,
}

impl EventPublisher {
    pub fn new() -> Self {
        Self {
            subscribers: Vec::new(),
        }
    }
    
    pub fn subscribe<E: DomainEvent + 'static>(&mut self, handler: impl EventHandler<Event = E> + 'static) {
        self.subscribers.push(Box::new(handler));
    }
    
    pub async fn publish(&self, event: impl DomainEvent + 'static) -> Result<()> {
        for subscriber in &self.subscribers {
            // 类型安全的订阅者调用
            // 这里需要实现类型擦除和动态分发
        }
        Ok(())
    }
}

// 全局事件发布器
lazy_static! {
    static ref EVENT_PUBLISHER: Arc<RwLock<EventPublisher>> = 
        Arc::new(RwLock::new(EventPublisher::new()));
}

pub async fn publish_event<E: DomainEvent + 'static>(event: E) -> Result<()> {
    let publisher = EVENT_PUBLISHER.read().await;
    publisher.publish(event).await
}

pub async fn subscribe_to_events<E: DomainEvent + 'static>(
    handler: impl EventHandler<Event = E> + 'static
) -> Result<()> {
    let mut publisher = EVENT_PUBLISHER.write().await;
    publisher.subscribe(handler);
    Ok(())
}
```

#### 1.2 配置项目

**更新 Cargo.toml**：
```toml
[dependencies]
# Loco 框架
loco-rs = "0.16"

# 异步运行时
tokio = { version = "1.0", features = ["full"] }

# 序列化
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# 错误处理
anyhow = "1.0"
thiserror = "1.0"

# 日期时间
chrono = { version = "0.4", features = ["serde"] }

# 验证
validator = { version = "0.16", features = ["derive"] }
regex = "1.0"

# 异步特征
async-trait = "0.1"

# 日志
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# 工具
lazy_static = "1.4"
uuid = { version = "1.0", features = ["v4", "serde"] }
strum = { version = "0.24", features = ["derive"] }

# 测试
mockall = "0.11"
tokio-test = "0.4"
```

**配置环境**：
```yaml
# config/development.yaml
application:
  host: 0.0.0.0
  port: 3000
  workers: 4

database:
  uri: "postgres://user:password@localhost:5432/myapp_development"
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
  level: debug
  format: pretty

ddd:
  event_store:
    driver: "database"  # database, redis, memory
  snapshots:
    enabled: true
    interval: 50
  domain_events:
    enabled: true
    async_handlers: true
```

### 阶段 2：核心领域开发

#### 2.1 识别限界上下文

**用户限界上下文**：
```rust
// src/domains/user/mod.rs
pub mod entities;
pub mod value_objects;
pub mod aggregates;
pub mod services;
pub mod events;
pub mod repositories;

pub use entities::*;
pub use value_objects::*;
pub use aggregates::*;
pub use services::*;
pub use events::*;
pub use repositories::*;

pub struct UserContext;

impl UserContext {
    pub fn new() -> Self {
        Self
    }
    
    pub fn get_name(&self) -> &'static str {
        "User"
    }
    
    pub fn get_description(&self) -> &'static str {
        "用户管理和认证上下文"
    }
}
```

**订单限界上下文**：
```rust
// src/domains/order/mod.rs
pub mod entities;
pub mod value_objects;
pub mod aggregates;
pub mod services;
pub mod events;
pub mod repositories;

pub use entities::*;
pub use value_objects::*;
pub use aggregates::*;
pub use services::*;
pub use events::*;
pub use repositories::*;

pub struct OrderContext;

impl OrderContext {
    pub fn new() -> Self {
        Self
    }
    
    pub fn get_name(&self) -> &'static str {
        "Order"
    }
    
    pub fn get_description(&self) -> &'static str {
        "订单管理和处理上下文"
    }
}
```

#### 2.2 实现值对象

**Email 值对象**：
```rust
// src/domains/user/value_objects/email.rs
use serde::{Deserialize, Serialize};
use validator::Validate;
use crate::ddd::value_object::ValueObject;
use anyhow::{Result, anyhow};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Validate)]
pub struct Email {
    #[validate(email)]
    value: String,
}

impl Email {
    pub fn new(value: String) -> Result<Self> {
        let email = Email { value };
        email.validate()?;
        Ok(email)
    }
    
    pub fn value(&self) -> &str {
        &self.value
    }
    
    pub fn domain(&self) -> Option<&str> {
        self.value.split('@').nth(1)
    }
    
    pub fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }
}

impl ValueObject for Email {
    fn equals(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Validate for Email {
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        use validator::Validate;
        self.validate()
    }
}

impl std::fmt::Display for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl std::str::FromStr for Email {
    type Err = anyhow::Error;
    
    fn from_str(s: &str) -> Result<Self> {
        Email::new(s.to_string())
    }
}
```

**金额值对象**：
```rust
// src/domains/order/value_objects/money.rs
use serde::{Deserialize, Serialize};
use crate::ddd::value_object::ValueObject;
use anyhow::{Result, anyhow};
use std::cmp::Ordering;
use std::ops::{Add, Sub, Mul};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Money {
    amount: i64, // 以分为单位存储
    currency: String,
}

impl Money {
    pub fn new(amount: i64, currency: String) -> Result<Self> {
        if amount < 0 {
            return Err(anyhow!("金额不能为负数"));
        }
        
        if currency.is_empty() {
            return Err(anyhow!("货币不能为空"));
        }
        
        Ok(Money { amount, currency })
    }
    
    pub fn zero(currency: String) -> Result<Self> {
        Money::new(0, currency)
    }
    
    pub fn from_dollars(amount: f64, currency: String) -> Result<Self> {
        let cents = (amount * 100.0) as i64;
        Money::new(cents, currency)
    }
    
    pub fn amount(&self) -> i64 {
        self.amount
    }
    
    pub fn currency(&self) -> &str {
        &self.currency
    }
    
    pub fn to_dollars(&self) -> f64 {
        self.amount as f64 / 100.0
    }
    
    pub fn is_zero(&self) -> bool {
        self.amount == 0
    }
    
    pub fn is_positive(&self) -> bool {
        self.amount > 0
    }
    
    pub fn is_negative(&self) -> bool {
        self.amount < 0
    }
}

impl ValueObject for Money {
    fn equals(&self, other: &Self) -> bool {
        self.amount == other.amount && self.currency == other.currency
    }
}

impl PartialOrd for Money {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.currency != other.currency {
            return None;
        }
        self.amount.partial_cmp(&other.amount)
    }
}

impl Add for Money {
    type Output = Result<Self>;
    
    fn add(self, other: Self) -> Result<Self> {
        if self.currency != other.currency {
            return Err(anyhow!("不能添加不同货币的金额"));
        }
        Money::new(self.amount + other.amount, self.currency)
    }
}

impl Sub for Money {
    type Output = Result<Self>;
    
    fn sub(self, other: Self) -> Result<Self> {
        if self.currency != other.currency {
            return Err(anyhow!("不能减去不同货币的金额"));
        }
        Money::new(self.amount - other.amount, self.currency)
    }
}

impl Mul<i64> for Money {
    type Output = Result<Self>;
    
    fn mul(self, multiplier: i64) -> Result<Self> {
        Money::new(self.amount * multiplier, self.currency)
    }
}

impl std::fmt::Display for Money {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2} {}", self.to_dollars(), self.currency)
    }
}
```

#### 2.3 实现实体

**用户实体**：
```rust
// src/domains/user/entities/user.rs
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::ddd::entity::Entity;
use super::{Email, UserId};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    id: UserId,
    email: Email,
    name: String,
    password_hash: String,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(
        email: Email,
        name: String,
        password_hash: String,
    ) -> Result<Self> {
        let now = Utc::now();
        
        Ok(User {
            id: UserId::new(),
            email,
            name,
            password_hash,
            is_active: true,
            created_at: now,
            updated_at: now,
        })
    }
    
    pub fn id(&self) -> &UserId {
        &self.id
    }
    
    pub fn email(&self) -> &Email {
        &self.email
    }
    
    pub fn name(&self) -> &str {
        &self.name
    }
    
    pub fn is_active(&self) -> bool {
        self.is_active
    }
    
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
    
    pub fn change_name(&mut self, new_name: String) -> Result<()> {
        if new_name.trim().is_empty() {
            return Err(anyhow!("用户名不能为空"));
        }
        
        self.name = new_name;
        self.updated_at = Utc::now();
        Ok(())
    }
    
    pub fn change_email(&mut self, new_email: Email) -> Result<()> {
        self.email = new_email;
        self.updated_at = Utc::now();
        Ok(())
    }
    
    pub fn activate(&mut self) {
        self.is_active = true;
        self.updated_at = Utc::now();
    }
    
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }
    
    pub fn change_password(&mut self, new_password_hash: String) -> Result<()> {
        if new_password_hash.is_empty() {
            return Err(anyhow!("密码哈希不能为空"));
        }
        
        self.password_hash = new_password_hash;
        self.updated_at = Utc::now();
        Ok(())
    }
    
    pub fn validate_password(&self, password: &str, hasher: &dyn PasswordHasher) -> Result<bool> {
        hasher.verify(password, &self.password_hash)
    }
}

impl Entity for User {
    type Id = UserId;
    
    fn id(&self) -> &Self::Id {
        &self.id
    }
    
    fn equals(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

// 用户ID值对象
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(Uuid);

impl UserId {
    pub fn new() -> Self {
        UserId(Uuid::new_v4())
    }
    
    pub fn from_uuid(uuid: Uuid) -> Self {
        UserId(uuid)
    }
    
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
    
    pub fn as_string(&self) -> String {
        self.0.to_string()
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// 密码哈希器特质
pub trait PasswordHasher: Send + Sync {
    fn hash(&self, password: &str) -> Result<String>;
    fn verify(&self, password: &str, hash: &str) -> Result<bool>;
}
```

#### 2.4 实现聚合根

**用户聚合根**：
```rust
// src/domains/user/aggregates/user_aggregate.rs
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::ddd::aggregate::AggregateRoot;
use super::{User, UserRegistered, UserUpdated, UserDeleted};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAggregate {
    user: User,
    version: u32,
    events: Vec<UserEvent>,
}

impl UserAggregate {
    pub fn new(user: User) -> Self {
        let mut aggregate = UserAggregate {
            user,
            version: 1,
            events: Vec::new(),
        };
        
        // 发布用户注册事件
        let event = UserRegistered {
            user_id: aggregate.user.id().clone(),
            email: aggregate.user.email().clone(),
            name: aggregate.user.name().to_string(),
            occurred_at: Utc::now(),
        };
        aggregate.events.push(UserEvent::Registered(event));
        
        aggregate
    }
    
    pub fn user(&self) -> &User {
        &self.user
    }
    
    pub fn change_name(&mut self, new_name: String) -> Result<()> {
        self.user.change_name(new_name)?;
        
        // 发布用户更新事件
        let event = UserUpdated {
            user_id: self.user.id().clone(),
            field: "name".to_string(),
            occurred_at: Utc::now(),
        };
        self.events.push(UserEvent::Updated(event));
        
        self.version += 1;
        Ok(())
    }
    
    pub fn change_email(&mut self, new_email: super::Email) -> Result<()> {
        self.user.change_email(new_email)?;
        
        // 发布用户更新事件
        let event = UserUpdated {
            user_id: self.user.id().clone(),
            field: "email".to_string(),
            occurred_at: Utc::now(),
        };
        self.events.push(UserEvent::Updated(event));
        
        self.version += 1;
        Ok(())
    }
    
    pub fn deactivate(&mut self) -> Result<()> {
        self.user.deactivate();
        
        // 发布用户删除事件
        let event = UserDeleted {
            user_id: self.user.id().clone(),
            occurred_at: Utc::now(),
        };
        self.events.push(UserEvent::Deleted(event));
        
        self.version += 1;
        Ok(())
    }
    
    pub fn activate(&mut self) -> Result<()> {
        self.user.activate();
        
        // 发布用户更新事件
        let event = UserUpdated {
            user_id: self.user.id().clone(),
            field: "status".to_string(),
            occurred_at: Utc::now(),
        };
        self.events.push(UserEvent::Updated(event));
        
        self.version += 1;
        Ok(())
    }
    
    pub fn change_password(&mut self, new_password_hash: String) -> Result<()> {
        self.user.change_password(new_password_hash)?;
        
        // 发布用户更新事件
        let event = UserUpdated {
            user_id: self.user.id().clone(),
            field: "password".to_string(),
            occurred_at: Utc::now(),
        };
        self.events.push(UserEvent::Updated(event));
        
        self.version += 1;
        Ok(())
    }
}

impl AggregateRoot for UserAggregate {
    type Id = super::UserId;
    type Event = UserEvent;
    
    fn id(&self) -> &Self::Id {
        self.user.id()
    }
    
    fn version(&self) -> u32 {
        self.version
    }
    
    fn events(&self) -> Vec<Self::Event> {
        self.events.clone()
    }
    
    fn clear_events(&mut self) {
        self.events.clear();
    }
}

impl Entity for UserAggregate {
    type Id = super::UserId;
    
    fn id(&self) -> &Self::Id {
        self.user.id()
    }
    
    fn equals(&self, other: &Self) -> bool {
        self.user.id() == other.user.id()
    }
}

// 用户事件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserEvent {
    Registered(UserRegistered),
    Updated(UserUpdated),
    Deleted(UserDeleted),
}

impl crate::ddd::event::DomainEvent for UserEvent {
    fn event_type(&self) -> &str {
        match self {
            UserEvent::Registered(_) => "user.registered",
            UserEvent::Updated(_) => "user.updated",
            UserEvent::Deleted(_) => "user.deleted",
        }
    }
    
    fn occurred_at(&self) -> DateTime<Utc> {
        match self {
            UserEvent::Registered(e) => e.occurred_at,
            UserEvent::Updated(e) => e.occurred_at,
            UserEvent::Deleted(e) => e.occurred_at,
        }
    }
    
    fn aggregate_id(&self) -> &str {
        match self {
            UserEvent::Registered(e) => &e.user_id.to_string(),
            UserEvent::Updated(e) => &e.user_id.to_string(),
            UserEvent::Deleted(e) => &e.user_id.to_string(),
        }
    }
}

// 用户事件定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRegistered {
    pub user_id: super::UserId,
    pub email: super::Email,
    pub name: String,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserUpdated {
    pub user_id: super::UserId,
    pub field: String,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDeleted {
    pub user_id: super::UserId,
    pub occurred_at: DateTime<Utc>,
}
```

### 阶段 3：应用层开发

#### 3.1 创建应用服务

**用户应用服务**：
```rust
// src/applications/services/user_application_service.rs
use crate::ddd::repository::Repository;
use crate::domains::user::aggregates::UserAggregate;
use crate::domains::user::repositories::UserRepository;
use crate::domains::user::value_objects::Email;
use crate::applications::commands::CreateUserCommand;
use crate::applications::commands::UpdateUserCommand;
use crate::applications::dtos::UserDto;
use crate::ddd::event::EventPublisher;
use anyhow::Result;

pub struct UserApplicationService {
    user_repository: Box<dyn Repository<UserAggregate>>,
    event_publisher: Box<dyn EventPublisher>,
}

impl UserApplicationService {
    pub fn new(
        user_repository: Box<dyn Repository<UserAggregate>>,
        event_publisher: Box<dyn EventPublisher>,
    ) -> Self {
        Self {
            user_repository,
            event_publisher,
        }
    }
    
    pub async fn create_user(&self, command: CreateUserCommand) -> Result<UserDto> {
        // 创建值对象
        let email = Email::new(command.email)?;
        
        // 创建实体
        let user = User::new(
            email,
            command.name,
            command.password_hash,
        )?;
        
        // 创建聚合
        let mut user_aggregate = UserAggregate::new(user);
        
        // 保存聚合
        self.user_repository.save(&user_aggregate).await?;
        
        // 发布领域事件
        for event in user_aggregate.events() {
            self.event_publisher.publish(event).await?;
        }
        
        user_aggregate.clear_events();
        
        // 返回DTO
        Ok(UserDto::from_aggregate(&user_aggregate))
    }
    
    pub async fn update_user(&self, command: UpdateUserCommand) -> Result<UserDto> {
        // 查找用户
        let mut user_aggregate = self.user_repository
            .find_by_id(&command.user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("用户不存在"))?;
        
        // 更新用户
        if let Some(name) = command.name {
            user_aggregate.change_name(name)?;
        }
        
        if let Some(email) = command.email {
            let email = Email::new(email)?;
            user_aggregate.change_email(email)?;
        }
        
        // 保存聚合
        self.user_repository.save(&user_aggregate).await?;
        
        // 发布领域事件
        for event in user_aggregate.events() {
            self.event_publisher.publish(event).await?;
        }
        
        user_aggregate.clear_events();
        
        // 返回DTO
        Ok(UserDto::from_aggregate(&user_aggregate))
    }
    
    pub async fn get_user(&self, user_id: &crate::domains::user::entities::UserId) -> Result<UserDto> {
        let user_aggregate = self.user_repository
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("用户不存在"))?;
        
        Ok(UserDto::from_aggregate(&user_aggregate))
    }
    
    pub async fn deactivate_user(&self, user_id: &crate::domains::user::entities::UserId) -> Result<()> {
        let mut user_aggregate = self.user_repository
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("用户不存在"))?;
        
        user_aggregate.deactivate()?;
        
        // 保存聚合
        self.user_repository.save(&user_aggregate).await?;
        
        // 发布领域事件
        for event in user_aggregate.events() {
            self.event_publisher.publish(event).await?;
        }
        
        user_aggregate.clear_events();
        
        Ok(())
    }
}
```

#### 3.2 创建命令和查询

**用户命令**：
```rust
// src/applications/commands/user_commands.rs
use serde::{Deserialize, Serialize};
use validator::Validate;
use crate::domains::user::entities::UserId;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateUserCommand {
    #[validate(length(min = 2, max = 50))]
    pub name: String,
    
    #[validate(email)]
    pub email: String,
    
    #[validate(length(min = 8))]
    pub password_hash: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateUserCommand {
    pub user_id: UserId,
    
    #[validate(length(min = 2, max = 50))]
    pub name: Option<String>,
    
    #[validate(email)]
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeactivateUserCommand {
    pub user_id: UserId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivateUserCommand {
    pub user_id: UserId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChangePasswordCommand {
    pub user_id: UserId,
    #[validate(length(min = 8))]
    pub new_password_hash: String,
}
```

**用户查询**：
```rust
// src/applications/queries/user_queries.rs
use serde::{Deserialize, Serialize};
use crate::domains::user::entities::UserId;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetUserQuery {
    pub user_id: UserId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListUsersQuery {
    pub page: u32,
    pub page_size: u32,
    pub active_only: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchUsersQuery {
    pub search_term: String,
    pub page: u32,
    pub page_size: u32,
}
```

#### 3.3 创建DTO

**用户DTO**：
```rust
// src/applications/dtos/user_dto.rs
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::domains::user::aggregates::UserAggregate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDto {
    pub id: String,
    pub email: String,
    pub name: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl UserDto {
    pub fn from_aggregate(aggregate: &UserAggregate) -> Self {
        Self {
            id: aggregate.user().id().to_string(),
            email: aggregate.user().email().to_string(),
            name: aggregate.user().name().to_string(),
            is_active: aggregate.user().is_active(),
            created_at: aggregate.user().created_at(),
            updated_at: aggregate.user().updated_at(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserListDto {
    pub users: Vec<UserDto>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSummaryDto {
    pub id: String,
    pub name: String,
    pub email: String,
    pub is_active: bool,
}
```

### 阶段 4：基础设施层开发

#### 4.1 实现仓库模式

**用户仓库实现**：
```rust
// src/infrastructure/persistence/user_repository_impl.rs
use std::sync::Arc;
use async_trait::async_trait;
use sea_orm::*;
use crate::ddd::repository::Repository;
use crate::domains::user::aggregates::UserAggregate;
use crate::domains::user::entities::UserId;
use crate::infrastructure::persistence::models::user_model;
use anyhow::Result;

pub struct UserRepositoryImpl {
    db: Arc<DatabaseConnection>,
}

impl UserRepositoryImpl {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
    
    async fn map_to_aggregate(&self, model: user_model::Model) -> Result<UserAggregate> {
        // 从数据库模型映射到领域聚合
        // 这里需要实现具体的映射逻辑
        todo!("实现从数据库模型到领域聚合的映射")
    }
    
    async fn map_to_model(&self, aggregate: &UserAggregate) -> Result<user_model::ActiveModel> {
        // 从领域聚合映射到数据库模型
        // 这里需要实现具体的映射逻辑
        todo!("实现从领域聚合到数据库模型的映射")
    }
}

#[async_trait]
impl Repository<UserAggregate> for UserRepositoryImpl {
    async fn save(&self, aggregate: &UserAggregate) -> Result<()> {
        let active_model = self.map_to_model(aggregate).await?;
        
        user_model::Entity::insert(active_model)
            .exec(self.db.as_ref())
            .await?;
        
        Ok(())
    }
    
    async fn find_by_id(&self, id: &UserId) -> Result<Option<UserAggregate>> {
        let uuid = uuid::Uuid::parse_str(&id.to_string())?;
        
        let model = user_model::Entity::find_by_id(uuid)
            .one(self.db.as_ref())
            .await?;
        
        match model {
            Some(model) => {
                let aggregate = self.map_to_aggregate(model).await?;
                Ok(Some(aggregate))
            }
            None => Ok(None),
        }
    }
    
    async fn delete(&self, id: &UserId) -> Result<()> {
        let uuid = uuid::Uuid::parse_str(&id.to_string())?;
        
        user_model::Entity::delete_by_id(uuid)
            .exec(self.db.as_ref())
            .await?;
        
        Ok(())
    }
}
```

#### 4.2 实现事件处理

**事件处理器**：
```rust
// src/infrastructure/events/event_handlers.rs
use async_trait::async_trait;
use crate::ddd::event::{DomainEvent, EventHandler};
use crate::domains::user::aggregates::{UserEvent, UserRegistered};
use crate::infrastructure::external::email_service::EmailService;
use anyhow::Result;

pub struct UserEventHandler {
    email_service: Arc<EmailService>,
}

impl UserEventHandler {
    pub fn new(email_service: Arc<EmailService>) -> Self {
        Self { email_service }
    }
}

#[async_trait]
impl EventHandler for UserEventHandler {
    type Event = UserEvent;
    
    async fn handle(&self, event: &Self::Event) -> Result<()> {
        match event {
            UserEvent::Registered(event) => {
                self.handle_user_registered(event).await
            }
            UserEvent::Updated(event) => {
                self.handle_user_updated(event).await
            }
            UserEvent::Deleted(event) => {
                self.handle_user_deleted(event).await
            }
        }
    }
}

impl UserEventHandler {
    async fn handle_user_registered(&self, event: &UserRegistered) -> Result<()> {
        // 发送欢迎邮件
        self.email_service.send_welcome_email(
            &event.email.to_string(),
            &event.name,
        ).await?;
        
        // 记录日志
        tracing::info!(
            user_id = %event.user_id,
            email = %event.email,
            "用户注册事件处理完成"
        );
        
        Ok(())
    }
    
    async fn handle_user_updated(&self, event: &crate::domains::user::aggregates::UserUpdated) -> Result<()> {
        // 处理用户更新事件
        tracing::info!(
            user_id = %event.user_id,
            field = %event.field,
            "用户更新事件处理完成"
        );
        
        Ok(())
    }
    
    async fn handle_user_deleted(&self, event: &crate::domains::user::aggregates::UserDeleted) -> Result<()> {
        // 处理用户删除事件
        tracing::info!(
            user_id = %event.user_id,
            "用户删除事件处理完成"
        );
        
        Ok(())
    }
}
```

### 阶段 5：接口层开发

#### 5.1 创建控制器

**用户控制器**：
```rust
// src/interfaces/controllers/user_controller.rs
use axum::extract::{Path, State, Query};
use axum::response::Json;
use loco_rs::prelude::*;
use crate::applications::commands::*;
use crate::applications::queries::*;
use crate::applications::services::UserApplicationService;
use crate::applications::dtos::{UserDto, UserListDto};
use crate::interfaces::routes::user_routes::UserParams;
use anyhow::Result;

pub struct UserController;

impl UserController {
    pub async fn create_user(
        State(ctx): State<AppContext>,
        Json(command): Json<CreateUserCommand>,
    ) -> Result<Json<UserDto>> {
        let user_service = ctx.services.user_service();
        let user = user_service.create_user(command).await?;
        
        Ok(Json(user))
    }
    
    pub async fn get_user(
        Path(user_id): Path<String>,
        State(ctx): State<AppContext>,
    ) -> Result<Json<UserDto>> {
        let user_id = crate::domains::user::entities::UserId::from_uuid(
            uuid::Uuid::parse_str(&user_id)?
        );
        
        let user_service = ctx.services.user_service();
        let user = user_service.get_user(&user_id).await?;
        
        Ok(Json(user))
    }
    
    pub async fn update_user(
        Path(user_id): Path<String>,
        State(ctx): State<AppContext>,
        Json(command): Json<UpdateUserCommand>,
    ) -> Result<Json<UserDto>> {
        let user_id = crate::domains::user::entities::UserId::from_uuid(
            uuid::Uuid::parse_str(&user_id)?
        );
        
        let mut command = command;
        command.user_id = user_id;
        
        let user_service = ctx.services.user_service();
        let user = user_service.update_user(command).await?;
        
        Ok(Json(user))
    }
    
    pub async fn list_users(
        Query(params): Query<UserParams>,
        State(ctx): State<AppContext>,
    ) -> Result<Json<UserListDto>> {
        let query = ListUsersQuery {
            page: params.page.unwrap_or(1),
            page_size: params.page_size.unwrap_or(10),
            active_only: params.active_only.unwrap_or(true),
        };
        
        let user_service = ctx.services.user_service();
        let users = user_service.list_users(query).await?;
        
        Ok(Json(users))
    }
    
    pub async fn deactivate_user(
        Path(user_id): Path<String>,
        State(ctx): State<AppContext>,
    ) -> Result<()> {
        let user_id = crate::domains::user::entities::UserId::from_uuid(
            uuid::Uuid::parse_str(&user_id)?
        );
        
        let user_service = ctx.services.user_service();
        user_service.deactivate_user(&user_id).await?;
        
        Ok(())
    }
}
```

#### 5.2 创建路由

**用户路由**：
```rust
// src/interfaces/routes/user_routes.rs
use axum::routing::{get, post, put, delete};
use super::user_controller::UserController;

pub struct UserRoutes;

impl UserRoutes {
    pub fn new() -> axum::Router<AppContext> {
        axum::Router::new()
            .route("/", post(UserController::create_user))
            .route("/", get(UserController::list_users))
            .route("/:id", get(UserController::get_user))
            .route("/:id", put(UserController::update_user))
            .route("/:id/deactivate", post(UserController::deactivate_user))
    }
}

#[derive(serde::Deserialize)]
pub struct UserParams {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub active_only: Option<bool>,
}
```

### 阶段 6：测试实施

#### 6.1 单元测试

**值对象测试**：
```rust
// tests/unit/domains/user/value_objects/email_test.rs
use super::super::super::domains::user::value_objects::Email;

#[tokio::test]
async fn test_email_creation_valid() {
    let email = Email::new("test@example.com".to_string()).unwrap();
    assert_eq!(email.value(), "test@example.com");
    assert!(email.is_valid());
}

#[tokio::test]
async fn test_email_creation_invalid() {
    let result = Email::new("invalid-email".to_string());
    assert!(result.is_err());
}

#[tokio::test]
async fn test_email_equality() {
    let email1 = Email::new("test@example.com".to_string()).unwrap();
    let email2 = Email::new("test@example.com".to_string()).unwrap();
    let email3 = Email::new("other@example.com".to_string()).unwrap();
    
    assert!(email1.equals(&email2));
    assert!(!email1.equals(&email3));
}

#[tokio::test]
async fn test_email_domain() {
    let email = Email::new("test@example.com".to_string()).unwrap();
    assert_eq!(email.domain(), Some("example.com"));
}
```

**聚合测试**：
```rust
// tests/unit/domains/user/aggregates/user_aggregate_test.rs
use super::super::super::domains::user::aggregates::UserAggregate;
use super::super::super::domains::user::entities::User;
use super::super::super::domains::user::value_objects::Email;

#[tokio::test]
async fn test_user_aggregate_creation() {
    let email = Email::new("test@example.com".to_string()).unwrap();
    let user = User::new(
        email,
        "Test User".to_string(),
        "hashed_password".to_string(),
    ).unwrap();
    
    let aggregate = UserAggregate::new(user);
    
    assert_eq!(aggregate.user().name(), "Test User");
    assert_eq!(aggregate.version(), 1);
    assert_eq!(aggregate.events().len(), 1);
}

#[tokio::test]
async fn test_user_aggregate_name_change() {
    let email = Email::new("test@example.com".to_string()).unwrap();
    let user = User::new(
        email,
        "Test User".to_string(),
        "hashed_password".to_string(),
    ).unwrap();
    
    let mut aggregate = UserAggregate::new(user);
    
    // 清除初始事件
    aggregate.clear_events();
    
    // 更新姓名
    aggregate.change_name("Updated Name".to_string()).unwrap();
    
    assert_eq!(aggregate.user().name(), "Updated Name");
    assert_eq!(aggregate.version(), 2);
    assert_eq!(aggregate.events().len(), 1);
}
```

#### 6.2 集成测试

**仓库测试**：
```rust
// tests/integration/infrastructure/persistence/user_repository_test.rs
use super::super::super::infrastructure::persistence::UserRepositoryImpl;
use super::super::super::domains::user::aggregates::UserAggregate;
use super::super::super::domains::user::entities::User;
use super::super::super::domains::user::value_objects::Email;

#[tokio::test]
async fn test_user_repository_save_and_find() {
    // 设置测试数据库
    let db = setup_test_db().await;
    let repository = UserRepositoryImpl::new(Arc::new(db));
    
    // 创建用户聚合
    let email = Email::new("test@example.com".to_string()).unwrap();
    let user = User::new(
        email,
        "Test User".to_string(),
        "hashed_password".to_string(),
    ).unwrap();
    
    let mut aggregate = UserAggregate::new(user);
    aggregate.clear_events(); // 清除初始事件
    
    // 保存用户
    repository.save(&aggregate).await.unwrap();
    
    // 查找用户
    let found_user = repository.find_by_id(aggregate.id()).await.unwrap();
    assert!(found_user.is_some());
    assert_eq!(found_user.unwrap().user().name(), "Test User");
}

#[tokio::test]
async fn test_user_repository_delete() {
    // 设置测试数据库
    let db = setup_test_db().await;
    let repository = UserRepositoryImpl::new(Arc::new(db));
    
    // 创建并保存用户
    let email = Email::new("test@example.com".to_string()).unwrap();
    let user = User::new(
        email,
        "Test User".to_string(),
        "hashed_password".to_string(),
    ).unwrap();
    
    let mut aggregate = UserAggregate::new(user);
    aggregate.clear_events();
    
    repository.save(&aggregate).await.unwrap();
    
    // 删除用户
    repository.delete(aggregate.id()).await.unwrap();
    
    // 验证用户已删除
    let found_user = repository.find_by_id(aggregate.id()).await.unwrap();
    assert!(found_user.is_none());
}

async fn setup_test_db() -> DatabaseConnection {
    // 设置测试数据库连接
    // 这里需要实现具体的测试数据库设置逻辑
    todo!("实现测试数据库设置")
}
```

#### 6.3 端到端测试

**用户注册流程测试**：
```rust
// tests/e2e/user_flows/registration_test.rs
use axum_test::TestServer;
use serde_json::json;

#[tokio::test]
async fn test_user_registration_flow() {
    // 创建测试服务器
    let app = create_test_app().await;
    let server = TestServer::new(app).unwrap();
    
    // 测试用户注册
    let response = server
        .post("/api/users")
        .json(&json!({
            "name": "Test User",
            "email": "test@example.com",
            "password_hash": "hashed_password_123"
        }))
        .expect_json::<serde_json::Value>()
        .await;
    
    // 验证响应
    assert_eq!(response["name"], "Test User");
    assert_eq!(response["email"], "test@example.com");
    assert!(response["is_active"].as_bool().unwrap());
    
    // 验证用户可以获取
    let user_id = response["id"].as_str().unwrap();
    let get_response = server
        .get(&format!("/api/users/{}", user_id))
        .expect_json::<serde_json::Value>()
        .await;
    
    assert_eq!(get_response["id"], user_id);
    assert_eq!(get_response["name"], "Test User");
}

#[tokio::test]
async fn test_user_update_flow() {
    // 创建测试服务器
    let app = create_test_app().await;
    let server = TestServer::new(app).unwrap();
    
    // 先创建用户
    let create_response = server
        .post("/api/users")
        .json(&json!({
            "name": "Test User",
            "email": "test@example.com",
            "password_hash": "hashed_password_123"
        }))
        .expect_json::<serde_json::Value>()
        .await;
    
    let user_id = create_response["id"].as_str().unwrap();
    
    // 更新用户
    let update_response = server
        .put(&format!("/api/users/{}", user_id))
        .json(&json!({
            "name": "Updated User"
        }))
        .expect_json::<serde_json::Value>()
        .await;
    
    assert_eq!(update_response["name"], "Updated User");
    
    // 验证更新
    let get_response = server
        .get(&format!("/api/users/{}", user_id))
        .expect_json::<serde_json::Value>()
        .await;
    
    assert_eq!(get_response["name"], "Updated User");
}

async fn create_test_app() -> axum::Router<AppContext> {
    // 创建测试应用
    // 这里需要实现具体的测试应用创建逻辑
    todo!("实现测试应用创建")
}
```

### 阶段 7：部署和监控

#### 7.1 部署配置

**Docker 配置**：
```dockerfile
# Dockerfile
FROM rust:1.70-slim AS builder

WORKDIR /app

# 安装依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# 复制 Cargo 文件
COPY Cargo.toml Cargo.lock ./

# 创建虚拟项目以缓存依赖
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

# 删除虚拟项目
RUN rm -rf src

# 复制源代码
COPY src ./src
COPY migrations ./migrations

# 构建应用
RUN cargo build --release

# 运行时镜像
FROM debian:bullseye-slim

WORKDIR /app

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# 从构建器复制二进制文件
COPY --from=builder /app/target/release/myapp ./myapp
COPY --from=builder /app/migrations ./migrations
COPY config ./config

# 创建非 root 用户
RUN useradd -m -u 1000 appuser
RUN chown -R appuser:appuser /app
USER appuser

# 暴露端口
EXPOSE 3000

# 健康检查
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# 启动应用
CMD ["./myapp", "start"]
```

**Docker Compose 配置**：
```yaml
# docker-compose.yml
version: '3.8'

services:
  app:
    build: .
    ports:
      - "3000:3000"
    environment:
      - RUST_ENV=production
      - DATABASE_URL=postgres://user:password@postgres:5432/myapp_production
      - REDIS_URL=redis://redis:6379/0
      - JWT_SECRET=your-jwt-secret-key
    depends_on:
      - postgres
      - redis
    volumes:
      - ./logs:/app/logs
    restart: unless-stopped

  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: myapp_production
      POSTGRES_USER: user
      POSTGRES_PASSWORD: password
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./init.sql:/docker-entrypoint-initdb.d/init.sql
    ports:
      - "5432:5432"
    restart: unless-stopped

  redis:
    image: redis:7-alpine
    volumes:
      - redis_data:/data
    ports:
      - "6379:6379"
    restart: unless-stopped

  nginx:
    image: nginx:alpine
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
      - ./ssl:/etc/nginx/ssl
    ports:
      - "80:80"
      - "443:443"
    depends_on:
      - app
    restart: unless-stopped

volumes:
  postgres_data:
  redis_data:
```

#### 7.2 监控配置

**健康检查端点**：
```rust
// src/interfaces/controllers/health_controller.rs
use axum::extract::State;
use axum::response::Json;
use loco_rs::prelude::*;
use serde_json::json;
use std::collections::HashMap;

pub struct HealthController;

impl HealthController {
    pub async fn check(State(ctx): State<AppContext>) -> Result<Json<serde_json::Value>> {
        let mut checks = HashMap::new();
        
        // 数据库检查
        let db_status = match check_database(&ctx).await {
            Ok(_) => "ok",
            Err(e) => {
                tracing::error!("数据库健康检查失败: {}", e);
                "error"
            }
        };
        checks.insert("database".to_string(), db_status.to_string());
        
        // 缓存检查
        let cache_status = match check_cache(&ctx).await {
            Ok(_) => "ok",
            Err(e) => {
                tracing::error!("缓存健康检查失败: {}", e);
                "error"
            }
        };
        checks.insert("cache".to_string(), cache_status.to_string());
        
        // 内存使用检查
        let memory_status = check_memory_usage();
        checks.insert("memory".to_string(), memory_status);
        
        // 整体状态
        let overall_status = if checks.values().all(|status| status == "ok") {
            "ok"
        } else {
            "error"
        };
        
        Ok(Json(json!({
            "status": overall_status,
            "checks": checks,
            "timestamp": chrono::Utc::now(),
            "version": env!("CARGO_PKG_VERSION")
        })))
    }
}

async fn check_database(ctx: &AppContext) -> Result<()> {
    // 执行简单的数据库查询
    let _result = sqlx::query("SELECT 1")
        .fetch_one(&ctx.db)
        .await?;
    
    Ok(())
}

async fn check_cache(ctx: &AppContext) -> Result<()> {
    // 执行简单的缓存操作
    ctx.cache.set("health_check", "ok", None).await?;
    
    Ok(())
}

fn check_memory_usage() -> String {
    // 检查内存使用情况
    let usage = psutil::memory::virtual_memory().unwrap();
    
    if usage.percent() > 90.0 {
        "critical"
    } else if usage.percent() > 70.0 {
        "warning"
    } else {
        "ok"
    }.to_string()
}
```

**Prometheus 监控**：
```rust
// src/interfaces/controllers/metrics_controller.rs
use axum::extract::State;
use axum::response::Json;
use loco_rs::prelude::*;
use prometheus::{Counter, Histogram, Gauge};
use std::sync::Arc;

pub struct MetricsController {
    request_count: Counter,
    request_duration: Histogram,
    active_connections: Gauge,
}

impl MetricsController {
    pub fn new() -> Self {
        Self {
            request_count: Counter::new(
                "http_requests_total",
                "Total number of HTTP requests"
            ).unwrap(),
            request_duration: Histogram::new(
                "http_request_duration_seconds",
                "HTTP request duration in seconds"
            ).unwrap(),
            active_connections: Gauge::new(
                "active_connections",
                "Number of active connections"
            ).unwrap(),
        }
    }
    
    pub async fn metrics() -> Result<String> {
        use prometheus::Encoder;
        
        let encoder = prometheus::TextEncoder::new();
        let metric_families = prometheus::gather();
        let mut buffer = Vec::new();
        
        encoder.encode(&metric_families, &mut buffer)?;
        
        Ok(String::from_utf8(buffer)?)
    }
    
    pub fn inc_request_count(&self, method: &str, endpoint: &str) {
        self.request_count
            .with_label_values(&[method, endpoint])
            .inc();
    }
    
    pub fn observe_request_duration(&self, method: &str, endpoint: &str, duration: f64) {
        self.request_duration
            .with_label_values(&[method, endpoint])
            .observe(duration);
    }
    
    pub fn set_active_connections(&self, count: i64) {
        self.active_connections.set(count as f64);
    }
}
```

## 🎯 最佳实践

### 1. 开发流程

#### TDD 开发流程
1. **Red**: 编写失败的测试
2. **Green**: 编写最少的代码使测试通过
3. **Refactor**: 重构代码，保持测试通过

#### 代码审查清单
- [ ] 所有测试通过
- [ ] 代码符合项目规范
- [ ] DDD 概念正确应用
- [ ] 错误处理完善
- [ ] 性能考虑充分
- [ ] 安全性检查通过

### 2. 性能优化

#### 数据库优化
- 使用适当的索引
- 实现查询缓存
- 使用连接池
- 避免N+1查询问题

#### 缓存策略
- 实现多级缓存
- 使用缓存过期策略
- 监控缓存命中率
- 实现缓存预热

### 3. 安全考虑

#### 认证和授权
- 使用 JWT 认证
- 实现基于角色的访问控制
- 定期更新密钥
- 实现密码哈希

#### 数据安全
- 使用 HTTPS
- 实现输入验证
- 防止 SQL 注入
- 实现敏感数据加密

### 4. 监控和日志

#### 日志记录
- 使用结构化日志
- 实现日志级别管理
- 记录关键业务事件
- 避免记录敏感信息

#### 监控指标
- 响应时间
- 错误率
- 并发用户数
- 系统资源使用率

## 📈 总结

本实施指南提供了在 Loco 框架中实施 DDD+TDD 的完整方案。通过遵循这个指南，您可以：

1. **构建清晰的领域模型**：使用 DDD 原则构建可维护的业务逻辑
2. **实现高质量的代码**：通过 TDD 确保代码质量和测试覆盖率
3. **保持架构清晰**：通过分层架构实现关注点分离
4. **支持业务增长**：通过事件驱动架构支持系统扩展
5. **确保系统稳定性**：通过全面的测试和监控确保系统质量

### 关键成功因素

1. **领域专家参与**：确保领域模型准确反映业务需求
2. **持续重构**：保持代码质量和架构清晰
3. **测试驱动**：确保代码质量和功能正确性
4. **团队协作**：确保团队成员理解 DDD 和 TDD 原则
5. **工具支持**：使用适当的工具支持开发流程

### 后续步骤

1. **持续学习**：深入学习 DDD 和 TDD 的高级概念
2. **实践改进**：根据项目经验调整实施策略
3. **知识分享**：与团队成员分享经验和最佳实践
4. **工具完善**：根据项目需求完善开发工具

---

这个完整的实施指南为您提供了在 Loco 框架中成功实施 DDD+TDD 所需的所有知识和工具。通过遵循这个指南，您可以构建高质量、可维护且业务对齐的应用程序。
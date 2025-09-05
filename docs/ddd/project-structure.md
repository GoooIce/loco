# DDD + TDD 项目结构和代码组织方案

## 🏗️ 项目整体架构

### 目录结构设计

```
myapp-ddd/
├── src/
│   ├── ddd/                          # DDD 基础设施
│   │   ├── lib.rs                    # DDD 核心特质和类型
│   │   ├── aggregate.rs              # 聚合基类
│   │   ├── entity.rs                 # 实体基类
│   │   ├── value_object.rs           # 值对象基类
│   │   ├── repository.rs             # 仓库接口
│   │   ├── service.rs                # 服务基类
│   │   ├── event.rs                  # 事件系统
│   │   ├── command.rs                # 命令系统
│   │   ├── query.rs                  # 查询系统
│   │   ├── error.rs                  # DDD 错误类型
│   │   └── testing.rs                # DDD 测试工具
│   │
│   ├── domains/                      # 领域层
│   │   ├── user/                     # 用户限界上下文
│   │   │   ├── mod.rs                # 限界上下文入口
│   │   │   ├── entities/             # 实体
│   │   │   │   ├── mod.rs
│   │   │   │   └── user.rs
│   │   │   ├── value_objects/        # 值对象
│   │   │   │   ├── mod.rs
│   │   │   │   ├── email.rs
│   │   │   │   └── user_id.rs
│   │   │   ├── aggregates/           # 聚合
│   │   │   │   ├── mod.rs
│   │   │   │   └── user_aggregate.rs
│   │   │   ├── services/             # 领域服务
│   │   │   │   ├── mod.rs
│   │   │   │   └── user_domain_service.rs
│   │   │   ├── events/               # 领域事件
│   │   │   │   ├── mod.rs
│   │   │   │   ├── user_events.rs
│   │   │   │   └── event_handlers.rs
│   │   │   └── repositories/         # 仓库接口
│   │   │       ├── mod.rs
│   │   │       └── user_repository.rs
│   │   │
│   │   ├── order/                    # 订单限界上下文
│   │   │   ├── mod.rs
│   │   │   ├── entities/
│   │   │   │   ├── mod.rs
│   │   │   │   └── order.rs
│   │   │   ├── value_objects/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── order_id.rs
│   │   │   │   ├── money.rs
│   │   │   │   └── order_status.rs
│   │   │   ├── aggregates/
│   │   │   │   ├── mod.rs
│   │   │   │   └── order_aggregate.rs
│   │   │   ├── services/
│   │   │   │   ├── mod.rs
│   │   │   │   └── order_domain_service.rs
│   │   │   ├── events/
│   │   │   │   ├── mod.rs
│   │   │   │   └── order_events.rs
│   │   │   └── repositories/
│   │   │       ├── mod.rs
│   │   │       └── order_repository.rs
│   │   │
│   │   └── shared/                    # 共享领域概念
│   │       ├── mod.rs
│   │       ├── value_objects/
│   │       │   ├── mod.rs
│   │       │   ├── email.rs
│   │       │   └── money.rs
│   │       └── events/
│   │           ├── mod.rs
│   │           └── shared_events.rs
│   │
│   ├── applications/                 # 应用层
│   │   ├── mod.rs                    # 应用层入口
│   │   ├── commands/                 # 命令
│   │   │   ├── mod.rs
│   │   │   ├── user_commands.rs
│   │   │   ├── order_commands.rs
│   │   │   └── command_handlers.rs
│   │   ├── queries/                  # 查询
│   │   │   ├── mod.rs
│   │   │   ├── user_queries.rs
│   │   │   ├── order_queries.rs
│   │   │   └── query_handlers.rs
│   │   ├── services/                 # 应用服务
│   │   │   ├── mod.rs
│   │   │   ├── user_application_service.rs
│   │   │   └── order_application_service.rs
│   │   ├── dtos/                     # 数据传输对象
│   │   │   ├── mod.rs
│   │   │   ├── user_dto.rs
│   │   │   ├── order_dto.rs
│   │   │   └── response_dto.rs
│   │   └── errors/                   # 应用层错误
│   │       ├── mod.rs
│   │       └── application_errors.rs
│   │
│   ├── infrastructure/              # 基础设施层
│   │   ├── mod.rs                    # 基础设施入口
│   │   ├── persistence/              # 持久化
│   │   │   ├── mod.rs
│   │   │   ├── database.rs
│   │   │   ├── user_repository_impl.rs
│   │   │   ├── order_repository_impl.rs
│   │   │   └── migrations/
│   │   ├── events/                   # 事件处理
│   │   │   ├── mod.rs
│   │   │   ├── event_store.rs
│   │   │   ├── event_publisher.rs
│   │   │   └── event_handlers.rs
│   │   ├── external/                 # 外部服务
│   │   │   ├── mod.rs
│   │   │   ├── email_service.rs
│   │   │   ├── payment_service.rs
│   │   │   └── notification_service.rs
│   │   ├── cache/                    # 缓存
│   │   │   ├── mod.rs
│   │   │   ├── redis_cache.rs
│   │   │   └── memory_cache.rs
│   │   └── auth/                     # 认证
│   │       ├── mod.rs
│   │       ├── jwt_service.rs
│   │       └── password_service.rs
│   │
│   ├── interfaces/                  # 接口层
│   │   ├── mod.rs                    # 接口层入口
│   │   ├── controllers/              # 控制器
│   │   │   ├── mod.rs
│   │   │   ├── user_controller.rs
│   │   │   ├── order_controller.rs
│   │   │   └── auth_controller.rs
│   │   ├── routes/                   # 路由
│   │   │   ├── mod.rs
│   │   │   ├── user_routes.rs
│   │   │   ├── order_routes.rs
│   │   │   └── auth_routes.rs
│   │   ├── middleware/               # 中间件
│   │   │   ├── mod.rs
│   │   │   ├── auth_middleware.rs
│   │   │   ├── logging_middleware.rs
│   │   │   ├── cors_middleware.rs
│   │   │   └── rate_limit_middleware.rs
│   │   └── views/                    # 视图
│   │       ├── mod.rs
│   │       ├── user_views.rs
│   │       ├── order_views.rs
│   │       └── layouts/
│   │
│   ├── config.rs                     # 配置管理
│   ├── app.rs                        # 应用配置
│   ├── boot.rs                       # 启动管理
│   ├── lib.rs                        # 库入口
│   └── errors.rs                     # 错误定义
│
├── tests/                            # 测试目录
│   ├── unit/                         # 单元测试
│   │   ├── domains/                  # 领域层测试
│   │   │   ├── user/
│   │   │   ├── order/
│   │   │   └── shared/
│   │   ├── applications/             # 应用层测试
│   │   │   ├── commands/
│   │   │   ├── queries/
│   │   │   └── services/
│   │   └── infrastructure/           # 基础设施层测试
│   │       ├── persistence/
│   │       ├── events/
│   │       └── external/
│   ├── integration/                  # 集成测试
│   │   ├── controllers/              # 控制器测试
│   │   ├── repositories/             # 仓库测试
│   │   ├── services/                 # 服务测试
│   │   └── events/                   # 事件测试
│   └── e2e/                          # 端到端测试
│       ├── user_flows/               # 用户流程测试
│       ├── order_flows/              # 订单流程测试
│       └── business_scenarios/       # 业务场景测试
│
├── config/                           # 配置文件
│   ├── development.yaml             # 开发配置
│   ├── test.yaml                    # 测试配置
│   └── production.yaml              # 生产配置
│
├── migrations/                       # 数据库迁移
│   ├── 20240101000000_create_users_table.sql
│   ├── 20240101000001_create_orders_table.sql
│   └── 20240101000002_create_domain_events_table.sql
│
├── docs/                            # 文档
│   ├── ddd/                         # DDD 相关文档
│   │   ├── thinking-process.md      # 思考过程
│   │   ├── framework-assessment.md  # 框架评估
│   │   ├── implementation-strategy.md # 实施策略
│   │   └── project-structure.md    # 项目结构
│   └── api/                         # API 文档
│
├── scripts/                          # 脚本
│   ├── setup.sh                     # 环境设置
│   ├── migrate.sh                   # 数据库迁移
│   ├── test.sh                      # 测试脚本
│   └── deploy.sh                    # 部署脚本
│
├── .env.example                     # 环境变量示例
├── .gitignore                       # Git 忽略文件
├── Cargo.toml                       # 项目配置
├── README.md                        # 项目说明
└── docker-compose.yml               # Docker 配置
```

## 📋 核心文件示例

### 1. DDD 基础设施 (`src/ddd/lib.rs`)

```rust
//! DDD 基础设施模块
//! 
//! 这个模块提供了领域驱动开发的核心基础设施，
//! 包括聚合、实体、值对象、仓库、事件等核心概念。

pub mod aggregate;
pub mod entity;
pub mod value_object;
pub mod repository;
pub mod service;
pub mod event;
pub mod command;
pub mod query;
pub mod error;
pub mod testing;

// 重新导出核心类型和特质
pub use aggregate::{AggregateRoot, AggregateId};
pub use entity::{Entity, EntityId};
pub use value_object::ValueObject;
pub use repository::{Repository, RepositoryError};
pub use service::{DomainService, ServiceError};
pub use event::{DomainEvent, EventPublisher, EventHandler};
pub use command::{Command, CommandHandler, CommandBus};
pub use query::{Query, QueryHandler, QueryBus};
pub use error::{DomainError, DomainResult};
pub use testing::{MockRepository, EventTestHelper};

// 常用类型别名
pub type DomainResult<T> = Result<T, DomainError>;
pub type EventResult = Result<(), DomainError>;
pub type CommandResult<T> = Result<T, DomainError>;
pub type QueryResult<T> = Result<T, DomainError>;
```

### 2. 聚合基类 (`src/ddd/aggregate.rs`)

```rust
//! 聚合基类和特质定义

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::fmt::Debug;
use crate::ddd::{entity::Entity, event::DomainEvent};

/// 聚合根特质
/// 
/// 聚合根是聚合的入口点，负责维护聚合的一致性
#[async_trait]
pub trait AggregateRoot: Entity + Send + Sync {
    /// 聚合ID类型
    type Id: AggregateId + Send + Sync;
    
    /// 聚合事件类型
    type Event: DomainEvent + Send + Sync + Clone;
    
    /// 获取聚合ID
    fn id(&self) -> &Self::Id;
    
    /// 获取聚合版本
    fn version(&self) -> u32;
    
    /// 获取未提交的事件
    fn events(&self) -> Vec<Self::Event>;
    
    /// 清除已提交的事件
    fn clear_events(&mut self);
    
    /// 增加版本号
    fn increment_version(&mut self);
}

/// 聚合ID特质
pub trait AggregateId: Debug + Clone + PartialEq + Eq + Send + Sync {
    /// 获取ID值
    fn value(&self) -> &str;
    
    /// 从字符串创建ID
    fn from_string(value: String) -> Self;
    
    /// 生成新ID
    fn new() -> Self;
}

/// 聚合基类实现
#[derive(Debug, Clone)]
pub struct AggregateBase<T: AggregateId> {
    id: T,
    version: u32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl<T: AggregateId> AggregateBase<T> {
    pub fn new(id: T) -> Self {
        let now = Utc::now();
        Self {
            id,
            version: 1,
            created_at: now,
            updated_at: now,
        }
    }
    
    pub fn id(&self) -> &T {
        &self.id
    }
    
    pub fn version(&self) -> u32 {
        self.version
    }
    
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
    
    pub fn increment_version(&mut self) {
        self.version += 1;
        self.updated_at = Utc::now();
    }
}
```

### 3. 用户限界上下文 (`src/domains/user/mod.rs`)

```rust
//! 用户限界上下文
//! 
//! 这个模块定义了用户管理相关的领域概念和业务逻辑

pub mod entities;
pub mod value_objects;
pub mod aggregates;
pub mod services;
pub mod events;
pub mod repositories;

// 重新导出公共接口
pub use self::aggregates::UserAggregate;
pub use self::entities::UserEntity;
pub use self::value_objects::{UserId, Email, UserStatus};
pub use self::services::UserDomainService;
pub use self::events::{UserEvent, UserRegisteredEvent, UserEmailChangedEvent};
pub use self::repositories::UserRepository;

// 用户限界上下文的通用语言
pub type UserName = String;
pub type UserPassword = String;
pub type UserCreatedAt = chrono::DateTime<chrono::Utc>;
pub type UserUpdatedAt = chrono::DateTime<chrono::Utc>;

// 用户相关的业务错误
#[derive(Debug, thiserror::Error)]
pub enum UserDomainError {
    #[error("用户邮箱格式无效: {0}")]
    InvalidEmail(String),
    
    #[error("用户密码格式无效: {0}")]
    InvalidPassword(String),
    
    #[error("用户邮箱已存在: {0}")]
    EmailAlreadyExists(String),
    
    #[error("用户不存在: {0}")]
    UserNotFound(String),
    
    #[error("用户状态无效")]
    InvalidUserStatus,
    
    #[error("用户权限不足")]
    InsufficientPermissions,
}

impl From<UserDomainError> for crate::ddd::error::DomainError {
    fn from(err: UserDomainError) -> Self {
        crate::ddd::error::DomainError::Business(err.to_string())
    }
}

// 用户相关的业务常量
pub const USER_PASSWORD_MIN_LENGTH: usize = 8;
pub const USER_PASSWORD_MAX_LENGTH: usize = 128;
pub const USER_NAME_MIN_LENGTH: usize = 2;
pub const USER_NAME_MAX_LENGTH: usize = 50;
```

### 4. 用户聚合 (`src/domains/user/aggregates/user_aggregate.rs`)

```rust
//! 用户聚合根
//! 
//! 用户聚合负责管理用户相关的业务逻辑和一致性

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use crate::ddd::{aggregate::AggregateRoot, event::DomainEvent};
use crate::domains::user::{
    entities::UserEntity,
    value_objects::{UserId, Email, UserStatus},
    events::{UserEvent, UserRegisteredEvent, UserEmailChangedEvent, UserStatusChangedEvent},
    UserDomainError,
};

/// 用户聚合根
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAggregate {
    /// 聚合基类
    base: AggregateBase<UserId>,
    
    /// 用户邮箱
    email: Email,
    
    /// 用户名
    name: String,
    
    /// 密码哈希
    password_hash: String,
    
    /// 用户状态
    status: UserStatus,
    
    /// 未提交的事件
    events: Vec<UserEvent>,
}

impl UserAggregate {
    /// 创建新用户
    pub fn new(
        id: UserId,
        email: Email,
        name: String,
        password: String,
    ) -> Result<Self, UserDomainError> {
        // 验证密码
        if password.len() < USER_PASSWORD_MIN_LENGTH {
            return Err(UserDomainError::InvalidPassword(
                format!("密码长度不能少于 {} 位", USER_PASSWORD_MIN_LENGTH)
            ));
        }
        
        if password.len() > USER_PASSWORD_MAX_LENGTH {
            return Err(UserDomainError::InvalidPassword(
                format!("密码长度不能超过 {} 位", USER_PASSWORD_MAX_LENGTH)
            ));
        }
        
        // 验证用户名
        if name.len() < USER_NAME_MIN_LENGTH {
            return Err(UserDomainError::InvalidPassword(
                format!("用户名长度不能少于 {} 位", USER_NAME_MIN_LENGTH)
            ));
        }
        
        if name.len() > USER_NAME_MAX_LENGTH {
            return Err(UserDomainError::InvalidPassword(
                format!("用户名长度不能超过 {} 位", USER_NAME_MAX_LENGTH)
            ));
        }
        
        let password_hash = hash_password(&password);
        let mut aggregate = Self {
            base: AggregateBase::new(id),
            email,
            name,
            password_hash,
            status: UserStatus::Active,
            events: Vec::new(),
        };
        
        // 发布用户注册事件
        aggregate.events.push(UserEvent::Registered(UserRegisteredEvent {
            user_id: aggregate.base.id.value().clone(),
            email: aggregate.email.value().clone(),
            name: aggregate.name.clone(),
            registered_at: Utc::now(),
        }));
        
        Ok(aggregate)
    }
    
    /// 修改用户邮箱
    pub fn change_email(&mut self, new_email: Email) -> Result<(), UserDomainError> {
        if self.email == new_email {
            return Ok(());
        }
        
        self.email = new_email.clone();
        self.base.increment_version();
        
        // 发布邮箱修改事件
        self.events.push(UserEvent::EmailChanged(UserEmailChangedEvent {
            user_id: self.base.id.value().clone(),
            old_email: self.email.value().clone(),
            new_email: new_email.value().clone(),
            changed_at: Utc::now(),
        }));
        
        Ok(())
    }
    
    /// 修改用户状态
    pub fn change_status(&mut self, new_status: UserStatus) -> Result<(), UserDomainError> {
        if self.status == new_status {
            return Ok(());
        }
        
        self.status = new_status;
        self.base.increment_version();
        
        // 发布状态修改事件
        self.events.push(UserEvent::StatusChanged(UserStatusChangedEvent {
            user_id: self.base.id.value().clone(),
            old_status: self.status.clone(),
            new_status: new_status.clone(),
            changed_at: Utc::now(),
        }));
        
        Ok(())
    }
    
    /// 验证密码
    pub fn verify_password(&self, password: &str) -> bool {
        verify_password_hash(password, &self.password_hash)
    }
    
    /// 获取用户邮箱
    pub fn email(&self) -> &Email {
        &self.email
    }
    
    /// 获取用户名
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// 获取用户状态
    pub fn status(&self) -> &UserStatus {
        &self.status
    }
}

#[async_trait]
impl AggregateRoot for UserAggregate {
    type Id = UserId;
    type Event = UserEvent;
    
    fn id(&self) -> &Self::Id {
        &self.base.id
    }
    
    fn version(&self) -> u32 {
        self.base.version()
    }
    
    fn events(&self) -> Vec<Self::Event> {
        self.events.clone()
    }
    
    fn clear_events(&mut self) {
        self.events.clear();
    }
    
    fn increment_version(&mut self) {
        self.base.increment_version();
    }
}

// 密码哈希函数
fn hash_password(password: &str) -> String {
    use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
    
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut rand::thread_rng());
    
    argon2
        .hash_password(password.as_bytes(), &salt)
        .expect("Unable to hash password")
        .to_string()
}

// 密码验证函数
fn verify_password_hash(password: &str, hash: &str) -> bool {
    use argon2::{Argon2, PasswordVerifier, password_hash::PasswordHash};
    
    let argon2 = Argon2::default();
    let parsed_hash = PasswordHash::new(hash).expect("Invalid password hash");
    
    argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok()
}
```

### 5. 应用服务 (`src/applications/services/user_application_service.rs`)

```rust
//! 用户应用服务
//! 
//! 用户应用服务负责协调领域对象和基础设施，
//! 提供用户管理的用例实现

use async_trait::async_trait;
use crate::ddd::{command::CommandHandler, query::QueryHandler, error::DomainResult};
use crate::domains::user::{
    aggregates::UserAggregate,
    value_objects::{UserId, Email},
    repositories::UserRepository,
    events::UserEvent,
};
use crate::applications::{
    commands::{RegisterUserCommand, UpdateUserEmailCommand, GetUserQuery},
    dtos::{UserDto, CreateUserDto, UpdateUserEmailDto},
    errors::ApplicationError,
};

/// 用户应用服务
pub struct UserApplicationService {
    user_repository: Arc<dyn UserRepository>,
    event_publisher: Arc<dyn EventPublisher<UserEvent>>,
}

impl UserApplicationService {
    pub fn new(
        user_repository: Arc<dyn UserRepository>,
        event_publisher: Arc<dyn EventPublisher<UserEvent>>,
    ) -> Self {
        Self {
            user_repository,
            event_publisher,
        }
    }
    
    /// 注册用户
    pub async fn register_user(&self, dto: CreateUserDto) -> DomainResult<UserDto> {
        // 验证邮箱是否已存在
        if let Some(existing_user) = self.user_repository.find_by_email(&dto.email).await? {
            return Err(ApplicationError::EmailAlreadyExists(dto.email).into());
        }
        
        // 创建用户聚合
        let user_id = UserId::new();
        let user = UserAggregate::new(
            user_id.clone(),
            dto.email,
            dto.name,
            dto.password,
        )?;
        
        // 保存用户
        self.user_repository.save(&user).await?;
        
        // 发布事件
        for event in user.events() {
            self.event_publisher.publish(event).await?;
        }
        
        // 返回DTO
        Ok(UserDto {
            id: user_id.value().clone(),
            email: user.email().value().clone(),
            name: user.name().to_string(),
            status: user.status().to_string(),
            created_at: user.created_at(),
            updated_at: user.updated_at(),
        })
    }
    
    /// 修改用户邮箱
    pub async fn update_user_email(&self, dto: UpdateUserEmailDto) -> DomainResult<UserDto> {
        // 查找用户
        let user_id = UserId::from_string(dto.user_id);
        let mut user = self.user_repository.find_by_id(&user_id).await?
            .ok_or(ApplicationError::UserNotFound(dto.user_id))?;
        
        // 修改邮箱
        let new_email = Email::new(dto.new_email)?;
        user.change_email(new_email)?;
        
        // 保存用户
        self.user_repository.save(&user).await?;
        
        // 发布事件
        for event in user.events() {
            self.event_publisher.publish(event).await?;
        }
        
        // 返回DTO
        Ok(UserDto {
            id: user.id().value().clone(),
            email: user.email().value().clone(),
            name: user.name().to_string(),
            status: user.status().to_string(),
            created_at: user.created_at(),
            updated_at: user.updated_at(),
        })
    }
    
    /// 获取用户信息
    pub async fn get_user(&self, user_id: String) -> DomainResult<UserDto> {
        let user_id = UserId::from_string(user_id);
        let user = self.user_repository.find_by_id(&user_id).await?
            .ok_or(ApplicationError::UserNotFound(user_id.value().clone()))?;
        
        Ok(UserDto {
            id: user.id().value().clone(),
            email: user.email().value().clone(),
            name: user.name().to_string(),
            status: user.status().to_string(),
            created_at: user.created_at(),
            updated_at: user.updated_at(),
        })
    }
}

#[async_trait]
impl CommandHandler<RegisterUserCommand> for UserApplicationService {
    type Result = UserDto;
    
    async fn handle(&self, command: RegisterUserCommand) -> DomainResult<Self::Result> {
        let dto = CreateUserDto {
            email: command.email,
            name: command.name,
            password: command.password,
        };
        
        self.register_user(dto).await
    }
}

#[async_trait]
impl CommandHandler<UpdateUserEmailCommand> for UserApplicationService {
    type Result = UserDto;
    
    async fn handle(&self, command: UpdateUserEmailCommand) -> DomainResult<Self::Result> {
        let dto = UpdateUserEmailDto {
            user_id: command.user_id,
            new_email: command.new_email,
        };
        
        self.update_user_email(dto).await
    }
}

#[async_trait]
impl QueryHandler<GetUserQuery> for UserApplicationService {
    type Result = UserDto;
    
    async fn handle(&self, query: GetUserQuery) -> DomainResult<Self::Result> {
        self.get_user(query.user_id).await
    }
}
```

### 6. 控制器 (`src/interfaces/controllers/user_controller.rs`)

```rust
//! 用户控制器
//! 
//! 用户控制器处理HTTP请求，调用应用服务，并返回HTTP响应

use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use crate::applications::{
    services::UserApplicationService,
    commands::{RegisterUserCommand, UpdateUserEmailCommand},
    queries::GetUserQuery,
    dtos::{UserDto, CreateUserDto, UpdateUserEmailDto},
    errors::ApplicationError,
};
use crate::interfaces::AppContext;

/// 用户控制器
pub struct UserController;

impl UserController {
    /// 注册用户
    pub async fn register_user(
        State(ctx): State<AppContext>,
        Json(dto): Json<CreateUserDto>,
    ) -> Result<impl IntoResponse, ApplicationError> {
        let command = RegisterUserCommand {
            email: dto.email,
            name: dto.name,
            password: dto.password,
        };
        
        let user_dto = ctx.user_service.register_user(command).await?;
        
        Ok((axum::http::StatusCode::CREATED, Json(user_dto)))
    }
    
    /// 获取用户信息
    pub async fn get_user(
        State(ctx): State<AppContext>,
        Path(user_id): Path<String>,
    ) -> Result<impl IntoResponse, ApplicationError> {
        let query = GetUserQuery { user_id };
        
        let user_dto = ctx.user_service.get_user(query).await?;
        
        Ok(Json(user_dto))
    }
    
    /// 修改用户邮箱
    pub async fn update_user_email(
        State(ctx): State<AppContext>,
        Path(user_id): Path<String>,
        Json(dto): Json<UpdateUserEmailDto>,
    ) -> Result<impl IntoResponse, ApplicationError> {
        let command = UpdateUserEmailCommand {
            user_id,
            new_email: dto.new_email,
        };
        
        let user_dto = ctx.user_service.update_user_email(command).await?;
        
        Ok(Json(user_dto))
    }
}

// 错误转换为HTTP响应
impl IntoResponse for ApplicationError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApplicationError::EmailAlreadyExists(_) => (axum::http::StatusCode::CONFLICT, "Email already exists"),
            ApplicationError::UserNotFound(_) => (axum::http::StatusCode::NOT_FOUND, "User not found"),
            ApplicationError::ValidationError(msg) => (axum::http::StatusCode::BAD_REQUEST, &msg),
            ApplicationError::InternalError(msg) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, &msg),
        };
        
        (status, Json(json!({ "error": error_message }))).into_response()
    }
}
```

## 🧪 测试文件示例

### 1. 聚合测试 (`tests/unit/domains/user/user_aggregate_test.rs`)

```rust
//! 用户聚合测试

use chrono::Utc;
use crate::domains::user::{
    aggregates::UserAggregate,
    value_objects::{UserId, Email, UserStatus},
    UserDomainError,
};

#[tokio::test]
async fn test_create_user_success() {
    // Arrange
    let user_id = UserId::new();
    let email = Email::new("test@example.com").unwrap();
    let name = "Test User".to_string();
    let password = "password123".to_string();
    
    // Act
    let result = UserAggregate::new(user_id, email, name, password);
    
    // Assert
    assert!(result.is_ok());
    let user = result.unwrap();
    
    assert_eq!(user.id(), &user_id);
    assert_eq!(user.email().value(), "test@example.com");
    assert_eq!(user.name(), "Test User");
    assert_eq!(user.status(), &UserStatus::Active);
    assert_eq!(user.events().len(), 1);
    
    // 验证注册事件
    match user.events().first() {
        Some(crate::domains::user::events::UserEvent::Registered(event)) => {
            assert_eq!(event.user_id, user_id.value());
            assert_eq!(event.email, "test@example.com");
            assert_eq!(event.name, "Test User");
        }
        _ => panic!("Expected UserRegistered event"),
    }
}

#[tokio::test]
async fn test_create_user_invalid_email() {
    // Arrange
    let user_id = UserId::new();
    let email = Email::new("invalid-email").unwrap();
    let name = "Test User".to_string();
    let password = "password123".to_string();
    
    // Act
    let result = UserAggregate::new(user_id, email, name, password);
    
    // Assert
    assert!(result.is_err());
    match result.err().unwrap() {
        UserDomainError::InvalidEmail(msg) => {
            assert!(msg.contains("邮箱格式无效"));
        }
        _ => panic!("Expected InvalidEmail error"),
    }
}

#[tokio::test]
async fn test_change_email_success() {
    // Arrange
    let user_id = UserId::new();
    let email = Email::new("old@example.com").unwrap();
    let name = "Test User".to_string();
    let password = "password123".to_string();
    
    let mut user = UserAggregate::new(user_id, email, name, password).unwrap();
    let new_email = Email::new("new@example.com").unwrap();
    
    // Act
    let result = user.change_email(new_email.clone());
    
    // Assert
    assert!(result.is_ok());
    assert_eq!(user.email(), &new_email);
    assert_eq!(user.events().len(), 2); // 注册事件 + 邮箱修改事件
    
    // 验证邮箱修改事件
    match user.events().last() {
        Some(crate::domains::user::events::UserEvent::EmailChanged(event)) => {
            assert_eq!(event.user_id, user_id.value());
            assert_eq!(event.old_email, "old@example.com");
            assert_eq!(event.new_email, "new@example.com");
        }
        _ => panic!("Expected EmailChanged event"),
    }
}

#[tokio::test]
async fn test_change_email_same_email() {
    // Arrange
    let user_id = UserId::new();
    let email = Email::new("test@example.com").unwrap();
    let name = "Test User".to_string();
    let password = "password123".to_string();
    
    let mut user = UserAggregate::new(user_id, email.clone(), name, password).unwrap();
    
    // Act
    let result = user.change_email(email);
    
    // Assert
    assert!(result.is_ok());
    assert_eq!(user.events().len(), 1); // 只有注册事件
}

#[tokio::test]
async fn test_verify_password_success() {
    // Arrange
    let user_id = UserId::new();
    let email = Email::new("test@example.com").unwrap();
    let name = "Test User".to_string();
    let password = "password123".to_string();
    
    let user = UserAggregate::new(user_id, email, name, password).unwrap();
    
    // Act
    let result = user.verify_password("password123");
    
    // Assert
    assert!(result);
}

#[tokio::test]
async fn test_verify_password_failure() {
    // Arrange
    let user_id = UserId::new();
    let email = Email::new("test@example.com").unwrap();
    let name = "Test User".to_string();
    let password = "password123".to_string();
    
    let user = UserAggregate::new(user_id, email, name, password).unwrap();
    
    // Act
    let result = user.verify_password("wrong_password");
    
    // Assert
    assert!(!result);
}
```

### 2. 集成测试 (`tests/integration/controllers/user_controller_test.rs`)

```rust
//! 用户控制器集成测试

use axum_test::TestServer;
use serde_json::json;
use crate::testing::create_test_app;
use crate::interfaces::controllers::UserController;

#[tokio::test]
async fn test_register_user_success() {
    // Arrange
    let app = create_test_app().await;
    let server = TestServer::new(app).unwrap();
    
    let new_user = json!({
        "email": "test@example.com",
        "name": "Test User",
        "password": "password123"
    });
    
    // Act
    let response = server
        .post("/api/users")
        .json(&new_user)
        .await;
    
    // Assert
    response.assert_status(201);
    response.assert_json::<serde_json::Value>();
    
    let user = response.json::<serde_json::Value>();
    assert_eq!(user["email"], "test@example.com");
    assert_eq!(user["name"], "Test User");
    assert!(user["id"].is_string());
    assert!(user["created_at"].is_string());
    assert!(user["updated_at"].is_string());
}

#[tokio::test]
async fn test_register_user_invalid_email() {
    // Arrange
    let app = create_test_app().await;
    let server = TestServer::new(app).unwrap();
    
    let new_user = json!({
        "email": "invalid-email",
        "name": "Test User",
        "password": "password123"
    });
    
    // Act
    let response = server
        .post("/api/users")
        .json(&new_user)
        .await;
    
    // Assert
    response.assert_status(400);
    
    let error = response.json::<serde_json::Value>();
    assert!(error["error"].is_string());
}

#[tokio::test]
async fn test_get_user_success() {
    // Arrange
    let app = create_test_app().await;
    let server = TestServer::new(app).unwrap();
    
    // 先创建用户
    let new_user = json!({
        "email": "test@example.com",
        "name": "Test User",
        "password": "password123"
    });
    
    let create_response = server
        .post("/api/users")
        .json(&new_user)
        .await;
    
    create_response.assert_status(201);
    let created_user = create_response.json::<serde_json::Value>();
    let user_id = created_user["id"].as_str().unwrap();
    
    // Act
    let response = server
        .get(&format!("/api/users/{}", user_id))
        .await;
    
    // Assert
    response.assert_status(200);
    
    let user = response.json::<serde_json::Value>();
    assert_eq!(user["id"], user_id);
    assert_eq!(user["email"], "test@example.com");
    assert_eq!(user["name"], "Test User");
}

#[tokio::test]
async fn test_get_user_not_found() {
    // Arrange
    let app = create_test_app().await;
    let server = TestServer::new(app).unwrap();
    
    // Act
    let response = server
        .get("/api/users/nonexistent-user")
        .await;
    
    // Assert
    response.assert_status(404);
    
    let error = response.json::<serde_json::Value>();
    assert_eq!(error["error"], "User not found");
}

#[tokio::test]
async fn test_update_user_email_success() {
    // Arrange
    let app = create_test_app().await;
    let server = TestServer::new(app).unwrap();
    
    // 先创建用户
    let new_user = json!({
        "email": "old@example.com",
        "name": "Test User",
        "password": "password123"
    });
    
    let create_response = server
        .post("/api/users")
        .json(&new_user)
        .await;
    
    create_response.assert_status(201);
    let created_user = create_response.json::<serde_json::Value>();
    let user_id = created_user["id"].as_str().unwrap();
    
    // Act
    let update_data = json!({
        "new_email": "new@example.com"
    });
    
    let response = server
        .put(&format!("/api/users/{}/email", user_id))
        .json(&update_data)
        .await;
    
    // Assert
    response.assert_status(200);
    
    let user = response.json::<serde_json::Value>();
    assert_eq!(user["id"], user_id);
    assert_eq!(user["email"], "new@example.com");
    assert_eq!(user["name"], "Test User");
}
```

## 📊 项目配置文件

### 1. Cargo.toml 配置

```toml
[package]
name = "myapp-ddd"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]

[dependencies]
# Loco 框架
loco-rs = "0.16"
loco-gen = "0.16"

# 异步运行时
tokio = { version = "1.0", features = ["full"] }
async-trait = "0.1"

# 序列化
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# 错误处理
thiserror = "1.0"
anyhow = "1.0"

# 日期时间
chrono = { version = "0.4", features = ["serde"] }

# 数据库
sea-orm = { version = "1.0", features = ["sqlx-postgres", "runtime-tokio-rustls", "macros"] }
sea-orm-migration = "1.0"
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-rustls", "chrono", "uuid"] }

# 密码哈希
argon2 = "0.5"
rand = "0.8"

# 验证
validator = { version = "0.16", features = ["derive"] }
regex = "1.0"

# 缓存
redis = { version = "0.24", features = ["tokio-comp"] }
bb8 = "0.8"
bb8-redis = "0.15"

# 日志
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

# 配置
config = "0.13"
dotenv = "0.15"

# 测试
mockall = "0.12"
tokio-test = "0.4"
testcontainers = "0.15"
axum-test = "0.7"

# 其他
uuid = { version = "1.0", features = ["v4", "serde"] }
lazy_static = "1.4"
parking_lot = "0.12"

[dev-dependencies]
# 测试
tokio-test = "0.4"
criterion = "0.5"
insta = "1.0"

# 代码质量
clippy = "0.0"
rustfmt = "0.0"

[features]
default = ["with-db", "auth-jwt", "cache-redis"]
with-db = []
auth-jwt = []
cache-redis = []
testing = []
```

### 2. 配置文件示例

```yaml
# config/development.yaml
application:
  name: "MyApp DDD"
  environment: "development"
  debug: true
  host: "0.0.0.0"
  port: 3000

database:
  url: "postgres://user:password@localhost:5432/myapp_development"
  max_connections: 10
  min_connections: 2
  connect_timeout: 30
  idle_timeout: 600

redis:
  url: "redis://localhost:6379/0"
  max_connections: 10

auth:
  jwt_secret: "your-secret-key-here"
  jwt_expiration: 3600

cache:
  default_ttl: 3600
  prefix: "myapp:"

logging:
  level: "debug"
  format: "pretty"
  file: "logs/app.log"

events:
  enabled: true
  store_events: true
  publish_events: true

monitoring:
  enabled: true
  metrics_port: 9090
```

---

*这份项目结构和代码组织方案提供了在 Loco 框架中实施 DDD + TDD 的完整目录结构和核心文件示例，为实际项目开发提供了详细的指导。*
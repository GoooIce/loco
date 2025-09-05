//! Loco DDD Prelude
//! 
//! 这个模块重新导出了常用的类型和特性，以便更容易使用。

// 核心类型
pub use crate::ddd::{
    Identifier, 
    BaseEntity, 
    EntityValidator,
    ValueObject,
    AggregateRoot,
    BaseAggregate,
    DomainEvent,
    EventData,
    BasicEvent,
    Command,
    Query,
    DomainService,
    BaseDomainService,
    Repository,
    InMemoryRepository,
    QuerySpecification,
    QueryExpression,
    QueryBuilder,
    QueryResult,
    QueryProcessor,
    InMemoryQueryProcessor,
};

// 错误类型
pub use crate::error::DddError;

// 事件系统
pub use crate::ddd::event::{
    EventBus,
    EventStore,
    InMemoryEventStore,
    EventSourcingHelper,
    EventHandler,
};

// 命令系统
pub use crate::ddd::command::{
    CommandBus,
    CommandHandler,
    QueryBus,
    QueryHandler,
    CqrsService,
    BasicCommand,
    BasicQuery,
};

// 查询系统
pub use crate::ddd::query::{
    SortOrder,
};

// 服务系统
pub use crate::ddd::service::{
    ServiceRegistry,
    ServiceCoordinator,
    ServiceMiddleware,
    LoggingServiceMiddleware,
    ValidationServiceMiddleware,
    ServiceFactory,
    ServiceHealthChecker,
};

// 常用的值对象
pub use crate::ddd::value_object::{
    Email,
    Money,
    Address,
    Percentage,
};

// 重新导出常用的特质和宏
pub use async_trait::async_trait;
pub use serde::{Serialize, Deserialize};
pub use uuid::Uuid;
pub use chrono::{DateTime, Utc};

// 常用的 Result 类型
pub type Result<T> = std::result::Result<T, crate::DddError>;

// 常用的标准库类型
pub use std::sync::Arc;
pub use std::collections::HashMap;

// 实用宏
#[macro_export]
macro_rules! ddd_result {
    ($expr:expr) => {
        $expr.map_err(|e| $crate::DddError::from(e))
    };
}

#[macro_export]
macro_rules! ensure {
    ($condition:expr, $error:expr) => {
        if !$condition {
            return Err($crate::DddError::validation($error));
        }
    };
}

#[macro_export]
macro_rules! ensure_domain {
    ($condition:expr, $error:expr) => {
        if !$condition {
            return Err($crate::DddError::domain($error));
        }
    };
}

#[macro_export]
macro_rules! ensure_not_found {
    ($condition:expr, $error:expr) => {
        if !$condition {
            return Err($crate::DddError::not_found($error));
        }
    };
}
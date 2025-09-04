#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::module_name_repetitions)]
#![doc = include_str!("../README.md")]

// Loco Rust Web Framework 应用程序库
//
// 这个 crate 提供了构建现代、高性能 Web 应用所需的基础设施组件。
//
// Loco 是一个受 Ruby on Rails 启发的 Rust Web 框架，采用约定优于配置的原则。
// 它基于 Axum 构建，并提供了包括控制器、模型、后台作业处理等在内的完整功能集。

pub use self::errors::Error;

mod banner;
pub mod bgworker; // 后台任务队列管理模块
mod depcheck; // 依赖检查工具模块
pub mod initializers; // 初始化器模块，用于系统初始化组件
pub mod prelude; // 预导入模块，包含常用的 trait 和类型定义

pub mod data; // 应用数据管理模块
pub mod doctor; // 系统诊断工具模块

#[cfg(feature = "with-db")]
pub mod db; // 数据库集成模块，基于 SeaORM 实现
#[cfg(feature = "with-db")]
pub mod model; // ORM 模型定义模块
#[cfg(feature = "with-db")]
pub mod schema; // 数据库 Schema 定义模块
mod tera; // 模板引擎集成模块（Tera）

pub mod app; // 应用程序核心模块，定义 App 结构和生命周期钩子
pub mod auth; // 认证授权模块，支持 JWT 等认证方式
pub mod boot; // 应用启动管理模块，配置和启动系统组件
pub mod cache; // 缓存支持模块，支持内存或 Redis 后端缓存
#[cfg(feature = "cli")]
pub mod cli; // 命令行接口模块，提供 CLI 工具功能
pub mod config; // 配置管理模块，读取和解析环境配置文件
pub mod controller; // HTTP 控制器模块，处理 Web 请求路由和响应逻辑
mod env_vars; // 环境变量处理模块
pub mod environment; // 环境管理模块，支持多环境配置加载
pub mod errors; // 错误处理模块，统一的错误类型定义和管理
pub mod hash; // 哈希处理模块，提供密码加密等功能
pub mod i18n; // 国际化支持模块，提供多语言能力
pub mod logger; // 日志记录模块，集成 tracing 和日志管理
pub mod mailer; // 邮件发送模块，支持背景邮件处理
pub mod scheduler; // 任务调度器模块，支持 Cron-like 调度任务
pub mod task; // 自定义任务处理模块，允许用户运行自定义命令逻辑
#[cfg(feature = "testing")]
pub mod testing; // 测试支持模块，提供测试工具和辅助方法
#[cfg(feature = "testing")]
pub use axum_test::TestServer; // Axum 测试服务器访问入口
pub mod storage; // 文件存储模块，支持多种存储提供商（本地、AWS S3 等）
#[cfg(feature = "testing")]
pub mod tests_cfg; // 测试配置模块，用于测试环境的参数设置
pub mod validation; // 数据验证模块，集成验证规则检查机制
pub use validator; // 导出外部验证库以便使用其功能
pub mod cargo_config;
#[cfg(feature = "mcp")]
pub mod mcp; // MCP API Server 模块 // Cargo 配置处理模块

/// 应用程序结果类型定义
///
/// 用于统一错误和成功返回的 Result 类型别名。
pub type Result<T, E = Error> = std::result::Result<T, E>;

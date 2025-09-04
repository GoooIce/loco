# Loco Rust Web Framework 中文说明文档

Loco 是一个受 Ruby on Rails 启发的 Rust Web 框架，遵循约定优于配置的设计原则。它基于 Axum 构建，并提供了完整的 Web 应用功能集，包括控制器、模型、后台任务处理等。

## 核心特性

### 应用结构
- **App 结构**: 中央的 `App` 结构体，通过实现 `Hooks` trait 来管理生命周期
- **控制器**: 处理 HTTP 请求，支持验证、格式化和中间件保护

### 核心组件
- **模型**: 基于 SeaORM 的数据库实体，支持关系和验证
- **视图**: Tera 模板引擎用于动态 HTML 生成  
- **后台任务处理器**: Redis、PostgreSQL 或 SQLite 支持的队列系统
- **调度器**: Cron-like 任务调度，支持英文转 cron 表达式转换
- **邮件器**: 背景邮件发送，支持模板渲染
- **存储**: 抽象文件存储（本地、AWS S3、GCP、Azure）
- **缓存**: 内存或 Redis 缓存层支持

### 架构模式
- **启动系统**: `boot.rs` 处理应用程序启动，支持多种运行模式（服务器只读、服务器和worker混合、仅worker、全部）
- **共享存储**: 使用 `DashMap` 提供类型安全的异构数据存储
- **中间件系统**: Tower-based 中间件栈，支持日志、CORS、安全头等
- **配置管理**: 基于环境的 YAML 配置文件
- **测试支持**: 集成 `axum-test` 和 snapshot testing 工具

## 功能特性标签
- `auth_jwt`: JWT 认证支持  
- `with-db`: 数据库功能（SeaORM）
- `cli`: 命令行界面支持
- `testing`: 测试工具集
- `bg_redis`/`bg_pg`/`bg_sqlt`: 后台工作队列后端支持
- `storage_*`: 云存储提供商（AWS S3、Azure Blob Storage、Google Cloud Storage等）
- `cache_*`: 缓存后端支持（内存缓存、Redis）

## 项目目录结构

```
src/                     # 核心框架代码
├── app.rs              # 应用程序核心结构和生命周期管理  
├── boot.rs             # 启动过程和运行模式处理
├── cli.rs              # 命令行接口定义
├── config.rs           # 配置管理器和加载系统  
├── controller/         # HTTP控制器处理模块
├── db.rs               # 数据库集成和连接管理
├── model/              # ORM模型定义模块  
├── storage/            # 文件存储抽象层
├── cache/              # 缓存处理模块
├── scheduler.rs        # 任务调度器实现  
└── task.rs             # 自定义任务管理

tests/                  # 框架测试用例
examples/               # 示例应用程序项目
starters/              # 应用模板生成器（已移除）
xtask/                  # 开发工具任务 runner

loco-cli/              # CLI 工具（已弃用，使用 loco-new 替代）
loco-gen/              # 代码生成工具集
loco-new/             # 应用程序模板生成器 
```

## 主要运行命令

```bash
cargo install loco                 # 安装 Loco CLI 工具

# 应用启动和管理命令
cargo loco start                   # 启动 Web 服务器  
cargo loco start --worker          # 启动单独的 worker 进程
cargo loco start --server-and-worker  # 同时启动服务器和 worker
cargo loco scheduler               # 运行调度器服务

# 数据库管理命令  
cargo loco db reset                # 重置并迁移数据库
cargo loco db entities             # 生成数据库实体定义

# 自定义任务命令  
cargo loco task <name>             # 运行自定义任务

# 代码生成命令
cargo loco generate model posts    # 生成模型文件及相关测试
cargo loco generate controller users   # 生成控制器及路由

# 环境诊断命令  
cargo loco doctor                  # 验证和诊断配置
cargo loco doctor --environment test   # 检查测试环境要求

# 代码质量检查命令
cargo fmt                          # 格式化 Rust 代码  
cargo clippy                       # 运行 Clippy 检查器
```

## 开发环境要求

- Redis 服务器（用于背景任务队列）
- 数据库（PostgreSQL 或 SQLite，如果需要数据库功能）  
- Node.js/pnpm（如果使用前端构建模板）

## 测试策略

- 单元测试: 针对各个组件的单独测试
- 集成测试: 使用 testcontainers 进行集成验证  
- snapshot 测试: 代码生成器的快照测试

## 编码规范  

遵循 Rust 最佳实践：
- 使用 `#![allow(clippy::missing_const_for_fn)]` 和 `#![allow(clippy::module_name_repetitions)]`
- 统一错误处理和 Result 类型定义
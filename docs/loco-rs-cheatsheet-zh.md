# Loco-rs 项目速记文档

## 📋 项目概览

Loco-rs 是一个受 Ruby on Rails 启发的 Rust Web 框架，遵循"约定优于配置"原则，构建在 Axum 之上，提供完整的 Web 应用开发体验。

### 🎯 设计哲学
- **约定优于配置**: 减少决策疲劳，提高开发效率
- **类型安全**: 利用 Rust 的类型系统确保编译时安全
- **异步优先**: 基于 Tokio 的全异步架构
- **模块化设计**: 清晰的模块边界和职责分离
- **开发体验**: 提供丰富的工具和脚手架

### 🏗️ 架构特点
- **基于 Axum**: 利用 Rust 生态中最受欢迎的 Web 框架
- **SeaORM 集成**: 类型安全的数据库操作
- **后台任务**: 支持多种队列后端
- **多环境支持**: 开发、测试、生产环境配置
- **CLI 工具**: 完整的命令行工具链

## 📦 核心组件架构

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

#### 依赖关系
```
loco-rs (主框架)
├── loco-gen (代码生成)
├── axum (Web 框架)
├── sea-orm (ORM)
├── tokio (异步运行时)
└── 其他核心依赖
```

### 2. **loco-gen** (代码生成器)
**版本**: 0.16.3 | **位置**: `/loco-gen`

#### 核心功能
- **组件生成**: 模型、控制器、迁移、脚手架
- **模板系统**: 基于 Tera 的代码模板
- **类型映射**: 智能类型推断和转换
- **部署配置**: Docker, Shuttle 等部署模板

#### 支持的生成器
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

#### 模板组织
```
templates/
├── controller/    # 控制器模板
├── model/         # 模型模板
├── migration/     # 迁移模板
├── scaffold/      # 脚手架模板
├── deployment/    # 部署模板
└── task/          # 任务模板
```

### 3. **loco-new** (项目生成器)
**版本**: 0.16.2 | **位置**: `/loco-new`

#### 应用模板类型
1. **SaaS App (Server Side Rendering)**
   - 完整的企业级应用模板
   - 服务端渲染，数据库 + 后台任务

2. **SaaS App (Client Side Rendering)**
   - 前后端分离架构
   - 客户端渲染，前端工程化

3. **REST API**
   - 纯 API 服务
   - 数据库 + 后台任务

4. **Lightweight Service**
   - 最小化服务
   - 无数据库，无后台任务

5. **Advanced**
   - 完全自定义配置
   - 最大灵活性

#### 交互式向导
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

## 🔧 技术栈

### 核心依赖
- **Web 框架**: Axum 0.8.1
- **异步运行时**: Tokio 1.45
- **ORM**: SeaORM 1.1.0
- **模板引擎**: Tera 1.19.1
- **序列化**: Serde 1.0
- **错误处理**: thiserror 1.0
- **日志**: tracing 0.1.40

### 可选特性
- **数据库**: `with-db` (SeaORM)
- **认证**: `auth_jwt` (JWT 支持)
- **CLI**: `cli` (命令行工具)
- **测试**: `testing` (测试工具)
- **后台任务**: `bg_redis`, `bg_pg`, `bg_sqlt`
- **存储**: `storage_aws_s3`, `storage_azure`, `storage_gcp`
- **缓存**: `cache_inmem`, `cache_redis`
- **MCP**: `mcp` (Model Context Protocol)

## 🚀 开发工作流

### 1. 项目创建
```bash
# 使用交互式向导
cargo install loco-cli
loco new

# 或直接使用模板
loco new --template saas-ssr
```

### 2. 开发周期
```bash
# 生成组件
cargo loco generate model user name:string email:string^
cargo loco generate scaffold post title:string content:text

# 运行开发服务器
cargo loco start

# 运行测试
cargo test
```

### 3. 部署准备
```bash
# 生成部署配置
cargo loco generate deployment docker

# 运行质量检查
cargo xtask test
```

## 📁 项目结构

```
loco/
├── src/                    # 主框架源码
│   ├── app.rs              # 应用核心结构
│   ├── boot.rs             # 启动管理
│   ├── controller/         # 控制器系统
│   ├── model/              # ORM 模型
│   ├── bgworker/           # 后台任务
│   ├── auth/               # 认证系统
│   ├── cache/              # 缓存系统
│   ├── mailer/             # 邮件系统
│   ├── storage/            # 存储系统
│   └── config.rs           # 配置管理
├── loco-gen/               # 代码生成器
├── loco-new/               # 项目生成器
├── xtask/                  # 开发任务管理
├── examples/               # 示例项目
├── tests/                  # 测试套件
└── docs/                   # 文档
```

## 🎨 设计模式

### 1. 生命周期模式
- **Hooks trait**: 应用生命周期管理
- **启动模式**: 多种启动配置
- **优雅关闭**: 信号处理和资源清理

### 2. 依赖注入模式
- **AppContext**: 应用状态容器
- **SharedStore**: 类型安全的数据存储
- **服务定位**: 统一的服务访问

### 3. 策略模式
- **存储后端**: 多种存储策略
- **缓存驱动**: 统一缓存接口
- **后台任务**: 多种队列后端

### 4. 模板方法模式
- **代码生成**: 标准化生成流程
- **控制器**: 统一的请求处理
- **中间件**: 可扩展的请求处理链

## 🛡️ 安全特性

### 1. 认证系统
- **JWT 支持**: HS512 算法
- **密码哈希**: Argon2 算法
- **API 密钥**: 自定义 API 密钥验证

### 2. 数据安全
- **SQL 注入防护**: SeaORM 参数化查询
- **XSS 防护**: 模板引擎自动转义
- **CSRF 保护**: 可选的 CSRF 中间件

### 3. 配置安全
- **环境变量**: 敏感信息环境变量管理
- **配置验证**: 配置项类型安全验证
- **默认安全**: 安全默认配置

## 📈 性能特性

### 1. 异步架构
- **Tokio 运行时**: 高性能异步处理
- **非阻塞 I/O**: 数据库、网络、文件操作
- **并发处理**: 请求并发处理

### 2. 资源管理
- **连接池**: 数据库连接池优化
- **缓存策略**: 多级缓存系统
- **内存管理**: 零拷贝和内存优化

### 3. 可扩展性
- **水平扩展**: 无状态设计支持
- **负载均衡**: 多实例部署支持
- **微服务**: 模块化架构支持

## 🔍 监控和日志

### 1. 日志系统
- **tracing**: 结构化日志记录
- **多级别**: DEBUG, INFO, WARN, ERROR
- **格式化**: JSON 和文本格式支持

### 2. 错误处理
- **统一错误类型**: 全框架错误类型
- **Backtrace 支持**: 调试信息保留
- **HTTP 错误**: 标准 HTTP 错误响应

### 3. 健康检查
- **内置检查**: 数据库连接、缓存状态
- **自定义检查**: 可扩展的健康检查
- **监控集成**: 与监控系统集成

## 🌐 国际化支持

### 1. i18n 模块
- **多语言**: 支持多种语言
- **本地化**: 日期、数字、货币格式化
- **翻译管理**: 翻译文件管理

### 2. 本地化特性
- **时区支持**: 多时区处理
- **字符编码**: UTF-8 全面支持
- **文化适应**: 文化特定的格式化

## 🔄 迁移和升级

### 1. 数据库迁移
- **自动迁移**: SeaORM 迁移系统
- **版本控制**: 迁移版本管理
- **回滚支持**: 迁移回滚功能

### 2. 框架升级
- **版本管理**: 语义化版本控制
- **向后兼容**: 保持 API 兼容性
- **迁移指南**: 详细的升级说明

## 📚 学习资源

### 1. 官方文档
- **用户指南**: 完整的使用指南
- **API 文档**: 详细的 API 参考
- **示例项目**: 实际应用示例

### 2. 社区支持
- **GitHub Issues**: 问题跟踪和讨论
- **Discussions**: 社区讨论和交流
- **贡献指南**: 如何贡献代码

### 3. 最佳实践
- **项目结构**: 推荐的项目组织方式
- **代码风格**: 统一的代码风格指南
- **测试策略**: 测试编写最佳实践

---

*这份速记文档提供了 Loco-rs 框架的全面概览，适合开发者快速了解框架的核心概念和功能特性。*
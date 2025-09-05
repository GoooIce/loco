# Loco DDD 使用指南

## 概述

Loco 框架现在支持 Domain-Driven Design (DDD) 模式，让您能够构建结构化、可维护的企业级应用程序。

## 当前状态

✅ **已完成**：
- 核心 DDD traits 编译成功
- 简化版 DDD 示例可正常运行
- 基本的命令查询分离模式

🚧 **进行中**：
- 完整的 DDD 库编译错误修复
- 高级功能（事件溯源、复杂聚合等）

## 快速开始

### 1. 运行基础示例

我们创建了一个可运行的基础 DDD 示例：

```bash
cd examples/ddd_basics
cargo run
```

示例输出：
```
=== Loco DDD 简化示例应用程序 ===

1. 创建用户示例
处理创建用户命令: CreateUserCommand { name: "张三", email: "zhangsan@example.com" }
用户创建成功: User { id: UserId("..."), name: "张三", email: "zhangsan@example.com", ... }
✅ 用户创建成功: ...

2. 查询用户示例
处理查询用户命令: GetUserByIdQuery { user_id: "..." }
✅ 找到用户: 模拟用户

3. 错误处理示例
处理创建用户命令: CreateUserCommand { name: "", email: "invalid-email" }
✅ 正确捕获错误: 用户名不能为空

=== DDD 示例完成 ===
```

### 2. 在您的项目中使用 DDD

创建新的 Rust 项目：

```bash
cargo new my_ddd_app --bin
cd my_ddd_app
```

配置 `Cargo.toml`：
```toml
[dependencies]
tokio = { version = "1.45", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
uuid = { version = "1.10", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
async-trait = "0.1"
```

### 3. 核心DDD 模式使用

#### 实体 (Entity)
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    id: UserId,
    name: String,
    email: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserId(String);

impl Entity for User {
    type Id = UserId;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn equals(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
```

#### 命令 (Command)
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserCommand {
    pub name: String,
    pub email: String,
}

impl Command for CreateUserCommand {
    type Result = User;
}
```

#### 命令处理器 (Command Handler)
```rust
pub struct CreateUserHandler;

impl Default for CreateUserHandler {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl CommandHandler<CreateUserCommand> for CreateUserHandler {
    async fn handle(&self, command: CreateUserCommand) -> Result<User, Box<dyn std::error::Error + Send + Sync>> {
        // 验证和处理逻辑
        if command.name.is_empty() {
            return Err("用户名不能为空".into());
        }
        
        let user = User::new(command.name, command.email);
        Ok(user)
    }
}
```

#### 查询 (Query)
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUserByIdQuery {
    pub user_id: String,
}

impl Query for GetUserByIdQuery {
    type Result = Option<User>;
}
```

#### 中介者模式 (Mediator)
```rust
let mediator = Mediator::new();

// 发送命令
let user = mediator.send_command::<CreateUserCommand, CreateUserHandler>(command).await?;

// 发送查询
let found_user = mediator.send_query::<GetUserByIdQuery, GetUserByIdHandler>(query).await?;
```

## DDD 模式说明

### 核心概念

- **Entity（实体）**: 具有唯一标识的领域对象
- **Value Object（值对象）**: 不可变的领域对象
- **Aggregate Root（聚合根）**: 管理聚合内部一致性
- **Repository（仓库）**: 封装持久化逻辑
- **Command（命令）**: 表示要执行的操作
- **Query（查询）**: 表示数据检索请求
- **Domain Event（领域事件）**: 领域中的重要事件
- **Mediator（中介者）**: 协调命令和查询的处理

### 优势

1. **分离关注点**: 命令和查询分离
2. **可测试性**: 每个组件都可以独立测试
3. **可维护性**: 清晰的架构分层
4. **可扩展性**: 易于添加新功能

## 下一步计划

1. **修复完整 DDD 库**: 解决 loco-ddd 中的编译错误
2. **集成到 Loco 主库**: 将 DDD 功能集成到 loco-rs 中
3. **添加更多示例**: 复杂聚合、事件溯源等
4. **文档完善**: 详细的 API 文档和最佳实践

## 示例项目结构

```
examples/
├── ddd_basics/              # 基础 DDD 示例（可运行）
│   ├── Cargo.toml
│   └── src/main.rs
└── ddd_usage.md             # 本文档
```

## 贡献

欢迎贡献代码和建议！请遵循以下步骤：

1. Fork 项目
2. 创建功能分支
3. 提交更改
4. 推送到分支
5. 创建 Pull Request

## 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件。

---

**注意**: 目前完整的 DDD 库（loco-ddd）还有一些编译错误需要修复。建议先使用简化版本来了解 DDD 模式的基本概念。
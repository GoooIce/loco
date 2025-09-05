# 简化 DDD 示例使用说明

## 概述

这个简化的 DDD (Domain-Driven Design) 示例展示了如何在 Rust 中实现核心的 DDD 模式，同时避免复杂的编译错误。

## 已解决的问题

1. **对象安全 (Object Safety)**：避免了 trait 对象中使用 async 方法
2. **静态变量初始化**：使用 `LazyLock` 正确处理运行时初始化
3. **类型擦除**：使用具体类型而非复杂的 trait 对象
4. **生命周期管理**：简化了生命周期约束

## 核心组件

### 1. 实体 (Entity)
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    id: String,
    name: String,
    email: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl Entity for User {
    type Id = String;
    // 实现 trait 方法...
}
```

### 2. 命令 (Command)
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserCommand {
    pub user_id: String,
    pub name: String,
    pub email: String,
}

impl Command for CreateUserCommand {
    type Result = String;
    // 实现 trait 方法...
}
```

### 3. 查询 (Query)
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUserQuery {
    pub user_id: String,
}

impl Query for GetUserQuery {
    type Result = Option<User>;
    // 实现 trait 方法...
}
```

### 4. 处理器 (Handlers)
```rust
pub struct CreateUserHandler {
    users: std::sync::Arc<tokio::sync::RwLock<Vec<User>>>,
}

#[async_trait::async_trait]
impl CommandHandler<CreateUserCommand> for CreateUserHandler {
    async fn handle(&self, command: CreateUserCommand) -> Result<String> {
        // 处理命令逻辑...
    }
}
```

### 5. Mediator 模式
```rust
pub struct Mediator {
    create_user_handler: Option<CreateUserHandler>,
    get_user_handler: Option<GetUserHandler>,
}

impl Mediator {
    pub async fn send_create_user_command(&self, command: CreateUserCommand) -> Result<String> {
        // 发送命令到处理器...
    }
    
    pub async fn send_get_user_query(&self, query: GetUserQuery) -> Result<Option<User>> {
        // 发送查询到处理器...
    }
}
```

## 运行示例

```bash
cd examples/simple_ddd_example
cargo run
```

## 输出示例

```
🎯 简化 DDD 示例启动...
📝 创建用户...
✅ 创建结果: User created successfully
🔍 获取用户...
✅ 找到用户: 张三 (zhangsan@example.com)
🧪 测试验证失败...
✅ 正确捕获错误: Name cannot be empty
🎉 DDD 示例完成！
```

## 关键设计决策

### 1. 避免复杂类型擦除
- 使用具体的处理器类型而非 trait 对象
- 避免了 `async fn` 在 trait 对象中的问题
- 提供了类型安全的 API

### 2. 简化生命周期
- 使用 `LazyLock` 处理静态变量
- 避免了复杂的生命周期注解
- 保持了代码的可读性

### 3. 错误处理
- 使用统一的 `Result<T>` 类型
- 简化了错误传播
- 提供了清晰的错误信息

### 4. 并发安全
- 使用 `Arc<RwLock<Vec<User>>>` 共享状态
- 确保线程安全
- 支持异步操作

## 扩展指南

要扩展这个示例，可以：

1. **添加新的实体类型**：创建新的 struct 并实现 `Entity` trait
2. **添加新的命令**：创建新的命令类型和对应的处理器
3. **添加新的查询**：创建新的查询类型和对应的处理器
4. **添加持久化**：将内存存储替换为数据库
5. **添加事件**：实现事件发布/订阅模式

## 学习要点

这个示例展示了：
- 如何在 Rust 中实现 DDD 模式
- 如何避免常见的编译错误
- 如何设计类型安全的 API
- 如何处理异步操作和并发

这个简化的实现为更复杂的 DDD 应用提供了坚实的基础。
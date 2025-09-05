use serde::{Deserialize, Serialize};
use async_trait::async_trait;

// 简化的 DDD traits
pub trait Entity: Send + Sync {
    type Id: Send + Sync + Clone + PartialEq;
    
    fn id(&self) -> &Self::Id;
    fn equals(&self, other: &Self) -> bool;
}

pub trait Command: Send + Sync {
    type Result: Send + Sync;
}

pub trait Query: Send + Sync {
    type Result: Send + Sync;
}

#[async_trait]
pub trait CommandHandler<C: Command>: Send + Sync {
    async fn handle(&self, command: C) -> Result<C::Result, Box<dyn std::error::Error + Send + Sync>>;
}

#[async_trait]
pub trait QueryHandler<Q: Query>: Send + Sync {
    async fn handle(&self, query: Q) -> Result<Q::Result, Box<dyn std::error::Error + Send + Sync>>;
}

// 简化的中介者
pub struct Mediator {
    // 在实际实现中，这里会有命令和查询处理器的注册表
}

impl Mediator {
    pub fn new() -> Self {
        Self {}
    }
    
    pub async fn send_command<C: Command, H: CommandHandler<C> + Default>(
        &self,
        command: C,
    ) -> Result<C::Result, Box<dyn std::error::Error + Send + Sync>> {
        let handler = H::default();
        handler.handle(command).await
    }
    
    pub async fn send_query<Q: Query, H: QueryHandler<Q> + Default>(
        &self,
        query: Q,
    ) -> Result<Q::Result, Box<dyn std::error::Error + Send + Sync>> {
        let handler = H::default();
        handler.handle(query).await
    }
}

// 用户ID值对象
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserId(String);

impl UserId {
    pub fn new(id: String) -> Self {
        UserId(id)
    }
    
    pub fn as_string(&self) -> String {
        self.0.clone()
    }
}

// 用户实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    id: UserId,
    name: String,
    email: String,
    created_at: chrono::DateTime<chrono::Utc>,
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

impl User {
    pub fn new(name: String, email: String) -> Self {
        Self {
            id: UserId(uuid::Uuid::new_v4().to_string()),
            name,
            email,
            created_at: chrono::Utc::now(),
        }
    }
    
    pub fn name(&self) -> &str {
        &self.name
    }
    
    pub fn email(&self) -> &str {
        &self.email
    }
}

// 创建用户命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserCommand {
    pub name: String,
    pub email: String,
}

impl Command for CreateUserCommand {
    type Result = User;
}

// 创建用户命令处理器
pub struct CreateUserHandler;

impl Default for CreateUserHandler {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl CommandHandler<CreateUserCommand> for CreateUserHandler {
    async fn handle(&self, command: CreateUserCommand) -> Result<User, Box<dyn std::error::Error + Send + Sync>> {
        println!("处理创建用户命令: {:?}", command);
        
        // 验证命令
        if command.name.is_empty() {
            return Err("用户名不能为空".into());
        }
        
        if !command.email.contains('@') {
            return Err("邮箱格式不正确".into());
        }
        
        // 创建用户实体
        let user = User::new(command.name, command.email);
        
        println!("用户创建成功: {:?}", user);
        Ok(user)
    }
}

// 根据ID查询用户
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUserByIdQuery {
    pub user_id: String,
}

impl Query for GetUserByIdQuery {
    type Result = Option<User>;
}

// 查询用户处理器
pub struct GetUserByIdHandler;

impl Default for GetUserByIdHandler {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl QueryHandler<GetUserByIdQuery> for GetUserByIdHandler {
    async fn handle(&self, query: GetUserByIdQuery) -> Result<Option<User>, Box<dyn std::error::Error + Send + Sync>> {
        println!("处理查询用户命令: {:?}", query);
        
        // 模拟从数据库查询
        // 在实际应用中，这里会查询数据库或其他存储
        let user_id = UserId(query.user_id);
        
        // 创建一个模拟用户用于演示
        let user = User {
            id: user_id,
            name: "模拟用户".to_string(),
            email: "simulated@example.com".to_string(),
            created_at: chrono::Utc::now(),
        };
        
        println!("查询到用户: {:?}", user);
        Ok(Some(user))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("=== Loco DDD 简化示例应用程序 ===\n");
    
    // 创建中介者
    let mediator = Mediator::new();
    
    // 示例1: 创建用户
    println!("1. 创建用户示例");
    let create_command = CreateUserCommand {
        name: "张三".to_string(),
        email: "zhangsan@example.com".to_string(),
    };
    
    match mediator.send_command::<CreateUserCommand, CreateUserHandler>(create_command).await {
        Ok(user) => {
            println!("✅ 用户创建成功: {}\n", user.id.as_string());
            
            // 示例2: 查询用户
            println!("2. 查询用户示例");
            let query = GetUserByIdQuery {
                user_id: user.id.as_string(),
            };
            
            match mediator.send_query::<GetUserByIdQuery, GetUserByIdHandler>(query).await {
                Ok(found_user) => {
                    match found_user {
                        Some(user) => println!("✅ 找到用户: {}", user.name()),
                        None => println!("❌ 未找到用户"),
                    }
                }
                Err(e) => println!("❌ 查询失败: {}", e),
            }
        }
        Err(e) => println!("❌ 用户创建失败: {}", e),
    }
    
    // 示例3: 错误处理
    println!("\n3. 错误处理示例");
    let invalid_command = CreateUserCommand {
        name: "".to_string(), // 空用户名
        email: "invalid-email".to_string(), // 无效邮箱
    };
    
    match mediator.send_command::<CreateUserCommand, CreateUserHandler>(invalid_command).await {
        Ok(_) => println!("❌ 应该失败但成功了"),
        Err(e) => println!("✅ 正确捕获错误: {}", e),
    }
    
    println!("\n=== DDD 示例完成 ===");
    
    Ok(())
}
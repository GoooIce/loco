// 简化的 DDD 核心类型
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

// 版本类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    value: u32,
}

impl Version {
    pub fn new(value: u32) -> Self {
        Self { value }
    }
    
    pub fn value(&self) -> u32 {
        self.value
    }
}

// 时间戳类型
#[derive(Debug, Clone)]
pub struct DomainTimestamp {
    value: chrono::DateTime<chrono::Utc>,
}

impl DomainTimestamp {
    pub fn new(value: chrono::DateTime<chrono::Utc>) -> Self {
        Self { value }
    }
    
    pub fn value(&self) -> &chrono::DateTime<chrono::Utc> {
        &self.value
    }
}

// 实体 trait
pub trait Entity: Send + Sync {
    type Id: Send + Sync + std::fmt::Debug + Clone + PartialEq + Eq;
    
    fn id(&self) -> &Self::Id;
    fn validate(&self) -> Result<()>;
    fn version(&self) -> &Version;
    fn created_at(&self) -> &DomainTimestamp;
    fn updated_at(&self) -> &DomainTimestamp;
}

// 命令 trait
pub trait Command: Send + Sync + Clone {
    type Result: Send + Sync;
    
    fn command_id(&self) -> &str;
    fn command_type(&self) -> &str;
    fn timestamp(&self) -> &chrono::DateTime<chrono::Utc>;
}

// 查询 trait
pub trait Query: Send + Sync + Clone {
    type Result: Send + Sync;
    
    fn query_id(&self) -> &str;
    fn query_type(&self) -> &str;
    fn timestamp(&self) -> &chrono::DateTime<chrono::Utc>;
}

// 命令处理器 trait
#[async_trait::async_trait]
pub trait CommandHandler<C: Command>: Send + Sync {
    async fn handle(&self, command: C) -> Result<C::Result>;
}

// 查询处理器 trait
#[async_trait::async_trait]
pub trait QueryHandler<Q: Query>: Send + Sync {
    async fn handle(&self, query: Q) -> Result<Q::Result>;
}

// 简化的 Mediator 实现 - 使用具体类型而非 trait 对象
pub struct Mediator {
    // 使用具体的处理器类型，避免 trait 对象
    create_user_handler: Option<CreateUserHandler>,
    get_user_handler: Option<GetUserHandler>,
}

impl Mediator {
    pub fn new() -> Self {
        Self {
            create_user_handler: None,
            get_user_handler: None,
        }
    }
    
    pub fn register_create_user_handler(&mut self, handler: CreateUserHandler) {
        self.create_user_handler = Some(handler);
    }
    
    pub fn register_get_user_handler(&mut self, handler: GetUserHandler) {
        self.get_user_handler = Some(handler);
    }
    
    pub async fn send_create_user_command(&self, command: CreateUserCommand) -> Result<String> {
        if let Some(handler) = &self.create_user_handler {
            handler.handle(command).await
        } else {
            Err("Create user handler not registered".into())
        }
    }
    
    pub async fn send_get_user_query(&self, query: GetUserQuery) -> Result<Option<User>> {
        if let Some(handler) = &self.get_user_handler {
            handler.handle(query).await
        } else {
            Err("Get user handler not registered".into())
        }
    }
}

impl Default for Mediator {
    fn default() -> Self {
        Self::new()
    }
}

// 用户实体定义
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct User {
    id: String,
    name: String,
    email: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl User {
    pub fn new(id: String, name: String, email: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            id,
            name,
            email,
            created_at: now,
            updated_at: now,
        }
    }
    
    pub fn name(&self) -> &str {
        &self.name
    }
    
    pub fn email(&self) -> &str {
        &self.email
    }
}

impl Entity for User {
    type Id = String;
    
    fn id(&self) -> &Self::Id {
        &self.id
    }
    
    fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".into());
        }
        if self.email.is_empty() {
            return Err("Email cannot be empty".into());
        }
        if !self.email.contains('@') {
            return Err("Email must be valid".into());
        }
        Ok(())
    }
    
    fn version(&self) -> &Version {
        static VERSION: std::sync::LazyLock<Version> = std::sync::LazyLock::new(|| Version::new(1));
        &VERSION
    }
    
    fn created_at(&self) -> &DomainTimestamp {
        static TIMESTAMP: std::sync::LazyLock<DomainTimestamp> = std::sync::LazyLock::new(|| DomainTimestamp::new(chrono::Utc::now()));
        &TIMESTAMP
    }
    
    fn updated_at(&self) -> &DomainTimestamp {
        static TIMESTAMP: std::sync::LazyLock<DomainTimestamp> = std::sync::LazyLock::new(|| DomainTimestamp::new(chrono::Utc::now()));
        &TIMESTAMP
    }
}

// 创建用户命令
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateUserCommand {
    pub user_id: String,
    pub name: String,
    pub email: String,
}

impl Command for CreateUserCommand {
    type Result = String;
    
    fn command_id(&self) -> &str {
        "create-user"
    }
    
    fn command_type(&self) -> &str {
        "CreateUser"
    }
    
    fn timestamp(&self) -> &chrono::DateTime<chrono::Utc> {
        static TIMESTAMP: std::sync::LazyLock<chrono::DateTime<chrono::Utc>> = std::sync::LazyLock::new(|| chrono::Utc::now());
        &TIMESTAMP
    }
}

// 获取用户查询
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GetUserQuery {
    pub user_id: String,
}

impl Query for GetUserQuery {
    type Result = Option<User>;
    
    fn query_id(&self) -> &str {
        "get-user"
    }
    
    fn query_type(&self) -> &str {
        "GetUser"
    }
    
    fn timestamp(&self) -> &chrono::DateTime<chrono::Utc> {
        static TIMESTAMP: std::sync::LazyLock<chrono::DateTime<chrono::Utc>> = std::sync::LazyLock::new(|| chrono::Utc::now());
        &TIMESTAMP
    }
}

// 创建用户命令处理器
pub struct CreateUserHandler {
    users: std::sync::Arc<tokio::sync::RwLock<Vec<User>>>,
}

impl CreateUserHandler {
    pub fn new(users: std::sync::Arc<tokio::sync::RwLock<Vec<User>>>) -> Self {
        Self { users }
    }
}

#[async_trait::async_trait]
impl CommandHandler<CreateUserCommand> for CreateUserHandler {
    async fn handle(&self, command: CreateUserCommand) -> Result<String> {
        let user = User::new(command.user_id, command.name, command.email);
        
        // 验证用户
        user.validate()?;
        
        // 存储用户
        let mut users = self.users.write().await;
        users.push(user);
        
        Ok("User created successfully".to_string())
    }
}

// 获取用户查询处理器
pub struct GetUserHandler {
    users: std::sync::Arc<tokio::sync::RwLock<Vec<User>>>,
}

impl GetUserHandler {
    pub fn new(users: std::sync::Arc<tokio::sync::RwLock<Vec<User>>>) -> Self {
        Self { users }
    }
}

#[async_trait::async_trait]
impl QueryHandler<GetUserQuery> for GetUserHandler {
    async fn handle(&self, query: GetUserQuery) -> Result<Option<User>> {
        let users = self.users.read().await;
        Ok(users.iter().find(|u| u.id == query.user_id).cloned())
    }
}
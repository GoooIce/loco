use simple_ddd_example::*;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🎯 简化 DDD 示例启动...");
    
    // 创建共享用户存储
    let users = std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new()));
    
    // 创建 Mediator
    let mut mediator = Mediator::new();
    
    // 注册处理器
    mediator.register_create_user_handler(CreateUserHandler::new(users.clone()));
    mediator.register_get_user_handler(GetUserHandler::new(users.clone()));
    
    // 测试创建用户命令
    println!("📝 创建用户...");
    let create_command = CreateUserCommand {
        user_id: "user-123".to_string(),
        name: "张三".to_string(),
        email: "zhangsan@example.com".to_string(),
    };
    
    let result = mediator.send_create_user_command(create_command).await?;
    println!("✅ 创建结果: {}", result);
    
    // 测试获取用户查询
    println!("🔍 获取用户...");
    let get_query = GetUserQuery {
        user_id: "user-123".to_string(),
    };
    
    let user = mediator.send_get_user_query(get_query).await?;
    match user {
        Some(user) => println!("✅ 找到用户: {} ({})", user.name(), user.email()),
        None => println!("❌ 用户未找到"),
    }
    
    // 测试验证失败的情况
    println!("🧪 测试验证失败...");
    let invalid_command = CreateUserCommand {
        user_id: "user-456".to_string(),
        name: "".to_string(), // 空名字，应该失败
        email: "invalid-email".to_string(), // 无效邮箱，应该失败
    };
    
    match mediator.send_create_user_command(invalid_command).await {
        Ok(_) => println!("❌ 应该失败但却成功了"),
        Err(e) => println!("✅ 正确捕获错误: {}", e),
    }
    
    println!("🎉 DDD 示例完成！");
    
    Ok(())
}
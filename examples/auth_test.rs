use dotenv::dotenv;
use std::env;
use notes_sync_server::auth;
use notes_sync_server::database::Database;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    // 初始化数据库连接
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = Database::new(&database_url).await;

    // 清理测试数据
    sqlx::query(
        "DELETE FROM users WHERE email LIKE 'test%@example.com'"
    )
    .execute(db.db()).await?;

    // 创建认证服务
    let auth_service = auth::service::AuthService::new(db);

    // 测试注册
    let register_request = auth::model::RegisterRequest {
        name: "test".to_string(),
        email: "test1@example.com".to_string(),
        password: "password123".to_string(),
    };

    let user = auth_service.register_user(register_request).await.expect("Register user failed");
    println!("Registered user: {:?}", user);

    // 测试重复注册
    let duplicate_register = auth::model::RegisterRequest {
        name: "test1".to_string(),
        email: " test1@example.com".to_string(),
        password: "password123".to_string(),
    };

    match auth_service.register_user(duplicate_register).await {
        Ok(_) => panic!("Should not allow duplicate registration"),
        Err(e) => println!("Expected error: {}", e),
    }

    // 测试登录
    let login_request = auth::model::LoginRequest {
        email: "test1@example.com".to_string(),
        password: "password123".to_string(),
    };

    let authenticated_user = auth_service.authenticate(
        &login_request.email, 
        &login_request.password
    ).await.expect("Authenticate failed");

    println!("Authenticated user: {:?}", authenticated_user);

    // 测试错误密码
    let wrong_password = auth::model::LoginRequest {
        email: "test1@example.com".to_string(),
        password: "wrongpassword".to_string(),
    };

    match auth_service.authenticate(&wrong_password.email, &wrong_password.password).await {
        Ok(_) => panic!("Should not authenticate with wrong password"),
        Err(e) => println!("Expected error: {}", e),
    }

    // 测试token生成和验证
    let token = auth_service.generate_token(&user.id).expect("Failed to generate token");
    println!("Generated token: {}", token);

    let claims = auth_service.validate_token(&token).await.expect("failed to validate token");
    println!("Token claims: {:?}", claims);

    // 测试获取用户
    let fetched_user = auth_service.get_user_by_id(&user.id).await.expect("failed to get user");
    println!("Fetched user: {:?}", fetched_user);

    Ok(())
}
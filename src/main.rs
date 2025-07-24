use actix_web::{web, App, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;
use dotenv::dotenv;
use std::env;

use notes_sync_server::{api, auth, middleware, sync};
use notes_sync_server::database::Database;
use notes_sync_server::utils::logging;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    // 初始化日志
    let _log_guard = logging::init_logging();
    tracing::info!("Starting notes server...");

    // 数据库连接池
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = Database::new(&database_url).await;
    db.migrate().await;

    // 初始化同步服务
    let sync_service = web::Data::new(sync::service::SyncService::new(db.clone()));

    // 初始化认证服务
    let auth_service = web::Data::new(auth::service::AuthService::new(db));

    let auth_service_clone = auth_service.clone();
    tokio::spawn(async move {
        auth::service::cleanup_expired_tokens(auth_service_clone).await
    });

    tracing::info!("Running api service");
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::logging::EnhancedLogging)
            .app_data(auth_service.clone())  
            .app_data(sync_service.clone())
            // 公开路由
            .service(
                web::scope("/api/auth")
                    .service(api::auth::register)
                    .service(api::auth::login)
            )
            // 受保护路由
            .service(
                web::scope("/api")
                    .wrap(HttpAuthentication::bearer(middleware::auth::validator))
                    .service(api::auth::get_me)
                    .service(api::auth::logout)
                    .configure(api::sync::configure)
            )
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
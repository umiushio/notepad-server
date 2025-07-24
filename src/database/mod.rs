pub(crate) mod auth_db;
pub(crate) use auth_db::AuthDatabase;
pub(crate) mod sync_db;
pub(crate) use sync_db::SyncDatabase;

use sqlx::{postgres::PgPoolOptions, PgPool};

#[derive(Debug, Clone)]
pub struct Database {
    db: PgPool,
}

impl Database {
    pub async fn new(url: &str) -> Self {
        let db = PgPoolOptions::new()
            .max_connections(5)
            .connect(url)
            .await
            .expect("Failed to create pool");

        let schema = sqlx::query("SELECT current_schema()").fetch_one(&db).await.unwrap();
        println!("current schema: {:?}", schema);

        Self { db }
    }

    // 运行数据库迁移
    pub async fn migrate(&self) {
        sqlx::migrate!()
            .run(&self.db)
            .await
            .expect("Failed to run migrations");
    }

    // 测试专用
    pub fn db(&self) -> &PgPool {
        &self.db
    }

    // 清理过期token
    pub async fn cleanup_expired_tokens(&self) {
        sqlx::query(
            "DELETE FROM blacklisted_tokens WHERE expires_at < NOW()"
        )
        .execute(&self.db)
        .await.expect("Failed to cleanup expired tokens");
    }
    
}
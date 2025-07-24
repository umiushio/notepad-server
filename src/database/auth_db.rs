use super::Database;
use crate::auth::{error::AuthError, model::User};

pub(crate) trait AuthDatabase {
    async fn get_user_by_email(&self, email: &str) -> Result<User, AuthError>;
    async fn get_user_by_id(&self, id: &str) -> Result<User, AuthError>;
    async fn insert_user(&self, id: &str, name: &str, email: &str, password_hash: &str) -> Result<User, AuthError>;
    async fn is_token_blacklisted(&self, token: &str) -> Result<bool, AuthError>;
    async fn logout_user(&self, token: &str, expires_at: chrono::DateTime<chrono::Utc>) -> Result<(), AuthError>;
}

impl AuthDatabase for Database {
    async fn get_user_by_email(&self, email: &str) -> Result<User, AuthError> {
        sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE email = $1"
        )
        .bind(email)
        .fetch_one(&self.db)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AuthError::UserNotFound,
            _ => AuthError::DatabaseError(e),
        })
    }

    async fn get_user_by_id(&self, id: &str) -> Result<User, AuthError> {
        sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = $1"
        )
        .bind(id)
        .fetch_one(&self.db)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AuthError::UserNotFound,
            _ => AuthError::DatabaseError(e),
        })
    }

    async fn insert_user(&self, id: &str, name: &str, email: &str, password_hash: &str) -> Result<User, AuthError> {
        let now = chrono::Utc::now();
        sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, user_name, email, password_hash, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(email)
        .bind(password_hash)
        .bind(now)
        .bind(now)
        .fetch_one(&self.db)
        .await
        .map_err(|e| AuthError::DatabaseError(e))
    }

    async fn logout_user(&self, token: &str, expires_at: chrono::DateTime<chrono::Utc>) -> Result<(), AuthError> {
        sqlx::query(
            "INSERT INTO blacklisted_tokens (token, expires_at) VALUES ($1, $2)"
        )
        .bind(token)
        .bind(expires_at)
        .execute(&self.db)
        .await?;
        Ok(())
    }

    async fn is_token_blacklisted(&self, token: &str) -> Result<bool, AuthError> {
        let exists = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM blacklisted_tokens WHERE token = $1)"
        )
        .bind(token)
        .fetch_one(&self.db)
        .await?;

        Ok(exists)
    }
}
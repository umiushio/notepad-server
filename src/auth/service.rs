use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use std::env;
use super::error::AuthError;
use crate::database::{AuthDatabase, Database};
use super::model::{User, RegisterRequest, Claims};

pub struct AuthService {
    jwt_secret: String,
    jwt_expiry: i64,    // 小时
    db: Database,
}

impl AuthService {
    pub fn new(db: Database) -> Self {
        let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let jwt_expiry = env::var("JWT_EXPIRY_HOURS")
            .unwrap_or_else(|_| "24".to_string())
            .parse()
            .unwrap_or(24);

        Self { jwt_secret, jwt_expiry, db }
    }

    // 注册用户
    pub async fn register_user(&self, request: RegisterRequest) -> Result<User, AuthError> {
        // 检查用户是否已经存在
        if let Ok(_) = self.db.get_user_by_email(&request.email).await {
            return Err(AuthError::UserExists);
        }

        let name = if request.name.chars().count() > 10 {
            request.name.chars().take(10).collect::<String>()
        } else {
            request.name.clone()
        };
        
        let password_hash = self.hash_password(&request.password)?;
        let user_id = uuid::Uuid::new_v4().to_string();

        self.db.insert_user(&user_id, &name, &request.email, &password_hash).await
    }

    // 用户认证
    pub async fn authenticate(&self, email: &str, password: &str) -> Result<User, AuthError> {
        let user = self.db.get_user_by_email(email).await?;

        self.verify_password(&user.password_hash, password)?;

        Ok(user)
    }

    pub async fn get_user_by_id(&self, user_id: &str) -> Result<User, AuthError> {
        self.db.get_user_by_id(user_id).await
    }

    // 密码哈希
    pub fn hash_password(&self, password: &str) -> Result<String, AuthError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        Ok(argon2.hash_password(password.as_bytes(), &salt)?.to_string())
    }

    // 校验密码
    pub fn verify_password(&self, hash: &str, password: &str) -> Result<(), AuthError> {
        let argon2 = Argon2::default();
        let hash = PasswordHash::new(hash)?;
        if let Err(_) = argon2.verify_password(password.as_bytes(), &hash) {
            Err(AuthError::InvalidCredentials)
        } else {
            Ok(())
        }
    }

    pub fn generate_token(&self, user_id: &str) -> Result<String, AuthError> {
        let now = chrono::Utc::now();
        let exp = now + chrono::Duration::hours(self.jwt_expiry);

        let claims = Claims {
            sub: user_id.to_owned(),
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
        };

        Ok(jsonwebtoken::encode(
            &Header::default(), 
            &claims, 
            &EncodingKey::from_secret(self.jwt_secret.as_ref())
        )?)
    }

    pub async fn validate_token(&self, token: &str) -> Result<Claims, AuthError> {
        // 先检查黑名单
        if self.db.is_token_blacklisted(token).await? {
            println!("Token is blacklisted");
            return Err(AuthError::Unauthorized);
        }
        
        Ok(jsonwebtoken::decode::<Claims>(
            token, 
            &DecodingKey::from_secret(self.jwt_secret.as_ref()), 
            &Validation::default(),
        )?.claims)
    }

    pub async fn logout(&self, token: &str) -> Result<(), AuthError> {
        let expires_at = chrono::Utc::now() + chrono::Duration::hours(self.jwt_expiry);
        Ok(self.db.logout_user(token, expires_at).await?)
    }
}

pub async fn cleanup_expired_tokens(auth_service: actix_web::web::Data<AuthService>) {
    // 每小时清理一次过期token
    loop {
        auth_service.db.cleanup_expired_tokens().await;
        tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
    }
}
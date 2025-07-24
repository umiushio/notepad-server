use actix_web::{get, post, web, HttpResponse, Responder};
use serde_json::json;

use crate::{auth::{error::AuthError, model::{AuthResponse, LoginRequest, RegisterRequest}, service::AuthService}, middleware::AuthToken};
use crate::log_error;

#[post("/register")]
pub async fn register(
    auth_service: web::Data<AuthService>,
    credentials: web::Json<RegisterRequest>,
) -> Result<impl Responder, AuthError> {
    tracing::info!("Starting user register");

    match auth_service.register_user(credentials.into_inner()).await {
        Ok(user) => {
            match auth_service.generate_token(&user.id) {
                Ok(token) => {
                    tracing::info!("User signed up successfully");
                    Ok(HttpResponse::Ok().json(AuthResponse {
                        token,
                        user_id: user.id,
                        user_name: user.user_name
                    }))
                }
                Err(e) => {
                    log_error!(e, "Failed to generate token");
                    Err(e)
                }
            }
        }
        Err(e) => {
            log_error!(e, "Failed to register user");
            Err(e)
        }
    }
}

#[get("/login")]
pub async fn login(
    auth_service: web::Data<AuthService>,
    credentials: web::Json<LoginRequest>,
) -> Result<impl Responder, AuthError> {
    tracing::info!("Starting user login");

    match auth_service.authenticate(&credentials.email, &credentials.password).await {
        Ok(user) => {
            match auth_service.generate_token(&user.id) {
                Ok(token) => {
                    tracing::info!("User signed in successfully");
                    Ok(HttpResponse::Ok().json(AuthResponse {
                        token,
                        user_id: user.id,
                        user_name: user.user_name,
                    }))
                }
                Err(e) => {
                    log_error!(e, "Failed to generate token");
                    Err(e)
                }
            }
        }
        Err(e) => {
            log_error!(e, "Failed to sign in");
            Err(e)
        }
    }
}

#[get("/me")]
pub async fn get_me(
    auth_service: web::Data<AuthService>,
    user_id: web::ReqData<String>,      // 从中间件获取的用户ID
) -> Result<impl Responder, AuthError> {
    tracing::info!("Starting getting me");

    match auth_service.get_user_by_id(&user_id.into_inner()).await {
        Ok(user) => {
            tracing::info!(user_id = %user.id, "User get me successfully");
            Ok(HttpResponse::Ok().json(user))
        }
        Err(e) => {
            log_error!(e, "Failed to get me");
            Err(e)
        }
    }
}

#[post("/logout")]
pub async fn logout(
    auth_service: web::Data<AuthService>,
    token: web::ReqData<AuthToken>,
) -> Result<impl Responder, AuthError> {
    tracing::info!("Starting user logout");

    match auth_service.logout(token.token()).await {
        Ok(_) => {
            tracing::info!(token = %token.token(), "User signed out successfully");
            Ok(HttpResponse::Ok().json(json!({"message": "Logout successful"})))
        }
        Err(e) => {
            log_error!(e, "Failed to logout");
            Err(e)
        }
    }
}
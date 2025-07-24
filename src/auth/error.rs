use derive_more::Display;
use jsonwebtoken::errors::Error as JwtError;
use argon2::password_hash::Error as ArgonError;
use sqlx::Error as SqlxError;
use actix_web::{HttpResponse, ResponseError};

#[derive(Debug, Display)]
pub enum AuthError {
    #[display("Invalid credentials")]
    InvalidCredentials,

    #[display("JWT creation error: {}", _0)]
    JwtCreationError(JwtError),

    #[display("JWT validation error: {}", _0)]
    JwtValidationError(JwtError),

    #[display("Password hashing error: {}", _0)]
    PasswordHashingError(ArgonError),

    #[display("Database error: {}", _0)]
    DatabaseError(SqlxError),

    #[display("User not found")]
    UserNotFound,

    #[display("User already exists")]
    UserExists,

    #[display("Unauthorized")]
    Unauthorized,
}

impl ResponseError for AuthError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AuthError::InvalidCredentials => HttpResponse::Unauthorized().json("Invalid credentials"),
            AuthError::JwtCreationError(_) => HttpResponse::InternalServerError().json("Token creation failed"),
            AuthError::JwtValidationError(_) => HttpResponse::Unauthorized().json("Invalid token"),
            AuthError::PasswordHashingError(_) => HttpResponse::InternalServerError().json("Password processing failed"),
            AuthError::DatabaseError(_) => HttpResponse::InternalServerError().json("Database operation failed"),
            AuthError::UserNotFound => HttpResponse::NotFound().json("User not found"),
            AuthError::UserExists => HttpResponse::Conflict().json("User already exists"),
            AuthError::Unauthorized => HttpResponse::Unauthorized().finish(),
        }
    }
}

impl From<JwtError> for AuthError {
    fn from(err: JwtError) -> Self {
        AuthError::JwtValidationError(err)
    }
}

impl From<ArgonError> for AuthError {
    fn from(err: ArgonError) -> Self {
        AuthError::PasswordHashingError(err)
    }
}

impl From<SqlxError> for AuthError {
    fn from(err: SqlxError) -> Self {
        AuthError::DatabaseError(err)
    }
}
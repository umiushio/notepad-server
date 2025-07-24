use derive_more::Display;
// use jsonwebtoken::errors::Error as JwtError;
// use argon2::password_hash::Error as ArgonError;
use sqlx::Error as SqlxError;
use actix_web::{HttpResponse, ResponseError};

#[derive(Debug, Display)]
pub enum SyncError {
    #[display("Invalid credentials")]
    InvalidCredentials,

    // #[display("JWT creation error: {}", _0)]
    // JwtCreationError(JwtError),

    // #[display("JWT validation error: {}", _0)]
    // JwtValidationError(JwtError),

    // #[display("Password hashing error: {}", _0)]
    // PasswordHashingError(ArgonError),

    #[display("Database error: {}", _0)]
    DatabaseError(SqlxError),

    // #[display("User not found")]
    // UserNotFound,

    // #[display("User already exists")]
    // UserExists,

    #[display("Unauthorized")]
    Unauthorized,
}

impl ResponseError for SyncError {
    fn error_response(&self) -> HttpResponse {
        match self {
            SyncError::InvalidCredentials => HttpResponse::Unauthorized().json("Invalid credentials"),
            // SyncError::JwtCreationError(_) => HttpResponse::InternalServerError().json("Token creation failed"),
            // SyncError::JwtValidationError(_) => HttpResponse::Unauthorized().json("Invalid token"),
            // SyncError::PasswordHashingError(_) => HttpResponse::InternalServerError().json("Password processing failed"),
            SyncError::DatabaseError(_) => HttpResponse::InternalServerError().json("Database operation failed"),
            // SyncError::UserNotFound => HttpResponse::NotFound().json("User not found"),
            // SyncError::UserExists => HttpResponse::Conflict().json("User already exists"),
            SyncError::Unauthorized => HttpResponse::Unauthorized().finish(),
        }
    }
}

// impl From<JwtError> for SyncError {
//     fn from(err: JwtError) -> Self {
//         SyncError::JwtValidationError(err)
//     }
// }

// impl From<ArgonError> for SyncError {
//     fn from(err: ArgonError) -> Self {
//         SyncError::PasswordHashingError(err)
//     }
// }

impl From<SqlxError> for SyncError {
    fn from(err: SqlxError) -> Self {
        SyncError::DatabaseError(err)
    }
}
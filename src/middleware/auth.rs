use actix_web::{web, dev::ServiceRequest, Error, HttpMessage};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use crate::{auth::service::AuthService, log_error};
use super::AuthToken;


pub async fn validator(
    req: ServiceRequest,        // 传入的请求
    credentials: BearerAuth,    // 提取的Bearer Token
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    tracing::debug!(token = %credentials.token(), "Starting validating");
    
    let auth_service = req.app_data::<web::Data<AuthService>>()
        .expect("AuthService not found in app data");
    let token = credentials.token().trim_matches('"');
    // validate_token 内部会检查黑名单
    match auth_service.validate_token(token).await {
        Ok(claims) => {
            tracing::info!(user = %claims.sub, "Token validated successfully");
            req.extensions_mut().insert(claims.sub);
            req.extensions_mut().insert(AuthToken(token.to_string()));
            Ok(req)
        }
        Err(e) => {
            log_error!(e, "Failed to validate token");
            Err((Error::from(e), req))
        }
    }
}
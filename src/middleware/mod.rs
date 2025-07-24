pub mod auth;
pub mod logging;

#[derive(Debug, Clone)]
pub struct AuthToken(String);

impl AuthToken {
    pub fn token(&self) -> &str {
        &self.0
    }
}
use async_trait::async_trait;
use crate::access::local_repo::{DbUserRepository, UserRepository};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, PartialEq)]
pub enum ManagerError {
    InvalidCredentials,
    DatabaseError,
    TokenError,
}

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait Manager: Send + Sync {
    async fn login(&self, username: &str) -> Result<(String, String), ManagerError>;
}

pub struct AppManager {
    pub user_repo: Arc<DbUserRepository>,
    pub jwt_secret: String,
}

impl AppManager {
    pub fn new(user_repo: Arc<DbUserRepository>, jwt_secret: String) -> Self {
        Self {
            user_repo,
            jwt_secret,
        }
    }

    fn create_jwt(
        username: &str,
        secret: &str,
        expiration_seconds: i64,
    ) -> Result<String, ManagerError> {
        let expiration = chrono::Utc::now().timestamp() + expiration_seconds;

        let claims = Claims {
            sub: username.to_owned(),
            exp: expiration as usize,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_ref()),
        )
        .map_err(|_| ManagerError::TokenError)
    }
}

#[async_trait]
impl Manager for AppManager {
    async fn login(&self, username: &str) -> Result<(String, String), ManagerError> {
        let user_result = self.user_repo.get_user_by_username(username).await;

        match user_result {
            Ok(Some(user)) => {
                let access_token = Self::create_jwt(&user.username, &self.jwt_secret, 3600)?;
                let refresh_token = Self::create_jwt(&user.username, &self.jwt_secret, 86400 * 7)?;

                Ok((access_token, refresh_token))
            }
            Ok(None) => Err(ManagerError::InvalidCredentials),
            Err(_) => Err(ManagerError::DatabaseError),
        }
    }
}

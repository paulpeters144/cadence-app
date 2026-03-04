use crate::Domain;
use crate::access::local_repo::{AccessError, DbUserRepository, UserRepository, ListRepository};
use crate::constants::JWT_EXPIRY_SECONDS;
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use async_trait::async_trait;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, PartialEq)]
pub enum ManagerError {
    InvalidCredentials,
    UserAlreadyExists,
    DatabaseError,
    TokenError,
    HashingError,
    UserNotFound,
}

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

#[async_trait]
pub trait Manager: Send + Sync {
    async fn login(&self, username: &str, password: &str) -> Result<String, ManagerError>;
    async fn register(&self, username: &str, password: &str) -> Result<String, ManagerError>;
    async fn get_user(&self, username: &str) -> Result<Domain::User, ManagerError>;
    async fn create_list(&self, username: &str, name: &str) -> Result<Domain::List, ManagerError>;
    async fn get_all_lists(&self, username: &str) -> Result<Vec<Domain::List>, ManagerError>;
    fn verify_jwt(&self, token: &str) -> Result<String, ManagerError>;
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

    fn hash_password(password: &str) -> Result<String, ManagerError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|_| ManagerError::HashingError)
    }

    fn verify_password(password: &str, hash: &str) -> Result<(), ManagerError> {
        let parsed_hash = PasswordHash::new(hash).map_err(|_| ManagerError::HashingError)?;
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| ManagerError::InvalidCredentials)
    }
}

#[async_trait]
impl Manager for AppManager {
    async fn login(&self, username: &str, password: &str) -> Result<String, ManagerError> {
        let user_result = self
            .user_repo
            .get_user_by_username_with_hash(username)
            .await;

        match user_result {
            Ok(Some((user, hash))) => {
                Self::verify_password(password, &hash)?;

                let access_token =
                    Self::create_jwt(&user.username, &self.jwt_secret, JWT_EXPIRY_SECONDS)?;

                Ok(access_token)
            }
            Ok(None) => Err(ManagerError::InvalidCredentials),
            Err(_) => Err(ManagerError::DatabaseError),
        }
    }

    async fn register(&self, username: &str, password: &str) -> Result<String, ManagerError> {
        let password_hash = Self::hash_password(password)?;

        let result = self.user_repo.create_user(username, &password_hash).await;

        match result {
            Ok(_) => {
                let access_token =
                    Self::create_jwt(username, &self.jwt_secret, JWT_EXPIRY_SECONDS)?;
                Ok(access_token)
            }
            Err(AccessError::AlreadyExists) => Err(ManagerError::UserAlreadyExists),
            Err(_) => Err(ManagerError::DatabaseError),
        }
    }

    async fn get_user(&self, username: &str) -> Result<Domain::User, ManagerError> {
        self.user_repo
            .get_user_by_username(username)
            .await
            .map_err(|_| ManagerError::DatabaseError)?
            .ok_or(ManagerError::UserNotFound)
    }

    async fn create_list(&self, username: &str, name: &str) -> Result<Domain::List, ManagerError> {
        self.user_repo
            .create_list(username, name)
            .await
            .map_err(|_| ManagerError::DatabaseError)
    }

    async fn get_all_lists(&self, username: &str) -> Result<Vec<Domain::List>, ManagerError> {
        self.user_repo
            .get_all_lists(username)
            .await
            .map_err(|_| ManagerError::DatabaseError)
    }

    fn verify_jwt(&self, token: &str) -> Result<String, ManagerError> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &Validation::default(),
        )
        .map(|data| data.claims.sub)
        .map_err(|_| ManagerError::TokenError)
    }
}

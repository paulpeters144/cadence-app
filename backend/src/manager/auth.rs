use crate::Domain;
use crate::access::{AccessError, UserQuery, UserQueryResult, UserRepository};
use crate::constants::JWT_EXPIRY_SECONDS;
use super::{AppManager, Claims, ManagerError};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rand::rngs::OsRng;

impl AppManager {
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

    pub async fn login(&self, username: &str, password: &str) -> Result<String, ManagerError> {
        let query = UserQuery::GetByUsername(username.to_string());
        let user_result = self.user_repo.execute(query).await;

        match user_result {
            Ok(UserQueryResult::User { password_hash, .. }) => {
                Self::verify_password(password, &password_hash)?;

                let access_token =
                    Self::create_jwt(username, &self.jwt_secret, JWT_EXPIRY_SECONDS)?;

                Ok(access_token)
            }
            Err(AccessError::NotFound) => Err(ManagerError::InvalidCredentials),
            Err(_) => Err(ManagerError::DatabaseError),
            _ => Err(ManagerError::DatabaseError),
        }
    }

    pub async fn register(&self, username: &str, password: &str) -> Result<String, ManagerError> {
        let password_hash = Self::hash_password(password)?;

        let query = UserQuery::Create {
            username: username.to_string(),
            password_hash,
        };
        let result = self.user_repo.execute(query).await;

        match result {
            Ok(UserQueryResult::Success) => {
                let access_token =
                    Self::create_jwt(username, &self.jwt_secret, JWT_EXPIRY_SECONDS)?;
                Ok(access_token)
            }
            Err(AccessError::AlreadyExists) => Err(ManagerError::UserAlreadyExists),
            Err(_) => Err(ManagerError::DatabaseError),
            _ => Err(ManagerError::DatabaseError),
        }
    }

    pub async fn get_user(&self, username: &str) -> Result<Domain::User, ManagerError> {
        let query = UserQuery::GetByUsername(username.to_string());
        let result = self.user_repo.execute(query).await;

        match result {
            Ok(UserQueryResult::User { id, username, .. }) => Ok(Domain::User {
                id: id.to_string(),
                username,
            }),
            Err(AccessError::NotFound) => Err(ManagerError::UserNotFound),
            Err(_) => Err(ManagerError::DatabaseError),
            _ => Err(ManagerError::DatabaseError),
        }
    }

    pub fn verify_jwt(&self, token: &str) -> Result<String, ManagerError> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &Validation::default(),
        )
        .map(|data| data.claims.sub)
        .map_err(|_| ManagerError::TokenError)
    }
}

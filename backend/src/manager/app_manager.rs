use crate::Domain;
use crate::access::{
    AccessError, ListRepository, AppRepository, TaskRepository, UserRepository,
};
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
use uuid::Uuid;

#[derive(Debug, PartialEq)]
pub enum ManagerError {
    InvalidCredentials,
    UserAlreadyExists,
    DatabaseError,
    TokenError,
    HashingError,
    UserNotFound,
    ListNotFound,
    TaskNotFound,
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
    async fn get_lists(
        &self,
        username: &str,
        start_id: Option<Uuid>,
        take: Option<i32>,
    ) -> Result<Vec<Domain::List>, ManagerError>;
    async fn update_list(
        &self,
        username: &str,
        id: Uuid,
        name: Option<String>,
        journal: Option<String>,
        archived: Option<bool>,
        position: Option<f32>,
    ) -> Result<Domain::List, ManagerError>;
    async fn create_task(
        &self,
        username: &str,
        list_id: Uuid,
        title: &str,
        points: Option<f32>,
    ) -> Result<Domain::Task, ManagerError>;
    async fn get_tasks(
        &self,
        username: &str,
        list_id: Uuid,
    ) -> Result<Vec<Domain::Task>, ManagerError>;
    async fn update_task(
        &self,
        username: &str,
        list_id: Uuid,
        task_id: Uuid,
        title: Option<String>,
        completed: Option<bool>,
        points: Option<f32>,
        position: Option<f32>,
    ) -> Result<Domain::Task, ManagerError>;
    async fn move_task(
        &self,
        username: &str,
        task_id: Uuid,
        from_list_id: Uuid,
        to_list_id: Uuid,
        position: Option<f32>,
    ) -> Result<Domain::Task, ManagerError>;
    async fn delete_task(
        &self,
        username: &str,
        list_id: Uuid,
        task_id: Uuid,
    ) -> Result<(), ManagerError>;
    async fn delete_list(&self, username: &str, id: Uuid) -> Result<(), ManagerError>;
    async fn duplicate_list(
        &self,
        username: &str,
        id: Uuid,
        new_name: &str,
    ) -> Result<Domain::List, ManagerError>;
    async fn reorder_lists(
        &self,
        username: &str,
        active_id: Uuid,
        over_id: Uuid,
    ) -> Result<Domain::List, ManagerError>;
    async fn reorder_tasks(
        &self,
        username: &str,
        list_id: Uuid,
        active_id: Uuid,
        over_id: Uuid,
    ) -> Result<Domain::Task, ManagerError>;
    fn verify_jwt(&self, token: &str) -> Result<String, ManagerError>;
}

pub struct AppManager {
    pub user_repo: Arc<AppRepository>,
    pub jwt_secret: String,
}

impl AppManager {
    pub fn new(user_repo: Arc<AppRepository>, jwt_secret: String) -> Self {
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
        let user_result = self.user_repo.get_user_pwd_hash(username).await;

        match user_result {
            Ok(Some(hash)) => {
                Self::verify_password(password, &hash)?;

                let access_token =
                    Self::create_jwt(username, &self.jwt_secret, JWT_EXPIRY_SECONDS)?;

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

    async fn get_lists(
        &self,
        username: &str,
        start_id: Option<Uuid>,
        take: Option<i32>,
    ) -> Result<Vec<Domain::List>, ManagerError> {
        self.user_repo
            .get_lists(username, start_id, take)
            .await
            .map_err(|_| ManagerError::DatabaseError)
    }

    async fn update_list(
        &self,
        username: &str,
        id: Uuid,
        name: Option<String>,
        journal: Option<String>,
        archived: Option<bool>,
        position: Option<f32>,
    ) -> Result<Domain::List, ManagerError> {
        self.user_repo
            .update_list(username, id, name, journal, archived, position)
            .await
            .map_err(|e| match e {
                AccessError::NotFound => ManagerError::ListNotFound,
                _ => ManagerError::DatabaseError,
            })
    }

    async fn create_task(
        &self,
        username: &str,
        list_id: Uuid,
        title: &str,
        points: Option<f32>,
    ) -> Result<Domain::Task, ManagerError> {
        self.user_repo
            .create_task(username, list_id, title, points)
            .await
            .map_err(|e| match e {
                AccessError::NotFound => ManagerError::ListNotFound,
                _ => ManagerError::DatabaseError,
            })
    }

    async fn get_tasks(
        &self,
        username: &str,
        list_id: Uuid,
    ) -> Result<Vec<Domain::Task>, ManagerError> {
        self.user_repo
            .get_tasks(username, list_id)
            .await
            .map_err(|_| ManagerError::DatabaseError)
    }

    async fn update_task(
        &self,
        username: &str,
        list_id: Uuid,
        task_id: Uuid,
        title: Option<String>,
        completed: Option<bool>,
        points: Option<f32>,
        position: Option<f32>,
    ) -> Result<Domain::Task, ManagerError> {
        self.user_repo
            .update_task(username, list_id, task_id, title, completed, points, position)
            .await
            .map_err(|e| match e {
                AccessError::NotFound => ManagerError::TaskNotFound,
                _ => ManagerError::DatabaseError,
            })
    }

    async fn move_task(
        &self,
        username: &str,
        task_id: Uuid,
        from_list_id: Uuid,
        to_list_id: Uuid,
        position: Option<f32>,
    ) -> Result<Domain::Task, ManagerError> {
        self.user_repo
            .move_task(username, task_id, from_list_id, to_list_id, position)
            .await
            .map_err(|e| match e {
                AccessError::NotFound => ManagerError::TaskNotFound,
                _ => ManagerError::DatabaseError,
            })
    }

    async fn delete_task(
        &self,
        username: &str,
        list_id: Uuid,
        task_id: Uuid,
    ) -> Result<(), ManagerError> {
        self.user_repo
            .delete_task(username, list_id, task_id)
            .await
            .map_err(|e| match e {
                AccessError::NotFound => ManagerError::TaskNotFound,
                _ => ManagerError::DatabaseError,
            })
    }

    async fn delete_list(&self, username: &str, id: Uuid) -> Result<(), ManagerError> {
        self.user_repo.delete_list(username, id).await.map_err(|e| {
            match e {
                AccessError::NotFound => ManagerError::ListNotFound,
                _ => ManagerError::DatabaseError,
            }
        })
    }

    async fn duplicate_list(
        &self,
        username: &str,
        id: Uuid,
        new_name: &str,
    ) -> Result<Domain::List, ManagerError> {
        self.user_repo
            .duplicate_list(username, id, new_name)
            .await
            .map_err(|e| match e {
                AccessError::NotFound => ManagerError::ListNotFound,
                _ => ManagerError::DatabaseError,
            })
    }

    async fn reorder_lists(
        &self,
        username: &str,
        active_id: Uuid,
        over_id: Uuid,
    ) -> Result<Domain::List, ManagerError> {
        self.user_repo
            .reorder_lists(username, active_id, over_id)
            .await
            .map_err(|e| match e {
                AccessError::NotFound => ManagerError::ListNotFound,
                _ => ManagerError::DatabaseError,
            })
    }

    async fn reorder_tasks(
        &self,
        username: &str,
        list_id: Uuid,
        active_id: Uuid,
        over_id: Uuid,
    ) -> Result<Domain::Task, ManagerError> {
        self.user_repo
            .reorder_tasks(username, list_id, active_id, over_id)
            .await
            .map_err(|e| match e {
                AccessError::NotFound => ManagerError::TaskNotFound,
                _ => ManagerError::DatabaseError,
            })
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

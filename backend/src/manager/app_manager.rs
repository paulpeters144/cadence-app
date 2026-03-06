use crate::Domain;
use crate::access::list::{
    CheckListOwnership, CreateList, DeleteList, GetList, GetLists, GetMaxListPosition, UpdateList,
};
use crate::access::task::{
    CheckTaskExists, CreateTask, DeleteTask, DeleteTasksByList, GetMaxTaskPosition, GetTasks,
    UpdateTask,
};
use crate::access::{
    AccessError, AppRepository, DbQuery, TransactionalRepository, UpdateListParams,
    UpdateTaskParams, UserQuery, UserQueryResult, UserRepository,
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
        params: UpdateListParams,
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
        params: UpdateTaskParams,
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

    async fn register(&self, username: &str, password: &str) -> Result<String, ManagerError> {
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

    async fn get_user(&self, username: &str) -> Result<Domain::User, ManagerError> {
        let query = UserQuery::GetByUsername(username.to_string());
        let result = self.user_repo.execute(query).await;

        match result {
            Ok(UserQueryResult::User { id, username, .. }) => Ok(Domain::User { id, username }),
            Err(AccessError::NotFound) => Err(ManagerError::UserNotFound),
            Err(_) => Err(ManagerError::DatabaseError),
            _ => Err(ManagerError::DatabaseError),
        }
    }

    async fn create_list(&self, username: &str, name: &str) -> Result<Domain::List, ManagerError> {
        let max_pos = GetMaxListPosition {
            username: username.to_string(),
        }
        .execute(&self.user_repo.pool)
        .await
        .map_err(|_| ManagerError::DatabaseError)?;

        let position = max_pos + 1024.0;
        let id = Uuid::new_v4();

        CreateList {
            id,
            username: username.to_string(),
            name: name.to_string(),
            position,
        }
        .execute(&self.user_repo.pool)
        .await
        .map_err(|_| ManagerError::DatabaseError)
    }

    async fn get_lists(
        &self,
        username: &str,
        start_id: Option<Uuid>,
        take: Option<i32>,
    ) -> Result<Vec<Domain::List>, ManagerError> {
        GetLists {
            username: username.to_string(),
            start_id,
            take,
        }
        .execute(&self.user_repo.pool)
        .await
        .map_err(|_| ManagerError::DatabaseError)
    }

    async fn update_list(
        &self,
        username: &str,
        id: Uuid,
        params: UpdateListParams,
    ) -> Result<Domain::List, ManagerError> {
        UpdateList {
            username: username.to_string(),
            id,
            name: params.name,
            journal: params.journal,
            archived: params.archived,
            position: params.position,
        }
        .execute(&self.user_repo.pool)
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
        let owns_list = CheckListOwnership {
            username: username.to_string(),
            id: list_id,
        }
        .execute(&self.user_repo.pool)
        .await
        .map_err(|_| ManagerError::DatabaseError)?;

        if !owns_list {
            return Err(ManagerError::ListNotFound);
        }

        let max_pos = GetMaxTaskPosition { list_id }
            .execute(&self.user_repo.pool)
            .await
            .map_err(|_| ManagerError::DatabaseError)?;

        let position = max_pos + 1024.0;
        let id = Uuid::new_v4();
        let created_at = chrono::Utc::now();

        CreateTask {
            id,
            list_id,
            title: title.to_string(),
            points,
            position,
            created_at,
        }
        .execute(&self.user_repo.pool)
        .await
        .map_err(|_| ManagerError::DatabaseError)
    }

    async fn get_tasks(
        &self,
        username: &str,
        list_id: Uuid,
    ) -> Result<Vec<Domain::Task>, ManagerError> {
        GetTasks {
            username: username.to_string(),
            list_id,
        }
        .execute(&self.user_repo.pool)
        .await
        .map_err(|_| ManagerError::DatabaseError)
    }

    async fn update_task(
        &self,
        username: &str,
        list_id: Uuid,
        task_id: Uuid,
        params: UpdateTaskParams,
    ) -> Result<Domain::Task, ManagerError> {
        UpdateTask {
            username: username.to_string(),
            list_id,
            task_id,
            title: params.title,
            completed: params.completed,
            points: params.points,
            position: params.position,
            new_list_id: None,
        }
        .execute(&self.user_repo.pool)
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
        let mut tx = self
            .user_repo
            .begin_transaction()
            .await
            .map_err(|_| ManagerError::DatabaseError)?;

        let task_exists = CheckTaskExists {
            username: username.to_string(),
            list_id: from_list_id,
            task_id,
        }
        .execute(&mut *tx)
        .await
        .map_err(|_| ManagerError::DatabaseError)?;

        if !task_exists {
            return Err(ManagerError::TaskNotFound);
        }

        let dest_owns = CheckListOwnership {
            username: username.to_string(),
            id: to_list_id,
        }
        .execute(&mut *tx)
        .await
        .map_err(|_| ManagerError::DatabaseError)?;

        if !dest_owns {
            return Err(ManagerError::ListNotFound);
        }

        let new_position = match position {
            Some(p) => p,
            None => {
                let max_pos = GetMaxTaskPosition {
                    list_id: to_list_id,
                }
                .execute(&mut *tx)
                .await
                .map_err(|_| ManagerError::DatabaseError)?;
                max_pos + 1024.0
            }
        };

        let updated_task = UpdateTask {
            username: username.to_string(),
            list_id: from_list_id,
            task_id,
            title: None,
            completed: None,
            points: None,
            position: Some(new_position),
            new_list_id: Some(to_list_id),
        }
        .execute(&mut *tx)
        .await
        .map_err(|e| match e {
            AccessError::NotFound => ManagerError::TaskNotFound,
            _ => ManagerError::DatabaseError,
        })?;

        tx.commit().await.map_err(|_| ManagerError::DatabaseError)?;

        Ok(updated_task)
    }

    async fn delete_task(
        &self,
        username: &str,
        list_id: Uuid,
        task_id: Uuid,
    ) -> Result<(), ManagerError> {
        DeleteTask {
            username: username.to_string(),
            list_id,
            task_id,
        }
        .execute(&self.user_repo.pool)
        .await
        .map_err(|e| match e {
            AccessError::NotFound => ManagerError::TaskNotFound,
            _ => ManagerError::DatabaseError,
        })
    }

    async fn delete_list(&self, username: &str, id: Uuid) -> Result<(), ManagerError> {
        let mut tx = self
            .user_repo
            .begin_transaction()
            .await
            .map_err(|_| ManagerError::DatabaseError)?;

        // First, check if the user owns the list
        let owns_list = CheckListOwnership {
            username: username.to_string(),
            id,
        }
        .execute(&mut *tx)
        .await
        .map_err(|_| ManagerError::DatabaseError)?;

        if !owns_list {
            return Err(ManagerError::ListNotFound);
        }

        DeleteTasksByList { list_id: id }
            .execute(&mut *tx)
            .await
            .map_err(|_| ManagerError::DatabaseError)?;

        DeleteList {
            username: username.to_string(),
            id,
        }
        .execute(&mut *tx)
        .await
        .map_err(|e| match e {
            AccessError::NotFound => ManagerError::ListNotFound,
            _ => ManagerError::DatabaseError,
        })?;

        tx.commit().await.map_err(|_| ManagerError::DatabaseError)?;
        Ok(())
    }

    async fn duplicate_list(
        &self,
        username: &str,
        id: Uuid,
        new_name: &str,
    ) -> Result<Domain::List, ManagerError> {
        let mut tx = self
            .user_repo
            .begin_transaction()
            .await
            .map_err(|_| ManagerError::DatabaseError)?;

        let owns_list = CheckListOwnership {
            username: username.to_string(),
            id,
        }
        .execute(&mut *tx)
        .await
        .map_err(|_| ManagerError::DatabaseError)?;

        if !owns_list {
            return Err(ManagerError::ListNotFound);
        }

        let max_pos = GetMaxListPosition {
            username: username.to_string(),
        }
        .execute(&mut *tx)
        .await
        .map_err(|_| ManagerError::DatabaseError)?;

        let position = max_pos + 1024.0;
        let new_id = Uuid::new_v4();

        let new_list = CreateList {
            id: new_id,
            username: username.to_string(),
            name: new_name.to_string(),
            position,
        }
        .execute(&mut *tx)
        .await
        .map_err(|_| ManagerError::DatabaseError)?;

        let tasks = GetTasks {
            username: username.to_string(),
            list_id: id,
        }
        .execute(&mut *tx)
        .await
        .map_err(|_| ManagerError::DatabaseError)?;

        for task in tasks {
            let new_task_id = Uuid::new_v4();
            CreateTask {
                id: new_task_id,
                list_id: new_id,
                title: task.title,
                points: task.points,
                position: task.position,
                created_at: chrono::Utc::now(),
            }
            .execute(&mut *tx)
            .await
            .map_err(|_| ManagerError::DatabaseError)?;
        }

        tx.commit().await.map_err(|_| ManagerError::DatabaseError)?;

        Ok(new_list)
    }

    async fn reorder_lists(
        &self,
        username: &str,
        active_id: Uuid,
        over_id: Uuid,
    ) -> Result<Domain::List, ManagerError> {
        if active_id == over_id {
            return GetList {
                username: username.to_string(),
                id: active_id,
            }
            .execute(&self.user_repo.pool)
            .await
            .map_err(|e| match e {
                AccessError::NotFound => ManagerError::ListNotFound,
                _ => ManagerError::DatabaseError,
            });
        }

        let lists = GetLists {
            username: username.to_string(),
            start_id: None,
            take: Some(1000),
        }
        .execute(&self.user_repo.pool)
        .await
        .map_err(|_| ManagerError::DatabaseError)?;

        let old_index = lists
            .iter()
            .position(|l| l.id == active_id)
            .ok_or(ManagerError::ListNotFound)?;
        let new_index = lists
            .iter()
            .position(|l| l.id == over_id)
            .ok_or(ManagerError::ListNotFound)?;

        let new_position = if new_index > old_index {
            let over_pos = lists[new_index].position;
            if new_index == lists.len() - 1 {
                over_pos + 1024.0
            } else {
                let next_pos = lists[new_index + 1].position;
                (over_pos + next_pos) / 2.0
            }
        } else {
            let over_pos = lists[new_index].position;
            if new_index == 0 {
                over_pos / 2.0
            } else {
                let prev_pos = lists[new_index - 1].position;
                (over_pos + prev_pos) / 2.0
            }
        };

        UpdateList {
            username: username.to_string(),
            id: active_id,
            name: None,
            journal: None,
            archived: None,
            position: Some(new_position),
        }
        .execute(&self.user_repo.pool)
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
        if active_id == over_id {
            return UpdateTask {
                username: username.to_string(),
                list_id,
                task_id: active_id,
                title: None,
                completed: None,
                points: None,
                position: None,
                new_list_id: None,
            }
            .execute(&self.user_repo.pool)
            .await
            .map_err(|e| match e {
                AccessError::NotFound => ManagerError::TaskNotFound,
                _ => ManagerError::DatabaseError,
            });
        }

        let tasks = GetTasks {
            username: username.to_string(),
            list_id,
        }
        .execute(&self.user_repo.pool)
        .await
        .map_err(|_| ManagerError::DatabaseError)?;

        let old_index = tasks
            .iter()
            .position(|t| t.id == active_id)
            .ok_or(ManagerError::TaskNotFound)?;
        let new_index = tasks
            .iter()
            .position(|t| t.id == over_id)
            .ok_or(ManagerError::TaskNotFound)?;

        let new_position = if new_index > old_index {
            let over_pos = tasks[new_index].position;
            if new_index == tasks.len() - 1 {
                over_pos + 1024.0
            } else {
                let next_pos = tasks[new_index + 1].position;
                (over_pos + next_pos) / 2.0
            }
        } else {
            let over_pos = tasks[new_index].position;
            if new_index == 0 {
                over_pos / 2.0
            } else {
                let prev_pos = tasks[new_index - 1].position;
                (over_pos + prev_pos) / 2.0
            }
        };

        UpdateTask {
            username: username.to_string(),
            list_id,
            task_id: active_id,
            title: None,
            completed: None,
            points: None,
            position: Some(new_position),
            new_list_id: None,
        }
        .execute(&self.user_repo.pool)
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

pub mod auth;
pub mod list;
pub mod task;

use crate::access::AppRepository;
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
    ListNotFound,
    TaskNotFound,
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
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
}

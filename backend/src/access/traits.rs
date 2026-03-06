use super::error::AccessError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateListParams {
    pub name: Option<String>,
    pub journal: Option<String>,
    pub archived: Option<bool>,
    pub position: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskParams {
    pub title: Option<String>,
    pub completed: Option<bool>,
    pub points: Option<f32>,
    pub position: Option<f32>,
}

pub trait DbQuery: Send + Sync {
    type Response;
    
    fn execute<'e, E>(
        &self,
        executor: E,
    ) -> impl std::future::Future<Output = Result<Self::Response, AccessError>> + Send
    where
        E: sqlx::Executor<'e, Database = sqlx::Sqlite>;
}

pub trait TransactionalRepository: Send + Sync {
    fn begin_transaction(
        &self,
    ) -> impl std::future::Future<Output = Result<sqlx::Transaction<'static, sqlx::Sqlite>, AccessError>> + Send;
}

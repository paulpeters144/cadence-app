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

pub enum DbExecutor<'a> {
    Connection(&'a libsql::Connection),
    Transaction(&'a libsql::Transaction),
}

impl<'a> DbExecutor<'a> {
    pub async fn query(&self, sql: &str, params: impl libsql::params::IntoParams) -> Result<libsql::Rows, libsql::Error> {
        match self {
            DbExecutor::Connection(c) => c.query(sql, params).await,
            DbExecutor::Transaction(t) => t.query(sql, params).await,
        }
    }

    pub async fn execute(&self, sql: &str, params: impl libsql::params::IntoParams) -> Result<u64, libsql::Error> {
        match self {
            DbExecutor::Connection(c) => c.execute(sql, params).await,
            DbExecutor::Transaction(t) => t.execute(sql, params).await,
        }
    }
}

pub trait DbQuery: Send + Sync {
    type Response;
    
    fn execute<'e>(
        &self,
        executor: &'e DbExecutor<'e>,
    ) -> impl std::future::Future<Output = Result<Self::Response, AccessError>> + Send;
}

pub trait TransactionalRepository: Send + Sync {
    fn begin_transaction(
        &self,
    ) -> impl std::future::Future<Output = Result<libsql::Transaction, AccessError>> + Send;
}

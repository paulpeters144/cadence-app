use super::error::AccessError;
use libsql::{Connection, Error, Rows, Transaction, params::IntoParams};
use serde::{Deserialize, Serialize};
use std::future::Future;

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
    Connection(&'a Connection),
    Transaction(&'a Transaction),
}

impl<'a> DbExecutor<'a> {
    pub async fn query(&self, sql: &str, params: impl IntoParams) -> Result<Rows, Error> {
        match self {
            DbExecutor::Connection(c) => c.query(sql, params).await,
            DbExecutor::Transaction(t) => t.query(sql, params).await,
        }
    }

    pub async fn execute(&self, sql: &str, params: impl IntoParams) -> Result<u64, Error> {
        match self {
            DbExecutor::Connection(c) => c.execute(sql, params).await,
            DbExecutor::Transaction(t) => t.execute(sql, params).await,
        }
    }
}

pub trait DbQuery: Send + Sync {
    type Response;

    fn execute<'e>(&self, executor: &'e DbExecutor<'e>) -> impl Future<Output = Result<Self::Response, AccessError>> + Send;
}

pub trait TransactionalRepository: Send + Sync {
    fn begin_transaction(
        &self,
    ) -> impl Future<Output = Result<Transaction, AccessError>> + Send;
}

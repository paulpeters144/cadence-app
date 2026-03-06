use sqlx::Row;
use crate::access::error::AccessError;
use super::AppRepository;

pub enum DbTable {
    Users,
    Lists,
    Tasks,
}

impl DbTable {
    pub fn table_name(&self) -> &'static str {
        match self {
            DbTable::Users => "users",
            DbTable::Lists => "lists",
            DbTable::Tasks => "tasks",
        }
    }
}

pub trait UtilRepository {
    fn count(&self, table: DbTable) -> impl std::future::Future<Output = Result<i64, AccessError>> + Send;
}

impl UtilRepository for AppRepository {
    async fn count(&self, table: DbTable) -> Result<i64, AccessError> {
        let query = format!("SELECT COUNT(*) as count FROM {}", table.table_name());
        let row = sqlx::query(&query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

        Ok(row.get("count"))
    }
}

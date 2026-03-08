use std::future::Future;
use crate::Domain;
use crate::access::error::AccessError;
use crate::access::traits::{DbQuery, DbExecutor};

fn row_to_list(row: libsql::Row) -> Result<Domain::List, AccessError> {
    let id: String = row.get(0).map_err(|e| AccessError::DatabaseError(e.to_string()))?;
    let name: String = row.get(1).map_err(|e| AccessError::DatabaseError(e.to_string()))?;
    let journal: Option<String> = row.get(2).map_err(|e| AccessError::DatabaseError(e.to_string()))?;
    let archived: bool = row.get(3).map_err(|e| AccessError::DatabaseError(e.to_string()))?;
    let archived_at_str: Option<String> = row.get(4).map_err(|e| AccessError::DatabaseError(e.to_string()))?;

    let archived_at = archived_at_str
        .map(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .map_err(|e| AccessError::DatabaseError(e.to_string()))
        })
        .transpose()?;
        
    let position: f64 = row.get(5).map_err(|e| AccessError::DatabaseError(e.to_string()))?;

    Ok(Domain::List {
        id,
        name,
        journal,
        archived,
        archived_at,
        position: position as f32,
    })
}

pub struct CreateList {
    pub id: String,
    pub username: String,
    pub name: String,
    pub position: f32,
}

impl DbQuery for CreateList {
    type Response = Domain::List;

    fn execute<'e>(&self, executor: &'e DbExecutor<'e>) -> impl Future<Output = Result<Self::Response, AccessError>> + Send {
        let id_str = self.id.to_string();
        let username = self.username.clone();
        let name = self.name.clone();
        let position = self.position;
        let id = self.id.clone();

        async move {
            executor.execute(
                "INSERT INTO lists (id, username, name, position) VALUES (?, ?, ?, ?)",
                libsql::params![id_str, username, name.clone(), position as f64]
            )
            .await
            .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

            Ok(Domain::List {
                id,
                name,
                journal: None,
                archived: false,
                archived_at: None,
                position,
            })
        }
    }
}

pub struct GetLists {
    pub username: String,
    pub start_id: Option<String>,
    pub take: Option<i32>,
}

impl DbQuery for GetLists {
    type Response = Vec<Domain::List>;

    fn execute<'e>(&self, executor: &'e DbExecutor<'e>) -> impl Future<Output = Result<Self::Response, AccessError>> + Send {
        let username = self.username.clone();
        let start_id = self.start_id.clone().map(|s| s.to_string());
        let take = self.take.unwrap_or(50);

        async move {
            let sql = if start_id.is_some() {
                format!(
                    "SELECT id, name, journal, archived, archived_at, position FROM lists WHERE username = ? AND id = ? ORDER BY position ASC LIMIT {}",
                    take
                )
            } else {
                format!(
                    "SELECT id, name, journal, archived, archived_at, position FROM lists WHERE username = ? ORDER BY position ASC LIMIT {}",
                    take
                )
            };

            let mut rows = if let Some(ref sid_str) = start_id {
                executor.query(&sql, libsql::params![username, sid_str.clone()]).await
            } else {
                executor.query(&sql, libsql::params![username]).await
            }.map_err(|e| AccessError::DatabaseError(e.to_string()))?;

            let mut lists = Vec::new();
            while let Some(row) = rows.next().await.map_err(|e| AccessError::DatabaseError(e.to_string()))? {
                lists.push(row_to_list(row)?);
            }

            Ok(lists)
        }
    }
}

pub struct GetList {
    pub username: String,
    pub id: String,
}

impl DbQuery for GetList {
    type Response = Domain::List;

    fn execute<'e>(&self, executor: &'e DbExecutor<'e>) -> impl Future<Output = Result<Self::Response, AccessError>> + Send {
        let username = self.username.clone();
        let id_str = self.id.to_string();

        async move {
            let mut rows = executor.query(
                "SELECT id, name, journal, archived, archived_at, position FROM lists WHERE id = ? AND username = ?",
                libsql::params![id_str, username]
            )
            .await
            .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

            let row = rows.next().await.map_err(|e| AccessError::DatabaseError(e.to_string()))?
                .ok_or(AccessError::NotFound)?;

            row_to_list(row)
        }
    }
}

pub struct UpdateList {
    pub username: String,
    pub id: String,
    pub name: Option<String>,
    pub journal: Option<String>,
    pub archived: Option<bool>,
    pub position: Option<f32>,
}

impl DbQuery for UpdateList {
    type Response = Domain::List;

    fn execute<'e>(&self, executor: &'e DbExecutor<'e>) -> impl Future<Output = Result<Self::Response, AccessError>> + Send {
        let username = self.username.clone();
        let id_str = self.id.to_string();
        let name = self.name.clone();
        let journal_is_some = self.journal.is_some();
        let journal = self.journal.clone();
        let archived = self.archived;
        let position = self.position;
        let archived_is_some = self.archived.is_some();
        
        let archived_at_str = self.archived
            .and_then(|a| a.then(|| chrono::Utc::now().to_rfc3339()));

        async move {
            let mut rows = executor.query(
                "UPDATE lists
                 SET name = COALESCE(?, name),
                     journal = CASE WHEN ? THEN ? ELSE journal END,
                     archived = COALESCE(?, archived),
                     position = COALESCE(?, position),
                     archived_at = CASE
                        WHEN ? THEN ?
                        ELSE archived_at
                     END
                 WHERE username = ? AND id = ?
                 RETURNING id, name, journal, archived, archived_at, position",
                 libsql::params![
                    name,
                    journal_is_some,
                    journal,
                    archived,
                    position.map(|p| p as f64),
                    archived_is_some,
                    archived_at_str,
                    username,
                    id_str
                 ]
            )
            .await
            .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

            let row = rows.next().await.map_err(|e| AccessError::DatabaseError(e.to_string()))?
                .ok_or(AccessError::NotFound)?;

            row_to_list(row)
        }
    }
}

pub struct DeleteList {
    pub username: String,
    pub id: String,
}

impl DbQuery for DeleteList {
    type Response = ();

    fn execute<'e>(&self, executor: &'e DbExecutor<'e>) -> impl Future<Output = Result<Self::Response, AccessError>> + Send {
        let username = self.username.clone();
        let id_str = self.id.to_string();

        async move {
            let rows_affected = executor.execute("DELETE FROM lists WHERE id = ? AND username = ?", libsql::params![id_str, username])
                .await
                .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

            if rows_affected == 0 {
                return Err(AccessError::NotFound);
            }

            Ok(())
        }
    }
}

pub struct GetMaxListPosition {
    pub username: String,
}

impl DbQuery for GetMaxListPosition {
    type Response = f32;

    fn execute<'e>(&self, executor: &'e DbExecutor<'e>) -> impl Future<Output = Result<Self::Response, AccessError>> + Send {
        let username = self.username.clone();

        async move {
            let mut rows = executor.query("SELECT COALESCE(MAX(position), 0.0) FROM lists WHERE username = ?", libsql::params![username])
                .await
                .map_err(|e| AccessError::DatabaseError(e.to_string()))?;
            
            let row = rows.next().await.map_err(|e| AccessError::DatabaseError(e.to_string()))?
                .ok_or(AccessError::NotFound)?;
            
            let max_pos: f64 = row.get(0).unwrap_or(0.0);
            Ok(max_pos as f32)
        }
    }
}

pub struct CheckListOwnership {
    pub username: String,
    pub id: String,
}

impl DbQuery for CheckListOwnership {
    type Response = bool;

    fn execute<'e>(&self, executor: &'e DbExecutor<'e>) -> impl Future<Output = Result<Self::Response, AccessError>> + Send {
        let username = self.username.clone();
        let id_str = self.id.to_string();

        async move {
            let mut rows = executor.query("SELECT 1 FROM lists WHERE id = ? AND username = ?", libsql::params![id_str, username])
                .await
                .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

            let row = rows.next().await.map_err(|e| AccessError::DatabaseError(e.to_string()))?;
            Ok(row.is_some())
        }
    }
}

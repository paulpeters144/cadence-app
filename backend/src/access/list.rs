use std::future::Future;
use sqlx::{Executor, Row, Sqlite};
use sqlx::sqlite::SqliteRow;
use crate::Domain;
use crate::access::error::AccessError;
use crate::access::traits::DbQuery;

fn row_to_list(row: SqliteRow) -> Result<Domain::List, AccessError> {
    let id: String = row.get("id");

    let archived_at = row
        .get::<Option<String>, _>("archived_at")
        .map(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .map_err(|e| AccessError::DatabaseError(e.to_string()))
        })
        .transpose()?;

    Ok(Domain::List {
        id,
        name: row.get("name"),
        journal: row.get("journal"),
        archived: row.get("archived"),
        archived_at,
        position: row.get("position"),
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

    fn execute<'e, E>(&self, executor: E) -> impl Future<Output = Result<Self::Response, AccessError>> + Send
    where
        E: Executor<'e, Database = Sqlite>
    {
        let id_str = self.id.to_string();
        let username = self.username.clone();
        let name = self.name.clone();
        let position = self.position;
        let id = self.id.clone();

        async move {
            sqlx::query("INSERT INTO lists (id, username, name, position) VALUES (?, ?, ?, ?)")
                .bind(&id_str)
                .bind(&username)
                .bind(&name)
                .bind(position)
                .execute(executor)
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

    fn execute<'e, E>(&self, executor: E) -> impl Future<Output = Result<Self::Response, AccessError>> + Send
    where
        E: Executor<'e, Database = Sqlite>
    {
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

            let mut query = sqlx::query::<Sqlite>(&sql).bind(&username);
            if let Some(ref sid_str) = start_id {
                query = query.bind(sid_str);
            }

            let rows = query
                .fetch_all(executor)
                .await
                .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

            rows.into_iter().map(row_to_list).collect()
        }
    }
}

pub struct GetList {
    pub username: String,
    pub id: String,
}

impl DbQuery for GetList {
    type Response = Domain::List;

    fn execute<'e, E>(&self, executor: E) -> impl Future<Output = Result<Self::Response, AccessError>> + Send
    where
        E: Executor<'e, Database = Sqlite>
    {
        let username = self.username.clone();
        let id_str = self.id.to_string();

        async move {
            let row = sqlx::query(
                "SELECT id, name, journal, archived, archived_at, position FROM lists WHERE id = ? AND username = ?",
            )
            .bind(&id_str)
            .bind(&username)
            .fetch_one(executor)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => AccessError::NotFound,
                _ => AccessError::DatabaseError(e.to_string()),
            })?;

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

    fn execute<'e, E>(&self, executor: E) -> impl Future<Output = Result<Self::Response, AccessError>> + Send
    where
        E: Executor<'e, Database = Sqlite>
    {
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
            let row = sqlx::query(
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
            )
            .bind(&name)
            .bind(journal_is_some)
            .bind(&journal)
            .bind(archived)
            .bind(position)
            .bind(archived_is_some)
            .bind(&archived_at_str)
            .bind(&username)
            .bind(&id_str)
            .fetch_optional(executor)
            .await
            .map_err(|e| AccessError::DatabaseError(e.to_string()))?
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

    fn execute<'e, E>(&self, executor: E) -> impl Future<Output = Result<Self::Response, AccessError>> + Send
    where
        E: Executor<'e, Database = Sqlite>
    {
        let username = self.username.clone();
        let id_str = self.id.to_string();

        async move {
            let result = sqlx::query("DELETE FROM lists WHERE id = ? AND username = ?")
                .bind(&id_str)
                .bind(&username)
                .execute(executor)
                .await
                .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

            if result.rows_affected() == 0 {
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

    fn execute<'e, E>(&self, executor: E) -> impl Future<Output = Result<Self::Response, AccessError>> + Send
    where
        E: Executor<'e, Database = Sqlite>
    {
        let username = self.username.clone();

        async move {
            let max_pos: (f32,) = sqlx::query_as("SELECT COALESCE(MAX(position), 0.0) FROM lists WHERE username = ?")
                .bind(&username)
                .fetch_one(executor)
                .await
                .map_err(|e| AccessError::DatabaseError(e.to_string()))?;
            
            Ok(max_pos.0)
        }
    }
}

pub struct CheckListOwnership {
    pub username: String,
    pub id: String,
}

impl DbQuery for CheckListOwnership {
    type Response = bool;

    fn execute<'e, E>(&self, executor: E) -> impl Future<Output = Result<Self::Response, AccessError>> + Send
    where
        E: Executor<'e, Database = Sqlite>
    {
        let username = self.username.clone();
        let id_str = self.id.to_string();

        async move {
            let row = sqlx::query("SELECT 1 FROM lists WHERE id = ? AND username = ?")
                .bind(&id_str)
                .bind(&username)
                .fetch_optional(executor)
                .await
                .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

            Ok(row.is_some())
        }
    }
}
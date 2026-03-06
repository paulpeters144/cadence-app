use super::AppRepository;
use crate::access::error::AccessError;
use serde::{Deserialize, Serialize};
use sqlx::{Row, sqlite::SqliteRow};
use std::future::Future;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserQuery {
    Get(Uuid),
    GetByUsername(String),
    Create {
        username: String,
        password_hash: String,
    },
    Update {
        id: Uuid,
        username: Option<String>,
        password_hash: Option<String>,
    },
    Delete(Uuid),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserQueryResult {
    User {
        id: Uuid,
        username: String,
        password_hash: String,
    },
    Success,
}

fn row_to_user(row: SqliteRow) -> Result<UserQueryResult, AccessError> {
    let id_str: String = row.get("id");
    let id: Uuid = Uuid::parse_str(&id_str).map_err(|e| AccessError::DatabaseError(e.to_string()))?;

    Ok(UserQueryResult::User {
        id,
        username: row.get("username"),
        password_hash: row.get("password_hash"),
    })
}

pub trait UserRepository {
    fn execute(&self, query: UserQuery) -> impl Future<Output = Result<UserQueryResult, AccessError>> + Send;
}

impl UserRepository for AppRepository {
    async fn execute(&self, query: UserQuery) -> Result<UserQueryResult, AccessError> {
        match query {
            UserQuery::Get(id) => {
                let row = sqlx::query("SELECT id, username, password_hash FROM users WHERE id = ?")
                    .bind(id.to_string())
                    .fetch_optional(&self.pool)
                    .await
                    .map_err(|e| AccessError::DatabaseError(e.to_string()))?
                    .ok_or(AccessError::NotFound)?;

                row_to_user(row)
            }
            UserQuery::GetByUsername(username) => {
                let row = sqlx::query("SELECT id, username, password_hash FROM users WHERE username = ?")
                    .bind(username)
                    .fetch_optional(&self.pool)
                    .await
                    .map_err(|e| AccessError::DatabaseError(e.to_string()))?
                    .ok_or(AccessError::NotFound)?;

                row_to_user(row)
            }
            UserQuery::Create {
                username,
                password_hash,
            } => {
                let id = Uuid::new_v4().to_string();
                sqlx::query("INSERT INTO users (id, username, password_hash) VALUES (?, ?, ?)")
                    .bind(id)
                    .bind(username)
                    .bind(password_hash)
                    .execute(&self.pool)
                    .await
                    .map_err(|e| {
                        if let Some(sqe) = e.as_database_error()
                            && sqe.is_unique_violation()
                        {
                            AccessError::AlreadyExists
                        } else {
                            AccessError::DatabaseError(e.to_string())
                        }
                    })?;
                Ok(UserQueryResult::Success)
            }
            UserQuery::Update {
                id,
                username,
                password_hash,
            } => {
                let mut qb = sqlx::QueryBuilder::new("UPDATE users SET ");
                let mut separated = qb.separated(", ");
                if let Some(u) = username {
                    separated.push("username = ").push_bind(u);
                }
                if let Some(p) = password_hash {
                    separated.push("password_hash = ").push_bind(p);
                }

                qb.push(" WHERE id = ").push_bind(id.to_string());

                let res = qb
                    .build()
                    .execute(&self.pool)
                    .await
                    .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

                if res.rows_affected() == 0 {
                    return Err(AccessError::NotFound);
                }
                Ok(UserQueryResult::Success)
            }
            UserQuery::Delete(id) => {
                let res = sqlx::query("DELETE FROM users WHERE id = ?")
                    .bind(id.to_string())
                    .execute(&self.pool)
                    .await
                    .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

                if res.rows_affected() == 0 {
                    return Err(AccessError::NotFound);
                }
                Ok(UserQueryResult::Success)
            }
        }
    }
}
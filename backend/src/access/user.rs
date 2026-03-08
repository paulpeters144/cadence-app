use super::AppRepository;
use crate::access::error::AccessError;
use serde::{Deserialize, Serialize};
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

fn row_to_user(row: libsql::Row) -> Result<UserQueryResult, AccessError> {
    let id_str: String = row.get(0).map_err(|e| AccessError::DatabaseError(e.to_string()))?;
    let id: Uuid = Uuid::parse_str(&id_str).map_err(|e| AccessError::DatabaseError(e.to_string()))?;

    let username: String = row.get(1).map_err(|e| AccessError::DatabaseError(e.to_string()))?;
    let password_hash: String = row.get(2).map_err(|e| AccessError::DatabaseError(e.to_string()))?;

    Ok(UserQueryResult::User {
        id,
        username,
        password_hash,
    })
}

pub trait UserRepository {
    fn execute(&self, query: UserQuery) -> impl Future<Output = Result<UserQueryResult, AccessError>> + Send;
}

impl UserRepository for AppRepository {
    async fn execute(&self, query: UserQuery) -> Result<UserQueryResult, AccessError> {
        match query {
            UserQuery::Get(id) => {
                let mut rows = self.conn.query("SELECT id, username, password_hash FROM users WHERE id = ?", libsql::params![id.to_string()])
                    .await
                    .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

                let row = rows.next().await.map_err(|e| AccessError::DatabaseError(e.to_string()))?
                    .ok_or(AccessError::NotFound)?;

                row_to_user(row)
            }
            UserQuery::GetByUsername(username) => {
                let mut rows = self.conn.query("SELECT id, username, password_hash FROM users WHERE username = ?", libsql::params![username])
                    .await
                    .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

                let row = rows.next().await.map_err(|e| AccessError::DatabaseError(e.to_string()))?
                    .ok_or(AccessError::NotFound)?;

                row_to_user(row)
            }
            UserQuery::Create {
                username,
                password_hash,
            } => {
                let id = Uuid::new_v4().to_string();
                self.conn.execute("INSERT INTO users (id, username, password_hash) VALUES (?, ?, ?)", libsql::params![id, username, password_hash])
                    .await
                    .map_err(|e| {
                        let err_str = e.to_string();
                        if err_str.contains("UNIQUE constraint failed") || err_str.contains("UNIQUE") {
                            AccessError::AlreadyExists
                        } else {
                            AccessError::DatabaseError(err_str)
                        }
                    })?;
                Ok(UserQueryResult::Success)
            }
            UserQuery::Update {
                id,
                username,
                password_hash,
            } => {
                let mut sql = String::from("UPDATE users SET ");
                let mut set_clauses = Vec::new();
                let mut params_user = None;
                let mut params_hash = None;

                if let Some(u) = username {
                    set_clauses.push("username = ?");
                    params_user = Some(u);
                }
                if let Some(p) = password_hash {
                    set_clauses.push("password_hash = ?");
                    params_hash = Some(p);
                }

                if set_clauses.is_empty() {
                    return Ok(UserQueryResult::Success);
                }

                sql.push_str(&set_clauses.join(", "));
                sql.push_str(" WHERE id = ?");

                let res = match (params_user, params_hash) {
                    (Some(u), Some(p)) => self.conn.execute(&sql, libsql::params![u, p, id.to_string()]).await,
                    (Some(u), None) => self.conn.execute(&sql, libsql::params![u, id.to_string()]).await,
                    (None, Some(p)) => self.conn.execute(&sql, libsql::params![p, id.to_string()]).await,
                    (None, None) => Ok(0),
                }.map_err(|e| AccessError::DatabaseError(e.to_string()))?;

                if res == 0 {
                    return Err(AccessError::NotFound);
                }
                Ok(UserQueryResult::Success)
            }
            UserQuery::Delete(id) => {
                let res = self.conn.execute("DELETE FROM users WHERE id = ?", libsql::params![id.to_string()])
                    .await
                    .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

                if res == 0 {
                    return Err(AccessError::NotFound);
                }
                Ok(UserQueryResult::Success)
            }
        }
    }
}
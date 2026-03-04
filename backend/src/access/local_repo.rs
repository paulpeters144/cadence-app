use crate::Domain;
use sqlx::{Execute, Row, SqlitePool, sqlite::SqlitePoolOptions};
use uuid::Uuid;

#[derive(Debug)]
pub enum AccessError {
    NotFound,
    AlreadyExists,
    DatabaseError(String),
}

pub trait UserRepository: Send + Sync {
    fn get_user_by_username(
        &self,
        username: &str,
    ) -> impl std::future::Future<Output = Result<Option<Domain::User>, AccessError>> + Send;

    fn get_user_by_username_with_hash(
        &self,
        username: &str,
    ) -> impl std::future::Future<Output = Result<Option<(Domain::User, String)>, AccessError>> + Send;

    fn create_user(
        &self,
        username: &str,
        password_hash: &str,
    ) -> impl std::future::Future<Output = Result<(), AccessError>> + Send;
}

pub trait ListRepository: Send + Sync {
    fn create_list(
        &self,
        username: &str,
        name: &str,
    ) -> impl std::future::Future<Output = Result<Domain::List, AccessError>> + Send;

    fn get_all_lists(
        &self,
        username: &str,
        start_id: Option<Uuid>,
        take: Option<i32>,
    ) -> impl std::future::Future<Output = Result<Vec<Domain::List>, AccessError>> + Send;
}

pub struct DbUserRepository {
    pool: SqlitePool,
}

impl DbUserRepository {
    pub async fn new(database_url: &str) -> Self {
        let path = database_url.strip_prefix("sqlite:");

        let needs_file_creation = |p: &str| p != ":memory:" && !std::path::Path::new(p).exists();

        if path.is_some_and(needs_file_creation) {
            tokio::fs::File::create(path.unwrap()).await.unwrap();
        }

        let pool = SqlitePoolOptions::new()
            .max_connections(2)
            .connect(database_url)
            .await
            .expect("Failed to connect to SQLite");

        Self { pool }
    }

    pub async fn init(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                username TEXT UNIQUE NOT NULL COLLATE NOCASE,
                password_hash TEXT NOT NULL
            );",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS lists (
                id TEXT PRIMARY KEY,
                username TEXT NOT NULL,
                name TEXT NOT NULL,
                journal TEXT,
                archived BOOLEAN NOT NULL DEFAULT 0,
                archived_at TEXT,
                FOREIGN KEY(username) REFERENCES users(username)
            );",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                list_id TEXT NOT NULL,
                title TEXT NOT NULL,
                completed BOOLEAN NOT NULL DEFAULT 0,
                points REAL,
                created_at TEXT NOT NULL,
                completed_at TEXT,
                FOREIGN KEY(list_id) REFERENCES lists(id)
            );",
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

impl UserRepository for DbUserRepository {
    async fn get_user_by_username(
        &self,
        username: &str,
    ) -> Result<Option<Domain::User>, AccessError> {
        let row = sqlx::query("SELECT username FROM users WHERE username = ?")
            .bind(username)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

        match row {
            Some(record) => Ok(Some(Domain::User {
                username: record.get("username"),
            })),
            None => Ok(None),
        }
    }

    async fn get_user_by_username_with_hash(
        &self,
        username: &str,
    ) -> Result<Option<(Domain::User, String)>, AccessError> {
        let row = sqlx::query("SELECT username, password_hash FROM users WHERE username = ?")
            .bind(username)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

        match row {
            Some(record) => Ok(Some((
                Domain::User {
                    username: record.get("username"),
                },
                record.get("password_hash"),
            ))),
            None => Ok(None),
        }
    }

    async fn create_user(&self, username: &str, password_hash: &str) -> Result<(), AccessError> {
        let id = Uuid::new_v4().to_string();
        let result =
            sqlx::query("INSERT INTO users (id, username, password_hash) VALUES (?, ?, ?)")
                .bind(id)
                .bind(username)
                .bind(password_hash)
                .execute(&self.pool)
                .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                if let Some(sqlite_error) = e.as_database_error()
                    && sqlite_error.is_unique_violation()
                {
                    return Err(AccessError::AlreadyExists);
                }
                Err(AccessError::DatabaseError(e.to_string()))
            }
        }
    }
}

impl ListRepository for DbUserRepository {
    async fn create_list(&self, username: &str, name: &str) -> Result<Domain::List, AccessError> {
        let id = Uuid::new_v4();
        let id_str = id.to_string();

        sqlx::query("INSERT INTO lists (id, username, name) VALUES (?, ?, ?)")
            .bind(id_str)
            .bind(username)
            .bind(name)
            .execute(&self.pool)
            .await
            .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

        Ok(Domain::List {
            id,
            name: name.to_string(),
            journal: None,
            archived: false,
            archived_at: None,
        })
    }

    async fn get_all_lists(
        &self,
        username: &str,
        start_id: Option<Uuid>,
        take: Option<i32>,
    ) -> Result<Vec<Domain::List>, AccessError> {
        let mut query = sqlx::query(
            "SELECT id, name, journal, archived, archived_at FROM lists WHERE username = ?",
        );
        query = query.bind(username);

        if let Some(start_id) = start_id {
            query = sqlx::query(
                "SELECT id, name, journal, archived, archived_at FROM lists WHERE username = ? AND id = ?",
            );
            query = query.bind(username);
            query = query.bind(start_id.to_string());
        }
        let take = take.unwrap_or(50);
        query = sqlx::query(&format!("{} LIMIT {}", query.sql(), take));

        let rows = query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

        let mut lists = Vec::new();
        for row in rows {
            let id_str: String = row.get("id");
            let id =
                Uuid::parse_str(&id_str).map_err(|e| AccessError::DatabaseError(e.to_string()))?;

            let archived_at_str: Option<String> = row.get("archived_at");
            let archived_at = match archived_at_str {
                Some(s) => Some(
                    chrono::DateTime::parse_from_rfc3339(&s)
                        .map_err(|e| AccessError::DatabaseError(e.to_string()))?
                        .with_timezone(&chrono::Utc),
                ),
                None => None,
            };

            lists.push(Domain::List {
                id,
                name: row.get("name"),
                journal: row.get("journal"),
                archived: row.get("archived"),
                archived_at,
            });
        }

        Ok(lists)
            .bind(username)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

        let mut lists = Vec::new();
        for row in rows {
            let id_str: String = row.get("id");
            let id =
                Uuid::parse_str(&id_str).map_err(|e| AccessError::DatabaseError(e.to_string()))?;

            let archived_at_str: Option<String> = row.get("archived_at");
            let archived_at = match archived_at_str {
                Some(s) => Some(
                    chrono::DateTime::parse_from_rfc3339(&s)
                        .map_err(|e| AccessError::DatabaseError(e.to_string()))?
                        .with_timezone(&chrono::Utc),
                ),
                None => None,
            };

            lists.push(Domain::List {
                id,
                name: row.get("name"),
                journal: row.get("journal"),
                archived: row.get("archived"),
                archived_at,
            });
        }

        Ok(lists)
    }
}

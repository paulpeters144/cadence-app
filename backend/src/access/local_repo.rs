use crate::Domain;
use sqlx::{Row, SqlitePool, sqlite::SqlitePoolOptions};
use uuid::Uuid;

#[derive(Debug)]
#[allow(dead_code)]
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

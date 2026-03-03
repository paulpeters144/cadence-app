use crate::domain::user::User;
use sqlx::{Row, SqlitePool, sqlite::SqlitePoolOptions};
use uuid::Uuid;

#[derive(Debug)]
#[allow(dead_code)]
pub enum DalError {
    NotFound,
    DatabaseError(String),
}

pub trait UserRepository: Send + Sync {
    fn get_user_by_username(
        &self,
        username: &str,
    ) -> impl std::future::Future<Output = Result<Option<User>, DalError>> + Send;
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
                username TEXT UNIQUE NOT NULL,
                password_hash TEXT
            );",
        )
        .execute(&self.pool)
        .await?;

        // Insert demo user if it doesn't exist
        sqlx::query("INSERT OR IGNORE INTO users (id, username, password_hash) VALUES (?, ?, ?)")
            .bind(Uuid::new_v4().to_string())
            .bind("demo_user")
            .bind("")
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

impl UserRepository for DbUserRepository {
    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, DalError> {
        let row = sqlx::query("SELECT username FROM users WHERE username = ?")
            .bind(username)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DalError::DatabaseError(e.to_string()))?;

        match row {
            Some(record) => Ok(Some(User {
                username: record.get("username"),
            })),
            None => Ok(None),
        }
    }
}

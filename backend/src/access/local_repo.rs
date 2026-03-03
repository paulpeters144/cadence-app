use crate::domain::user::User;
use sqlx::{Row, SqlitePool, sqlite::SqlitePoolOptions};
use uuid::Uuid;

#[derive(Debug)]
#[allow(dead_code)]
pub enum DalError {
    NotFound,
    AlreadyExists,
    DatabaseError(String),
}

pub trait UserRepository: Send + Sync {
    fn get_user_by_username(
        &self,
        username: &str,
    ) -> impl std::future::Future<Output = Result<Option<User>, DalError>> + Send;

    fn get_user_by_username_with_hash(
        &self,
        username: &str,
    ) -> impl std::future::Future<Output = Result<Option<(User, String)>, DalError>> + Send;

    fn create_user(
        &self,
        username: &str,
        password_hash: &str,
    ) -> impl std::future::Future<Output = Result<(), DalError>> + Send;
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

        // Using a valid argon2 hash for "password123"
        let demo_password_hash = "$argon2id$v=19$m=19456,t=2,p=1$MmpS4mtt28qceHgV2OWZCg$3GB4vVNyFb2asA1kNUPGlw96imkXtVAtx5jalemz27U"; 
        
        sqlx::query("INSERT OR IGNORE INTO users (id, username, password_hash) VALUES (?, ?, ?)")
            .bind(Uuid::new_v4().to_string())
            .bind("demo_user")
            .bind(demo_password_hash)
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

    async fn get_user_by_username_with_hash(
        &self,
        username: &str,
    ) -> Result<Option<(User, String)>, DalError> {
        let row = sqlx::query("SELECT username, password_hash FROM users WHERE username = ?")
            .bind(username)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DalError::DatabaseError(e.to_string()))?;

        match row {
            Some(record) => Ok(Some((
                User {
                    username: record.get("username"),
                },
                record.get("password_hash"),
            ))),
            None => Ok(None),
        }
    }

    async fn create_user(&self, username: &str, password_hash: &str) -> Result<(), DalError> {
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
                if let Some(sqlite_error) = e.as_database_error() {
                    if sqlite_error.is_unique_violation() {
                        return Err(DalError::AlreadyExists);
                    }
                }
                Err(DalError::DatabaseError(e.to_string()))
            }
        }
    }
}

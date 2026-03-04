use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

pub struct AppRepository {
    pub(crate) pool: SqlitePool,
}

impl AppRepository {
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

pub mod user;
pub mod list;
pub mod task;

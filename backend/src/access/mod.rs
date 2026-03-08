pub mod error;
pub mod list;
pub mod task;
pub mod traits;
pub mod user;

pub use error::AccessError;
pub use traits::{DbExecutor, DbQuery, TransactionalRepository, UpdateListParams, UpdateTaskParams};
pub use user::{UserRepository, UserQuery, UserQueryResult};

use libsql::{Builder, Connection};

#[derive(Clone)]
pub struct AppRepository {
    pub(crate) conn: Connection,
}

impl traits::TransactionalRepository for AppRepository {
    async fn begin_transaction(
        &self,
    ) -> Result<libsql::Transaction, AccessError> {
        self.conn.transaction().await.map_err(|e| AccessError::DatabaseError(e.to_string()))
    }
}

impl AppRepository {
    pub async fn new() -> Self {
        let database_url = std::env::var("TURSO_DATABASE_URL").expect("TURSO_DATABASE_URL must be set");
        let auth_token = std::env::var("TURSO_AUTH_TOKEN").unwrap_or_default();
        
        if auth_token.is_empty() {
            panic!("TURSO_AUTH_TOKEN is required for remote database");
        }

        let db = Builder::new_remote(database_url, auth_token)
            .build()
            .await
            .expect("Failed to build remote libsql connection");

        let conn = db.connect().unwrap();
        Self { conn }
    }

    pub async fn new_in_memory() -> Self {
        let db = Builder::new_local(":memory:")
            .build()
            .await
            .expect("Failed to build in-memory libsql connection");
        let conn = db.connect().unwrap();
        Self { conn }
    }

    pub async fn init(&self) -> Result<(), libsql::Error> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                username TEXT UNIQUE NOT NULL COLLATE NOCASE,
                password_hash TEXT NOT NULL
            );", ()
        )
        .await?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS lists (
                id TEXT PRIMARY KEY,
                username TEXT NOT NULL,
                name TEXT NOT NULL,
                journal TEXT,
                archived BOOLEAN NOT NULL DEFAULT 0,
                archived_at TEXT,
                position REAL NOT NULL DEFAULT 0,
                FOREIGN KEY(username) REFERENCES users(username)
            );", ()
        )
        .await?;

        // Add position column if it doesn't exist (for existing databases)
        let _ = self.conn.execute("ALTER TABLE lists ADD COLUMN position REAL NOT NULL DEFAULT 0", ())
            .await;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                list_id TEXT NOT NULL,
                title TEXT NOT NULL,
                completed BOOLEAN NOT NULL DEFAULT 0,
                points REAL,
                created_at TEXT NOT NULL,
                completed_at TEXT,
                position REAL NOT NULL DEFAULT 0,
                FOREIGN KEY(list_id) REFERENCES lists(id)
            );", ()
        )
        .await?;

        // Add position column if it doesn't exist (for existing databases)
        let _ = self.conn.execute("ALTER TABLE tasks ADD COLUMN position REAL NOT NULL DEFAULT 0", ())
            .await;

        Ok(())
    }
}

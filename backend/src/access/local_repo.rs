use crate::Domain;
use sqlx::{Row, SqlitePool, sqlite::SqlitePoolOptions};
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

    fn get_user_pwd_hash(
        &self,
        username: &str,
    ) -> impl std::future::Future<Output = Result<Option<String>, AccessError>> + Send;

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

    fn get_lists(
        &self,
        username: &str,
        start_id: Option<Uuid>,
        take: Option<i32>,
    ) -> impl std::future::Future<Output = Result<Vec<Domain::List>, AccessError>> + Send;

    fn update_list(
        &self,
        username: &str,
        id: Uuid,
        name: Option<String>,
        journal: Option<String>,
        archived: Option<bool>,
    ) -> impl std::future::Future<Output = Result<Domain::List, AccessError>> + Send;
}

pub trait TaskRepository: Send + Sync {
    fn create_task(
        &self,
        username: &str,
        list_id: Uuid,
        title: &str,
        points: Option<f32>,
    ) -> impl std::future::Future<Output = Result<Domain::Task, AccessError>> + Send;

    fn get_tasks(
        &self,
        username: &str,
        list_id: Uuid,
    ) -> impl std::future::Future<Output = Result<Vec<Domain::Task>, AccessError>> + Send;

    fn update_task(
        &self,
        username: &str,
        list_id: Uuid,
        task_id: Uuid,
        title: Option<String>,
        completed: Option<bool>,
        points: Option<f32>,
    ) -> impl std::future::Future<Output = Result<Domain::Task, AccessError>> + Send;

    fn delete_task(
        &self,
        username: &str,
        list_id: Uuid,
        task_id: Uuid,
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

    async fn get_user_pwd_hash(&self, username: &str) -> Result<Option<String>, AccessError> {
        let row = sqlx::query("SELECT password_hash FROM users WHERE username = ?")
            .bind(username)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

        match row {
            Some(record) => Ok(Some(record.get("password_hash"))),
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

    async fn get_lists(
        &self,
        username: &str,
        start_id: Option<Uuid>,
        take: Option<i32>,
    ) -> Result<Vec<Domain::List>, AccessError> {
        let take = take.unwrap_or(50);
        let sql = if start_id.is_some() {
            format!(
                "SELECT id, name, journal, archived, archived_at FROM lists WHERE username = ? AND id = ? LIMIT {}",
                take
            )
        } else {
            format!(
                "SELECT id, name, journal, archived, archived_at FROM lists WHERE username = ? LIMIT {}",
                take
            )
        };

        let mut query = sqlx::query(&sql);
        query = query.bind(username);

        if let Some(start_id) = start_id {
            query = query.bind(start_id.to_string());
        }

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
    }

    async fn update_list(
        &self,
        username: &str,
        id: Uuid,
        name: Option<String>,
        journal: Option<String>,
        archived: Option<bool>,
    ) -> Result<Domain::List, AccessError> {
        let archived_at = archived.map(|a| if a { Some(chrono::Utc::now()) } else { None });

        let row = sqlx::query(
            "UPDATE lists
             SET name = COALESCE(?, name),
                 journal = CASE WHEN ? IS NOT NULL THEN ? ELSE journal END,
                 archived = COALESCE(?, archived),
                 archived_at = CASE
                    WHEN ? IS NOT NULL THEN ?
                    ELSE archived_at
                 END
             WHERE username = ? AND id = ?
             RETURNING id, name, journal, archived, archived_at",
        )
        .bind(name)
        .bind(journal.is_some()) // Using a flag for journal because COALESCE(?, journal) would not allow setting it to NULL if we wanted to (but here journal is Option<String> and we might want to clear it)
        .bind(journal)
        .bind(archived)
        .bind(archived.is_some())
        .bind(archived_at.flatten().map(|dt| dt.to_rfc3339()))
        .bind(username)
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

        let row = row.ok_or(AccessError::NotFound)?;

        let id_str: String = row.get("id");
        let id = Uuid::parse_str(&id_str).map_err(|e| AccessError::DatabaseError(e.to_string()))?;

        let archived_at_str: Option<String> = row.get("archived_at");
        let archived_at = match archived_at_str {
            Some(s) => Some(
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map_err(|e| AccessError::DatabaseError(e.to_string()))?
                    .with_timezone(&chrono::Utc),
            ),
            None => None,
        };

        Ok(Domain::List {
            id,
            name: row.get("name"),
            journal: row.get("journal"),
            archived: row.get("archived"),
            archived_at,
        })
    }
}

impl TaskRepository for DbUserRepository {
    async fn create_task(
        &self,
        username: &str,
        list_id: Uuid,
        title: &str,
        points: Option<f32>,
    ) -> Result<Domain::Task, AccessError> {
        // Verify list ownership first
        let list_exists = sqlx::query("SELECT 1 FROM lists WHERE id = ? AND username = ?")
            .bind(list_id.to_string())
            .bind(username)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

        if list_exists.is_none() {
            return Err(AccessError::NotFound);
        }

        let id = Uuid::new_v4();
        let created_at = chrono::Utc::now();

        sqlx::query(
            "INSERT INTO tasks (id, list_id, title, points, created_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(id.to_string())
        .bind(list_id.to_string())
        .bind(title)
        .bind(points)
        .bind(created_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

        Ok(Domain::Task {
            id,
            title: title.to_string(),
            completed: false,
            points,
            created_at,
            completed_at: None,
        })
    }

    async fn get_tasks(
        &self,
        username: &str,
        list_id: Uuid,
    ) -> Result<Vec<Domain::Task>, AccessError> {
        let rows = sqlx::query(
            "SELECT t.id, t.title, t.completed, t.points, t.created_at, t.completed_at
             FROM tasks t
             JOIN lists l ON t.list_id = l.id
             WHERE l.username = ? AND t.list_id = ?",
        )
        .bind(username)
        .bind(list_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

        let mut tasks = Vec::new();
        for row in rows {
            let id_str: String = row.get("id");
            let id = Uuid::parse_str(&id_str).map_err(|e| AccessError::DatabaseError(e.to_string()))?;

            let created_at_str: String = row.get("created_at");
            let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|e| AccessError::DatabaseError(e.to_string()))?
                .with_timezone(&chrono::Utc);

            let completed_at_str: Option<String> = row.get("completed_at");
            let completed_at = match completed_at_str {
                Some(s) => Some(
                    chrono::DateTime::parse_from_rfc3339(&s)
                        .map_err(|e| AccessError::DatabaseError(e.to_string()))?
                        .with_timezone(&chrono::Utc),
                ),
                None => None,
            };

            tasks.push(Domain::Task {
                id,
                title: row.get("title"),
                completed: row.get("completed"),
                points: row.get("points"),
                created_at,
                completed_at,
            });
        }

        Ok(tasks)
    }

    async fn update_task(
        &self,
        username: &str,
        list_id: Uuid,
        task_id: Uuid,
        title: Option<String>,
        completed: Option<bool>,
        points: Option<f32>,
    ) -> Result<Domain::Task, AccessError> {
        let completed_at = completed.map(|c| if c { Some(chrono::Utc::now()) } else { None });

        let row = sqlx::query(
            "UPDATE tasks
             SET title = COALESCE(?, title),
                 completed = COALESCE(?, completed),
                 points = CASE WHEN ? IS NOT NULL THEN ? ELSE points END,
                 completed_at = CASE
                    WHEN ? IS NOT NULL THEN ?
                    ELSE completed_at
                 END
             WHERE id = ? AND list_id IN (SELECT id FROM lists WHERE id = ? AND username = ?)
             RETURNING id, title, completed, points, created_at, completed_at",
        )
        .bind(title)
        .bind(completed)
        .bind(points.is_some())
        .bind(points)
        .bind(completed.is_some())
        .bind(completed_at.flatten().map(|dt| dt.to_rfc3339()))
        .bind(task_id.to_string())
        .bind(list_id.to_string())
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

        let row = row.ok_or(AccessError::NotFound)?;

        let id_str: String = row.get("id");
        let id = Uuid::parse_str(&id_str).map_err(|e| AccessError::DatabaseError(e.to_string()))?;

        let created_at_str: String = row.get("created_at");
        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| AccessError::DatabaseError(e.to_string()))?
            .with_timezone(&chrono::Utc);

        let completed_at_str: Option<String> = row.get("completed_at");
        let completed_at = match completed_at_str {
            Some(s) => Some(
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map_err(|e| AccessError::DatabaseError(e.to_string()))?
                    .with_timezone(&chrono::Utc),
            ),
            None => None,
        };

        Ok(Domain::Task {
            id,
            title: row.get("title"),
            completed: row.get("completed"),
            points: row.get("points"),
            created_at,
            completed_at,
        })
    }

    async fn delete_task(
        &self,
        username: &str,
        list_id: Uuid,
        task_id: Uuid,
    ) -> Result<(), AccessError> {
        let result = sqlx::query(
            "DELETE FROM tasks
             WHERE id = ? AND list_id IN (SELECT id FROM lists WHERE id = ? AND username = ?)",
        )
        .bind(task_id.to_string())
        .bind(list_id.to_string())
        .bind(username)
        .execute(&self.pool)
        .await
        .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(AccessError::NotFound);
        }

        Ok(())
    }
}

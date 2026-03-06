use std::future::Future;
use sqlx::{Executor, Row, Sqlite};
use sqlx::sqlite::SqliteRow;
use crate::Domain;
use crate::access::error::AccessError;
use crate::access::traits::DbQuery;

fn row_to_task(row: SqliteRow) -> Result<Domain::Task, AccessError> {
    let id: String = row.get("id");

    let created_at_str: String = row.get("created_at");
    let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
        .map_err(|e| AccessError::DatabaseError(e.to_string()))?
        .with_timezone(&chrono::Utc);

    let completed_at = row
        .get::<Option<String>, _>("completed_at")
        .map(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .map_err(|e| AccessError::DatabaseError(e.to_string()))
        })
        .transpose()?;

    Ok(Domain::Task {
        id,
        title: row.get("title"),
        completed: row.get("completed"),
        points: row.get("points"),
        position: row.get("position"),
        created_at,
        completed_at,
    })
}

pub struct CreateTask {
    pub id: String,
    pub list_id: String,
    pub title: String,
    pub points: Option<f32>,
    pub position: f32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl DbQuery for CreateTask {
    type Response = Domain::Task;

    fn execute<'e, E>(&self, executor: E) -> impl Future<Output = Result<Self::Response, AccessError>> + Send
    where
        E: Executor<'e, Database = Sqlite>
    {
        let id_str = self.id.to_string();
        let list_id_str = self.list_id.to_string();
        let title = self.title.clone();
        let points = self.points;
        let position = self.position;
        let created_at = self.created_at;
        
        let id = self.id.clone();

        async move {
            sqlx::query(
                "INSERT INTO tasks (id, list_id, title, points, created_at, position)
                 VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(&id_str)
            .bind(&list_id_str)
            .bind(&title)
            .bind(points)
            .bind(created_at.to_rfc3339())
            .bind(position)
            .execute(executor)
            .await
            .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

            Ok(Domain::Task {
                id,
                title,
                completed: false,
                points,
                position,
                created_at,
                completed_at: None,
            })
        }
    }
}

pub struct GetTasks {
    pub username: String,
    pub list_id: String,
}

impl DbQuery for GetTasks {
    type Response = Vec<Domain::Task>;

    fn execute<'e, E>(&self, executor: E) -> impl Future<Output = Result<Self::Response, AccessError>> + Send
    where
        E: Executor<'e, Database = Sqlite>
    {
        let username = self.username.clone();
        let list_id_str = self.list_id.to_string();

        async move {
            let rows = sqlx::query(
                "SELECT t.id, t.title, t.completed, t.points, t.created_at, t.completed_at, t.position
                 FROM tasks t
                 JOIN lists l ON t.list_id = l.id
                 WHERE l.username = ? AND t.list_id = ?
                 ORDER BY t.position ASC",
            )
            .bind(&username)
            .bind(&list_id_str)
            .fetch_all(executor)
            .await
            .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

            rows.into_iter().map(row_to_task).collect()
        }
    }
}

pub struct UpdateTask {
    pub username: String,
    pub list_id: String,
    pub task_id: String,
    pub title: Option<String>,
    pub completed: Option<bool>,
    pub points: Option<f32>,
    pub position: Option<f32>,
    pub new_list_id: Option<String>,
}

impl DbQuery for UpdateTask {
    type Response = Domain::Task;

    fn execute<'e, E>(&self, executor: E) -> impl Future<Output = Result<Self::Response, AccessError>> + Send
    where
        E: Executor<'e, Database = Sqlite>
    {
        let completed_at_str = self.completed
            .and_then(|c| c.then(|| chrono::Utc::now().to_rfc3339()));

        let title = self.title.clone();
        let completed = self.completed;
        let points_is_some = self.points.is_some();
        let points = self.points;
        let position = self.position;
        let completed_is_some = self.completed.is_some();
        
        let new_list_id_str = self.new_list_id.clone().map(|id| id.to_string());
        
        let task_id_str = self.task_id.to_string();
        let list_id_str = self.list_id.to_string();
        let username = self.username.clone();

        async move {
            let row = sqlx::query(
                "UPDATE tasks
                 SET title = COALESCE(?, title),
                     completed = COALESCE(?, completed),
                     points = CASE WHEN ? THEN ? ELSE points END,
                     position = COALESCE(?, position),
                     completed_at = CASE
                        WHEN ? THEN ?
                        ELSE completed_at
                     END,
                     list_id = COALESCE(?, list_id)
                 WHERE id = ? AND list_id IN (SELECT id FROM lists WHERE id = ? AND username = ?)
                 RETURNING id, title, completed, points, created_at, completed_at, position",
            )
            .bind(&title)
            .bind(completed)
            .bind(points_is_some)
            .bind(points)
            .bind(position)
            .bind(completed_is_some)
            .bind(&completed_at_str)
            .bind(&new_list_id_str)
            .bind(&task_id_str)
            .bind(&list_id_str)
            .bind(&username)
            .fetch_optional(executor)
            .await
            .map_err(|e| AccessError::DatabaseError(e.to_string()))?
            .ok_or(AccessError::NotFound)?;

            row_to_task(row)
        }
    }
}

pub struct DeleteTask {
    pub username: String,
    pub list_id: String,
    pub task_id: String,
}

impl DbQuery for DeleteTask {
    type Response = ();

    fn execute<'e, E>(&self, executor: E) -> impl Future<Output = Result<Self::Response, AccessError>> + Send
    where
        E: Executor<'e, Database = Sqlite>
    {
        let task_id_str = self.task_id.to_string();
        let list_id_str = self.list_id.to_string();
        let username = self.username.clone();

        async move {
            let result = sqlx::query(
                "DELETE FROM tasks
                 WHERE id = ? AND list_id IN (SELECT id FROM lists WHERE id = ? AND username = ?)",
            )
            .bind(&task_id_str)
            .bind(&list_id_str)
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

pub struct DeleteTasksByList {
    pub list_id: String,
}

impl DbQuery for DeleteTasksByList {
    type Response = ();

    fn execute<'e, E>(&self, executor: E) -> impl Future<Output = Result<Self::Response, AccessError>> + Send
    where
        E: Executor<'e, Database = Sqlite>
    {
        let list_id_str = self.list_id.to_string();

        async move {
            sqlx::query("DELETE FROM tasks WHERE list_id = ?")
                .bind(&list_id_str)
                .execute(executor)
                .await
                .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

            Ok(())
        }
    }
}

pub struct GetMaxTaskPosition {
    pub list_id: String,
}

impl DbQuery for GetMaxTaskPosition {
    type Response = f32;

    fn execute<'e, E>(&self, executor: E) -> impl Future<Output = Result<Self::Response, AccessError>> + Send
    where
        E: Executor<'e, Database = Sqlite>
    {
        let list_id_str = self.list_id.to_string();

        async move {
            let max_pos: (f32,) = sqlx::query_as("SELECT COALESCE(MAX(position), 0.0) FROM tasks WHERE list_id = ?")
                .bind(&list_id_str)
                .fetch_one(executor)
                .await
                .map_err(|e| AccessError::DatabaseError(e.to_string()))?;
            
            Ok(max_pos.0)
        }
    }
}

pub struct CheckTaskExists {
    pub username: String,
    pub list_id: String,
    pub task_id: String,
}

impl DbQuery for CheckTaskExists {
    type Response = bool;

    fn execute<'e, E>(&self, executor: E) -> impl Future<Output = Result<Self::Response, AccessError>> + Send
    where
        E: Executor<'e, Database = Sqlite>
    {
        let task_id_str = self.task_id.to_string();
        let list_id_str = self.list_id.to_string();
        let username = self.username.clone();

        async move {
            let row = sqlx::query("SELECT 1 FROM tasks t JOIN lists l ON t.list_id = l.id WHERE t.id = ? AND t.list_id = ? AND l.username = ?")
                .bind(&task_id_str)
                .bind(&list_id_str)
                .bind(&username)
                .fetch_optional(executor)
                .await
                .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

            Ok(row.is_some())
        }
    }
}
use std::future::Future;
use crate::Domain;
use crate::access::error::AccessError;
use crate::access::traits::{DbQuery, DbExecutor};

fn row_to_task(row: libsql::Row) -> Result<Domain::Task, AccessError> {
    let id: String = row.get(0).map_err(|e| AccessError::DatabaseError(e.to_string()))?;
    let title: String = row.get(1).map_err(|e| AccessError::DatabaseError(e.to_string()))?;
    let completed: bool = row.get(2).map_err(|e| AccessError::DatabaseError(e.to_string()))?;
    let points: Option<f64> = row.get(3).map_err(|e| AccessError::DatabaseError(e.to_string()))?;
    
    let created_at_str: String = row.get(4).map_err(|e| AccessError::DatabaseError(e.to_string()))?;
    let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
        .map_err(|e| AccessError::DatabaseError(e.to_string()))?
        .with_timezone(&chrono::Utc);

    let completed_at_str: Option<String> = row.get(5).map_err(|e| AccessError::DatabaseError(e.to_string()))?;
    let completed_at = completed_at_str
        .map(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .map_err(|e| AccessError::DatabaseError(e.to_string()))
        })
        .transpose()?;

    let position: f64 = row.get(6).map_err(|e| AccessError::DatabaseError(e.to_string()))?;

    Ok(Domain::Task {
        id,
        title,
        completed,
        points: points.map(|p| p as f32),
        position: position as f32,
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

    fn execute<'e>(&self, executor: &'e DbExecutor<'e>) -> impl Future<Output = Result<Self::Response, AccessError>> + Send {
        let id_str = self.id.to_string();
        let list_id_str = self.list_id.to_string();
        let title = self.title.clone();
        let points = self.points;
        let position = self.position;
        let created_at = self.created_at;
        
        let id = self.id.clone();

        async move {
            executor.execute(
                "INSERT INTO tasks (id, list_id, title, points, created_at, position)
                 VALUES (?, ?, ?, ?, ?, ?)",
                libsql::params![
                    id_str,
                    list_id_str,
                    title.clone(),
                    points.map(|p| p as f64),
                    created_at.to_rfc3339(),
                    position as f64
                ]
            )
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

    fn execute<'e>(&self, executor: &'e DbExecutor<'e>) -> impl Future<Output = Result<Self::Response, AccessError>> + Send {
        let username = self.username.clone();
        let list_id_str = self.list_id.to_string();

        async move {
            let mut rows = executor.query(
                "SELECT t.id, t.title, t.completed, t.points, t.created_at, t.completed_at, t.position
                 FROM tasks t
                 JOIN lists l ON t.list_id = l.id
                 WHERE l.username = ? AND t.list_id = ?
                 ORDER BY t.position ASC",
                 libsql::params![username, list_id_str]
            )
            .await
            .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

            let mut tasks = Vec::new();
            while let Some(row) = rows.next().await.map_err(|e| AccessError::DatabaseError(e.to_string()))? {
                tasks.push(row_to_task(row)?);
            }

            Ok(tasks)
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

    fn execute<'e>(&self, executor: &'e DbExecutor<'e>) -> impl Future<Output = Result<Self::Response, AccessError>> + Send {
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
            let mut rows = executor.query(
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
                 libsql::params![
                     title,
                     completed,
                     points_is_some,
                     points.map(|p| p as f64),
                     position.map(|p| p as f64),
                     completed_is_some,
                     completed_at_str,
                     new_list_id_str,
                     task_id_str,
                     list_id_str,
                     username
                 ]
            )
            .await
            .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

            let row = rows.next().await.map_err(|e| AccessError::DatabaseError(e.to_string()))?
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

    fn execute<'e>(&self, executor: &'e DbExecutor<'e>) -> impl Future<Output = Result<Self::Response, AccessError>> + Send {
        let task_id_str = self.task_id.to_string();
        let list_id_str = self.list_id.to_string();
        let username = self.username.clone();

        async move {
            let rows_affected = executor.execute(
                "DELETE FROM tasks
                 WHERE id = ? AND list_id IN (SELECT id FROM lists WHERE id = ? AND username = ?)",
                 libsql::params![task_id_str, list_id_str, username]
            )
            .await
            .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

            if rows_affected == 0 {
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

    fn execute<'e>(&self, executor: &'e DbExecutor<'e>) -> impl Future<Output = Result<Self::Response, AccessError>> + Send {
        let list_id_str = self.list_id.to_string();

        async move {
            executor.execute("DELETE FROM tasks WHERE list_id = ?", libsql::params![list_id_str])
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

    fn execute<'e>(&self, executor: &'e DbExecutor<'e>) -> impl Future<Output = Result<Self::Response, AccessError>> + Send {
        let list_id_str = self.list_id.to_string();

        async move {
            let mut rows = executor.query("SELECT COALESCE(MAX(position), 0.0) FROM tasks WHERE list_id = ?", libsql::params![list_id_str])
                .await
                .map_err(|e| AccessError::DatabaseError(e.to_string()))?;
            
            let row = rows.next().await.map_err(|e| AccessError::DatabaseError(e.to_string()))?
                .ok_or(AccessError::NotFound)?;
            
            let max_pos: f64 = row.get(0).unwrap_or(0.0);
            Ok(max_pos as f32)
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

    fn execute<'e>(&self, executor: &'e DbExecutor<'e>) -> impl Future<Output = Result<Self::Response, AccessError>> + Send {
        let task_id_str = self.task_id.to_string();
        let list_id_str = self.list_id.to_string();
        let username = self.username.clone();

        async move {
            let mut rows = executor.query("SELECT 1 FROM tasks t JOIN lists l ON t.list_id = l.id WHERE t.id = ? AND t.list_id = ? AND l.username = ?", 
                libsql::params![task_id_str, list_id_str, username])
                .await
                .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

            let row = rows.next().await.map_err(|e| AccessError::DatabaseError(e.to_string()))?;
            Ok(row.is_some())
        }
    }
}

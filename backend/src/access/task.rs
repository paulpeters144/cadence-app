use sqlx::Row;
use uuid::Uuid;
use crate::Domain;
use crate::access::error::AccessError;
use crate::access::traits::{TaskRepository, UpdateTaskParams};
use super::AppRepository;

impl TaskRepository for AppRepository {
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

        // Get max position to place new task at the end
        let max_pos: (f32,) = sqlx::query_as("SELECT COALESCE(MAX(position), 0.0) FROM tasks WHERE list_id = ?")
            .bind(list_id.to_string())
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AccessError::DatabaseError(e.to_string()))?;
        
        let position = max_pos.0 + 1024.0; // Use 1024 as spacing for fractional indexing

        sqlx::query(
            "INSERT INTO tasks (id, list_id, title, points, created_at, position)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(id.to_string())
        .bind(list_id.to_string())
        .bind(title)
        .bind(points)
        .bind(created_at.to_rfc3339())
        .bind(position)
        .execute(&self.pool)
        .await
        .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

        Ok(Domain::Task {
            id,
            title: title.to_string(),
            completed: false,
            points,
            position,
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
            "SELECT t.id, t.title, t.completed, t.points, t.created_at, t.completed_at, t.position
             FROM tasks t
             JOIN lists l ON t.list_id = l.id
             WHERE l.username = ? AND t.list_id = ?
             ORDER BY t.position ASC",
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
                position: row.get("position"),
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
        params: UpdateTaskParams,
    ) -> Result<Domain::Task, AccessError> {
        let completed_at = params.completed.map(|c| if c { Some(chrono::Utc::now()) } else { None });

        let row = sqlx::query(
            "UPDATE tasks
             SET title = COALESCE(?, title),
                 completed = COALESCE(?, completed),
                 points = CASE WHEN ? IS NOT NULL THEN ? ELSE points END,
                 position = COALESCE(?, position),
                 completed_at = CASE
                    WHEN ? IS NOT NULL THEN ?
                    ELSE completed_at
                 END
             WHERE id = ? AND list_id IN (SELECT id FROM lists WHERE id = ? AND username = ?)
             RETURNING id, title, completed, points, created_at, completed_at, position",
        )
        .bind(params.title)
        .bind(params.completed)
        .bind(params.points.is_some())
        .bind(params.points)
        .bind(params.position)
        .bind(params.completed.is_some())
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
            position: row.get("position"),
            created_at,
            completed_at,
        })
    }

    async fn move_task(
        &self,
        username: &str,
        task_id: Uuid,
        from_list_id: Uuid,
        to_list_id: Uuid,
        position: Option<f32>,
    ) -> Result<Domain::Task, AccessError> {
        let mut tx = self.pool.begin().await.map_err(|e| AccessError::DatabaseError(e.to_string()))?;

        // 1. Verify source list ownership and task existence
        let task_check = sqlx::query("SELECT 1 FROM tasks t JOIN lists l ON t.list_id = l.id WHERE t.id = ? AND t.list_id = ? AND l.username = ?")
            .bind(task_id.to_string())
            .bind(from_list_id.to_string())
            .bind(username)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

        if task_check.is_none() {
            return Err(AccessError::NotFound);
        }

        // 2. Verify destination list ownership
        let dest_check = sqlx::query("SELECT 1 FROM lists WHERE id = ? AND username = ?")
            .bind(to_list_id.to_string())
            .bind(username)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

        if dest_check.is_none() {
            return Err(AccessError::NotFound);
        }

        // 3. Determine new position if not provided
        let new_position = match position {
            Some(p) => p,
            None => {
                let max_pos: (f32,) = sqlx::query_as("SELECT COALESCE(MAX(position), 0.0) FROM tasks WHERE list_id = ?")
                    .bind(to_list_id.to_string())
                    .fetch_one(&mut *tx)
                    .await
                    .map_err(|e| AccessError::DatabaseError(e.to_string()))?;
                max_pos.0 + 1024.0
            }
        };

        // 4. Update the task
        let row = sqlx::query(
            "UPDATE tasks
             SET list_id = ?,
                 position = ?
             WHERE id = ?
             RETURNING id, title, completed, points, created_at, completed_at, position",
        )
        .bind(to_list_id.to_string())
        .bind(new_position)
        .bind(task_id.to_string())
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

        tx.commit().await.map_err(|e| AccessError::DatabaseError(e.to_string()))?;

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
            position: row.get("position"),
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

    async fn reorder_tasks(
        &self,
        username: &str,
        list_id: Uuid,
        active_id: Uuid,
        over_id: Uuid,
    ) -> Result<Domain::Task, AccessError> {
        if active_id == over_id {
            return self.update_task(username, list_id, active_id, UpdateTaskParams {
                title: None,
                completed: None,
                points: None,
                position: None,
            }).await;
        }

        let tasks = self.get_tasks(username, list_id).await?;

        let old_index = tasks.iter().position(|t| t.id == active_id).ok_or(AccessError::NotFound)?;
        let new_index = tasks.iter().position(|t| t.id == over_id).ok_or(AccessError::NotFound)?;

        let new_position = if new_index > old_index {
            // Moving down - place after over_id
            let over_pos = tasks[new_index].position;
            if new_index == tasks.len() - 1 {
                over_pos + 1024.0
            } else {
                let next_pos = tasks[new_index + 1].position;
                (over_pos + next_pos) / 2.0
            }
        } else {
            // Moving up - place before over_id
            let over_pos = tasks[new_index].position;
            if new_index == 0 {
                over_pos / 2.0
            } else {
                let prev_pos = tasks[new_index - 1].position;
                (over_pos + prev_pos) / 2.0
            }
        };

        self.update_task(username, list_id, active_id, UpdateTaskParams {
            title: None,
            completed: None,
            points: None,
            position: Some(new_position),
        }).await
    }
}

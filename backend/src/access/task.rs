use sqlx::Row;
use uuid::Uuid;
use crate::Domain;
use crate::access::error::AccessError;
use crate::access::traits::TaskRepository;
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

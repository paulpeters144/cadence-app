use sqlx::Row;
use uuid::Uuid;
use crate::Domain;
use crate::access::error::AccessError;
use crate::access::traits::{ListRepository, UpdateListParams};
use super::AppRepository;

impl ListRepository for AppRepository {
    async fn create_list(&self, username: &str, name: &str) -> Result<Domain::List, AccessError> {
        let id = Uuid::new_v4();
        let id_str = id.to_string();

        // Get max position
        let max_pos: (f32,) = sqlx::query_as("SELECT COALESCE(MAX(position), 0.0) FROM lists WHERE username = ?")
            .bind(username)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AccessError::DatabaseError(e.to_string()))?;
        
        let position = max_pos.0 + 1024.0;

        sqlx::query("INSERT INTO lists (id, username, name, position) VALUES (?, ?, ?, ?)")
            .bind(id_str)
            .bind(username)
            .bind(name)
            .bind(position)
            .execute(&self.pool)
            .await
            .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

        Ok(Domain::List {
            id,
            name: name.to_string(),
            journal: None,
            archived: false,
            archived_at: None,
            position,
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
                "SELECT id, name, journal, archived, archived_at, position FROM lists WHERE username = ? AND id = ? ORDER BY position ASC LIMIT {}",
                take
            )
        } else {
            format!(
                "SELECT id, name, journal, archived, archived_at, position FROM lists WHERE username = ? ORDER BY position ASC LIMIT {}",
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
                position: row.get("position"),
            });
        }

        Ok(lists)
    }

    async fn update_list(
        &self,
        username: &str,
        id: Uuid,
        params: UpdateListParams,
    ) -> Result<Domain::List, AccessError> {
        let archived_at = params.archived.map(|a| if a { Some(chrono::Utc::now()) } else { None });

        let row = sqlx::query(
            "UPDATE lists
             SET name = COALESCE(?, name),
                 journal = CASE WHEN ? IS NOT NULL THEN ? ELSE journal END,
                 archived = COALESCE(?, archived),
                 position = COALESCE(?, position),
                 archived_at = CASE
                    WHEN ? IS NOT NULL THEN ?
                    ELSE archived_at
                 END
             WHERE username = ? AND id = ?
             RETURNING id, name, journal, archived, archived_at, position",
        )
        .bind(params.name)
        .bind(params.journal.is_some())
        .bind(params.journal)
        .bind(params.archived)
        .bind(params.position)
        .bind(params.archived.is_some())
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
            position: row.get("position"),
        })
    }

    async fn delete_list(&self, username: &str, id: Uuid) -> Result<(), AccessError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

        // 1. Delete tasks belonging to the list
        sqlx::query("DELETE FROM tasks WHERE list_id = ?")
            .bind(id.to_string())
            .execute(&mut *tx)
            .await
            .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

        // 2. Delete the list itself
        let result = sqlx::query("DELETE FROM lists WHERE id = ? AND username = ?")
            .bind(id.to_string())
            .bind(username)
            .execute(&mut *tx)
            .await
            .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(AccessError::NotFound);
        }

        tx.commit()
            .await
            .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn duplicate_list(
        &self,
        username: &str,
        id: Uuid,
        new_name: &str,
    ) -> Result<Domain::List, AccessError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

        // 1. Verify source list exists and belongs to the user
        let list_exists = sqlx::query("SELECT 1 FROM lists WHERE id = ? AND username = ?")
            .bind(id.to_string())
            .bind(username)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

        if list_exists.is_none() {
            return Err(AccessError::NotFound);
        }

        // 2. Create new list
        let new_id = Uuid::new_v4();
        
        // Get max position for lists for this user
        let max_pos: (f32,) = sqlx::query_as("SELECT COALESCE(MAX(position), 0.0) FROM lists WHERE username = ?")
            .bind(username)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| AccessError::DatabaseError(e.to_string()))?;
        
        let position = max_pos.0 + 1024.0;

        sqlx::query("INSERT INTO lists (id, username, name, position) VALUES (?, ?, ?, ?)")
            .bind(new_id.to_string())
            .bind(username)
            .bind(new_name)
            .bind(position)
            .execute(&mut *tx)
            .await
            .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

        // 3. Fetch tasks from the source list
        let rows = sqlx::query(
            "SELECT title, points, position FROM tasks WHERE list_id = ? ORDER BY position ASC"
        )
        .bind(id.to_string())
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

        // 4. Batch insert tasks into the new list (or loop)
        for row in rows {
            let task_id = Uuid::new_v4();
            let created_at = chrono::Utc::now();
            let title: String = row.get("title");
            let points: Option<f32> = row.get("points");
            let task_position: f32 = row.get("position");

            sqlx::query(
                "INSERT INTO tasks (id, list_id, title, points, created_at, position)
                 VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(task_id.to_string())
            .bind(new_id.to_string())
            .bind(title)
            .bind(points)
            .bind(created_at.to_rfc3339())
            .bind(task_position)
            .execute(&mut *tx)
            .await
            .map_err(|e| AccessError::DatabaseError(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| AccessError::DatabaseError(e.to_string()))?;

        Ok(Domain::List {
            id: new_id,
            name: new_name.to_string(),
            journal: None,
            archived: false,
            archived_at: None,
            position,
        })
    }

    async fn reorder_lists(
        &self,
        username: &str,
        active_id: Uuid,
        over_id: Uuid,
    ) -> Result<Domain::List, AccessError> {
        if active_id == over_id {
            let row = sqlx::query(
                "SELECT id, name, journal, archived, archived_at, position FROM lists WHERE id = ? AND username = ?",
            )
            .bind(active_id.to_string())
            .bind(username)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => AccessError::NotFound,
                _ => AccessError::DatabaseError(e.to_string()),
            })?;

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

            return Ok(Domain::List {
                id,
                name: row.get("name"),
                journal: row.get("journal"),
                archived: row.get("archived"),
                archived_at,
                position: row.get("position"),
            });
        }

        let lists = self.get_lists(username, None, Some(1000)).await?;

        let old_index = lists.iter().position(|l| l.id == active_id).ok_or(AccessError::NotFound)?;
        let new_index = lists.iter().position(|l| l.id == over_id).ok_or(AccessError::NotFound)?;

        let new_position = if new_index > old_index {
            // Moving down - place after over_id
            let over_pos = lists[new_index].position;
            if new_index == lists.len() - 1 {
                over_pos + 1024.0
            } else {
                let next_pos = lists[new_index + 1].position;
                (over_pos + next_pos) / 2.0
            }
        } else {
            // Moving up - place before over_id
            let over_pos = lists[new_index].position;
            if new_index == 0 {
                over_pos / 2.0
            } else {
                let prev_pos = lists[new_index - 1].position;
                (over_pos + prev_pos) / 2.0
            }
        };

        self.update_list(username, active_id, UpdateListParams {
            name: None,
            journal: None,
            archived: None,
            position: Some(new_position),
        }).await
    }
}

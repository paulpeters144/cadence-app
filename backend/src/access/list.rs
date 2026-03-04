use sqlx::Row;
use uuid::Uuid;
use crate::Domain;
use crate::access::error::AccessError;
use crate::access::traits::ListRepository;
use super::AppRepository;

impl ListRepository for AppRepository {
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
        .bind(journal.is_some())
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
}

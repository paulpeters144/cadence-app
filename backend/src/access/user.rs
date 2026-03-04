use sqlx::Row;
use uuid::Uuid;
use crate::Domain;
use crate::access::error::AccessError;
use crate::access::traits::UserRepository;
use super::AppRepository;

impl UserRepository for AppRepository {
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

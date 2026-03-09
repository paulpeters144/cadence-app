use crate::Domain;
use crate::access::list::CheckListOwnership;
use crate::access::task::{
    CheckTaskExists, CreateTask, DeleteTask, GetMaxTaskPosition, GetTasks, UpdateTask,
};
use crate::access::traits::DbQuery;
use crate::access::{AccessError, DbExecutor, TransactionalRepository, UpdateTaskParams};
use super::{AppManager, ManagerError};

impl AppManager {
    pub async fn create_task(
        &self,
        username: &str,
        list_id: String,
        title: &str,
        points: Option<f32>,
    ) -> Result<Domain::Task, ManagerError> {
        let exec = DbExecutor::Connection(&self.user_repo.conn);
        let owns_list = CheckListOwnership {
            username: username.to_string(),
            id: list_id.clone(),
        }
        .execute(&exec)
        .await
        .map_err(|_| ManagerError::DatabaseError)?;

        if !owns_list {
            return Err(ManagerError::ListNotFound);
        }

        let max_pos = GetMaxTaskPosition {
            list_id: list_id.clone(),
        }
        .execute(&exec)
        .await
        .map_err(|_| ManagerError::DatabaseError)?;

        let position = max_pos + 1024.0;
        let id = crate::handlers::util::id::generate_short_id();
        let created_at = chrono::Utc::now();

        CreateTask {
            id,
            list_id,
            title: title.to_string(),
            points,
            position,
            created_at,
        }
        .execute(&exec)
        .await
        .map_err(|_| ManagerError::DatabaseError)
    }

    pub async fn get_tasks(
        &self,
        username: &str,
        list_id: String,
    ) -> Result<Vec<Domain::Task>, ManagerError> {
        let exec = DbExecutor::Connection(&self.user_repo.conn);
        GetTasks {
            username: username.to_string(),
            list_id,
        }
        .execute(&exec)
        .await
        .map_err(|_| ManagerError::DatabaseError)
    }

    pub async fn update_task(
        &self,
        username: &str,
        list_id: String,
        task_id: String,
        params: UpdateTaskParams,
    ) -> Result<Domain::Task, ManagerError> {
        let exec = DbExecutor::Connection(&self.user_repo.conn);
        UpdateTask {
            username: username.to_string(),
            list_id,
            task_id,
            title: params.title,
            completed: params.completed,
            points: params.points,
            position: params.position,
            new_list_id: None,
        }
        .execute(&exec)
        .await
        .map_err(|e| match e {
            AccessError::NotFound => ManagerError::TaskNotFound,
            _ => ManagerError::DatabaseError,
        })
    }

    pub async fn move_task(
        &self,
        username: &str,
        task_id: String,
        from_list_id: String,
        to_list_id: String,
        position: Option<f32>,
    ) -> Result<Domain::Task, ManagerError> {
        let tx = self
            .user_repo
            .begin_transaction()
            .await
            .map_err(|_| ManagerError::DatabaseError)?;

        let exec = DbExecutor::Transaction(&tx);

        let task_exists = CheckTaskExists {
            username: username.to_string(),
            list_id: from_list_id.clone(),
            task_id: task_id.clone(),
        }
        .execute(&exec)
        .await
        .map_err(|_| ManagerError::DatabaseError)?;

        if !task_exists {
            return Err(ManagerError::TaskNotFound);
        }

        let dest_owns = CheckListOwnership {
            username: username.to_string(),
            id: to_list_id.clone(),
        }
        .execute(&exec)
        .await
        .map_err(|_| ManagerError::DatabaseError)?;

        if !dest_owns {
            return Err(ManagerError::ListNotFound);
        }

        let new_position = match position {
            Some(p) => p,
            None => {
                let max_pos = GetMaxTaskPosition {
                    list_id: to_list_id.clone(),
                }
                .execute(&exec)
                .await
                .map_err(|_| ManagerError::DatabaseError)?;
                max_pos + 1024.0
            }
        };

        let updated_task = UpdateTask {
            username: username.to_string(),
            list_id: from_list_id,
            task_id,
            title: None,
            completed: None,
            points: None,
            position: Some(new_position),
            new_list_id: Some(to_list_id),
        }
        .execute(&exec)
        .await
        .map_err(|e| match e {
            AccessError::NotFound => ManagerError::TaskNotFound,
            _ => ManagerError::DatabaseError,
        })?;

        tx.commit().await.map_err(|_| ManagerError::DatabaseError)?;

        Ok(updated_task)
    }

    pub async fn delete_task(
        &self,
        username: &str,
        list_id: String,
        task_id: String,
    ) -> Result<(), ManagerError> {
        let exec = DbExecutor::Connection(&self.user_repo.conn);
        DeleteTask {
            username: username.to_string(),
            list_id,
            task_id,
        }
        .execute(&exec)
        .await
        .map_err(|e| match e {
            AccessError::NotFound => ManagerError::TaskNotFound,
            _ => ManagerError::DatabaseError,
        })
    }

    pub async fn reorder_tasks(
        &self,
        username: &str,
        list_id: String,
        active_id: String,
        over_id: String,
    ) -> Result<Domain::Task, ManagerError> {
        let exec = DbExecutor::Connection(&self.user_repo.conn);
        if active_id == over_id {
            return UpdateTask {
                username: username.to_string(),
                list_id: list_id.clone(),
                task_id: active_id,
                title: None,
                completed: None,
                points: None,
                position: None,
                new_list_id: None,
            }
            .execute(&exec)
            .await
            .map_err(|e| match e {
                AccessError::NotFound => ManagerError::TaskNotFound,
                _ => ManagerError::DatabaseError,
            });
        }

        let tasks = GetTasks {
            username: username.to_string(),
            list_id: list_id.clone(),
        }
        .execute(&exec)
        .await
        .map_err(|_| ManagerError::DatabaseError)?;

        let old_index = tasks
            .iter()
            .position(|t| t.id == active_id)
            .ok_or(ManagerError::TaskNotFound)?;
        let new_index = tasks
            .iter()
            .position(|t| t.id == over_id)
            .ok_or(ManagerError::TaskNotFound)?;

        let new_position = if new_index > old_index {
            let over_pos = tasks[new_index].position;
            if new_index == tasks.len() - 1 {
                over_pos + 1024.0
            } else {
                let next_pos = tasks[new_index + 1].position;
                (over_pos + next_pos) / 2.0
            }
        } else {
            let over_pos = tasks[new_index].position;
            if new_index == 0 {
                over_pos / 2.0
            } else {
                let prev_pos = tasks[new_index - 1].position;
                (over_pos + prev_pos) / 2.0
            }
        };

        UpdateTask {
            username: username.to_string(),
            list_id,
            task_id: active_id,
            title: None,
            completed: None,
            points: None,
            position: Some(new_position),
            new_list_id: None,
        }
        .execute(&exec)
        .await
        .map_err(|e| match e {
            AccessError::NotFound => ManagerError::TaskNotFound,
            _ => ManagerError::DatabaseError,
        })
    }
}

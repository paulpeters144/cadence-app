use crate::Domain;
use crate::access::list::{
    CheckListOwnership, CreateList, DeleteList, GetList, GetLists, GetMaxListPosition, UpdateList,
};
use crate::access::task::{CreateTask, DeleteTasksByList, GetTasks};
use crate::access::traits::DbQuery;
use crate::access::{AccessError, DbExecutor, TransactionalRepository, UpdateListParams};
use super::{AppManager, ManagerError};

impl AppManager {
    pub async fn create_list(
        &self,
        username: &str,
        name: &str,
    ) -> Result<Domain::List, ManagerError> {
        let exec = DbExecutor::Connection(&self.user_repo.conn);
        let max_pos = GetMaxListPosition {
            username: username.to_string(),
        }
        .execute(&exec)
        .await
        .map_err(|_| ManagerError::DatabaseError)?;

        let position = max_pos + 1024.0;
        let id = crate::handlers::util::id::generate_short_id();

        CreateList {
            id,
            username: username.to_string(),
            name: name.to_string(),
            position,
        }
        .execute(&exec)
        .await
        .map_err(|_| ManagerError::DatabaseError)
    }

    pub async fn get_lists(
        &self,
        username: &str,
        start_id: Option<String>,
        take: Option<i32>,
    ) -> Result<Vec<Domain::List>, ManagerError> {
        let exec = DbExecutor::Connection(&self.user_repo.conn);
        GetLists {
            username: username.to_string(),
            start_id,
            take,
        }
        .execute(&exec)
        .await
        .map_err(|_| ManagerError::DatabaseError)
    }

    pub async fn update_list(
        &self,
        username: &str,
        id: String,
        params: UpdateListParams,
    ) -> Result<Domain::List, ManagerError> {
        let exec = DbExecutor::Connection(&self.user_repo.conn);
        UpdateList {
            username: username.to_string(),
            id,
            name: params.name,
            journal: params.journal,
            archived: params.archived,
            position: params.position,
        }
        .execute(&exec)
        .await
        .map_err(|e| match e {
            AccessError::NotFound => ManagerError::ListNotFound,
            _ => ManagerError::DatabaseError,
        })
    }

    pub async fn delete_list(&self, username: &str, id: String) -> Result<(), ManagerError> {
        let tx = self
            .user_repo
            .begin_transaction()
            .await
            .map_err(|_| ManagerError::DatabaseError)?;

        let exec = DbExecutor::Transaction(&tx);

        // First, check if the user owns the list
        let owns_list = CheckListOwnership {
            username: username.to_string(),
            id: id.clone(),
        }
        .execute(&exec)
        .await
        .map_err(|_| ManagerError::DatabaseError)?;

        if !owns_list {
            return Err(ManagerError::ListNotFound);
        }

        DeleteTasksByList {
            list_id: id.clone(),
        }
        .execute(&exec)
        .await
        .map_err(|_| ManagerError::DatabaseError)?;

        DeleteList {
            username: username.to_string(),
            id,
        }
        .execute(&exec)
        .await
        .map_err(|e| match e {
            AccessError::NotFound => ManagerError::ListNotFound,
            _ => ManagerError::DatabaseError,
        })?;

        tx.commit().await.map_err(|_| ManagerError::DatabaseError)?;
        Ok(())
    }

    pub async fn duplicate_list(
        &self,
        username: &str,
        id: String,
        new_name: &str,
    ) -> Result<Domain::List, ManagerError> {
        let tx = self
            .user_repo
            .begin_transaction()
            .await
            .map_err(|_| ManagerError::DatabaseError)?;

        let exec = DbExecutor::Transaction(&tx);

        let owns_list = CheckListOwnership {
            username: username.to_string(),
            id: id.clone(),
        }
        .execute(&exec)
        .await
        .map_err(|_| ManagerError::DatabaseError)?;

        if !owns_list {
            return Err(ManagerError::ListNotFound);
        }

        let max_pos = GetMaxListPosition {
            username: username.to_string(),
        }
        .execute(&exec)
        .await
        .map_err(|_| ManagerError::DatabaseError)?;

        let position = max_pos + 1024.0;
        let new_id = crate::handlers::util::id::generate_short_id();

        let new_list = CreateList {
            id: new_id.clone(),
            username: username.to_string(),
            name: new_name.to_string(),
            position,
        }
        .execute(&exec)
        .await
        .map_err(|_| ManagerError::DatabaseError)?;

        let tasks = GetTasks {
            username: username.to_string(),
            list_id: id,
        }
        .execute(&exec)
        .await
        .map_err(|_| ManagerError::DatabaseError)?;

        for task in tasks {
            let new_task_id = crate::handlers::util::id::generate_short_id();
            CreateTask {
                id: new_task_id,
                list_id: new_id.clone(),
                title: task.title,
                points: task.points,
                position: task.position,
                created_at: chrono::Utc::now(),
            }
            .execute(&exec)
            .await
            .map_err(|_| ManagerError::DatabaseError)?;
        }

        tx.commit().await.map_err(|_| ManagerError::DatabaseError)?;

        Ok(new_list)
    }

    pub async fn reorder_lists(
        &self,
        username: &str,
        active_id: String,
        over_id: String,
    ) -> Result<Domain::List, ManagerError> {
        let exec = DbExecutor::Connection(&self.user_repo.conn);
        if active_id == over_id {
            return GetList {
                username: username.to_string(),
                id: active_id,
            }
            .execute(&exec)
            .await
            .map_err(|e| match e {
                AccessError::NotFound => ManagerError::ListNotFound,
                _ => ManagerError::DatabaseError,
            });
        }

        let lists = GetLists {
            username: username.to_string(),
            start_id: None,
            take: Some(1000),
        }
        .execute(&exec)
        .await
        .map_err(|_| ManagerError::DatabaseError)?;

        let old_index = lists
            .iter()
            .position(|l| l.id == active_id)
            .ok_or(ManagerError::ListNotFound)?;
        let new_index = lists
            .iter()
            .position(|l| l.id == over_id)
            .ok_or(ManagerError::ListNotFound)?;

        let new_position = if new_index > old_index {
            let over_pos = lists[new_index].position;
            if new_index == lists.len() - 1 {
                over_pos + 1024.0
            } else {
                let next_pos = lists[new_index + 1].position;
                (over_pos + next_pos) / 2.0
            }
        } else {
            let over_pos = lists[new_index].position;
            if new_index == 0 {
                over_pos / 2.0
            } else {
                let prev_pos = lists[new_index - 1].position;
                (over_pos + prev_pos) / 2.0
            }
        };

        UpdateList {
            username: username.to_string(),
            id: active_id,
            name: None,
            journal: None,
            archived: None,
            position: Some(new_position),
        }
        .execute(&exec)
        .await
        .map_err(|e| match e {
            AccessError::NotFound => ManagerError::ListNotFound,
            _ => ManagerError::DatabaseError,
        })
    }
}

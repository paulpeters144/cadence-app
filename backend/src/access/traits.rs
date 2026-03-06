use super::error::AccessError;
use crate::Domain;
use uuid::Uuid;

pub struct UpdateListParams {
    pub name: Option<String>,
    pub journal: Option<String>,
    pub archived: Option<bool>,
    pub position: Option<f32>,
}

pub struct UpdateTaskParams {
    pub title: Option<String>,
    pub completed: Option<bool>,
    pub points: Option<f32>,
    pub position: Option<f32>,
}

pub trait UserRepository {
    fn create_user(
        &self,
        username: &str,
        password_hash: &str,
    ) -> impl std::future::Future<Output = Result<(), AccessError>> + Send;

    fn get_user_by_username(
        &self,
        username: &str,
    ) -> impl std::future::Future<Output = Result<Option<Domain::User>, AccessError>> + Send;

    fn get_user_pwd_hash(
        &self,
        username: &str,
    ) -> impl std::future::Future<Output = Result<Option<String>, AccessError>> + Send;
}

pub trait ListRepository {
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
        params: UpdateListParams,
    ) -> impl std::future::Future<Output = Result<Domain::List, AccessError>> + Send;

    fn delete_list(
        &self,
        username: &str,
        id: Uuid,
    ) -> impl std::future::Future<Output = Result<(), AccessError>> + Send;

    fn duplicate_list(
        &self,
        username: &str,
        id: Uuid,
        new_name: &str,
    ) -> impl std::future::Future<Output = Result<Domain::List, AccessError>> + Send;

    fn reorder_lists(
        &self,
        username: &str,
        active_id: Uuid,
        over_id: Uuid,
    ) -> impl std::future::Future<Output = Result<Domain::List, AccessError>> + Send;
}

pub trait TaskRepository {
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
        params: UpdateTaskParams,
    ) -> impl std::future::Future<Output = Result<Domain::Task, AccessError>> + Send;

    fn move_task(
        &self,
        username: &str,
        task_id: Uuid,
        from_list_id: Uuid,
        to_list_id: Uuid,
        position: Option<f32>,
    ) -> impl std::future::Future<Output = Result<Domain::Task, AccessError>> + Send;

    fn delete_task(
        &self,
        username: &str,
        list_id: Uuid,
        task_id: Uuid,
    ) -> impl std::future::Future<Output = Result<(), AccessError>> + Send;

    fn reorder_tasks(
        &self,
        username: &str,
        list_id: Uuid,
        active_id: Uuid,
        over_id: Uuid,
    ) -> impl std::future::Future<Output = Result<Domain::Task, AccessError>> + Send;
}


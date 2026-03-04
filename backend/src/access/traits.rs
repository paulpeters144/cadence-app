use crate::Domain;
use uuid::Uuid;
use super::error::AccessError;

pub trait UserRepository: Send + Sync {
    fn get_user_by_username(
        &self,
        username: &str,
    ) -> impl std::future::Future<Output = Result<Option<Domain::User>, AccessError>> + Send;

    fn get_user_pwd_hash(
        &self,
        username: &str,
    ) -> impl std::future::Future<Output = Result<Option<String>, AccessError>> + Send;

    fn create_user(
        &self,
        username: &str,
        password_hash: &str,
    ) -> impl std::future::Future<Output = Result<(), AccessError>> + Send;
}

pub trait ListRepository: Send + Sync {
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
        name: Option<String>,
        journal: Option<String>,
        archived: Option<bool>,
    ) -> impl std::future::Future<Output = Result<Domain::List, AccessError>> + Send;

    fn delete_list(
        &self,
        username: &str,
        id: Uuid,
    ) -> impl std::future::Future<Output = Result<(), AccessError>> + Send;
}

pub trait TaskRepository: Send + Sync {
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
        title: Option<String>,
        completed: Option<bool>,
        points: Option<f32>,
    ) -> impl std::future::Future<Output = Result<Domain::Task, AccessError>> + Send;

    fn delete_task(
        &self,
        username: &str,
        list_id: Uuid,
        task_id: Uuid,
    ) -> impl std::future::Future<Output = Result<(), AccessError>> + Send;
}

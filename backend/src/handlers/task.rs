use crate::AppState;
use crate::Domain;
use crate::error::{AppError, ErrorResponse};
use crate::handlers::util::auth::AuthenticatedUser;
use axum::{Json, extract::State, extract::Path};
use axum_valid::Valid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

pub const PATH_TASKS: &str = "/api/lists/{listId}/tasks";
pub const PATH_TASK_ID: &str = "/api/lists/{listId}/tasks/{taskId}";

// -----------------------------------------------------------------------------
// SCHEMAS
// -----------------------------------------------------------------------------

#[derive(Deserialize, Validate, utoipa::ToSchema)]
pub struct CreateTaskRequest {
    #[validate(length(min = 1, message = "Title cannot be empty"))]
    pub title: String,
    pub points: Option<f32>,
}

#[derive(Deserialize, Validate, utoipa::ToSchema)]
pub struct UpdateTaskRequest {
    pub title: Option<String>,
    pub completed: Option<bool>,
    pub points: Option<f32>,
}

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TaskResponse {
    pub id: Uuid,
    pub title: String,
    pub completed: bool,
    pub points: Option<f32>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl From<Domain::Task> for TaskResponse {
    fn from(task: Domain::Task) -> Self {
        Self {
            id: task.id,
            title: task.title,
            completed: task.completed,
            points: task.points,
            created_at: task.created_at,
            completed_at: task.completed_at,
        }
    }
}

// -----------------------------------------------------------------------------
// HANDLERS
// -----------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/api/lists/{listId}/tasks",
    tag = "Tasks",
    params(
        ("listId" = Uuid, Path, description = "List ID")
    ),
    request_body = CreateTaskRequest,
    responses(
        (status = 201, description = "Task created successfully", body = TaskResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "List not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn create_task(
    State(manager): State<AppState>,
    auth: AuthenticatedUser,
    Path(list_id): Path<Uuid>,
    Valid(Json(payload)): Valid<Json<CreateTaskRequest>>,
) -> Result<(axum::http::StatusCode, Json<TaskResponse>), AppError> {
    let task = manager
        .create_task(&auth.username, list_id, &payload.title, payload.points)
        .await
        .map_err(|e| match e {
            crate::manager::app_manager::ManagerError::ListNotFound => {
                AppError::NotFound("List not found".to_string())
            }
            _ => AppError::InternalServerError("Failed to create task".to_string()),
        })?;

    Ok((
        axum::http::StatusCode::CREATED,
        Json(TaskResponse::from(task)),
    ))
}

#[utoipa::path(
    get,
    path = "/api/lists/{listId}/tasks",
    tag = "Tasks",
    params(
        ("listId" = Uuid, Path, description = "List ID")
    ),
    responses(
        (status = 200, description = "Tasks retrieved successfully", body = [TaskResponse]),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn get_tasks(
    State(manager): State<AppState>,
    auth: AuthenticatedUser,
    Path(list_id): Path<Uuid>,
) -> Result<Json<Vec<TaskResponse>>, AppError> {
    let tasks = manager
        .get_tasks(&auth.username, list_id)
        .await
        .map_err(|_| AppError::InternalServerError("Failed to fetch tasks".to_string()))?;

    Ok(Json(tasks.into_iter().map(TaskResponse::from).collect()))
}

#[utoipa::path(
    patch,
    path = "/api/lists/{listId}/tasks/{taskId}",
    tag = "Tasks",
    params(
        ("listId" = Uuid, Path, description = "List ID"),
        ("taskId" = Uuid, Path, description = "Task ID")
    ),
    request_body = UpdateTaskRequest,
    responses(
        (status = 200, description = "Task updated successfully", body = TaskResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Task not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn update_task(
    State(manager): State<AppState>,
    auth: AuthenticatedUser,
    Path((list_id, task_id)): Path<(Uuid, Uuid)>,
    Valid(Json(payload)): Valid<Json<UpdateTaskRequest>>,
) -> Result<Json<TaskResponse>, AppError> {
    let task = manager
        .update_task(
            &auth.username,
            list_id,
            task_id,
            payload.title,
            payload.completed,
            payload.points,
        )
        .await
        .map_err(|e| match e {
            crate::manager::app_manager::ManagerError::TaskNotFound => {
                AppError::NotFound("Task not found".to_string())
            }
            _ => AppError::InternalServerError("Failed to update task".to_string()),
        })?;

    Ok(Json(TaskResponse::from(task)))
}

#[utoipa::path(
    delete,
    path = "/api/lists/{listId}/tasks/{taskId}",
    tag = "Tasks",
    params(
        ("listId" = Uuid, Path, description = "List ID"),
        ("taskId" = Uuid, Path, description = "Task ID")
    ),
    responses(
        (status = 204, description = "Task deleted successfully"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Task not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn delete_task(
    State(manager): State<AppState>,
    auth: AuthenticatedUser,
    Path((list_id, task_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, AppError> {
    manager
        .delete_task(&auth.username, list_id, task_id)
        .await
        .map_err(|e| match e {
            crate::manager::app_manager::ManagerError::TaskNotFound => {
                AppError::NotFound("Task not found".to_string())
            }
            _ => AppError::InternalServerError("Failed to delete task".to_string()),
        })?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}

use crate::AppState;
use crate::Domain;
use crate::access::UpdateTaskParams;
use crate::error::{AppError, ErrorResponse};
use crate::handlers::util::auth::AuthenticatedUser;
use axum::{Json, extract::State, extract::Path};
use axum_valid::Valid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// -----------------------------------------------------------------------------
// RESPONSE SCHEMAS (SHARED)
// -----------------------------------------------------------------------------

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TaskResponse {
    pub id: Uuid,
    pub title: String,
    pub completed: bool,
    pub points: Option<f32>,
    pub position: f32,
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
            position: task.position,
            created_at: task.created_at,
            completed_at: task.completed_at,
        }
    }
}

// -----------------------------------------------------------------------------
// GET TASKS FOR A LIST
// -----------------------------------------------------------------------------
pub const PATH_TASKS: &str = "/api/lists/{listId}/tasks";

#[utoipa::path(
    get,
    path = PATH_TASKS,
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

// -----------------------------------------------------------------------------
// CREATE TASK
// -----------------------------------------------------------------------------

#[derive(Deserialize, Validate, utoipa::ToSchema)]
pub struct CreateTaskRequest {
    #[validate(length(min = 1, message = "Title cannot be empty"))]
    pub title: String,
    pub points: Option<f32>,
}

#[utoipa::path(
    post,
    path = PATH_TASKS,
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

// -----------------------------------------------------------------------------
// UPDATE TASK
// -----------------------------------------------------------------------------
pub const PATH_TASK_ID: &str = "/api/lists/{listId}/tasks/{taskId}";

#[derive(Deserialize, Validate, utoipa::ToSchema)]
pub struct UpdateTaskRequest {
    pub title: Option<String>,
    pub completed: Option<bool>,
    pub points: Option<f32>,
    pub position: Option<f32>,
}

#[utoipa::path(
    patch,
    path = PATH_TASK_ID,
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
            UpdateTaskParams {
                title: payload.title,
                completed: payload.completed,
                points: payload.points,
                position: payload.position,
            },
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

// -----------------------------------------------------------------------------
// DELETE TASK
// -----------------------------------------------------------------------------

#[utoipa::path(
    delete,
    path = PATH_TASK_ID,
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

// -----------------------------------------------------------------------------
// MOVE TASK
// -----------------------------------------------------------------------------
pub const PATH_TASK_MOVE: &str = "/api/tasks/{taskId}/move";

#[derive(Deserialize, Validate, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MoveTaskRequest {
    pub from_list_id: Uuid,
    pub to_list_id: Uuid,
    pub position: Option<f32>,
}

#[utoipa::path(
    post,
    path = PATH_TASK_MOVE,
    tag = "Tasks",
    params(
        ("taskId" = Uuid, Path, description = "Task ID")
    ),
    request_body = MoveTaskRequest,
    responses(
        (status = 200, description = "Task moved successfully", body = TaskResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Task or List not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn move_task(
    State(manager): State<AppState>,
    auth: AuthenticatedUser,
    Path(task_id): Path<Uuid>,
    Valid(Json(payload)): Valid<Json<MoveTaskRequest>>,
) -> Result<Json<TaskResponse>, AppError> {
    let task = manager
        .move_task(
            &auth.username,
            task_id,
            payload.from_list_id,
            payload.to_list_id,
            payload.position,
        )
        .await
        .map_err(|e| match e {
            crate::manager::app_manager::ManagerError::TaskNotFound => {
                AppError::NotFound("Task or list not found".to_string())
            }
            _ => AppError::InternalServerError("Failed to move task".to_string()),
        })?;

    Ok(Json(TaskResponse::from(task)))
}

// -----------------------------------------------------------------------------
// REORDER TASKS
// -----------------------------------------------------------------------------
pub const PATH_TASKS_REORDER: &str = "/api/lists/{listId}/tasks/reorder";

#[derive(Deserialize, Validate, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TaskReorderRequest {
    pub active_id: Uuid,
    pub over_id: Uuid,
}

#[utoipa::path(
    post,
    path = PATH_TASKS_REORDER,
    tag = "Tasks",
    params(
        ("listId" = Uuid, Path, description = "List ID")
    ),
    request_body = TaskReorderRequest,
    responses(
        (status = 200, description = "Tasks reordered successfully", body = TaskResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Task or List not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn reorder_tasks(
    State(manager): State<AppState>,
    auth: AuthenticatedUser,
    Path(list_id): Path<Uuid>,
    Valid(Json(payload)): Valid<Json<TaskReorderRequest>>,
) -> Result<Json<TaskResponse>, AppError> {
    let task = manager
        .reorder_tasks(&auth.username, list_id, payload.active_id, payload.over_id)
        .await
        .map_err(|e| match e {
            crate::manager::app_manager::ManagerError::TaskNotFound => {
                AppError::NotFound("Task not found".to_string())
            }
            crate::manager::app_manager::ManagerError::ListNotFound => {
                AppError::NotFound("List not found".to_string())
            }
            _ => AppError::InternalServerError("Failed to reorder tasks".to_string()),
        })?;

    Ok(Json(TaskResponse::from(task)))
}

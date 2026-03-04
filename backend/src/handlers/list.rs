use crate::AppState;
use crate::Domain;
use crate::error::{AppError, ErrorResponse};
use crate::handlers::util::auth::AuthenticatedUser;
use axum::{Json, extract::State};
use axum_valid::Valid;
use serde::{Deserialize, Serialize};
use validator::Validate;
use uuid::Uuid;
use chrono::{DateTime, Utc};

pub const PATH_LISTS: &str = "/api/lists";

// -----------------------------------------------------------------------------
// SCHEMAS
// -----------------------------------------------------------------------------

#[derive(Deserialize, Validate, utoipa::ToSchema)]
pub struct CreateListRequest {
    #[validate(length(min = 1, message = "Name cannot be empty"))]
    pub name: String,
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

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ListResponse {
    pub id: Uuid,
    pub name: String,
    pub journal: Option<String>,
    pub archived: bool,
    pub archived_at: Option<DateTime<Utc>>,
    pub tasks: Vec<TaskResponse>,
}

impl From<Domain::List> for ListResponse {
    fn from(list: Domain::List) -> Self {
        Self {
            id: list.id,
            name: list.name,
            journal: list.journal,
            archived: list.archived,
            archived_at: list.archived_at,
            tasks: list.tasks.into_iter().map(TaskResponse::from).collect(),
        }
    }
}

// -----------------------------------------------------------------------------
// HANDLERS
// -----------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = PATH_LISTS,
    tag = "Lists",
    request_body = CreateListRequest,
    responses(
        (status = 201, description = "List created successfully", body = ListResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn create_list(
    State(manager): State<AppState>,
    auth: AuthenticatedUser,
    Valid(Json(payload)): Valid<Json<CreateListRequest>>,
) -> Result<(axum::http::StatusCode, Json<ListResponse>), AppError> {
    let list = manager
        .create_list(&auth.username, &payload.name)
        .await
        .map_err(|_| AppError::InternalServerError("Failed to create list".to_string()))?;

    Ok((axum::http::StatusCode::CREATED, Json(ListResponse::from(list))))
}

#[utoipa::path(
    get,
    path = PATH_LISTS,
    tag = "Lists",
    responses(
        (status = 200, description = "Lists retrieved successfully", body = [ListResponse]),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn get_all_lists(
    State(manager): State<AppState>,
    auth: AuthenticatedUser,
) -> Result<Json<Vec<ListResponse>>, AppError> {
    let lists = manager
        .get_all_lists(&auth.username)
        .await
        .map_err(|_| AppError::InternalServerError("Failed to fetch lists".to_string()))?;

    Ok(Json(lists.into_iter().map(ListResponse::from).collect()))
}

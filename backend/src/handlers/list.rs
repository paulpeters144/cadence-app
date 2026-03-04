use crate::AppState;
use crate::Domain;
use crate::error::{AppError, ErrorResponse};
use crate::handlers::util::auth::AuthenticatedUser;
use axum::extract::Query;
use axum::{Json, extract::State};
use axum_valid::Valid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

pub const PATH_LISTS: &str = "/api/lists";

// -----------------------------------------------------------------------------
// SCHEMAS
// -----------------------------------------------------------------------------

#[derive(Deserialize, Validate, utoipa::ToSchema)]
pub struct CreateListRequest {
    #[validate(length(min = 1, message = "Name cannot be empty"))]
    pub name: String,
}

#[derive(Deserialize, Validate, utoipa::ToSchema)]
pub struct UpdateListRequest {
    #[validate(length(min = 1, message = "Name cannot be empty"))]
    pub name: Option<String>,
    pub journal: Option<String>,
    pub archived: Option<bool>,
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
}

impl From<Domain::List> for ListResponse {
    fn from(list: Domain::List) -> Self {
        Self {
            id: list.id,
            name: list.name,
            journal: list.journal,
            archived: list.archived,
            archived_at: list.archived_at,
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

    Ok((
        axum::http::StatusCode::CREATED,
        Json(ListResponse::from(list)),
    ))
}

#[derive(Deserialize, Validate)]
pub struct ListQueryParams {
    pub start_id: Option<Uuid>,
    #[validate(range(max = 500, message = "Cannot take more than 500 lists"))]
    pub take: Option<i32>,
}

#[utoipa::path(
    get,
    path = PATH_LISTS,
    tag = "Lists",
    params(
        ("startId" = Option<String>, Query, description = "Starting ID for pagination"),
        ("take" = Option<i32>, Query, description = "Number of items to retrieve"),
    ),
    responses(
        (status = 200, description = "Lists retrieved successfully", body = [ListResponse]),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn get_lists(
    State(manager): State<AppState>,
    auth: AuthenticatedUser,
    Valid(Query(params)): Valid<Query<ListQueryParams>>,
) -> Result<Json<Vec<ListResponse>>, AppError> {
    let ListQueryParams { start_id, take } = params;
    let lists = manager
        .get_lists(&auth.username, start_id, take)
        .await
        .map_err(|_| AppError::InternalServerError("Failed to fetch lists".to_string()))?;

    Ok(Json(lists.into_iter().map(ListResponse::from).collect()))
}

#[utoipa::path(
    patch,
    path = "/api/lists/{id}",
    tag = "Lists",
    params(
        ("id" = Uuid, Path, description = "List ID")
    ),
    request_body = UpdateListRequest,
    responses(
        (status = 200, description = "List updated successfully", body = ListResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "List not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn update_list(
    State(manager): State<AppState>,
    auth: AuthenticatedUser,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    Valid(Json(payload)): Valid<Json<UpdateListRequest>>,
) -> Result<Json<ListResponse>, AppError> {
    let list = manager
        .update_list(
            &auth.username,
            id,
            payload.name,
            payload.journal,
            payload.archived,
        )
        .await
        .map_err(|e| match e {
            crate::manager::app_manager::ManagerError::ListNotFound => {
                AppError::NotFound("List not found".to_string())
            }
            crate::manager::app_manager::ManagerError::UserNotFound => {
                AppError::NotFound("User not found".to_string())
            }
            _ => AppError::InternalServerError("Failed to update list".to_string()),
        })?;

    Ok(Json(ListResponse::from(list)))
}

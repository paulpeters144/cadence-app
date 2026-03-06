use crate::AppState;
use crate::Domain;
use crate::access::UpdateListParams;
use crate::error::{AppError, ErrorResponse};
use crate::handlers::util::auth::AuthenticatedUser;
use axum::extract::Query;
use axum::{Json, extract::State};
use axum_valid::Valid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::handlers::task::TaskResponse;

// -----------------------------------------------------------------------------
// RESPONSE SCHEMAS (SHARED)
// -----------------------------------------------------------------------------

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ListResponse {
    pub id: String,
    pub name: String,
    pub journal: Option<String>,
    pub archived: bool,
    pub archived_at: Option<DateTime<Utc>>,
    pub position: f32,
    pub tasks: Vec<TaskResponse>,
}

impl ListResponse {
    pub fn new(list: Domain::List, tasks: Vec<Domain::Task>) -> Self {
        Self {
            id: list.id,
            name: list.name,
            journal: list.journal,
            archived: list.archived,
            archived_at: list.archived_at,
            position: list.position,
            tasks: tasks.into_iter().map(TaskResponse::from).collect(),
        }
    }
}

// -----------------------------------------------------------------------------
// GET ALL LISTS
// -----------------------------------------------------------------------------
pub const PATH_LISTS: &str = "/api/lists";

#[derive(Deserialize, Validate)]
pub struct ListQueryParams {
    pub start_id: Option<String>,
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

    let mut response = Vec::new();
    for list in lists {
        let tasks = manager
            .get_tasks(&auth.username, list.id.clone())
            .await
            .unwrap_or_default();
        response.push(ListResponse::new(list, tasks));
    }

    Ok(Json(response))
}

// -----------------------------------------------------------------------------
// CREATE LIST
// -----------------------------------------------------------------------------

#[derive(Deserialize, Validate, utoipa::ToSchema)]
pub struct CreateListRequest {
    #[validate(length(min = 1, message = "Name cannot be empty"))]
    pub name: String,
}

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
        Json(ListResponse::new(list, vec![])),
    ))
}

// -----------------------------------------------------------------------------
// UPDATE LIST
// -----------------------------------------------------------------------------
pub const PATH_LIST_ID: &str = "/api/lists/{id}";

#[derive(Deserialize, Validate, utoipa::ToSchema)]
pub struct UpdateListRequest {
    #[validate(length(min = 1, message = "Name cannot be empty"))]
    pub name: Option<String>,
    pub journal: Option<String>,
    pub archived: Option<bool>,
    pub position: Option<f32>,
}

#[utoipa::path(
    patch,
    path = PATH_LIST_ID,
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
    axum::extract::Path(id): axum::extract::Path<String>,
    Valid(Json(payload)): Valid<Json<UpdateListRequest>>,
) -> Result<Json<ListResponse>, AppError> {
    let list = manager
        .update_list(
            &auth.username,
            id,
            UpdateListParams {
                name: payload.name,
                journal: payload.journal,
                archived: payload.archived,
                position: payload.position,
            },
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

    let tasks = manager
        .get_tasks(&auth.username, list.id.clone())
        .await
        .unwrap_or_default();

    Ok(Json(ListResponse::new(list, tasks)))
}

// -----------------------------------------------------------------------------
// DELETE LIST
// -----------------------------------------------------------------------------

#[utoipa::path(
    delete,
    path = PATH_LIST_ID,
    tag = "Lists",
    params(
        ("id" = Uuid, Path, description = "List ID")
    ),
    responses(
        (status = 204, description = "List deleted successfully"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "List not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn delete_list(
    State(manager): State<AppState>,
    auth: AuthenticatedUser,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<axum::http::StatusCode, AppError> {
    manager
        .delete_list(&auth.username, id)
        .await
        .map_err(|e| match e {
            crate::manager::app_manager::ManagerError::ListNotFound => {
                AppError::NotFound("List not found".to_string())
            }
            _ => AppError::InternalServerError("Failed to delete list".to_string()),
        })?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}

// -----------------------------------------------------------------------------
// DUPLICATE LIST
// -----------------------------------------------------------------------------
pub const PATH_LIST_DUPLICATE: &str = "/api/lists/{id}/duplicate";

#[derive(Deserialize, Validate, utoipa::ToSchema)]
pub struct DuplicateListRequest {
    #[validate(length(min = 1, message = "Name cannot be empty"))]
    pub name: String,
}

#[utoipa::path(
    post,
    path = PATH_LIST_DUPLICATE,
    tag = "Lists",
    params(
        ("id" = Uuid, Path, description = "List ID")
    ),
    request_body = DuplicateListRequest,
    responses(
        (status = 201, description = "List duplicated successfully", body = ListResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "List not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn duplicate_list(
    State(manager): State<AppState>,
    auth: AuthenticatedUser,
    axum::extract::Path(id): axum::extract::Path<String>,
    Valid(Json(payload)): Valid<Json<DuplicateListRequest>>,
) -> Result<(axum::http::StatusCode, Json<ListResponse>), AppError> {
    let list = manager
        .duplicate_list(&auth.username, id, &payload.name)
        .await
        .map_err(|e| match e {
            crate::manager::app_manager::ManagerError::ListNotFound => {
                AppError::NotFound("List not found".to_string())
            }
            _ => AppError::InternalServerError("Failed to duplicate list".to_string()),
        })?;

    let tasks = manager
        .get_tasks(&auth.username, list.id.clone())
        .await
        .unwrap_or_default();

    Ok((
        axum::http::StatusCode::CREATED,
        Json(ListResponse::new(list, tasks)),
    ))
}

// -----------------------------------------------------------------------------
// REORDER LISTS
// -----------------------------------------------------------------------------
pub const PATH_LISTS_REORDER: &str = "/api/lists/reorder";

#[derive(Deserialize, Validate, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ListReorderRequest {
    pub active_id: String,
    pub over_id: String,
}

#[utoipa::path(
    post,
    path = PATH_LISTS_REORDER,
    tag = "Lists",
    request_body = ListReorderRequest,
    responses(
        (status = 200, description = "Lists reordered successfully", body = ListResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "List not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn reorder_lists(
    State(manager): State<AppState>,
    auth: AuthenticatedUser,
    Valid(Json(payload)): Valid<Json<ListReorderRequest>>,
) -> Result<Json<ListResponse>, AppError> {
    let list = manager
        .reorder_lists(&auth.username, payload.active_id, payload.over_id)
        .await
        .map_err(|e| match e {
            crate::manager::app_manager::ManagerError::ListNotFound => {
                AppError::NotFound("List not found".to_string())
            }
            _ => AppError::InternalServerError("Failed to reorder lists".to_string()),
        })?;

    let tasks = manager
        .get_tasks(&auth.username, list.id.clone())
        .await
        .unwrap_or_default();

    Ok(Json(ListResponse::new(list, tasks)))
}

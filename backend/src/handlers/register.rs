use axum::{Json, extract::State};
use axum_valid::Valid;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::AppState;
use crate::error::{AppError, ErrorResponse};
use crate::manager::app_manager::ManagerError;

#[derive(Deserialize, Validate, utoipa::ToSchema)]
pub struct RegisterRequest {
    #[validate(length(min = 3, message = "Username too short"))]
    pub username: String,
    #[validate(length(min = 8, message = "Password too short"))]
    pub password: String,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct RegisterResponse {
    pub username: String,
    pub access_token: String,
}

pub const PATH: &str = "/api/user/register";

#[utoipa::path(
    post,
    path = PATH,
    tag = "User",
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "Registration successful", body = RegisterResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 409, description = "User already exists", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    )
)]
pub async fn register(
    State(state): State<AppState>,
    Valid(Json(payload)): Valid<Json<RegisterRequest>>,
) -> Result<Json<RegisterResponse>, AppError> {
    let access_token = state
        .app_manager
        .register(&payload.username, &payload.password)
        .await
        .map_err(|e| match e {
            ManagerError::UserAlreadyExists => AppError::Conflict("User already exists".to_string()),
            _ => AppError::InternalServerError("Failed to register user".to_string()),
        })?;

    let response = RegisterResponse {
        username: payload.username,
        access_token,
    };
    Ok(Json(response))
}

use axum::{Json, extract::State};
use axum_valid::Valid;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::AppState;
use crate::error::{AppError, ErrorResponse};
use crate::manager::app_manager::ManagerError;

#[derive(Deserialize, Validate, utoipa::ToSchema)]
pub struct LoginRequest {
    #[validate(length(min = 3, message = "Username too short"))]
    pub username: String,
    #[validate(length(min = 8, message = "Password too short"))]
    pub password: String,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct LoginResponse {
    pub username: String,
    pub access_token: String,
}

pub const PATH: &str = "/api/user/login";

#[utoipa::path(
    post,
    path = PATH ,
    tag = "User",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    )
)]
pub async fn login(
    State(state): State<AppState>,
    Valid(Json(payload)): Valid<Json<LoginRequest>>,
) -> Result<Json<LoginResponse>, AppError> {
    let result = state.login(&payload.username, &payload.password).await;

    match result {
        Ok(access_token) => {
            let result = LoginResponse {
                username: payload.username,
                access_token,
            };
            Ok(Json(result))
        }

        Err(ManagerError::InvalidCredentials) => {
            let msg = "Invalid credentials".to_string();
            Err(AppError::Unauthorized(msg))
        }
        Err(_) => {
            let msg = "Internal server error".to_string();
            Err(AppError::InternalServerError(msg))
        }
    }
}

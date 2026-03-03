use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::error::AppError;
use crate::manager::app_manager::{Manager, ManagerError};

#[derive(Deserialize, utoipa::ToSchema)]
pub struct RegisterRequest {
    pub username: String,
    #[allow(dead_code)]
    pub password: Option<String>,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct RegisterResponse {
    pub username: String,
    pub access_token: String,
    pub refresh_token: String,
}

#[utoipa::path(
    post,
    path = "/api/user/register",
    tag = "Register",
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 400, description = "Bad request", body = crate::error::ErrorResponse),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::error::ErrorResponse),
    )
)]
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    todo!();
}

use axum::{Json, extract::State};
use axum_valid::Valid;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::AppState;
use crate::error::AppError;
use crate::manager::app_manager::ManagerError;

#[derive(Deserialize, Validate, utoipa::ToSchema)]
pub struct LoginRequest {
    #[validate(length(min = 3, message = "Username too short"))]
    pub username: String,
    #[allow(dead_code)]
    pub password: Option<String>,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct LoginResponse {
    pub username: String,
    pub access_token: String,
    pub refresh_token: String,
}

pub const PATH: &str = "/api/user/login";

#[utoipa::path(
    post,
    path = PATH ,
    tag = "User",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse, ),
    )
)]
pub async fn login(
    State(state): State<AppState>,
    Valid(Json(payload)): Valid<Json<LoginRequest>>,
) -> Result<Json<LoginResponse>, AppError> {
    let result = state.app_manager.login(&payload.username).await;

    match result {
        Ok((access_token, refresh_token)) => {
            let result = LoginResponse {
                username: payload.username,
                access_token,
                refresh_token,
            };
            Ok(Json(result))
        }

        Err(ManagerError::InvalidCredentials) => {
            let msg = "Invalid credentials".to_string();
            Err(AppError::Unauthorized(msg))
        }
        Err(ManagerError::DatabaseError) | Err(ManagerError::TokenError) => {
            let msg = "Internal server error".to_string();
            Err(AppError::InternalServerError(msg))
        }
    }
}

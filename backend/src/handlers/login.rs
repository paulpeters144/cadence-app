use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::error::AppError;
use crate::manager::app_manager::{Manager, ManagerError};

#[derive(Deserialize, utoipa::ToSchema)]
pub struct LoginRequest {
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

#[utoipa::path(
    post,
    path = "/api/user/login",
    tag = "Login",
    request_body = LoginRequest,
    responses(
        (
            status = 200, 
            description = "Login successful", 
            body = LoginResponse,
        ),
    )
)]
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    if payload.username.trim().is_empty() {
        let msg = "Username cannot be empty".to_string();
        return Err(AppError::BadRequest(msg));
    }

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

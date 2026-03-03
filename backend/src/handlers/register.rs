use axum::{Json, extract::State};
use axum_valid::Valid;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::AppState;
use crate::error::AppError;

#[derive(Deserialize, Validate, utoipa::ToSchema)]
pub struct RegisterRequest {
    #[validate(length(min = 3, message = "Username too short"))]
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

pub const PATH: &str = "/api/user/register";

#[utoipa::path(
    post,
    path = PATH,
    tag = "User",
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "Registration successful", body = RegisterResponse),
    )
)]
pub async fn register(
    State(_state): State<AppState>,
    Valid(Json(payload)): Valid<Json<RegisterRequest>>,
) -> Result<Json<RegisterResponse>, AppError> {
    let response = RegisterResponse {
        username: payload.username.clone(),
        access_token: "not_implemented".to_string(),
        refresh_token: "not_implemented".to_string(),
    };
    Ok(Json(response))
}

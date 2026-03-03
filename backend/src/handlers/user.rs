use crate::AppState;
use crate::domain::user::User;
use crate::error::{AppError, ErrorResponse};
use crate::handlers::util::auth::AuthenticatedUser;
use crate::manager::app_manager::ManagerError;
use axum::{Json, extract::State};
use axum_valid::Valid;
use serde::{Deserialize, Serialize};
use validator::Validate;

// -----------------------------------------------------------------------------
// GET ME
// -----------------------------------------------------------------------------
pub const PATH_ME: &str = "/api/user/me";

#[utoipa::path(
    get,
    path = PATH_ME,
    tag = "User",
    responses(
        (status = 200, description = "Current user retrieved successfully", body = User),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn get_me(
    State(manager): State<AppState>,
    auth: AuthenticatedUser,
) -> Result<Json<User>, AppError> {
    let user = manager
        .get_user(&auth.username)
        .await
        .map_err(|e| match e {
            ManagerError::UserNotFound => AppError::Unauthorized("User not found".to_string()),
            _ => AppError::InternalServerError("Failed to fetch user".to_string()),
        })?;
    Ok(Json(user))
}

// -----------------------------------------------------------------------------
// LOGIN
// -----------------------------------------------------------------------------
pub const PATH_LOGIN: &str = "/api/user/login";

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

#[utoipa::path(
    post,
    path = PATH_LOGIN,
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
    let access_token = state
        .login(&payload.username, &payload.password)
        .await
        .map_err(|e| match e {
            ManagerError::InvalidCredentials => {
                AppError::Unauthorized("Invalid credentials".to_string())
            }
            _ => AppError::InternalServerError("Internal server error".to_string()),
        })?;

    Ok(Json(LoginResponse {
        username: payload.username,
        access_token,
    }))
}

// -----------------------------------------------------------------------------
// REGISTER
// -----------------------------------------------------------------------------
pub const PATH_REGISTER: &str = "/api/user/register";

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

#[utoipa::path(
    post,
    path = PATH_REGISTER,
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
        .register(&payload.username, &payload.password)
        .await
        .map_err(|e| match e {
            ManagerError::UserAlreadyExists => {
                AppError::Conflict("User already exists".to_string())
            }
            _ => AppError::InternalServerError("Failed to register user".to_string()),
        })?;

    Ok(Json(RegisterResponse {
        username: payload.username,
        access_token,
    }))
}

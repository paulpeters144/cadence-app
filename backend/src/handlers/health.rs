use axum::{http::StatusCode, Json};
use serde::Serialize;
use utoipa::ToSchema;

pub const PATH_HEALTH: &str = "/api/health";

#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    status: &'static str,
    build_date: &'static str,
}

#[utoipa::path(
    get,
    path = PATH_HEALTH,
    tag = "Health",
    responses(
        (status = 200, description = "Health check passed", body = HealthResponse),
    )
)]
pub async fn health() -> (StatusCode, Json<HealthResponse>) {
    let build_date = option_env!("BUILD_DATE").unwrap_or("unknown");

    let response = HealthResponse {
        status: "ok",
        build_date,
    };

    (StatusCode::OK, Json(response))
}

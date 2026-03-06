use axum::http::StatusCode;

pub const PATH_HEALTH: &str = "/api/health";

#[utoipa::path(
    get,
    path = PATH_HEALTH,
    tag = "Health",
    responses(
        (status = 200, description = "Health check passed"),
    )
)]
pub async fn health() -> StatusCode {
    StatusCode::OK
}

pub mod access;
pub mod domain;
pub mod error;
pub mod handlers;
pub mod manager;

use axum::{Router, routing::post};
use handlers::{login, register};
use manager::app_manager::Manager;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(Clone)]
pub struct AppState {
    pub app_manager: Arc<dyn Manager>,
}

#[derive(OpenApi)]
#[openapi(
    paths(login::login, register::register),
    components(schemas(login::LoginRequest, login::LoginResponse, error::ErrorResponse,))
)]
struct ApiDoc;

pub fn app(state: AppState) -> Router {
    Router::new()
        .route(login::PATH, post(login::login))
        .route(register::PATH, post(register::register))
        .merge(SwaggerUi::new("/swagger").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(state)
}

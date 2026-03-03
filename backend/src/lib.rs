pub mod access;
pub mod domain;
pub mod error;
pub mod handlers;
pub mod manager;

use axum::{Router, routing::post};
use manager::app_manager::AppManager;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub app_manager: Arc<AppManager>,
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/api/login", post(handlers::login::login))
        .with_state(state)
}

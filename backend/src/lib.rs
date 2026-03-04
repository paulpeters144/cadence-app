pub mod access;
pub mod constants;
pub mod domain;
pub use domain as Domain;
pub mod error;
pub mod handlers;
pub mod manager;

use axum::{
    Router,
    routing::{get, post},
};
use handlers::{list, user};
use manager::app_manager::Manager;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub type AppState = Arc<dyn Manager>;

#[derive(OpenApi)]
#[openapi(
    paths(
        user::login,
        user::register,
        user::get_me,
        list::create_list,
        list::get_lists
    ),
    components(schemas(
        user::LoginRequest,
        user::LoginResponse,
        user::RegisterRequest,
        user::RegisterResponse,
        user::UserResponse,
        list::CreateListRequest,
        list::ListResponse,
        list::TaskResponse,
        error::ErrorResponse,
    ))
)]
struct ApiDoc;

pub fn app(state: AppState) -> Router {
    Router::new()
        .route(user::PATH_LOGIN, post(user::login))
        .route(user::PATH_REGISTER, post(user::register))
        .route(user::PATH_ME, get(user::get_me))
        .route(
            list::PATH_LISTS,
            post(list::create_list).get(list::get_lists),
        )
        .merge(SwaggerUi::new("/swagger").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(state)
}

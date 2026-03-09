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
use handlers::{health, list, task, user};
use manager::AppManager;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub type AppState = Arc<AppManager>;

#[derive(OpenApi)]
#[openapi(
    paths(
        user::login,
        user::register,
        user::get_me,
        list::create_list,
        list::get_lists,
        list::update_list,
        list::delete_list,
        list::duplicate_list,
        list::reorder_lists,
        task::create_task,
        task::get_tasks,
        task::update_task,
        task::delete_task,
        task::move_task,
        task::reorder_tasks,
        health::health
    ),
    components(schemas(
        user::LoginRequest,
        user::LoginResponse,
        user::RegisterRequest,
        user::RegisterResponse,
        user::UserResponse,
        list::CreateListRequest,
        list::DuplicateListRequest,
        list::ListReorderRequest,
        list::UpdateListRequest,
        list::ListResponse,
        task::CreateTaskRequest,
        task::TaskReorderRequest,
        task::UpdateTaskRequest,
        task::MoveTaskRequest,
        task::TaskResponse,
        error::ErrorResponse,
        health::HealthResponse,
    ))
)]
pub struct ApiDoc;

use tower_http::cors::{Any, CorsLayer};

fn user_routes() -> Router<AppState> {
    Router::new()
        .route(user::PATH_LOGIN, post(user::login))
        .route(user::PATH_REGISTER, post(user::register))
        .route(user::PATH_ME, get(user::get_me))
}

fn health_routes() -> Router<AppState> {
    Router::new().route(health::PATH_HEALTH, get(health::health))
}

fn list_routes() -> Router<AppState> {
    Router::new()
        .route(
            list::PATH_LISTS,
            post(list::create_list).get(list::get_lists),
        )
        .route(
            list::PATH_LIST_ID,
            axum::routing::patch(list::update_list).delete(list::delete_list),
        )
        .route(list::PATH_LIST_DUPLICATE, post(list::duplicate_list))
        .route(list::PATH_LISTS_REORDER, post(list::reorder_lists))
}

fn task_routes() -> Router<AppState> {
    Router::new()
        .route(
            task::PATH_TASKS,
            post(task::create_task).get(task::get_tasks),
        )
        .route(
            task::PATH_TASK_ID,
            axum::routing::patch(task::update_task).delete(task::delete_task),
        )
        .route(task::PATH_TASKS_REORDER, post(task::reorder_tasks))
        .route(task::PATH_TASK_MOVE, post(task::move_task))
}

fn swagger_routes() -> SwaggerUi {
    let openapi_doc = ApiDoc::openapi();
    SwaggerUi::new("/swagger").url("/api-docs/openapi.json", openapi_doc)
}

fn create_cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .merge(user_routes())
        .merge(health_routes())
        .merge(list_routes())
        .merge(task_routes())
        .merge(swagger_routes())
        .with_state(state)
        .layer(create_cors_layer())
}

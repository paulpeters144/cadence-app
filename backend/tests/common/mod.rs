use axum::Router;
use backend::{
    AppState, access::local_repo::DbUserRepository, app, manager::app_manager::AppManager,
};
use std::sync::Arc;

pub async fn setup_app() -> Router {
    let db_url = "sqlite::memory:";
    let repo = DbUserRepository::new(db_url).await;
    repo.init().await.expect("Failed to init DB");

    let repo = Arc::new(repo);
    let secret = "test_secret".to_string();
    let state: AppState = Arc::new(AppManager::new(repo, secret));
    app(state)
}

use axum::Router;
use backend::access::local_repo::DbUserRepository;
use backend::manager::app_manager::AppManager;
use backend::{AppState, app};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let router: Router;
    {
        let db_url =
            std::env::var("DATABASE_URL").expect("DATABASE_URL not set in the environment");
        let jwt_secret =
            std::env::var("JWT_SECRET").expect("JWT_SECRET not set in the environment");

        let repo = DbUserRepository::new(&db_url).await;
        repo.init().await.expect("Failed to initialize database");

        let user_repo = Arc::new(repo);
        let app_manager = AppManager::new(user_repo, jwt_secret);
        let state: AppState = Arc::new(app_manager);

        router = app(state);
    }

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, router).await.unwrap();
}

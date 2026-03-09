use backend::access::AppRepository;
use backend::manager::AppManager;
use backend::{AppState, app};
use std::sync::Arc;

async fn setup_state() -> AppState {
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET not set in the environment");

    let repo = AppRepository::new().await;
    repo.init().await.expect("Failed to initialize database");

    let user_repo = Arc::new(repo);
    let app_manager = AppManager::new(user_repo, jwt_secret);
    Arc::new(app_manager)
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let state = setup_state().await;
    let router = app(state);

    #[cfg(not(feature = "lambda"))]
    {
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
        println!("listening on {}", listener.local_addr().unwrap());
        axum::serve(listener, router).await.unwrap();
    }

    #[cfg(feature = "lambda")]
    {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .with_target(false)
            .without_time()
            .init();
        lambda_http::run(router).await.unwrap();
    }
}

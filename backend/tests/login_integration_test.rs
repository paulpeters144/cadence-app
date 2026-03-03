use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use backend::{
    AppState, access::local_repo::DbUserRepository, app, manager::app_manager::AppManager,
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use std::sync::Arc;
use tower::ServiceExt;

mod login_test_suite {
    use super::*;

    async fn setup_app() -> Router {
        let db_url = "sqlite::memory:";
        let repo = DbUserRepository::new(db_url).await;
        repo.init().await.expect("Failed to init DB");

        let repo = Arc::new(repo);
        let secret = "test_secret".to_string();
        let app_manager = Arc::new(AppManager::new(repo, secret));

        let state = AppState { app_manager };
        app(state)
    }

    #[tokio::test]
    async fn test_login_empty_username_returns_bad_request() {
        let app = setup_app().await;

        let req = Request::builder()
            .method("POST")
            .uri("/api/login")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&json!({
                    "username": "",
                    "password": "password123"
                }))
                .unwrap(),
            ))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_login_invalid_credentials_returns_unauthorized() {
        let app = setup_app().await;

        let req = Request::builder()
            .method("POST")
            .uri("/api/login")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&json!({
                    "username": "unknown_user",
                    "password": "password123"
                }))
                .unwrap(),
            ))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_login_valid_credentials_returns_ok_with_tokens() {
        let app = setup_app().await;

        // demo_user is seeded in init()
        let req = Request::builder()
            .method("POST")
            .uri("/api/login")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&json!({
                    "username": "demo_user",
                    "password": "password123"
                }))
                .unwrap(),
            ))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body_json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(body_json["username"], "demo_user");
        assert!(body_json.get("access_token").is_some());
        assert!(body_json.get("refresh_token").is_some());
    }
}

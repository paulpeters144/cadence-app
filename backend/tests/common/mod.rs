use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use backend::{
    AppState, access::AppRepository, app, manager::app_manager::AppManager,
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use std::sync::Arc;
use tower::ServiceExt;

pub async fn setup_app() -> Router {
    let db_url = "sqlite::memory:";
    let repo = AppRepository::new(db_url).await;
    repo.init().await.expect("Failed to init DB");

    let repo = Arc::new(repo);
    let secret = "test_secret".to_string();
    let state: AppState = Arc::new(AppManager::new(repo, secret));
    app(state)
}

#[allow(dead_code)]
pub async fn register_user(app: &Router, username: &str) -> String {
    let payload = json!({
        "username": username,
        "password": "password123"
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/user/register")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_json: Value = serde_json::from_slice(&body).unwrap();
    body_json["access_token"].as_str().unwrap().to_string()
}

#[allow(dead_code)]
pub async fn create_list(app: &Router, token: &str, name: &str) -> String {
    let payload = json!({ "name": name });
    let req = Request::builder()
        .method("POST")
        .uri("/api/lists")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_json: Value = serde_json::from_slice(&body).unwrap();
    body_json["id"].as_str().unwrap().to_string()
}

#[allow(dead_code)]
pub async fn create_task(app: &Router, token: &str, list_id: &str, title: &str) -> String {
    let payload = json!({ "title": title });
    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/lists/{}/tasks", list_id))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_json: Value = serde_json::from_slice(&body).unwrap();
    body_json["id"].as_str().unwrap().to_string()
}

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

mod common;

async fn register_user(app: &axum::Router, username: &str, password: &str) {
    let payload = json!({
        "username": username,
        "password": password
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/user/register")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_login_success() {
    let app = common::setup_app().await;
    register_user(&app, "test_user", "password123").await;

    let payload = json!({
        "username": "test_user",
        "password": "password123"
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/user/login")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body_json["username"], "test_user");
    assert!(body_json.get("access_token").is_some());
}

#[tokio::test]
async fn test_login_invalid_password() {
    let app = common::setup_app().await;
    register_user(&app, "test_user", "password123").await;

    let payload = json!({
        "username": "test_user",
        "password": "wrongpassword"
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/user/login")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_login_non_existent_user() {
    let app = common::setup_app().await;

    let payload = json!({
        "username": "ghost",
        "password": "password123"
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/user/login")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_login_case_insensitive() {
    let app = common::setup_app().await;
    register_user(&app, "TestUser", "password123").await;

    let payload = json!({
        "username": "testuser",
        "password": "password123"
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/user/login")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Login should be case-insensitive"
    );
}

#[tokio::test]
async fn test_login_validation_errors() {
    let app = common::setup_app().await;

    // Username too short
    let payload = json!({
        "username": "ab",
        "password": "password123"
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/user/login")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Password too short
    let payload = json!({
        "username": "valid_user",
        "password": "short"
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/user/login")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

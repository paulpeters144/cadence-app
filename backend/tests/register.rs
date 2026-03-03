use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

mod common;

#[tokio::test]
async fn test_register_success() {
    let app = common::setup_app().await;

    let payload = json!({
        "username": "new_user",
        "password": "password123"
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/user/register")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body_json["username"], "new_user");
    assert!(body_json.get("access_token").is_some());
}

#[tokio::test]
async fn test_register_duplicate_username() {
    let app = common::setup_app().await;

    let payload = json!({
        "username": "duplicate_user",
        "password": "password123"
    });

    // First registration
    let req = Request::builder()
        .method("POST")
        .uri("/api/user/register")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();
    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Second registration (exact match)
    let req = Request::builder()
        .method("POST")
        .uri("/api/user/register")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();
    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);

    // Third registration (case insensitive check)
    let payload_lower = json!({
        "username": "DUPLICATE_USER",
        "password": "password123"
    });
    let req = Request::builder()
        .method("POST")
        .uri("/api/user/register")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&payload_lower).unwrap()))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_register_validation_username_too_short() {
    let app = common::setup_app().await;

    let payload = json!({
        "username": "ab",
        "password": "password123"
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/user/register")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_register_validation_password_too_short() {
    let app = common::setup_app().await;

    let payload = json!({
        "username": "valid_user",
        "password": "short"
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/user/register")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_register_invalid_json() {
    let app = common::setup_app().await;

    let req = Request::builder()
        .method("POST")
        .uri("/api/user/register")
        .header("content-type", "application/json")
        .body(Body::from("invalid json"))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

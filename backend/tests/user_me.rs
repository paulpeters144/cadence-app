use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

mod common;

async fn get_token(app: &axum::Router, username: &str, password: &str) -> String {
    // First register
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

    let _ = app.clone().oneshot(req).await.unwrap();

    // Then login
    let req = Request::builder()
        .method("POST")
        .uri("/api/user/login")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_json: Value = serde_json::from_slice(&body).unwrap();
    body_json["access_token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn test_get_me_success() {
    let app = common::setup_app().await;
    let token = get_token(&app, "test_user", "password123").await;

    let req = Request::builder()
        .method("GET")
        .uri("/api/user/me")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body_json["username"], "test_user");
}

#[tokio::test]
async fn test_get_me_no_token() {
    let app = common::setup_app().await;

    let req = Request::builder()
        .method("GET")
        .uri("/api/user/me")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_me_invalid_token() {
    let app = common::setup_app().await;

    let req = Request::builder()
        .method("GET")
        .uri("/api/user/me")
        .header("Authorization", "Bearer invalidtoken")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_me_wrong_prefix() {
    let app = common::setup_app().await;
    let token = get_token(&app, "test_user", "password123").await;

    let req = Request::builder()
        .method("GET")
        .uri("/api/user/me")
        .header("Authorization", format!("Token {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

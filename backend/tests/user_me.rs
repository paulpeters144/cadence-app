use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{Value};
use tower::ServiceExt;

mod common;

#[tokio::test]
async fn test_get_me_success() {
    let app = common::setup_app().await;
    let token = common::register_user(&app, "test_user").await;

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
    let token = common::register_user(&app, "test_user").await;

    let req = Request::builder()
        .method("GET")
        .uri("/api/user/me")
        .header("Authorization", format!("Token {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

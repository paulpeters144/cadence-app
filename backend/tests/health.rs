mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::setup_app;
use tower::ServiceExt;

#[tokio::test]
async fn test_health_check() {
    let app = setup_app().await;

    let req = Request::builder()
        .method("GET")
        .uri("/api/health")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

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

    // Also verify we get a build_date and status ok in the response
    let body_bytes = http_body_util::BodyExt::collect(response.into_body()).await.unwrap().to_bytes();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body_json["status"], "ok");
    assert!(body_json["build_date"].is_string());
}

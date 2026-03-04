use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

mod common;

async fn register_user(app: &axum::Router, username: &str) -> String {
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

#[tokio::test]
async fn test_create_list_success() {
    let app = common::setup_app().await;
    let token = register_user(&app, "test_user").await;

    let payload = json!({
        "name": "My New List"
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/lists")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body_json["name"], "My New List");
    assert!(body_json.get("id").is_some());
    assert_eq!(body_json["archived"], false);
}

#[tokio::test]
async fn test_create_list_unauthorized() {
    let app = common::setup_app().await;

    let payload = json!({
        "name": "My New List"
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/lists")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_create_list_validation_empty_name() {
    let app = common::setup_app().await;
    let token = register_user(&app, "test_user").await;

    let payload = json!({
        "name": ""
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/lists")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_all_lists_success() {
    let app = common::setup_app().await;
    let token = register_user(&app, "test_user").await;

    // Create two lists
    for name in ["List 1", "List 2"] {
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
    }

    // Get all lists
    let req = Request::builder()
        .method("GET")
        .uri("/api/lists")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_json: Value = serde_json::from_slice(&body).unwrap();
    assert!(body_json.is_array());
    let lists = body_json.as_array().unwrap();
    assert_eq!(lists.len(), 2);
    assert!(lists.iter().any(|l| l["name"] == "List 1"));
    assert!(lists.iter().any(|l| l["name"] == "List 2"));
}

#[tokio::test]
async fn test_get_lists_pagination_take() {
    let app = common::setup_app().await;
    let token = register_user(&app, "test_user_take").await;

    for i in 1..=5 {
        let payload = json!({ "name": format!("List {}", i) });
        let req = Request::builder()
            .method("POST")
            .uri("/api/lists")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();
        app.clone().oneshot(req).await.unwrap();
    }

    let req = Request::builder()
        .method("GET")
        .uri("/api/lists?take=3")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_json: Value = serde_json::from_slice(&body).unwrap();
    let lists = body_json.as_array().unwrap();
    assert_eq!(lists.len(), 3);
}

#[tokio::test]
async fn test_get_lists_pagination_start_id() {
    let app = common::setup_app().await;
    let token = register_user(&app, "test_user_start_id").await;

    let payload = json!({ "name": "Target List" });
    let req = Request::builder()
        .method("POST")
        .uri("/api/lists")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();
    let create_res = app.clone().oneshot(req).await.unwrap();
    let create_body = create_res.into_body().collect().await.unwrap().to_bytes();
    let create_json: Value = serde_json::from_slice(&create_body).unwrap();
    let target_id = create_json["id"].as_str().unwrap();

    let req = Request::builder()
        .method("GET")
        .uri(format!("/api/lists?start_id={}", target_id))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_json: Value = serde_json::from_slice(&body).unwrap();
    let lists = body_json.as_array().unwrap();
    
    // Based on current backend implementation, start_id returns exactly one list (the matched id)
    assert_eq!(lists.len(), 1);
    assert_eq!(lists[0]["id"].as_str().unwrap(), target_id);
    assert_eq!(lists[0]["name"].as_str().unwrap(), "Target List");
}

#[tokio::test]
async fn test_get_lists_take_validation() {
    let app = common::setup_app().await;
    let token = register_user(&app, "test_user_validation").await;

    let req = Request::builder()
        .method("GET")
        .uri("/api/lists?take=501")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

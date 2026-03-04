use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

mod common;

#[tokio::test]
async fn test_create_list_success() {
    let app = common::setup_app().await;
    let token = common::register_user(&app, "test_user").await;

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
    let token = common::register_user(&app, "test_user").await;

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
async fn test_get_lists_success() {
    let app = common::setup_app().await;
    let token = common::register_user(&app, "test_user").await;

    // Create two lists
    for name in ["List 1", "List 2"] {
        common::create_list(&app, &token, name).await;
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
}

#[tokio::test]
async fn test_get_lists_pagination_take() {
    let app = common::setup_app().await;
    let token = common::register_user(&app, "test_user_take").await;

    for i in 1..=5 {
        common::create_list(&app, &token, &format!("List {}", i)).await;
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
    let token = common::register_user(&app, "test_user_start_id").await;

    let target_id = common::create_list(&app, &token, "Target List").await;

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

    assert_eq!(lists.len(), 1);
    assert_eq!(lists[0]["id"].as_str().unwrap(), target_id);
}

#[tokio::test]
async fn test_update_list_success() {
    let app = common::setup_app().await;
    let token = common::register_user(&app, "test_user_update").await;

    let list_id = common::create_list(&app, &token, "Original Name").await;

    // Update the list (partial: name only)
    let update_payload = json!({ "name": "Updated Name" });
    let req = Request::builder()
        .method("PATCH")
        .uri(format!("/api/lists/{}", list_id))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(serde_json::to_vec(&update_payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body_json["name"], "Updated Name");
}

#[tokio::test]
async fn test_update_list_not_found() {
    let app = common::setup_app().await;
    let token = common::register_user(&app, "test_user_nf").await;
    let random_id = uuid::Uuid::new_v4();

    let update_payload = json!({ "name": "New Name" });
    let req = Request::builder()
        .method("PATCH")
        .uri(format!("/api/lists/{}", random_id))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(serde_json::to_vec(&update_payload).unwrap()))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_update_list_validation_empty_name() {
    let app = common::setup_app().await;
    let token = common::register_user(&app, "test_user_val").await;
    let list_id = common::create_list(&app, &token, "Original Name").await;

    let update_payload = json!({ "name": "" });
    let req = Request::builder()
        .method("PATCH")
        .uri(format!("/api/lists/{}", list_id))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(serde_json::to_vec(&update_payload).unwrap()))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_lists_pagination_foreign_id() {
    let app = common::setup_app().await;
    
    // User 1 creates a list
    let token1 = common::register_user(&app, "user1").await;
    let list_id1 = common::create_list(&app, &token1, "User 1 List").await;
    
    // User 2 tries to access User 1's list via start_id
    let token2 = common::register_user(&app, "user2").await;
    let req = Request::builder()
        .method("GET")
        .uri(format!("/api/lists?start_id={}", list_id1))
        .header("authorization", format!("Bearer {}", token2))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_json: Value = serde_json::from_slice(&body).unwrap();
    let lists = body_json.as_array().unwrap();
    assert_eq!(lists.len(), 0, "User 2 should not see User 1's list");
}

#[tokio::test]
async fn test_delete_list_success() {
    let app = common::setup_app().await;
    let token = common::register_user(&app, "test_user_delete").await;

    let list_id = common::create_list(&app, &token, "List to Delete").await;

    // Delete the list
    let req = Request::builder()
        .method("DELETE")
        .uri(format!("/api/lists/{}", list_id))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Verify it's gone
    let req = Request::builder()
        .method("GET")
        .uri("/api/lists")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_json: Value = serde_json::from_slice(&body).unwrap();
    let lists = body_json.as_array().unwrap();
    assert_eq!(lists.len(), 0);
}

#[tokio::test]
async fn test_delete_list_not_found() {
    let app = common::setup_app().await;
    let token = common::register_user(&app, "test_user_del_nf").await;
    let random_id = uuid::Uuid::new_v4();

    let req = Request::builder()
        .method("DELETE")
        .uri(format!("/api/lists/{}", random_id))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_duplicate_list_success() {
    let app = common::setup_app().await;
    let token = common::register_user(&app, "test_user_duplicate").await;

    let list_id = common::create_list(&app, &token, "Original List").await;
    common::create_task(&app, &token, &list_id, "Task 1").await;
    common::create_task(&app, &token, &list_id, "Task 2").await;

    // Duplicate the list
    let duplicate_payload = json!({ "name": "Duplicated List" });
    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/lists/{}/duplicate", list_id))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(serde_json::to_vec(&duplicate_payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_json: Value = serde_json::from_slice(&body).unwrap();
    let new_list_id = body_json["id"].as_str().unwrap();
    assert_eq!(body_json["name"], "Duplicated List");
    assert_ne!(new_list_id, list_id);

    // Verify tasks are duplicated
    let req = Request::builder()
        .method("GET")
        .uri(format!("/api/lists/{}/tasks", new_list_id))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_json: Value = serde_json::from_slice(&body).unwrap();
    let tasks = body_json.as_array().unwrap();
    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0]["title"], "Task 1");
    assert_eq!(tasks[1]["title"], "Task 2");
}

#[tokio::test]
async fn test_duplicate_list_unauthorized_owner() {
    let app = common::setup_app().await;
    
    // User 1 creates a list
    let token1 = common::register_user(&app, "user1_dup").await;
    let list_id1 = common::create_list(&app, &token1, "User 1 List").await;
    
    // User 2 tries to duplicate User 1's list
    let token2 = common::register_user(&app, "user2_dup").await;
    let duplicate_payload = json!({ "name": "Duplicated List" });
    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/lists/{}/duplicate", list_id1))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token2))
        .body(Body::from(serde_json::to_vec(&duplicate_payload).unwrap()))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_reorder_lists_success() {
    let app = common::setup_app().await;
    let token = common::register_user(&app, "test_user_reorder").await;

    let list_id1 = common::create_list(&app, &token, "List 1").await;
    let list_id2 = common::create_list(&app, &token, "List 2").await;
    let list_id3 = common::create_list(&app, &token, "List 3").await;

    // Initial order: 1, 2, 3
    // Move 1 to after 2: new order 2, 1, 3
    let reorder_payload = json!({
        "activeId": list_id1,
        "overId": list_id2
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/lists/reorder")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(serde_json::to_vec(&reorder_payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Verify final order
    let req = Request::builder()
        .method("GET")
        .uri("/api/lists")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_json: Value = serde_json::from_slice(&body).unwrap();
    let lists = body_json.as_array().unwrap();
    
    assert_eq!(lists[0]["id"].as_str().unwrap(), list_id2);
    assert_eq!(lists[1]["id"].as_str().unwrap(), list_id1);
    assert_eq!(lists[2]["id"].as_str().unwrap(), list_id3);
}

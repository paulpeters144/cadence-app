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

async fn create_list(app: &axum::Router, token: &str, name: &str) -> String {
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

#[tokio::test]
async fn test_task_lifecycle() {
    let app = common::setup_app().await;
    let token = register_user(&app, "task_user").await;
    let list_id = create_list(&app, &token, "My Tasks").await;

    // 1. Create Task
    let payload = json!({
        "title": "Test Task",
        "points": 5.0
    });
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
    let task_json: Value = serde_json::from_slice(&body).unwrap();
    let task_id = task_json["id"].as_str().unwrap();
    assert_eq!(task_json["title"], "Test Task");
    assert_eq!(task_json["points"], 5.0);
    assert_eq!(task_json["completed"], false);

    // 2. Get Tasks
    let req = Request::builder()
        .method("GET")
        .uri(format!("/api/lists/{}/tasks", list_id))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let tasks: Value = serde_json::from_slice(&body).unwrap();
    assert!(tasks.is_array());
    assert_eq!(tasks.as_array().unwrap().len(), 1);
    assert_eq!(tasks[0]["id"], task_id);

    // 3. Update Task (Complete)
    let payload = json!({
        "completed": true,
        "title": "Updated Task Name"
    });
    let req = Request::builder()
        .method("PATCH")
        .uri(format!("/api/lists/{}/tasks/{}", list_id, task_id))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let updated_task: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated_task["completed"], true);
    assert_eq!(updated_task["title"], "Updated Task Name");
    assert!(updated_task["completedAt"].is_string());

    // 4. Delete Task
    let req = Request::builder()
        .method("DELETE")
        .uri(format!("/api/lists/{}/tasks/{}", list_id, task_id))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // 5. Verify Deletion
    let req = Request::builder()
        .method("GET")
        .uri(format!("/api/lists/{}/tasks", list_id))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let tasks: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(tasks.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_task_security() {
    let app = common::setup_app().await;
    
    let token1 = register_user(&app, "user1").await;
    let list_id1 = create_list(&app, &token1, "User 1 List").await;
    
    let token2 = register_user(&app, "user2").await;
    
    // User 2 tries to create a task in User 1's list
    let payload = json!({ "title": "Sneaky Task" });
    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/lists/{}/tasks", list_id1))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token2))
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND); // Should be NOT_FOUND because the list doesn't "exist" for user2

    // Create a task for user 1
    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/lists/{}/tasks", list_id1))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token1))
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();
    let response = app.clone().oneshot(req).await.unwrap();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let task_json: Value = serde_json::from_slice(&body).unwrap();
    let task_id1 = task_json["id"].as_str().unwrap();

    // User 2 tries to update User 1's task
    let payload = json!({ "completed": true });
    let req = Request::builder()
        .method("PATCH")
        .uri(format!("/api/lists/{}/tasks/{}", list_id1, task_id1))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token2))
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // User 2 tries to delete User 1's task
    let req = Request::builder()
        .method("DELETE")
        .uri(format!("/api/lists/{}/tasks/{}", list_id1, task_id1))
        .header("authorization", format!("Bearer {}", token2))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

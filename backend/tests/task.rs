use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

mod common;

#[tokio::test]
async fn test_task_lifecycle() {
    let app = common::setup_app().await;
    let token = common::register_user(&app, "task_user").await;
    let list_id = common::create_list(&app, &token, "My Tasks").await;

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
    assert_eq!(tasks.as_array().unwrap().len(), 1);

    // 3. Update Task
    let payload = json!({ "completed": true });
    let req = Request::builder()
        .method("PATCH")
        .uri(format!("/api/lists/{}/tasks/{}", list_id, task_id))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 4. Delete Task
    let req = Request::builder()
        .method("DELETE")
        .uri(format!("/api/lists/{}/tasks/{}", list_id, task_id))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_task_security_isolation() {
    let app = common::setup_app().await;
    
    let token1 = common::register_user(&app, "user1").await;
    let list_id1 = common::create_list(&app, &token1, "User 1 List").await;
    
    let token2 = common::register_user(&app, "user2").await;
    
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
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // User 2 tries to GET User 1's tasks
    let req = Request::builder()
        .method("GET")
        .uri(format!("/api/lists/{}/tasks", list_id1))
        .header("authorization", format!("Bearer {}", token2))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let tasks: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(tasks.as_array().unwrap().len(), 0, "User 2 should see 0 tasks in foreign list");
}

#[tokio::test]
async fn test_task_unauthorized() {
    let app = common::setup_app().await;
    let list_id = uuid::Uuid::new_v4();

    let endpoints = [
        ("POST", format!("/api/lists/{}/tasks", list_id), Some(json!({"title": "T"}))),
        ("GET", format!("/api/lists/{}/tasks", list_id), None),
        ("PATCH", format!("/api/lists/{}/tasks/{}", list_id, uuid::Uuid::new_v4()), Some(json!({"completed": true}))),
        ("DELETE", format!("/api/lists/{}/tasks/{}", list_id, uuid::Uuid::new_v4()), None),
    ];

    for (method, uri, body) in endpoints {
        let mut builder = Request::builder().method(method).uri(uri);
        if body.is_some() {
            builder = builder.header("content-type", "application/json");
        }
        let req = builder.body(Body::from(body.map(|b| serde_json::to_vec(&b).unwrap()).unwrap_or_default())).unwrap();
        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "Endpoint {} should be unauthorized", method);
    }
}

#[tokio::test]
async fn test_create_task_validation_empty_title() {
    let app = common::setup_app().await;
    let token = common::register_user(&app, "test_user").await;
    let list_id = common::create_list(&app, &token, "List").await;

    let payload = json!({ "title": "" });
    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/lists/{}/tasks", list_id))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_task_not_found() {
    let app = common::setup_app().await;
    let token = common::register_user(&app, "test_user").await;
    let list_id = common::create_list(&app, &token, "List").await;
    let fake_id = uuid::Uuid::new_v4();

    // Update non-existent task
    let payload = json!({ "completed": true });
    let req = Request::builder()
        .method("PATCH")
        .uri(format!("/api/lists/{}/tasks/{}", list_id, fake_id))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();
    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // Delete non-existent task
    let req = Request::builder()
        .method("DELETE")
        .uri(format!("/api/lists/{}/tasks/{}", list_id, fake_id))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_reorder_tasks_success() {
    let app = common::setup_app().await;
    let token = common::register_user(&app, "test_user_reorder_tasks").await;
    let list_id = common::create_list(&app, &token, "Task List").await;

    let task_id1 = common::create_task(&app, &token, &list_id, "Task 1").await;
    let task_id2 = common::create_task(&app, &token, &list_id, "Task 2").await;
    let task_id3 = common::create_task(&app, &token, &list_id, "Task 3").await;

    // Initial order: 1, 2, 3
    // Move 1 to after 2: new order 2, 1, 3
    let reorder_payload = json!({
        "activeId": task_id1,
        "overId": task_id2
    });

    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/lists/{}/tasks/reorder", list_id))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(serde_json::to_vec(&reorder_payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Verify final order
    let req = Request::builder()
        .method("GET")
        .uri(format!("/api/lists/{}/tasks", list_id))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_json: Value = serde_json::from_slice(&body).unwrap();
    let tasks = body_json.as_array().unwrap();
    
    assert_eq!(tasks[0]["id"].as_str().unwrap(), task_id2);
    assert_eq!(tasks[1]["id"].as_str().unwrap(), task_id1);
    assert_eq!(tasks[2]["id"].as_str().unwrap(), task_id3);
}

#[tokio::test]
async fn test_move_task() {
    let app = common::setup_app().await;
    let token = common::register_user(&app, "move_user").await;
    let list_id1 = common::create_list(&app, &token, "List 1").await;
    let list_id2 = common::create_list(&app, &token, "List 2").await;

    // 1. Create Task in List 1
    let payload = json!({ "title": "Move Me" });
    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/lists/{}/tasks", list_id1))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let task_json: Value = serde_json::from_slice(&body).unwrap();
    let task_id = task_json["id"].as_str().unwrap();

    // 2. Move Task to List 2
    let move_payload = json!({
        "fromListId": list_id1,
        "toListId": list_id2,
        "position": 500.0
    });
    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/tasks/{}/move", task_id))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(serde_json::to_vec(&move_payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let moved_task: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(moved_task["position"], 500.0);

    // 3. Verify in List 2
    let req = Request::builder()
        .method("GET")
        .uri(format!("/api/lists/{}/tasks", list_id2))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let tasks: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(tasks.as_array().unwrap().len(), 1);
    assert_eq!(tasks[0]["id"], task_id);

    // 4. Verify NOT in List 1
    let req = Request::builder()
        .method("GET")
        .uri(format!("/api/lists/{}/tasks", list_id1))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let tasks: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(tasks.as_array().unwrap().len(), 0);
}

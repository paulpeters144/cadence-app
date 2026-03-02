use axum::{Json, Router, routing::get};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct JsonResponse {
    message: String,
}

fn app() -> Router {
    Router::new().route("/", get(get_hello))
}

async fn get_hello() -> Json<JsonResponse> {
    let result = JsonResponse {
        message: "Hello, Rust Backend!".to_string(),
    };
    Json(result)
}

#[tokio::main]
async fn main() {
    let app = app();

    // run it with hyper on localhost:3001
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt; // for `oneshot` and `ready` // for `collect`

    #[tokio::test]
    async fn hello_world_json() {
        let app = app();

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "application/json"
        );

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body: JsonResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            body,
            JsonResponse {
                message: "Hello, Rust Backend!".to_string()
            }
        );
    }
}

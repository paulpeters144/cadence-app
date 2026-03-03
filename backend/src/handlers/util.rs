use axum::{async_trait, extract::FromRequest, http::Request};

pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<S, B, T> FromRequest<S, B> for ValidatedJson<T>
where
    B: Send + 'static,
    S: Send + Sync,
    T: DeserializeOwned + Validate + 'static,
{
    type Rejection = AppError; // Map your validation errors to your custom AppError

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|rejection| {
                // Convert Axum Json rejection to your AppError
                AppError::from(rejection)
            })?;

        value.validate().map_err(|e| AppError::ValidationError(e))?;
        Ok(ValidatedJson(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use serde::Deserialize;
    use tower::ServiceExt;
    use validator::Validate;

    #[derive(Deserialize, Validate, Debug)]
    struct TestData {
        #[validate(length(min = 1))]
        name: String,
    }

    #[tokio::test]
    async fn test_validated_json_success() {
        let json_str = r#"{"name": "test"}"#;
        let request = Request::builder()
            .method(http::Method::POST)
            .header("Content-Type", "application/json")
            .body(Body::from(json_str))
            .unwrap();

        let state = (); // Dummy state

        let result = ValidatedJson::<TestData>::from_request(request, &state).await;
        assert!(result.is_ok());
        let validated_json = result.unwrap();
        assert_eq!(validated_json.0.name, "test");
    }

    #[tokio::test]
    async fn test_validated_json_failure() {
        let json_str = r#"{"name": ""}"#; // Empty name should fail validation
        let request = Request::builder()
            .method(http::Method::POST)
            .header("Content-Type", "application/json")
            .body(Body::from(json_str))
            .unwrap();

        let state = (); // Dummy state

        let result = ValidatedJson::<TestData>::from_request(request, &state).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::ValidationError(_) => {
                // Expected
            }
            _ => panic!("Expected ValidationError"),
        }
    }
}

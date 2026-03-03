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

use ax_extract::FromRequest;
use axum::{async_trait, extract::FromRequest, Json};
use validator::Validate;
use crate::AppError;

pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    S: Send + Sync,
    T: Validate + serde::de::DeserializeOwned,
{
    type Rejection = AppError;

    async fn from_request(req: axum::http::Request<axum::body::Body>, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state).await
            .map_err(|_| AppError::BadRequest("Invalid JSON".into()))?;
        
        value.validate()?; // 自动校验
        
        Ok(ValidatedJson(value))
    }
}
// src/validation.rs
use axum::{
    async_trait,
    extract::{FromRequest, Request}, // 修正为 Request
    Json,
};
use validator::Validate;
use crate::AppError;

pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    S: Send + Sync,
    T: Validate + serde::de::DeserializeOwned + 'static, // 增加 'static 约束
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        // 1. 利用 Axum 原生的 Json 提取器解析 Body
        // 如果 JSON 格式非法（比如少括号），这里会直接返回 BadRequest
        let Json(value) = Json::<T>::from_request(req, state).await
            .map_err(|rejection| AppError::BadRequest(rejection.body_text()))?;
        
        // 2. 执行 validator 的校验逻辑
        // 如果校验失败（比如标题太短），会通过 AppError::ValidationError 自动转换
        value.validate()?; 
        
        Ok(ValidatedJson(value))
    }
}
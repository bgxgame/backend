// src/error.rs
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Authentication failed: {0}")]
    Auth(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unauthorized: {0}")]
    Forbidden(String),

    #[error("Internal server error")]
    Internal,

    #[error("Validation error: {0}")]
    BadRequest(String),

    #[error("Validation error: {0}")]
    ValidationError(#[from] validator::ValidationErrors), // 新增：自动转换校验错误
}

// 核心逻辑：将我们的错误转换为 HTTP 响应
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Database(ref e) => {
                // 后台记录详细错误
                tracing::error!("Database Error: {:?}", e);

                // 对外根据具体情况返回信息
                if e.to_string().contains("duplicate key") {
                    (StatusCode::CONFLICT, "Record already exists".to_string())
                } else {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Database operation failed".to_string(),
                    )
                }
            },
            AppError::Auth(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
            AppError::ValidationError(ref e) => {
                // 将复杂的校验错误对象转为简单易读的字符串或 JSON
                (StatusCode::BAD_REQUEST, format!("输入参数有误: {}", e))
            },
        };

        let body = Json(json!({
            "status": "error",
            "message": error_message,
        }));

        (status, body).into_response()
    }
}

// 方便将 String 错误转换为 AppError::Auth
impl From<String> for AppError {
    fn from(err: String) -> Self {
        AppError::Auth(err)
    }
}

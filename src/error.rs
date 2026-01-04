// src/error.rs
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, Value};
use thiserror::Error;
use validator::ValidationErrors;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error")]
    Database(#[from] sqlx::Error),

    #[error("Authentication failed: {0}")]
    Auth(String),

    #[error("Not found")]
    NotFound(String),

    #[error("Internal server error")]
    Internal,

    #[error("Validation error")]
    ValidationError(#[from] ValidationErrors),

    #[error("Bad request: {0}")]
    BadRequest(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message, details) = match self {
            AppError::Database(ref e) => {
                tracing::error!("DB Error: {:?}", e);
                if e.to_string().contains("duplicate key") {
                    (StatusCode::CONFLICT, "记录已存在".to_string(), None)
                } else {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "数据库操作失败".to_string(),
                        None,
                    )
                }
            }
            AppError::Auth(msg) => (StatusCode::UNAUTHORIZED, msg, None),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg, None),
            AppError::ValidationError(e) => {
                // 将 validator 的错误提取为 field -> message 的 Map
                let mut errors = std::collections::HashMap::new();
                for (field, field_errors) in e.field_errors() {
                    let msgs: Vec<String> = field_errors
                        .iter()
                        .map(|fe| {
                            fe.message
                                .as_ref()
                                .map(|m| m.to_string())
                                .unwrap_or_else(|| "格式不正确".to_string())
                        })
                        .collect();
                    errors.insert(field, msgs);
                }
                (
                    StatusCode::BAD_REQUEST,
                    "输入校验失败".to_string(),
                    Some(json!(errors)),
                )
            }
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg, None),
            AppError::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "服务器内部错误".to_string(),
                None,
            ),
        };

        let body = Json(json!({
            "status": "error",
            "message": message,
            "errors": details // 这里的具体错误字段供前端解析
        }));

        (status, body).into_response()
    }
}

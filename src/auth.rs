// src/auth.rs
use argon2::{
    password_hash::{
        rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2,
};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use chrono::{Utc, Duration};
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json, RequestPartsExt,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use serde_json::json;

// --- 1. 密码处理 ---

// 加密密码
pub fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)
        .map_err(|e| e.to_string())?
        .to_string();
    Ok(password_hash)
}

// 验证密码
pub fn verify_password(password: &str, password_hash: &str) -> bool {
    let parsed_hash = match PasswordHash::new(password_hash) {
        Ok(h) => h,
        Err(_) => return false,
    };
    Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok()
}

// --- 2. JWT 处理 ---

// Token 里的数据结构 (Claims)
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,    // 将 sub 改为用户 ID (i32)
    pub username: String, // 保留用户名用于显示
    pub exp: usize,
}

// 创建 Token
pub fn create_jwt(user_id: i32, username: &str) -> Result<String, String> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24)) // Token 24小时有效
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user_id,
        username: username.to_owned(),
        exp: expiration as usize,
    };

    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| e.to_string())
}

// --- 3. 核心：认证提取器 (Extractor) ---
// 这是一个“守门员”。任何在参数里写了 AuthUser 的接口，都会先经过这里。
// 如果 Token 无效，直接拒绝请求，代码都不会进入 handler。

pub struct AuthUser {
    pub id: i32,      // 包含 ID
    pub username: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // 1. 从 Header 提取 Bearer Token
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| (StatusCode::UNAUTHORIZED, Json(json!({"error": "Missing or invalid token"}))))?;

        // 2. 验证 Token
        let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".into());
        let token_data = decode::<Claims>(
            bearer.token(),
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|_| (StatusCode::UNAUTHORIZED, Json(json!({"error": "Invalid token"}))))?;

        // 3. 验证通过，返回 AuthUser
        Ok(AuthUser {
            id: token_data.claims.sub,
            username: token_data.claims.username,
        })
    }
}
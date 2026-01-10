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
    http::request::Parts,
    RequestPartsExt,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use crate::AppError; 
use uuid::Uuid;

// --- 1. 密码处理 (Argon2) ---

pub fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)
        .map_err(|e| e.to_string())?
        .to_string();
    Ok(password_hash)
}

pub fn verify_password(password: &str, password_hash: &str) -> bool {
    let parsed_hash = match PasswordHash::new(password_hash) {
        Ok(h) => h,
        Err(_) => return false,
    };
    Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok()
}

// --- 2. JWT (Access Token) 处理 ---

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,         // 用户 ID
    pub username: String, 
    pub exp: usize,       // 过期时间
}

/// 生成短效的 Access Token (用于 API 请求)
/// 建议有效期：15 分钟
pub fn create_jwt(user_id: i32, username: &str) -> Result<String, String> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::minutes(15)) 
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

// --- 3. Refresh Token 处理 ---

/// 生成唯一的随机字符串作为刷新令牌
pub fn generate_refresh_token() -> String {
    Uuid::new_v4().to_string()
}

// --- 4. 核心：认证提取器 (AuthUser Extractor) ---
// 用于在 Handler 中通过 (user: AuthUser) 自动获取当前登录用户

pub struct AuthUser {
    pub id: i32,
    #[allow(dead_code)] 
    pub username: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // 1. 从 HTTP Header 提取 Bearer Token
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AppError::Auth("Token 缺失或格式错误".into()))?;

        // 2. 验证 Token 有效性
        let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".into());
        let token_data = decode::<Claims>(
            bearer.token(),
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|e| {
            // 如果 Token 过期，jsonwebtoken 会返回特定错误，前端拦截器会捕获并处理
            tracing::warn!("JWT 验证失败: {}", e);
            AppError::Auth("Token 已过期或无效".into())
        })?;

        // 3. 验证通过，构建 AuthUser
        Ok(AuthUser {
            id: token_data.claims.sub,
            username: token_data.claims.username,
        })
    }
}
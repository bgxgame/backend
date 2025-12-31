// src/models.rs
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};

// --- 1. User 模型 (数据库对应) ---
#[derive(Debug, FromRow, Deserialize, Serialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    #[serde(skip)] // 序列化时跳过密码
    pub password_hash: String,
    pub created_at: Option<DateTime<Utc>>,
}

// --- 2. Plan 模型 (数据库对应) ---
#[derive(Debug, FromRow, Serialize)]
pub struct Plan {
    pub id: i32,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub category: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub is_public: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

// --- 3. CRUD 请求结构体 ---
#[derive(Debug, Deserialize)]
pub struct CreatePlanSchema {
    pub title: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    #[serde(default = "default_is_public")]
    pub is_public: bool,
}

fn default_is_public() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct UpdatePlanSchema {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub category: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub is_public: Option<bool>,
}

// --- 4. 认证相关结构体 ---
#[derive(Debug, Deserialize)]
pub struct RegisterSchema {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginSchema {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub username: String,
}
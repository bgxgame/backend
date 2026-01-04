// src/models.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate; // 引入 Validate trait

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
    pub user_id: i32, // 新增字段
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub category: Option<String>,
    pub priority: i32,
    pub due_date: Option<DateTime<Utc>>,
    pub is_public: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

// --- 3. CRUD 请求结构体 ---
#[derive(Debug, Deserialize, Validate)]
pub struct CreatePlanSchema {
    #[validate(length(min = 1, max = 150, message = "标题不能为空且不能超过 150 字"))]
    pub title: String,
    #[validate(length(min = 5, message = "描述内容至少需要 5 个字"))]
    pub description: Option<String>,
    #[validate(length(max = 50, message = "分类名称过长"))]
    pub category: Option<String>,
    pub priority: Option<i32>,
    pub due_date: Option<DateTime<Utc>>,
    #[serde(default = "default_is_public")]
    pub is_public: bool,
}

fn default_is_public() -> bool {
    true
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePlanSchema {
    // 更新时，如果传了标题，则必须满足长度
    #[validate(length(min = 2, max = 150, message = "标题至少需要 2 个字"))]
    pub title: Option<String>,
    // 修复点：增加描述校验，确保更新时如果修改了描述，长度也要够
    #[validate(length(min = 5, message = "描述内容太短了，至少 5 个字"))]
    pub description: Option<String>,
    pub status: Option<String>,
    pub category: Option<String>,
    pub priority: Option<i32>,
    pub due_date: Option<DateTime<Utc>>,
    pub is_public: Option<bool>,
}

// --- 4. 认证相关结构体 ---
#[derive(Debug, Deserialize, Validate)]
pub struct RegisterSchema {
    #[validate(length(min = 3, max = 20, message = "用户名长度需在 3-20 位之间"))]
    pub username: String,
    #[validate(length(min = 6, message = "密码至少需要 6 位"))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginSchema {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct PlanQuery {
    pub q: Option<String>,        // 搜索关键词
    pub status: Option<String>,   // 状态过滤
    pub category: Option<String>, // 分类过滤
}

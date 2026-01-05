// src/models.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

// --- 1. Project 模型 ---
#[derive(Debug, FromRow, Serialize)]
pub struct Project {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub color: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateProjectSchema {
    #[validate(length(min = 1, max = 100, message = "项目名称不能为空"))]
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateProjectSchema {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub color: Option<String>,
}

// --- 2. Issue 模型 (由原 Plan 升级) ---
#[derive(Debug, FromRow, Serialize)]
pub struct Issue {
    pub id: i32,
    pub project_id: i32,
    pub user_id: i32,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: i32,
    pub due_date: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateIssueSchema {
    pub project_id: i32, // 必须指定所属项目
    #[validate(length(min = 1, max = 255, message = "标题不能为空"))]
    pub title: String,
    #[validate(length(min = 5, message = "描述内容至少需要 5 个字"))]
    pub description: Option<String>,
    pub priority: Option<i32>,
    pub due_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateIssueSchema {
    #[validate(length(min = 1, max = 255, message = "标题不能为空"))]
    pub title: Option<String>,
    // #[validate(length(min = 5, message = "描述内容至少需要 5 个字"))]
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<i32>,
    pub due_date: Option<DateTime<Utc>>,
}

// --- 3. 认证与查询模型 ---
#[derive(Debug, FromRow, Deserialize, Serialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    #[serde(skip)]
    pub password_hash: String,
    pub created_at: Option<DateTime<Utc>>,
}

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
pub struct IssueQuery {
    pub q: Option<String>,
    pub status: Option<String>,
}
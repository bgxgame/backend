// src/handlers.rs
use crate::auth::{create_jwt, hash_password, verify_password, AuthUser}; // 引入 AuthUser
use crate::models::{
    AuthResponse,
    CreatePlanSchema,
    LoginSchema,
    Plan,
    RegisterSchema,
    UpdatePlanSchema,
    User, // <--- 确保引入 User
};
use crate::AppError;
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::json; // <--- 修复报错：引入 json! 宏 // 引入自定义错误

// --- 1. 获取列表 (GET /plans) ---
pub async fn get_plans_handler(
    auth: Option<AuthUser>, // 可选认证：登录后能看到自己的私有计划
    State(state): State<AppState>,
) -> Result<Json<Vec<Plan>>, AppError> {
    let user_id = auth.map(|u| u.id).unwrap_or(-1); // 如果没登录，user_id 设为不可能的值

    // 查询所有计划，按创建时间倒序
    let plans = sqlx::query_as::<_, Plan>(
        "SELECT * FROM plans WHERE is_public = 't' OR user_id = $1  ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(plans))
}

// --- 2. 创建计划 (POST /plans) ---
pub async fn create_plan_handler(
    user: AuthUser, // 强制要求登录
    State(state): State<AppState>,
    Json(body): Json<CreatePlanSchema>,
) -> Result<Json<Plan>, AppError> {
    // 插入数据并返回新创建的记录
    let plan = sqlx::query_as::<_, Plan>(
        "INSERT INTO plans (title, description, category, due_date, is_public, user_id) 
         VALUES ($1, $2, $3, $4, $5, $6) 
         RETURNING *",
    )
    .bind(body.title)
    .bind(body.description)
    .bind(body.category)
    .bind(body.due_date)
    .bind(body.is_public)
    .bind(user.id) // 绑定所有权
    .fetch_one(&state.db)
    .await?;

    Ok(Json(plan))
}

// --- 3. 更新计划 (PATCH /plans/:id) ---
pub async fn update_plan_handler(
    Path(id): Path<i32>,
    user: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<UpdatePlanSchema>,
) -> Result<Json<Plan>, AppError> {
    // 使用 fetch_optional 来判断更新是否成功（是否找到属于该用户的记录）
    let plan = sqlx::query_as::<_, Plan>(
        "UPDATE plans SET 
            title = COALESCE($1, title),
            description = COALESCE($2, description),
            status = COALESCE($3, status),
            category = COALESCE($4, category),
            due_date = COALESCE($5, due_date),
            is_public = COALESCE($6, is_public),
            updated_at = NOW()
         WHERE id = $7 AND user_id = $8
         RETURNING *",
    )
    .bind(body.title)
    .bind(body.description)
    .bind(body.status)
    .bind(body.category)
    .bind(body.due_date)
    .bind(body.is_public)
    .bind(id)
    .bind(user.id) // 权限核心：匹配 ID 和 UserID
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Plan not found or unauthorized".to_string()))?;

    Ok(Json(plan))
}

// --- 4. 删除计划 (DELETE /plans/:id) ---
pub async fn delete_plan_handler(
    Path(id): Path<i32>,
    user: AuthUser,
    State(state): State<AppState>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query("DELETE FROM plans WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user.id)
        .execute(&state.db)
        .await?;

    // 检查是否有行被删除
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Plan not found or you don't have permission".to_string(),
        ));
    }

    Ok(StatusCode::NO_CONTENT) // 204 No Content
}

// --- 5. 用户注册 ---
pub async fn register_handler(
    State(state): State<AppState>,
    Json(payload): Json<RegisterSchema>,
) -> Result<Json<serde_json::Value>, AppError> {
    // 验证逻辑（简单示例：长度校验）
    if payload.password.len() < 6 {
        return Err(AppError::BadRequest("Password too short".into()));
    }

    // 1. 哈希密码
    // 修复报错：使用 map_err 将 String 错误转换为 (StatusCode, String)
    let hashed_password =
        hash_password(&payload.password).map_err(|e| AppError::Internal)?; // 转换加密库的错误

    // 2. 存入数据库
    let _ = sqlx::query("INSERT INTO users (username, password_hash) VALUES ($1, $2)")
        .bind(&payload.username)
        .bind(hashed_password)
        .execute(&state.db)
        .await?;

    Ok(Json(json!({"message": "User registered successfully"})))
}

// --- 6. 用户登录 ---
pub async fn login_handler(
    State(state): State<AppState>,
    Json(payload): Json<LoginSchema>,
) -> Result<Json<AuthResponse>, AppError> {
    // 1. 查找用户
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
        .bind(&payload.username)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::Auth("Invalid username or password".into()))?;

    // 2. 验证密码
    if !verify_password(&payload.password, &user.password_hash) {
        return Err(AppError::Auth("Invalid username or password".into()));
    }

    // 3. 生成 Token
    // 修复报错：使用 map_err 将 String 错误转换为 (StatusCode, String)
    let token =
        create_jwt(user.id, &user.username).map_err(|e| AppError::Internal)?;

    Ok(Json(AuthResponse {
        token,
        username: user.username,
    }))
}

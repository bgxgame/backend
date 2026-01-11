// src/handlers.rs
use crate::auth::{create_jwt, hash_password, verify_password, AuthUser, generate_refresh_token};
use crate::models::*;
use crate::AppError;
use crate::AppState;
use crate::validation::ValidatedJson;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde_json::json;
use chrono::{Utc, Duration};

// ======= PROJECTS HANDLERS =======

pub async fn get_projects_handler(
    user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<Project>>, AppError> {
    let projects = sqlx::query_as::<_, Project>(
        "SELECT * FROM projects WHERE user_id = $1 ORDER BY updated_at DESC",
    )
    .bind(user.id)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(projects))
}

pub async fn create_project_handler(
    user: AuthUser,
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<CreateProjectSchema>,
) -> Result<Json<Project>, AppError> {
    let project = sqlx::query_as::<_, Project>(
        "INSERT INTO projects (user_id, name, description, color) VALUES ($1, $2, $3, $4) RETURNING *",
    )
    .bind(user.id)
    .bind(body.name)
    .bind(body.description)
    .bind(body.color.unwrap_or_else(|| "#5E6AD2".to_string()))
    .fetch_one(&state.db)
    .await?;
    Ok(Json(project))
}

pub async fn update_project_handler(
    user: AuthUser,
    Path(id): Path<i32>,
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<UpdateProjectSchema>,
) -> Result<Json<Project>, AppError> {
    let project = sqlx::query_as::<_, Project>(
        r#"UPDATE projects SET 
            name = COALESCE($1, name),
            description = COALESCE($2, description),
            status = COALESCE($3, status),
            color = COALESCE($4, color),
            updated_at = NOW()
         WHERE id = $5 AND user_id = $6
         RETURNING *"#,
    )
    .bind(body.name).bind(body.description).bind(body.status).bind(body.color)
    .bind(id).bind(user.id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("项目未找到".into()))?;

    Ok(Json(project))
}

pub async fn delete_project_handler(
    user: AuthUser,
    Path(id): Path<i32>,
    State(state): State<AppState>,
) -> Result<StatusCode, AppError> {
    let res = sqlx::query("DELETE FROM projects WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user.id)
        .execute(&state.db)
        .await?;
    if res.rows_affected() == 0 { return Err(AppError::NotFound("项目不存在或无权操作".into())); }
    Ok(StatusCode::NO_CONTENT)
}

// ======= ISSUES HANDLERS =======

pub async fn get_all_my_issues_handler(
    user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<Issue>>, AppError> {
    let issues = sqlx::query_as::<_, Issue>(
        "SELECT * FROM issues WHERE user_id = $1 ORDER BY updated_at DESC",
    )
    .bind(user.id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(issues))
}

pub async fn get_project_issues_handler(
    user: AuthUser,
    Path(project_id): Path<i32>,
    Query(query): Query<IssueQuery>,
    State(state): State<AppState>,
) -> Result<Json<Vec<Issue>>, AppError> {
    let project_exists = sqlx::query("SELECT id FROM projects WHERE id = $1 AND user_id = $2")
        .bind(project_id)
        .bind(user.id)
        .fetch_optional(&state.db)
        .await?;

    if project_exists.is_none() { return Err(AppError::Forbidden("无权访问该项目".into())); }

    let issues = sqlx::query_as::<_, Issue>(
        r#"
        SELECT * FROM issues 
        WHERE project_id = $1 
          AND ($2 IS NULL OR title ILIKE $2 OR description ILIKE $2)
          AND ($3 IS NULL OR status = $3)
        ORDER BY priority DESC, created_at DESC
        "#,
    )
    .bind(project_id)
    .bind(query.q.map(|s| format!("%{}%", s)))
    .bind(query.status)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(issues))
}

pub async fn create_issue_handler(
    user: AuthUser,
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<CreateIssueSchema>,
) -> Result<Json<Issue>, AppError> {
    let project_owned = sqlx::query("SELECT id FROM projects WHERE id = $1 AND user_id = $2")
        .bind(body.project_id)
        .bind(user.id)
        .fetch_optional(&state.db)
        .await?;

    if project_owned.is_none() { return Err(AppError::BadRequest("目标项目不存在".into())); }

    let issue = sqlx::query_as::<_, Issue>(
        r#"INSERT INTO issues (project_id, user_id, title, description, priority, due_date) 
           VALUES ($1, $2, $3, $4, $5, $6) RETURNING *"#,
    )
    .bind(body.project_id)
    .bind(user.id)
    .bind(body.title)
    .bind(body.description)
    .bind(body.priority.unwrap_or(0))
    .bind(body.due_date)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(issue))
}

pub async fn update_issue_handler(
    user: AuthUser,
    Path(id): Path<i32>,
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<UpdateIssueSchema>,
) -> Result<Json<Issue>, AppError> {
    let issue = sqlx::query_as::<_, Issue>(
        r#"UPDATE issues SET 
            title = COALESCE($1, title),
            description = CASE WHEN $2 IS NULL THEN description ELSE $2 END,
            status = COALESCE($3, status),
            priority = COALESCE($4, priority),
            due_date = COALESCE($5, due_date),
            updated_at = NOW()
         WHERE id = $6 AND user_id = $7
         RETURNING *"#,
    )
    .bind(body.title).bind(body.description).bind(body.status).bind(body.priority).bind(body.due_date)
    .bind(id).bind(user.id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("任务未找到".into()))?;

    Ok(Json(issue))
}

pub async fn delete_issue_handler(
    user: AuthUser,
    Path(id): Path<i32>,
    State(state): State<AppState>,
) -> Result<StatusCode, AppError> {
    let res = sqlx::query("DELETE FROM issues WHERE id = $1 AND user_id = $2")
        .bind(id).bind(user.id).execute(&state.db).await?;
    if res.rows_affected() == 0 { return Err(AppError::NotFound("任务未找到".into())); }
    Ok(StatusCode::NO_CONTENT)
}

// ======= AUTH HANDLERS (无感刷新版本) =======

pub async fn register_handler(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<RegisterSchema>,
) -> Result<Json<serde_json::Value>, AppError> {
    let hashed_password = hash_password(&payload.password).map_err(|_| AppError::Internal)?;
    sqlx::query("INSERT INTO users (username, password_hash) VALUES ($1, $2)")
        .bind(&payload.username).bind(hashed_password).execute(&state.db).await?;
    Ok(Json(json!({"message": "User registered successfully"})))
}

pub async fn login_handler(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<LoginSchema>,
) -> Result<Json<AuthResponse>, AppError> {
    // 1. 验证用户
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
        .bind(&payload.username).fetch_optional(&state.db).await?
        .ok_or_else(|| AppError::Auth("用户名或密码错误".into()))?;

    if !verify_password(&payload.password, &user.password_hash) {
        return Err(AppError::Auth("用户名或密码错误".into()));
    }

    // 2. 生成 Access Token (短效)
    let token = create_jwt(user.id, &user.username).map_err(|_| AppError::Internal)?;

    // 3. 生成并存储 Refresh Token (长效)
    let refresh_token_str = generate_refresh_token();
    let expires_at = Utc::now() + Duration::days(7); // 7天有效期

    sqlx::query(
        "INSERT INTO refresh_tokens (user_id, token, expires_at) VALUES ($1, $2, $3)"
    )
    .bind(user.id)
    .bind(&refresh_token_str)
    .bind(expires_at)
    .execute(&state.db)
    .await?;

    Ok(Json(AuthResponse { 
        token, 
        refresh_token: Some(refresh_token_str),
        username: user.username 
    }))
}

// 核心：无感刷新接口
pub async fn refresh_handler(
    State(state): State<AppState>,
    Json(payload): Json<RefreshRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    // 1. 检查数据库中是否存在该 Token 且未过期
    let row = sqlx::query!(
        r#"SELECT r.user_id, u.username FROM refresh_tokens r
           JOIN users u ON r.user_id = u.id
           WHERE r.token = $1 AND r.expires_at > NOW()"#,
        payload.refresh_token
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::Auth("登录已过期，请重新登录".into()))?;

    // 2. 签发新的 Access Token
    let new_access_token = create_jwt(row.user_id, &row.username).map_err(|_| AppError::Internal)?;

    // 3. 返回新 Token (这里沿用旧的 Refresh Token，也可以在这里进行滚动更新)
    Ok(Json(AuthResponse {
        token: new_access_token,
        refresh_token: Some(payload.refresh_token),
        username: row.username,
    }))
}

pub async fn get_issue_comments_handler(
    user: AuthUser,
    Path(issue_id): Path<i32>,
    State(state): State<AppState>,
) -> Result<Json<Vec<Comment>>, AppError> {
    // 检查 Issue 是否存在且用户有权访问（通过项目所属权判断）
    let comments = sqlx::query_as::<_, Comment>(
        r#"
        SELECT c.*, u.username 
        FROM comments c
        JOIN users u ON c.user_id = u.id
        JOIN issues i ON c.issue_id = i.id
        JOIN projects p ON i.project_id = p.id
        WHERE c.issue_id = $1 AND p.user_id = $2
        ORDER BY c.created_at ASC
        "#
    )
    .bind(issue_id)
    .bind(user.id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(comments))
}

pub async fn create_comment_handler(
    user: AuthUser,
    Path(issue_id): Path<i32>,
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<CreateCommentSchema>,
) -> Result<Json<Comment>, AppError> {
    // 插入评论
    let comment = sqlx::query_as::<_, Comment>(
        r#"
        WITH inserted AS (
            INSERT INTO comments (issue_id, user_id, content)
            VALUES ($1, $2, $3)
            RETURNING *
        )
        SELECT i.*, u.username FROM inserted i
        JOIN users u ON i.user_id = u.id
        "#
    )
    .bind(issue_id)
    .bind(user.id)
    .bind(body.content)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(comment))
}

pub async fn unified_search_handler(
    user: AuthUser,
    Query(query): Query<IssueQuery>, // 复用包含 q 的 Query 结构
    State(state): State<AppState>,
) -> Result<Json<Vec<UnifiedSearchResult>>, AppError> {
    let q = query.q.unwrap_or_default();
    if q.trim().is_empty() {
        return Ok(Json(vec![]));
    }

    let search_pattern = format!("%{}%", q);

    // 使用 UNION ALL 将项目和任务的结果合并
    // 注意：字段数量和类型必须对齐
    let results = sqlx::query_as::<_, UnifiedSearchResult>(
        r#"
        SELECT 'project' as type, id, name as title, description, status, color 
        FROM projects 
        WHERE user_id = $1 AND (name ILIKE $2 OR description ILIKE $2)
        
        UNION ALL
        
        SELECT 'issue' as type, id, title, description, status, NULL as color 
        FROM issues 
        WHERE user_id = $1 AND (title ILIKE $2 OR description ILIKE $2)
        
        ORDER BY title ASC
        LIMIT 15
        "#
    )
    .bind(user.id)
    .bind(search_pattern)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(results))
}

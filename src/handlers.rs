// src/handlers.rs
use crate::auth::{create_jwt, hash_password, verify_password, AuthUser};
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
    // 查询该用户创建的所有任务，不分项目
    let issues = sqlx::query_as::<_, Issue>(
        "SELECT * FROM issues WHERE user_id = $1 ORDER BY updated_at DESC",
    )
    .bind(user.id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(issues))
}

// 获取指定项目下的所有 Issue
pub async fn get_project_issues_handler(
    user: AuthUser,
    Path(project_id): Path<i32>,
    Query(query): Query<IssueQuery>,
    State(state): State<AppState>,
) -> Result<Json<Vec<Issue>>, AppError> {
    // 权限校验：确保该项目属于该用户
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
    // 确保项目属于该用户
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
    // 业务层校验：如果传了描述且不是空的，则必须大于 5 字
    if let Some(ref desc) = body.description {
        if !desc.is_empty() && desc.len() < 5 {
            return Err(AppError::BadRequest("描述内容如果填写，则至少需要 5 个字".into()));
        }
    }

    let issue = sqlx::query_as::<_, Issue>(
        r#"UPDATE issues SET 
            title = COALESCE($1, title),
            -- 这里的处理逻辑：如果是 None 则不改动，如果是 Some("") 则会更新为 SQL NULL 或空字符串
            description = CASE WHEN $2 IS NULL THEN description ELSE $2 END,
            status = COALESCE($3, status),
            priority = COALESCE($4, priority),
            due_date = COALESCE($5, due_date),
            updated_at = NOW()
         WHERE id = $6 AND user_id = $7
         RETURNING *"#,
    )
    .bind(body.title)       // $1
    .bind(body.description) // $2 (可以是 Some(""), Some("内容"), 或 None)
    .bind(body.status)      // $3
    .bind(body.priority)    // $4
    .bind(body.due_date)    // $5
    .bind(id)               // $6
    .bind(user.id)          // $7
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

// ======= AUTH HANDLERS =======

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
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
        .bind(&payload.username).fetch_optional(&state.db).await?
        .ok_or_else(|| AppError::Auth("用户名或密码错误".into()))?;

    if !verify_password(&payload.password, &user.password_hash) {
        return Err(AppError::Auth("用户名或密码错误".into()));
    }

    let token = create_jwt(user.id, &user.username).map_err(|_| AppError::Internal)?;
    Ok(Json(AuthResponse { token, username: user.username }))
}
// src/main.rs
use axum::{
    http::Method,
    routing::{delete, get, patch, post},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod auth;
mod error;
mod handlers;
mod models;
mod validation;

pub use error::AppError;

use handlers::*;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to connect to Postgres");

    tracing::info!("✅ 数据库连接成功!");

    let state = AppState { db: pool };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_headers(Any);

    let app = Router::new()
        // 认证
        .route("/api/register", post(register_handler))
        .route("/api/login", post(login_handler))
        .route("/api/refresh", post(refresh_handler))
        // 项目路由
        .route("/api/projects", get(get_projects_handler))
        .route("/api/projects", post(create_project_handler))
        .route("/api/projects/:id", patch(update_project_handler))
        .route("/api/projects/:id", delete(delete_project_handler))
        // 任务路由
        .route("/api/issues", get(get_all_my_issues_handler))
        .route("/api/projects/:id/issues", get(get_project_issues_handler))
        .route("/api/search", get(unified_search_handler))
        .route("/api/issues", post(create_issue_handler))
        .route("/api/issues/:id", patch(update_issue_handler))
        .route("/api/issues/:id", delete(delete_issue_handler))
        .route("/api/issues/:id/comments", get(get_issue_comments_handler))
        .route("/api/issues/:id/comments", post(create_comment_handler))
        .with_state(state)
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("🚀 服务器运行在: {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// src/main.rs
use axum::{
    routing::{get, post, patch, delete}, // 引入更多路由方法
    Router,
    http::{Method, HeaderValue}, // 引入 Method 和 HeaderValue
};
use tower_http::cors::{CorsLayer, Any}; // 引入 CORS 相关
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// 引入模块
mod handlers;
mod models;
mod auth;
mod error; // 引入新模块
pub use error::AppError; // 导出方便其他地方使用
mod validation; // <--- 添加这一行，让编译器知道 validation.rs 的存在

// 使用 handlers 中的函数
use handlers::{
    create_plan_handler, delete_plan_handler, get_plans_handler, update_plan_handler,
    login_handler, register_handler
};

#[derive(Clone)]
pub struct AppState { // 注意加上 pub，因为 handlers 里要用
    pub db: PgPool,   // 注意加上 pub
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to connect to Postgres");

    tracing::info!("✅ 成功连接到数据库!");

    let state = AppState { db: pool };

        // --- CORS 配置 (关键步骤) ---
    // 允许前端 (http://localhost:5173) 访问后端
    let cors = CorsLayer::new()
        // 允许的来源：为了开发方便，这里先设为 Any (允许所有)，
        // 生产环境建议改为 specific origin: "http://localhost:5173".parse::<HeaderValue>().unwrap()
        .allow_origin(Any) 
        // 允许的方法
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        // 允许的头 (Authorization 等)
        .allow_headers(Any);

    // 定义路由
    let app = Router::new()
        // 公开路由
        .route("/api/plans", get(get_plans_handler))
        .route("/api/register", post(register_handler)) // 注册
        .route("/api/login", post(login_handler))       // 登录
        // 受保护路由 (在 handlers 内部通过 AuthUser 参数保护，这里路由写法看起来一样)
        .route("/api/plans", post(create_plan_handler))
        .route("/api/plans/:id", patch(update_plan_handler))
        .route("/api/plans/:id", delete(delete_plan_handler))
        .with_state(state)
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("🚀 服务器正在监听: {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
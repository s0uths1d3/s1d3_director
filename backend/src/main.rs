mod handlers;
mod models;
mod llm_client;
mod emotion_analyzer;
mod script_generator;
mod causal_graph;
mod relation_network;
mod co_pilot;

use axum::{Router, routing::get, routing::post};
use handlers::AppState;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // 初始化日志
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "novel2script_pro=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 加载 .env 文件
    dotenvy::dotenv().ok();

    // 读取 DeepSeek API 配置
    let deepseek_api_key = std::env::var("DEEPSEEK_API_KEY").unwrap_or_default();
    let deepseek_base_url = std::env::var("DEEPSEEK_BASE_URL")
        .unwrap_or_else(|_| "https://api.deepseek.com/v1".to_string());

    tracing::info!(
        api_key_set = !deepseek_api_key.is_empty(),
        base_url = %deepseek_base_url,
        "DeepSeek API 配置已加载"
    );

    // 读取数据库连接字符串
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:@123666@localhost:5432/novel2script_pro".to_string());

    tracing::info!("正在连接 PostgreSQL 数据库...");

    // 连接数据库并创建连接池
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await;

    match pool {
        Ok(db_pool) => {
            tracing::info!("PostgreSQL 数据库连接成功!");

            // 自动创建表（如果不存在）
            migrate_database(&db_pool).await;

            // 构建应用状态（包含数据库连接池）
            let state = AppState {
                db: Some(db_pool),
                deepseek_api_key: deepseek_api_key.clone(),
                deepseek_base_url: deepseek_base_url.clone(),
            };

            // 构建路由
            let app = Router::new()
                // 健康检查
                .route("/api/health", get(handlers::health_check))
                // 分析接口
                .route("/api/analyze", post(handlers::analyze_text))
                // 剧本生成
                .route("/api/generate-script", post(handlers::generate_script_handler))
                // 因果图谱
                .route("/api/causal/impact", post(handlers::causal_impact))
                // 关系网络更新
                .route("/api/relation/update", post(handlers::update_relation))
                // AI 副编剧
                .route("/api/co-pilot/chat", post(handlers::copilot_chat_handler))
                .route("/api/co-pilot/suggest", post(handlers::copilot_suggest_handler))
                // CORS 中间件
                .layer(axum::middleware::from_fn(handlers::cors_middleware))
                // 注入共享状态
                .with_state(state);

            let addr = "0.0.0.0:8081";
          tracing::info!("Novel2Script Pro 后端服务启动于 {}", addr);

            let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
            axum::serve(listener, app).await.unwrap();
        }
        Err(e) => {
            tracing::error!("无法连接到 PostgreSQL 数据库: {}", e);
            tracing::warn!("将以无数据库模式启动（部分功能不可用）...");
            
            // 无数据库模式：使用默认状态
            let state = AppState {
                db: None,
                deepseek_api_key: deepseek_api_key.clone(),
                deepseek_base_url: deepseek_base_url.clone(),
            };

            let app = Router::new()
                .route("/api/health", get(handlers::health_check))
                .route("/api/analyze", post(handlers::analyze_text))
                .route("/api/generate-script", post(handlers::generate_script_handler))
                .route("/api/causal/impact", post(handlers::causal_impact))
                .route("/api/relation/update", post(handlers::update_relation))
                .route("/api/co-pilot/chat", post(handlers::copilot_chat_handler))
                .route("/api/co-pilot/suggest", post(handlers::copilot_suggest_handler))
                .layer(axum::middleware::from_fn(handlers::cors_middleware))
                .with_state(state);

            let addr = "0.0.0.0:8081";
          tracing::info!("Novel2Script Pro 后端服务启动于 {} (无数据库模式)", addr);

            let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
            axum::serve(listener, app).await.unwrap();
        }
    }
}

/// 数据库迁移：自动创建必要的表
async fn migrate_database(pool: &sqlx::PgPool) {
    // 剧本存储表：保存用户生成的剧本
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS scripts (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            title VARCHAR(255) NOT NULL,
            source_novel TEXT,
            style VARCHAR(50) DEFAULT 'short_drama',
            yaml_content TEXT NOT NULL,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        )
    "#).execute(pool).await.expect("创建 scripts 表失败");

    // 分析历史表：记录每次分析的结果摘要
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS analysis_history (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            script_id UUID REFERENCES scripts(id) ON DELETE CASCADE,
            novel_preview TEXT,
            emotional_curve JSONB,
            characters_count INTEGER DEFAULT 0,
            scenes_count INTEGER DEFAULT 0,
            created_at TIMESTAMPTZ DEFAULT NOW()
        )
    "#).execute(pool).await.expect("创建 analysis_history 表失败");

    // 聊天记录表：保存副编剧对话
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS copilot_chats (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            session_id VARCHAR(100) NOT NULL,
            role VARCHAR(20) NOT NULL,
            content TEXT NOT NULL,
            created_at TIMESTAMPTZ DEFAULT NOW()
        )
    "#).execute(pool).await.expect("创建 copilot_chats 表失败");

    // 创建索引
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_scripts_created ON scripts(created_at)")
        .execute(pool).await.ok();
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_copilot_session ON copilot_chats(session_id)")
        .execute(pool).await.ok();

    tracing::info!("数据库表结构初始化完成");
}

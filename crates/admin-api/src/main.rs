mod error;
mod openapi;
mod routes;
mod state;

use anyhow::Context;
use axum::Router;
use deadpool_redis::{Config as RedisConfig, Runtime};
use karateway_config::{init_env, AppConfig, DatabaseConfig};
use state::AppState;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use tracing_subscriber::EnvFilter;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables
    init_env();

    // Load configuration
    let config = AppConfig::from_env().context("Failed to load configuration")?;

    // Initialize tracing with log level from config
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(&config.rust_log))
        .init();

    info!(
        "Starting Karateway Admin API v{}",
        env!("CARGO_PKG_VERSION")
    );
    info!("Configuration loaded successfully");

    // Create database connection pool
    let db_config = DatabaseConfig::new(config.clone());
    let pool = db_config
        .create_pool()
        .await
        .context("Failed to create database pool")?;

    info!("Database connection pool created");

    // Create Redis connection pool
    let redis_cfg = RedisConfig::from_url(&config.redis_url());
    let redis_pool = redis_cfg
        .create_pool(Some(Runtime::Tokio1))
        .context("Failed to create Redis pool")?;

    info!("Redis connection pool created");

    // Create application state
    let state = AppState::new(pool, redis_pool);

    // Create router with CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .merge(
            SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi::ApiDoc::openapi()),
        )
        .merge(routes::create_router(state))
        .layer(cors);

    // Get bind address from config
    let addr = format!("{}:{}", config.admin_api_host, config.admin_api_port);

    info!("Admin API listening on {}", addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .context("Failed to bind to address")?;

    axum::serve(listener, app).await.context("Server error")?;

    Ok(())
}

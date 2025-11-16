pub mod app_config;
pub mod audit_logger;
pub mod database;
pub mod redis;
pub mod repository;

pub use app_config::AppConfig;
pub use audit_logger::AuditLogger;
pub use database::DatabaseConfig;
pub use redis::RedisConfig;

use dotenvy::dotenv;

/// Initialize environment variables from .env file
pub fn init_env() {
    dotenv().ok();
}

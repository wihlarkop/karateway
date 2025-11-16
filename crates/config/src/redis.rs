use deadpool_redis::{Config, Pool, Runtime};
use redis::RedisError;

use crate::app_config::AppConfig;

pub struct RedisConfig {
    config: AppConfig,
}

impl RedisConfig {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    pub fn create_pool(&self) -> Result<Pool, RedisError> {
        let cfg = Config::from_url(&self.config.redis_url);
        cfg.create_pool(Some(Runtime::Tokio1)).map_err(|e| {
            RedisError::from((
                redis::ErrorKind::IoError,
                "Pool creation failed",
                e.to_string(),
            ))
        })
    }
}

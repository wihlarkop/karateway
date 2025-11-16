use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::Error as SqlxError;
use std::time::Duration;

use crate::app_config::AppConfig;

pub struct DatabaseConfig {
    config: AppConfig,
}

impl DatabaseConfig {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    pub async fn create_pool(&self) -> Result<PgPool, SqlxError> {
        PgPoolOptions::new()
            .max_connections(self.config.db_max_connections)
            .min_connections(self.config.db_min_connections)
            .acquire_timeout(Duration::from_secs(self.config.db_connect_timeout_seconds))
            .idle_timeout(Duration::from_secs(self.config.db_idle_timeout_seconds))
            .connect(&self.config.database_url)
            .await
    }
}

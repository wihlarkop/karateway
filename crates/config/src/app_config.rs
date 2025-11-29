use envconfig::Envconfig;

#[derive(Envconfig, Clone, Debug)]
pub struct AppConfig {
    // Database Configuration
    #[envconfig(from = "DB_USERNAME", default = "karateway")]
    pub db_username: String,

    #[envconfig(from = "DB_PASSWORD")]
    pub db_password: String,

    #[envconfig(from = "DB_HOST", default = "localhost")]
    pub db_host: String,

    #[envconfig(from = "DB_PORT", default = "5432")]
    pub db_port: u16,

    #[envconfig(from = "DB_NAME", default = "karateway")]
    pub db_name: String,

    #[envconfig(from = "DB_MAX_CONNECTIONS", default = "20")]
    pub db_max_connections: u32,

    #[envconfig(from = "DB_MIN_CONNECTIONS", default = "5")]
    pub db_min_connections: u32,

    #[envconfig(from = "DB_CONNECT_TIMEOUT_SECONDS", default = "10")]
    pub db_connect_timeout_seconds: u64,

    #[envconfig(from = "DB_IDLE_TIMEOUT_SECONDS", default = "600")]
    pub db_idle_timeout_seconds: u64,

    // Redis Configuration
    #[envconfig(from = "REDIS_HOST", default = "localhost")]
    pub redis_host: String,

    #[envconfig(from = "REDIS_PORT", default = "6379")]
    pub redis_port: u16,

    #[envconfig(from = "REDIS_PASSWORD", default = "")]
    pub redis_password: String,

    #[envconfig(from = "REDIS_POOL_SIZE", default = "10")]
    pub redis_pool_size: usize,

    // Gateway Configuration
    #[envconfig(from = "GATEWAY_HOST", default = "0.0.0.0")]
    pub gateway_host: String,

    #[envconfig(from = "GATEWAY_PORT", default = "8080")]
    pub gateway_port: u16,

    // Admin API Configuration
    #[envconfig(from = "ADMIN_API_HOST", default = "0.0.0.0")]
    pub admin_api_host: String,

    #[envconfig(from = "ADMIN_API_PORT", default = "8081")]
    pub admin_api_port: u16,

    // JWT Secret
    #[envconfig(from = "JWT_SECRET")]
    pub jwt_secret: String,

    // Log Level
    #[envconfig(from = "RUST_LOG", default = "info")]
    pub rust_log: String,
}

impl AppConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, envconfig::Error> {
        Self::init_from_env()
    }

    /// Build PostgreSQL connection URL
    pub fn database_url(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}/{}",
            self.db_username, self.db_password, self.db_host, self.db_port, self.db_name
        )
    }

    /// Build Redis connection URL
    pub fn redis_url(&self) -> String {
        if self.redis_password.is_empty() {
            format!("redis://{}:{}", self.redis_host, self.redis_port)
        } else {
            format!("redis://:{}@{}:{}", self.redis_password, self.redis_host, self.redis_port)
        }
    }
}

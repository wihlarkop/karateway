use envconfig::Envconfig;

#[derive(Envconfig, Clone, Debug)]
pub struct AppConfig {
    // Database Configuration
    #[envconfig(from = "DATABASE_URL")]
    pub database_url: String,

    #[envconfig(from = "DB_MAX_CONNECTIONS", default = "20")]
    pub db_max_connections: u32,

    #[envconfig(from = "DB_MIN_CONNECTIONS", default = "5")]
    pub db_min_connections: u32,

    #[envconfig(from = "DB_CONNECT_TIMEOUT_SECONDS", default = "10")]
    pub db_connect_timeout_seconds: u64,

    #[envconfig(from = "DB_IDLE_TIMEOUT_SECONDS", default = "600")]
    pub db_idle_timeout_seconds: u64,

    // Redis Configuration
    #[envconfig(from = "REDIS_URL")]
    pub redis_url: String,

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
}

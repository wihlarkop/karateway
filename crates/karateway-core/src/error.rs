use thiserror::Error;

#[derive(Error, Debug)]
pub enum KaratewayError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),
}

pub type Result<T> = std::result::Result<T, KaratewayError>;

impl From<validator::ValidationErrors> for KaratewayError {
    fn from(err: validator::ValidationErrors) -> Self {
        KaratewayError::Validation(err.to_string())
    }
}

impl KaratewayError {
    /// Get HTTP status code for this error
    pub fn status_code(&self) -> u16 {
        match self {
            KaratewayError::Database(_) => 500,
            KaratewayError::Redis(_) => 500,
            KaratewayError::Validation(_) => 400,
            KaratewayError::NotFound(_) => 404,
            KaratewayError::Conflict(_) => 409,
            KaratewayError::Internal(_) => 500,
            KaratewayError::Configuration(_) => 500,
            KaratewayError::Unauthorized(_) => 401,
            KaratewayError::Forbidden(_) => 403,
        }
    }

    /// Get error code string
    pub fn error_code(&self) -> String {
        match self {
            KaratewayError::Database(_) => "DATABASE_ERROR",
            KaratewayError::Redis(_) => "REDIS_ERROR",
            KaratewayError::Validation(_) => "VALIDATION_ERROR",
            KaratewayError::NotFound(_) => "NOT_FOUND",
            KaratewayError::Conflict(_) => "CONFLICT",
            KaratewayError::Internal(_) => "INTERNAL_ERROR",
            KaratewayError::Configuration(_) => "CONFIGURATION_ERROR",
            KaratewayError::Unauthorized(_) => "UNAUTHORIZED",
            KaratewayError::Forbidden(_) => "FORBIDDEN",
        }
        .to_string()
    }
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct BackendService {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub base_url: String,
    pub health_check_url: Option<String>,
    pub health_check_interval_seconds: Option<i32>,
    pub timeout_ms: Option<i32>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateBackendServiceRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,

    pub description: Option<String>,

    #[validate(url)]
    pub base_url: String,

    #[validate(url)]
    pub health_check_url: Option<String>,

    #[validate(range(min = 10, max = 3600))]
    pub health_check_interval_seconds: Option<i32>,

    #[validate(range(min = 100, max = 60000))]
    pub timeout_ms: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateBackendServiceRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,

    pub description: Option<String>,

    #[validate(url)]
    pub base_url: Option<String>,

    #[validate(url)]
    pub health_check_url: Option<String>,

    #[validate(range(min = 10, max = 3600))]
    pub health_check_interval_seconds: Option<i32>,

    #[validate(range(min = 100, max = 60000))]
    pub timeout_ms: Option<i32>,

    pub is_active: Option<bool>,
}

impl BackendService {
    pub fn is_healthy(&self) -> bool {
        self.is_active
    }
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq, ToSchema)]
#[sqlx(type_name = "varchar")]
pub enum IdentifierType {
    #[sqlx(rename = "ip")]
    Ip,
    #[sqlx(rename = "api_key")]
    ApiKey,
    #[sqlx(rename = "user_id")]
    UserId,
    #[sqlx(rename = "global")]
    Global,
}

impl std::fmt::Display for IdentifierType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdentifierType::Ip => write!(f, "ip"),
            IdentifierType::ApiKey => write!(f, "api_key"),
            IdentifierType::UserId => write!(f, "user_id"),
            IdentifierType::Global => write!(f, "global"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct RateLimit {
    pub id: Uuid,
    pub name: String,
    pub api_route_id: Option<Uuid>,
    pub max_requests: i32,
    pub window_seconds: i32,
    pub identifier_type: IdentifierType,
    pub is_active: bool,
    pub burst_size: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateRateLimitRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,

    pub api_route_id: Option<Uuid>,

    #[validate(range(min = 1, max = 1000000))]
    pub max_requests: i32,

    #[validate(range(min = 1, max = 86400))]
    pub window_seconds: i32,

    pub identifier_type: IdentifierType,

    #[validate(range(min = 1, max = 1000000))]
    pub burst_size: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateRateLimitRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,

    pub api_route_id: Option<Uuid>,

    #[validate(range(min = 1, max = 1000000))]
    pub max_requests: Option<i32>,

    #[validate(range(min = 1, max = 86400))]
    pub window_seconds: Option<i32>,

    pub identifier_type: Option<IdentifierType>,

    pub is_active: Option<bool>,

    #[validate(range(min = 1, max = 1000000))]
    pub burst_size: Option<i32>,
}

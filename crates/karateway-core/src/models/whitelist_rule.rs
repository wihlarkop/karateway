use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq, ToSchema)]
#[sqlx(type_name = "varchar")]
pub enum RuleType {
    #[sqlx(rename = "ip")]
    Ip,
    #[sqlx(rename = "api_key")]
    ApiKey,
    #[sqlx(rename = "jwt")]
    Jwt,
    #[sqlx(rename = "custom")]
    Custom,
}

impl std::fmt::Display for RuleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuleType::Ip => write!(f, "ip"),
            RuleType::ApiKey => write!(f, "api_key"),
            RuleType::Jwt => write!(f, "jwt"),
            RuleType::Custom => write!(f, "custom"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct WhitelistRule {
    pub id: Uuid,
    pub rule_name: String,
    pub rule_type: RuleType,
    pub api_route_id: Option<Uuid>,
    pub config: serde_json::Value,
    pub is_active: bool,
    pub priority: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateWhitelistRuleRequest {
    #[validate(length(min = 1, max = 100))]
    pub rule_name: String,

    pub rule_type: RuleType,

    pub api_route_id: Option<Uuid>,

    pub config: serde_json::Value,

    pub priority: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateWhitelistRuleRequest {
    #[validate(length(min = 1, max = 100))]
    pub rule_name: Option<String>,

    pub rule_type: Option<RuleType>,

    pub api_route_id: Option<Uuid>,

    pub config: Option<serde_json::Value>,

    pub is_active: Option<bool>,

    pub priority: Option<i32>,
}

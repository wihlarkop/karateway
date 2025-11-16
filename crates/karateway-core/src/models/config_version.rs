use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ConfigVersion {
    pub id: Uuid,
    pub version_name: String,
    pub description: Option<String>,
    pub config_snapshot: serde_json::Value,
    pub created_by: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateConfigVersionRequest {
    #[validate(length(min = 1, max = 100))]
    pub version_name: String,

    pub description: Option<String>,

    pub created_by: Option<String>,
}

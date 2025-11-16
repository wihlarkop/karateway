use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "varchar")]
pub enum LoadBalancerAlgorithm {
    #[sqlx(rename = "round_robin")]
    RoundRobin,
    #[sqlx(rename = "least_conn")]
    LeastConn,
    #[sqlx(rename = "ip_hash")]
    IpHash,
    #[sqlx(rename = "weighted")]
    Weighted,
}

impl std::fmt::Display for LoadBalancerAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadBalancerAlgorithm::RoundRobin => write!(f, "round_robin"),
            LoadBalancerAlgorithm::LeastConn => write!(f, "least_conn"),
            LoadBalancerAlgorithm::IpHash => write!(f, "ip_hash"),
            LoadBalancerAlgorithm::Weighted => write!(f, "weighted"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LoadBalancerConfig {
    pub id: Uuid,
    pub backend_service_id: Uuid,
    pub algorithm: LoadBalancerAlgorithm,
    pub health_check_enabled: bool,
    pub config: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

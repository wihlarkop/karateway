use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq, ToSchema)]
#[sqlx(type_name = "varchar")]
pub enum HttpMethod {
    #[sqlx(rename = "GET")]
    GET,
    #[sqlx(rename = "POST")]
    POST,
    #[sqlx(rename = "PUT")]
    PUT,
    #[sqlx(rename = "DELETE")]
    DELETE,
    #[sqlx(rename = "PATCH")]
    PATCH,
    #[sqlx(rename = "HEAD")]
    HEAD,
    #[sqlx(rename = "OPTIONS")]
    OPTIONS,
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::GET => write!(f, "GET"),
            HttpMethod::POST => write!(f, "POST"),
            HttpMethod::PUT => write!(f, "PUT"),
            HttpMethod::DELETE => write!(f, "DELETE"),
            HttpMethod::PATCH => write!(f, "PATCH"),
            HttpMethod::HEAD => write!(f, "HEAD"),
            HttpMethod::OPTIONS => write!(f, "OPTIONS"),
        }
    }
}

impl std::str::FromStr for HttpMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(HttpMethod::GET),
            "POST" => Ok(HttpMethod::POST),
            "PUT" => Ok(HttpMethod::PUT),
            "DELETE" => Ok(HttpMethod::DELETE),
            "PATCH" => Ok(HttpMethod::PATCH),
            "HEAD" => Ok(HttpMethod::HEAD),
            "OPTIONS" => Ok(HttpMethod::OPTIONS),
            _ => Err(format!("Invalid HTTP method: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct ApiRoute {
    pub id: Uuid,
    pub path_pattern: String,
    pub method: HttpMethod,
    pub backend_service_id: Uuid,
    pub strip_path_prefix: bool,
    pub preserve_host_header: bool,
    pub timeout_ms: Option<i32>,
    pub is_active: bool,
    pub priority: i32,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateApiRouteRequest {
    #[validate(length(min = 1, max = 500))]
    pub path_pattern: String,

    pub method: HttpMethod,

    pub backend_service_id: Uuid,

    pub strip_path_prefix: Option<bool>,

    pub preserve_host_header: Option<bool>,

    #[validate(range(min = 100, max = 120000))]
    pub timeout_ms: Option<i32>,

    pub priority: Option<i32>,

    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateApiRouteRequest {
    #[validate(length(min = 1, max = 500))]
    pub path_pattern: Option<String>,

    pub method: Option<HttpMethod>,

    pub backend_service_id: Option<Uuid>,

    pub strip_path_prefix: Option<bool>,

    pub preserve_host_header: Option<bool>,

    #[validate(range(min = 100, max = 120000))]
    pub timeout_ms: Option<i32>,

    pub is_active: Option<bool>,

    pub priority: Option<i32>,

    pub metadata: Option<serde_json::Value>,
}

use chrono::{DateTime, Utc};
use sea_query::Iden;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ConfigAuditLog {
    pub id: Uuid,
    pub table_name: String,
    pub record_id: Uuid,
    pub operation: String,
    pub old_data: Option<serde_json::Value>,
    pub new_data: Option<serde_json::Value>,
    pub changed_by: Option<String>,
    pub changed_at: DateTime<Utc>,
}

/// Security audit log for tracking security events in the gateway
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct AuditLog {
    pub id: Uuid,
    pub event_type: String,
    pub event_category: String,
    pub severity: String,
    pub request_method: Option<String>,
    pub request_path: Option<String>,
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
    pub api_route_id: Option<Uuid>,
    pub backend_service_id: Option<Uuid>,
    pub message: String,
    pub metadata: serde_json::Value,
    pub status_code: Option<i32>,
    pub created_at: DateTime<Utc>,
}

/// Event types for audit logging
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    RateLimitExceeded,
    WhitelistDenied,
    AuthenticationFailed,
    AuthorizationDenied,
    InvalidRequest,
    BackendError,
    ConfigurationChanged,
}

impl ToString for AuditEventType {
    fn to_string(&self) -> String {
        match self {
            AuditEventType::RateLimitExceeded => "rate_limit_exceeded".to_string(),
            AuditEventType::WhitelistDenied => "whitelist_denied".to_string(),
            AuditEventType::AuthenticationFailed => "authentication_failed".to_string(),
            AuditEventType::AuthorizationDenied => "authorization_denied".to_string(),
            AuditEventType::InvalidRequest => "invalid_request".to_string(),
            AuditEventType::BackendError => "backend_error".to_string(),
            AuditEventType::ConfigurationChanged => "configuration_changed".to_string(),
        }
    }
}

/// Event categories for audit logging
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventCategory {
    Authentication,
    RateLimit,
    Whitelist,
    Admin,
}

impl ToString for AuditEventCategory {
    fn to_string(&self) -> String {
        match self {
            AuditEventCategory::Authentication => "authentication".to_string(),
            AuditEventCategory::RateLimit => "rate_limit".to_string(),
            AuditEventCategory::Whitelist => "whitelist".to_string(),
            AuditEventCategory::Admin => "admin".to_string(),
        }
    }
}

/// Severity levels for audit logs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditSeverity {
    Info,
    Warning,
    Critical,
}

impl ToString for AuditSeverity {
    fn to_string(&self) -> String {
        match self {
            AuditSeverity::Info => "info".to_string(),
            AuditSeverity::Warning => "warning".to_string(),
            AuditSeverity::Critical => "critical".to_string(),
        }
    }
}

/// Builder for creating audit log entries
#[derive(Debug, Clone)]
pub struct AuditLogBuilder {
    event_type: String,
    event_category: String,
    severity: String,
    request_method: Option<String>,
    request_path: Option<String>,
    client_ip: Option<String>,
    user_agent: Option<String>,
    api_route_id: Option<Uuid>,
    backend_service_id: Option<Uuid>,
    message: String,
    metadata: serde_json::Value,
    status_code: Option<i32>,
}

impl AuditLogBuilder {
    pub fn new(
        event_type: AuditEventType,
        event_category: AuditEventCategory,
        severity: AuditSeverity,
        message: impl Into<String>,
    ) -> Self {
        Self {
            event_type: event_type.to_string(),
            event_category: event_category.to_string(),
            severity: severity.to_string(),
            request_method: None,
            request_path: None,
            client_ip: None,
            user_agent: None,
            api_route_id: None,
            backend_service_id: None,
            message: message.into(),
            metadata: serde_json::Value::Object(serde_json::Map::new()),
            status_code: None,
        }
    }

    pub fn request_method(mut self, method: impl Into<String>) -> Self {
        self.request_method = Some(method.into());
        self
    }

    pub fn request_path(mut self, path: impl Into<String>) -> Self {
        self.request_path = Some(path.into());
        self
    }

    pub fn client_ip(mut self, ip: impl Into<String>) -> Self {
        self.client_ip = Some(ip.into());
        self
    }

    pub fn user_agent(mut self, ua: impl Into<String>) -> Self {
        self.user_agent = Some(ua.into());
        self
    }

    pub fn api_route_id(mut self, id: Uuid) -> Self {
        self.api_route_id = Some(id);
        self
    }

    pub fn backend_service_id(mut self, id: Uuid) -> Self {
        self.backend_service_id = Some(id);
        self
    }

    pub fn metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn status_code(mut self, code: i32) -> Self {
        self.status_code = Some(code);
        self
    }

    pub fn build(self) -> AuditLog {
        AuditLog {
            id: Uuid::new_v4(),
            event_type: self.event_type,
            event_category: self.event_category,
            severity: self.severity,
            request_method: self.request_method,
            request_path: self.request_path,
            client_ip: self.client_ip,
            user_agent: self.user_agent,
            api_route_id: self.api_route_id,
            backend_service_id: self.backend_service_id,
            message: self.message,
            metadata: self.metadata,
            status_code: self.status_code,
            created_at: Utc::now(),
        }
    }
}

/// Table identifier for audit_logs table
#[derive(Iden)]
pub enum AuditLogs {
    Table,
    Id,
    EventType,
    EventCategory,
    Severity,
    RequestMethod,
    RequestPath,
    ClientIp,
    UserAgent,
    ApiRouteId,
    BackendServiceId,
    Message,
    Metadata,
    StatusCode,
    CreatedAt,
}

/// Table identifier for config_audit_log table
#[derive(Iden)]
pub enum ConfigAuditLogs {
    Table,
    Id,
    TableName,
    RecordId,
    Operation,
    OldData,
    NewData,
    ChangedBy,
    ChangedAt,
}

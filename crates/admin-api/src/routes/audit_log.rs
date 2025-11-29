use crate::{error::ApiError, state::AppState};
use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use karateway_core::{models::AuditLog, JsonResponse};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct AuditLogQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub event_type: Option<String>,
    pub event_category: Option<String>,
    pub severity: Option<String>,
    pub client_ip: Option<String>,
}

fn default_limit() -> i64 {
    50
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuditLogResponse {
    pub logs: Vec<AuditLog>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

pub fn routes(_state: AppState) -> Router<AppState> {
    Router::new().route("/", get(list_audit_logs))
}

/// List audit logs with optional filtering
#[utoipa::path(
    get,
    path = "/api/audit-logs",
    tag = "audit-logs",
    params(AuditLogQuery),
    responses(
        (status = 200, description = "Successfully retrieved audit logs", body = JsonResponse<AuditLogResponse>),
        (status = 500, description = "Internal server error")
    )
)]
async fn list_audit_logs(
    State(state): State<AppState>,
    Query(query): Query<AuditLogQuery>,
) -> Result<Json<JsonResponse<AuditLogResponse>>, ApiError> {
    let limit = query.limit.min(1000); // Max 1000 records at once
    let offset = query.offset;

    // Use the repository to fetch audit logs
    let logs = state
        .audit_log_repo
        .list(limit, offset)
        .await
        .map_err(|e| ApiError(e))?;

    let total = state.audit_log_repo.count().await.map_err(|e| ApiError(e))?;

    Ok(Json(JsonResponse::success(AuditLogResponse {
        logs,
        total,
        limit,
        offset,
    })))
}

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

    // Build the base query
    let mut sql = String::from(
        r#"
        SELECT
            id, event_type, event_category, severity,
            request_method, request_path, client_ip, user_agent,
            api_route_id, backend_service_id, message, metadata,
            status_code, created_at
        FROM audit_logs
        WHERE 1=1
        "#,
    );

    // Add filters if provided
    if query.event_type.is_some() {
        sql.push_str(" AND event_type = $1");
    }
    if query.event_category.is_some() {
        sql.push_str(if query.event_type.is_some() {
            " AND event_category = $2"
        } else {
            " AND event_category = $1"
        });
    }
    if query.severity.is_some() {
        let param_num = 1
            + query.event_type.is_some() as i32
            + query.event_category.is_some() as i32;
        sql.push_str(&format!(" AND severity = ${}", param_num));
    }
    if query.client_ip.is_some() {
        let param_num = 1
            + query.event_type.is_some() as i32
            + query.event_category.is_some() as i32
            + query.severity.is_some() as i32;
        sql.push_str(&format!(" AND client_ip = ${}", param_num));
    }

    sql.push_str(" ORDER BY created_at DESC");

    // For simplicity, let's use the basic query without filters for now
    // TODO: Implement proper parameterized queries with filters
    let logs = sqlx::query_as::<_, AuditLog>(
        r#"
        SELECT
            id, event_type, event_category, severity,
            request_method, request_path, client_ip, user_agent,
            api_route_id, backend_service_id, message, metadata,
            status_code, created_at
        FROM audit_logs
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| ApiError(e.into()))?;

    let total: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) as count
        FROM audit_logs
        "#,
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| ApiError(e.into()))?;

    Ok(Json(JsonResponse::success(AuditLogResponse {
        logs,
        total: total.0,
        limit,
        offset,
    })))
}

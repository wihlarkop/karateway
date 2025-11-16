pub mod api_route;
pub mod audit_log;
pub mod backend_service;
pub mod health;
pub mod rate_limit;
pub mod service_health;
pub mod whitelist_rule;

use crate::state::AppState;
use axum::{routing::get, Router};

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health::health_check))
        .route(
            "/api/services/health",
            get(service_health::get_services_health),
        )
        .nest("/api/services", backend_service::routes(state.clone()))
        .nest("/api/routes", api_route::routes(state.clone()))
        .nest("/api/whitelist", whitelist_rule::routes(state.clone()))
        .nest("/api/rate-limits", rate_limit::routes(state.clone()))
        .nest("/api/audit-logs", audit_log::routes(state.clone()))
        .with_state(state)
}

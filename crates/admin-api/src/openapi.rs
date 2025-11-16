use utoipa::OpenApi;

use karateway_core::{
    models::{
        ApiRoute, AuditLog, BackendService, CreateApiRouteRequest, CreateBackendServiceRequest,
        CreateRateLimitRequest, CreateWhitelistRuleRequest, HttpMethod, IdentifierType, RateLimit,
        RuleType, UpdateApiRouteRequest, UpdateBackendServiceRequest, UpdateRateLimitRequest,
        UpdateWhitelistRuleRequest, WhitelistRule,
    },
    JsonResponse, MetaResponse,
};

use crate::routes::{
    audit_log::{AuditLogQuery, AuditLogResponse},
    backend_service::BackendServiceWithRoutes,
    health::{DatabaseStatus, HealthResponse},
};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::health::health_check,
        crate::routes::backend_service::create_service,
        crate::routes::backend_service::list_services,
        crate::routes::backend_service::get_service,
        crate::routes::backend_service::update_service,
        crate::routes::backend_service::delete_service,
        crate::routes::backend_service::get_service_with_routes,
        crate::routes::api_route::create_route,
        crate::routes::api_route::list_routes,
        crate::routes::api_route::get_route,
        crate::routes::api_route::update_route,
        crate::routes::api_route::delete_route,
        crate::routes::rate_limit::create_limit,
        crate::routes::rate_limit::list_limits,
        crate::routes::rate_limit::get_limit,
        crate::routes::rate_limit::update_limit,
        crate::routes::rate_limit::delete_limit,
        crate::routes::whitelist_rule::create_rule,
        crate::routes::whitelist_rule::list_rules,
        crate::routes::whitelist_rule::get_rule,
        crate::routes::whitelist_rule::update_rule,
        crate::routes::whitelist_rule::delete_rule,
        crate::routes::audit_log::list_audit_logs,
    ),
    components(
        schemas(
            // Core models
            BackendService,
            BackendServiceWithRoutes,
            CreateBackendServiceRequest,
            UpdateBackendServiceRequest,
            ApiRoute,
            CreateApiRouteRequest,
            UpdateApiRouteRequest,
            HttpMethod,
            RateLimit,
            CreateRateLimitRequest,
            UpdateRateLimitRequest,
            IdentifierType,
            WhitelistRule,
            CreateWhitelistRuleRequest,
            UpdateWhitelistRuleRequest,
            RuleType,
            AuditLog,
            AuditLogQuery,
            AuditLogResponse,
            // Response wrappers
            JsonResponse<BackendService>,
            JsonResponse<BackendServiceWithRoutes>,
            JsonResponse<Vec<BackendService>>,
            JsonResponse<ApiRoute>,
            JsonResponse<Vec<ApiRoute>>,
            JsonResponse<RateLimit>,
            JsonResponse<Vec<RateLimit>>,
            JsonResponse<WhitelistRule>,
            JsonResponse<Vec<WhitelistRule>>,
            JsonResponse<HealthResponse>,
            MetaResponse,
            HealthResponse,
            DatabaseStatus,
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "backend-services", description = "Backend service management"),
        (name = "api-routes", description = "API route management"),
        (name = "rate-limits", description = "Rate limiting configuration"),
        (name = "whitelist-rules", description = "Whitelist and access control rules"),
        (name = "audit-logs", description = "Security audit logs"),
    ),
    info(
        title = "Karateway Admin API",
        version = env!("CARGO_PKG_VERSION"),
        description = "Admin API for managing Karateway API Gateway configuration",
        contact(
            name = "Karateway Team",
            email = "support@karateway.io"
        ),
        license(
            name = "MIT",
        )
    )
)]
pub struct ApiDoc;

use deadpool_redis::Pool as RedisPool;
use karateway_config::repository::{
    ApiRouteRepository, AuditLogRepository, BackendServiceRepository, RateLimitRepository,
    WhitelistRuleRepository,
};
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub redis_pool: RedisPool,
    pub backend_service_repo: BackendServiceRepository,
    pub api_route_repo: ApiRouteRepository,
    pub whitelist_rule_repo: WhitelistRuleRepository,
    pub rate_limit_repo: RateLimitRepository,
    pub audit_log_repo: AuditLogRepository,
}

impl AppState {
    pub fn new(pool: PgPool, redis_pool: RedisPool) -> Self {
        Self {
            db_pool: pool.clone(),
            redis_pool,
            backend_service_repo: BackendServiceRepository::new(pool.clone()),
            api_route_repo: ApiRouteRepository::new(pool.clone()),
            whitelist_rule_repo: WhitelistRuleRepository::new(pool.clone()),
            rate_limit_repo: RateLimitRepository::new(pool.clone()),
            audit_log_repo: AuditLogRepository::new(pool),
        }
    }
}

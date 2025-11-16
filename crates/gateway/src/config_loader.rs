use anyhow::Result;
use arc_swap::ArcSwap;
use karateway_config::repository::{
    ApiRouteRepository, BackendServiceRepository, RateLimitRepository, WhitelistRuleRepository,
};
use karateway_core::models::{ApiRoute, BackendService, RateLimit, WhitelistRule};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info};
use uuid::Uuid;

/// Configuration snapshot loaded from database
#[derive(Clone, Debug)]
pub struct GatewayConfig {
    /// All backend services indexed by ID
    pub services: HashMap<Uuid, BackendService>,
    /// All active API routes
    pub routes: Vec<ApiRoute>,
    /// All active rate limits indexed by route ID
    pub rate_limits: HashMap<Option<Uuid>, Vec<RateLimit>>,
    /// All active whitelist rules indexed by route ID
    pub whitelist_rules: HashMap<Option<Uuid>, Vec<WhitelistRule>>,
}

impl GatewayConfig {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
            routes: Vec::new(),
            rate_limits: HashMap::new(),
            whitelist_rules: HashMap::new(),
        }
    }
}

/// Loads and manages configuration from PostgreSQL
pub struct ConfigLoader {
    db_pool: PgPool,
    config: Arc<ArcSwap<GatewayConfig>>,
}

impl ConfigLoader {
    pub fn new(db_pool: PgPool) -> Self {
        Self {
            db_pool,
            config: Arc::new(ArcSwap::from_pointee(GatewayConfig::new())),
        }
    }

    /// Load configuration from database
    pub async fn load_config(&self) -> Result<()> {
        debug!("Loading configuration from database");

        // Load backend services
        let service_repo = BackendServiceRepository::new(self.db_pool.clone());
        let services_result = service_repo.list(1, 1000).await?;

        let mut services_map = HashMap::new();
        for service in services_result {
            if service.is_active {
                services_map.insert(service.id.clone(), service);
            }
        }

        info!("Loaded {} active backend services", services_map.len());

        // Load API routes
        let route_repo = ApiRouteRepository::new(self.db_pool.clone());
        let routes_result = route_repo.list(1, 1000).await?;

        let active_routes: Vec<ApiRoute> =
            routes_result.into_iter().filter(|r| r.is_active).collect();

        info!("Loaded {} active API routes", active_routes.len());

        // Load rate limits
        let rate_limit_repo = RateLimitRepository::new(self.db_pool.clone());
        let rate_limits_result = rate_limit_repo.list_active().await?;

        // Group rate limits by route_id
        let mut rate_limits_map: HashMap<Option<Uuid>, Vec<RateLimit>> = HashMap::new();
        for limit in rate_limits_result {
            debug!(
                "Loading rate limit: name={}, route_id={:?}, identifier_type={}",
                limit.name, limit.api_route_id, limit.identifier_type
            );
            rate_limits_map
                .entry(limit.api_route_id.clone())
                .or_insert_with(Vec::new)
                .push(limit);
        }

        info!(
            "Loaded {} active rate limits",
            rate_limits_map.values().map(|v| v.len()).sum::<usize>()
        );
        debug!(
            "Rate limits map keys: {:?}",
            rate_limits_map.keys().collect::<Vec<_>>()
        );

        // Load whitelist rules
        let whitelist_repo = WhitelistRuleRepository::new(self.db_pool.clone());
        let whitelist_result = whitelist_repo.list_active().await?;

        // Group whitelist rules by route_id
        let mut whitelist_map: HashMap<Option<Uuid>, Vec<WhitelistRule>> = HashMap::new();
        for rule in whitelist_result {
            debug!(
                "Loading whitelist rule: name={}, route_id={:?}, type={}",
                rule.rule_name, rule.api_route_id, rule.rule_type
            );
            whitelist_map
                .entry(rule.api_route_id.clone())
                .or_insert_with(Vec::new)
                .push(rule);
        }

        info!(
            "Loaded {} active whitelist rules",
            whitelist_map.values().map(|v| v.len()).sum::<usize>()
        );

        // Create new config snapshot
        let new_config = GatewayConfig {
            services: services_map,
            routes: active_routes,
            rate_limits: rate_limits_map,
            whitelist_rules: whitelist_map,
        };

        // Atomically swap the configuration
        self.config.store(Arc::new(new_config));

        info!("Configuration updated successfully");
        Ok(())
    }

    /// Get current configuration snapshot
    pub fn get_config(&self) -> Arc<GatewayConfig> {
        self.config.load_full()
    }

    /// Start background task to watch for configuration changes
    pub async fn start_reload_watcher(&self) {
        info!("Starting configuration reload watcher");

        // PostgreSQL LISTEN/NOTIFY implementation would go here
        // For now, use periodic polling as a fallback
        let mut interval = tokio::time::interval(Duration::from_secs(10));

        loop {
            interval.tick().await;

            debug!("Checking for configuration updates");

            if let Err(e) = self.load_config().await {
                error!("Failed to reload configuration: {}", e);
            } else {
                debug!("Configuration check complete");
            }
        }
    }

    /// Get a backend service by ID
    pub fn get_service(&self, service_id: &Uuid) -> Option<BackendService> {
        let config = self.get_config();
        config.services.get(service_id).cloned()
    }

    /// Find matching route for a request
    pub fn find_route(&self, path: &str, method: &str) -> Option<ApiRoute> {
        let config = self.get_config();

        // Find routes matching the method and path pattern
        // TODO: Implement proper pattern matching with wildcards
        // For now, use exact match
        config
            .routes
            .iter()
            .filter(|route| {
                route.method.to_string() == method.to_uppercase()
                    && path.starts_with(&route.path_pattern)
            })
            .max_by_key(|route| route.priority)
            .cloned()
    }
}

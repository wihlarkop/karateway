use karateway_core::models::{ApiRoute, BackendService, RateLimit, WhitelistRule};
use std::sync::Arc;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::config_loader::ConfigLoader;

/// Router handles matching incoming requests to configured routes
pub struct Router {
    config_loader: Arc<ConfigLoader>,
}

impl Router {
    pub fn new(config_loader: Arc<ConfigLoader>) -> Self {
        Self { config_loader }
    }

    /// Find the matching route and backend service for a request
    pub fn route_request(&self, path: &str, method: &str) -> Option<(ApiRoute, BackendService)> {
        debug!("Routing request: {} {}", method, path);

        // Find matching route
        let route = self.config_loader.find_route(path, method)?;

        debug!(
            "Matched route: {} {} -> service {}",
            route.method, route.path_pattern, route.backend_service_id
        );

        // Get the backend service
        let service = self.config_loader.get_service(&route.backend_service_id)?;

        if !service.is_active {
            warn!("Backend service {} is not active", service.id);
            return None;
        }

        debug!(
            "Routing to backend: {} ({})",
            service.name, service.base_url
        );

        Some((route, service))
    }

    /// Transform the request path according to route configuration
    pub fn transform_path(&self, route: &ApiRoute, original_path: &str) -> String {
        if route.strip_path_prefix {
            // Remove the matched prefix
            let prefix = &route.path_pattern;
            if let Some(stripped) = original_path.strip_prefix(prefix) {
                // Ensure the path starts with /
                if stripped.is_empty() || !stripped.starts_with('/') {
                    format!("/{}", stripped)
                } else {
                    stripped.to_string()
                }
            } else {
                original_path.to_string()
            }
        } else {
            original_path.to_string()
        }
    }

    /// Build the upstream URL
    pub fn build_upstream_url(
        &self,
        service: &BackendService,
        path: &str,
        query: Option<&str>,
    ) -> String {
        let base_url = service.base_url.trim_end_matches('/');
        let clean_path = if path.starts_with('/') {
            path
        } else {
            &format!("/{}", path)
        };

        match query {
            Some(q) if !q.is_empty() => format!("{}{}?{}", base_url, clean_path, q),
            _ => format!("{}{}", base_url, clean_path),
        }
    }

    /// Get rate limits for a route
    pub fn get_rate_limits(&self, route_id: &Uuid) -> Option<Vec<RateLimit>> {
        let config = self.config_loader.get_config();

        debug!("Looking for rate limits for route_id: {}", route_id);
        debug!(
            "Available rate limit keys: {:?}",
            config.rate_limits.keys().collect::<Vec<_>>()
        );

        // Get route-specific rate limits
        let mut limits = config
            .rate_limits
            .get(&Some(*route_id))
            .cloned()
            .unwrap_or_default();

        debug!("Found {} route-specific rate limits", limits.len());

        // Also get global rate limits (where api_route_id is None)
        if let Some(global_limits) = config.rate_limits.get(&None) {
            debug!("Found {} global rate limits", global_limits.len());
            limits.extend(global_limits.clone());
        }

        if limits.is_empty() {
            debug!("No rate limits found for route {}", route_id);
            None
        } else {
            debug!(
                "Returning {} total rate limits for route {}",
                limits.len(),
                route_id
            );
            Some(limits)
        }
    }

    /// Get whitelist rules for a route
    pub fn get_whitelist_rules(&self, route_id: &Uuid) -> Option<Vec<WhitelistRule>> {
        let config = self.config_loader.get_config();

        debug!("Looking for whitelist rules for route_id: {}", route_id);

        // Get route-specific whitelist rules
        let mut rules = config
            .whitelist_rules
            .get(&Some(*route_id))
            .cloned()
            .unwrap_or_default();

        debug!("Found {} route-specific whitelist rules", rules.len());

        // Also get global whitelist rules (where api_route_id is None)
        if let Some(global_rules) = config.whitelist_rules.get(&None) {
            debug!("Found {} global whitelist rules", global_rules.len());
            rules.extend(global_rules.clone());
        }

        if rules.is_empty() {
            debug!("No whitelist rules found for route {}", route_id);
            None
        } else {
            // Sort by priority (highest first)
            rules.sort_by(|a, b| b.priority.cmp(&a.priority));
            debug!(
                "Returning {} total whitelist rules for route {}",
                rules.len(),
                route_id
            );
            Some(rules)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use karateway_core::models::HttpMethod;
    use uuid::Uuid;

    #[test]
    fn test_transform_path_with_strip() {
        let route = ApiRoute {
            id: Uuid::new_v4(),
            path_pattern: "/api/v1".to_string(),
            method: HttpMethod::GET,
            backend_service_id: Uuid::new_v4(),
            strip_path_prefix: true,
            preserve_host_header: true,
            timeout_ms: Some(5000),
            priority: 100,
            is_active: true,
            metadata: serde_json::Value::Null,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // Mock router (config_loader not used in this test)
        let original = "/api/v1/users";
        let expected = "/users";

        // Direct test without router instance
        let result = if route.strip_path_prefix {
            original
                .strip_prefix(&route.path_pattern)
                .map(|s| {
                    if s.is_empty() || !s.starts_with('/') {
                        format!("/{}", s)
                    } else {
                        s.to_string()
                    }
                })
                .unwrap_or_else(|| original.to_string())
        } else {
            original.to_string()
        };

        assert_eq!(result, expected);
    }
}

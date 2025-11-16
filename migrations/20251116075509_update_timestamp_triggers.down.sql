-- Drop all update triggers
DROP TRIGGER IF EXISTS update_load_balancer_config_updated_at ON load_balancer_config;
DROP TRIGGER IF EXISTS update_rate_limits_updated_at ON rate_limits;
DROP TRIGGER IF EXISTS update_whitelist_rules_updated_at ON whitelist_rules;
DROP TRIGGER IF EXISTS update_api_routes_updated_at ON api_routes;
DROP TRIGGER IF EXISTS update_backend_services_updated_at ON backend_services;

-- Drop trigger function
DROP FUNCTION IF EXISTS update_updated_at_column();

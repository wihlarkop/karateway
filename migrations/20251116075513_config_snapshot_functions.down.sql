-- Drop notification triggers
DROP TRIGGER IF EXISTS notify_load_balancer_config_change ON load_balancer_config;
DROP TRIGGER IF EXISTS notify_rate_limits_change ON rate_limits;
DROP TRIGGER IF EXISTS notify_whitelist_rules_change ON whitelist_rules;
DROP TRIGGER IF EXISTS notify_api_routes_change ON api_routes;
DROP TRIGGER IF EXISTS notify_backend_services_change ON backend_services;

-- Drop notification function
DROP FUNCTION IF EXISTS notify_config_change();

-- Drop snapshot function
DROP FUNCTION IF EXISTS create_config_snapshot(VARCHAR, TEXT, VARCHAR);

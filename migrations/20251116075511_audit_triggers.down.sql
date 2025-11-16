-- Drop audit triggers
DROP TRIGGER IF EXISTS audit_load_balancer_config ON load_balancer_config;
DROP TRIGGER IF EXISTS audit_rate_limits ON rate_limits;
DROP TRIGGER IF EXISTS audit_whitelist_rules ON whitelist_rules;
DROP TRIGGER IF EXISTS audit_api_routes ON api_routes;
DROP TRIGGER IF EXISTS audit_backend_services ON backend_services;

-- Drop audit trigger function
DROP FUNCTION IF EXISTS audit_trigger_func();

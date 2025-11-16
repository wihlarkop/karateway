-- Drop indexes
DROP INDEX IF EXISTS idx_gateway_metrics_status;
DROP INDEX IF EXISTS idx_gateway_metrics_route;
DROP INDEX IF EXISTS idx_gateway_metrics_timestamp;
DROP INDEX IF EXISTS idx_config_audit_log_timestamp;
DROP INDEX IF EXISTS idx_config_audit_log_table_record;
DROP INDEX IF EXISTS idx_rate_limits_active;
DROP INDEX IF EXISTS idx_rate_limits_route;
DROP INDEX IF EXISTS idx_whitelist_rules_active;
DROP INDEX IF EXISTS idx_whitelist_rules_route;
DROP INDEX IF EXISTS idx_api_routes_active;
DROP INDEX IF EXISTS idx_api_routes_backend_service;
DROP INDEX IF EXISTS idx_api_routes_path_method;

-- Indexes for Performance
CREATE INDEX idx_api_routes_path_method ON api_routes(path_pattern, method);
CREATE INDEX idx_api_routes_backend_service ON api_routes(backend_service_id);
CREATE INDEX idx_api_routes_active ON api_routes(is_active);
CREATE INDEX idx_whitelist_rules_route ON whitelist_rules(api_route_id);
CREATE INDEX idx_whitelist_rules_active ON whitelist_rules(is_active);
CREATE INDEX idx_rate_limits_route ON rate_limits(api_route_id);
CREATE INDEX idx_rate_limits_active ON rate_limits(is_active);
CREATE INDEX idx_config_audit_log_table_record ON config_audit_log(table_name, record_id);
CREATE INDEX idx_config_audit_log_timestamp ON config_audit_log(changed_at);
CREATE INDEX idx_gateway_metrics_timestamp ON gateway_metrics(timestamp);
CREATE INDEX idx_gateway_metrics_route ON gateway_metrics(route_id);
CREATE INDEX idx_gateway_metrics_status ON gateway_metrics(status_code);

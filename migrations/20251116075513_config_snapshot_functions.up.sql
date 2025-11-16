-- Function to create a configuration snapshot
CREATE OR REPLACE FUNCTION create_config_snapshot(
    p_version_name VARCHAR(100),
    p_description TEXT DEFAULT NULL,
    p_created_by VARCHAR(100) DEFAULT NULL
)
RETURNS UUID AS $$
DECLARE
    v_snapshot_id UUID;
    v_snapshot JSONB;
BEGIN
    -- Build complete config snapshot
    SELECT jsonb_build_object(
        'backend_services', (SELECT jsonb_agg(row_to_json(t.*)) FROM backend_services t WHERE is_active = true),
        'api_routes', (SELECT jsonb_agg(row_to_json(t.*)) FROM api_routes t WHERE is_active = true),
        'whitelist_rules', (SELECT jsonb_agg(row_to_json(t.*)) FROM whitelist_rules t WHERE is_active = true),
        'rate_limits', (SELECT jsonb_agg(row_to_json(t.*)) FROM rate_limits t WHERE is_active = true),
        'load_balancer_config', (SELECT jsonb_agg(row_to_json(t.*)) FROM load_balancer_config t)
    ) INTO v_snapshot;

    -- Insert snapshot
    INSERT INTO config_versions (version_name, description, config_snapshot, created_by)
    VALUES (p_version_name, p_description, v_snapshot, p_created_by)
    RETURNING id INTO v_snapshot_id;

    RETURN v_snapshot_id;
END;
$$ LANGUAGE plpgsql;

-- Function to notify config changes (for Pingora gateway to reload)
CREATE OR REPLACE FUNCTION notify_config_change()
RETURNS TRIGGER AS $$
BEGIN
    PERFORM pg_notify('config_update', json_build_object(
        'table', TG_TABLE_NAME,
        'operation', TG_OP,
        'timestamp', CURRENT_TIMESTAMP
    )::text);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply notification triggers
CREATE TRIGGER notify_backend_services_change
    AFTER INSERT OR UPDATE OR DELETE ON backend_services
    FOR EACH ROW EXECUTE FUNCTION notify_config_change();

CREATE TRIGGER notify_api_routes_change
    AFTER INSERT OR UPDATE OR DELETE ON api_routes
    FOR EACH ROW EXECUTE FUNCTION notify_config_change();

CREATE TRIGGER notify_whitelist_rules_change
    AFTER INSERT OR UPDATE OR DELETE ON whitelist_rules
    FOR EACH ROW EXECUTE FUNCTION notify_config_change();

CREATE TRIGGER notify_rate_limits_change
    AFTER INSERT OR UPDATE OR DELETE ON rate_limits
    FOR EACH ROW EXECUTE FUNCTION notify_config_change();

CREATE TRIGGER notify_load_balancer_config_change
    AFTER INSERT OR UPDATE OR DELETE ON load_balancer_config
    FOR EACH ROW EXECUTE FUNCTION notify_config_change();

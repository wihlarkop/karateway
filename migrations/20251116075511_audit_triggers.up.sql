-- Generic audit trigger function
CREATE OR REPLACE FUNCTION audit_trigger_func()
RETURNS TRIGGER AS $$
BEGIN
    IF (TG_OP = 'DELETE') THEN
        INSERT INTO config_audit_log (table_name, record_id, operation, old_data, changed_by)
        VALUES (TG_TABLE_NAME, OLD.id, TG_OP, row_to_json(OLD), current_user);
        RETURN OLD;
    ELSIF (TG_OP = 'UPDATE') THEN
        INSERT INTO config_audit_log (table_name, record_id, operation, old_data, new_data, changed_by)
        VALUES (TG_TABLE_NAME, NEW.id, TG_OP, row_to_json(OLD), row_to_json(NEW), current_user);
        RETURN NEW;
    ELSIF (TG_OP = 'INSERT') THEN
        INSERT INTO config_audit_log (table_name, record_id, operation, new_data, changed_by)
        VALUES (TG_TABLE_NAME, NEW.id, TG_OP, row_to_json(NEW), current_user);
        RETURN NEW;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Apply audit triggers to configuration tables
CREATE TRIGGER audit_backend_services
    AFTER INSERT OR UPDATE OR DELETE ON backend_services
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_func();

CREATE TRIGGER audit_api_routes
    AFTER INSERT OR UPDATE OR DELETE ON api_routes
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_func();

CREATE TRIGGER audit_whitelist_rules
    AFTER INSERT OR UPDATE OR DELETE ON whitelist_rules
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_func();

CREATE TRIGGER audit_rate_limits
    AFTER INSERT OR UPDATE OR DELETE ON rate_limits
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_func();

CREATE TRIGGER audit_load_balancer_config
    AFTER INSERT OR UPDATE OR DELETE ON load_balancer_config
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_func();

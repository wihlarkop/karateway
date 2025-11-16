-- Audit logs table for security events
CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Event information
    event_type VARCHAR(50) NOT NULL,
    event_category VARCHAR(50) NOT NULL,
    severity VARCHAR(20) NOT NULL,

    -- Request details
    request_method VARCHAR(10),
    request_path TEXT,
    client_ip VARCHAR(45),
    user_agent TEXT,

    -- Related entities
    api_route_id UUID REFERENCES api_routes(id) ON DELETE SET NULL,
    backend_service_id UUID REFERENCES backend_services(id) ON DELETE SET NULL,

    -- Event details
    message TEXT NOT NULL,
    metadata JSONB DEFAULT '{}'::jsonb,

    -- Response details
    status_code INTEGER,

    -- Timestamp
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for common queries
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at DESC);
CREATE INDEX idx_audit_logs_event_type ON audit_logs(event_type);
CREATE INDEX idx_audit_logs_event_category ON audit_logs(event_category);
CREATE INDEX idx_audit_logs_client_ip ON audit_logs(client_ip);
CREATE INDEX idx_audit_logs_severity ON audit_logs(severity);

-- Create a function to automatically delete old audit logs (retention policy: 90 days)
CREATE OR REPLACE FUNCTION cleanup_old_audit_logs() RETURNS void AS $$
BEGIN
    DELETE FROM audit_logs
    WHERE created_at < NOW() - INTERVAL '90 days';
END;
$$ LANGUAGE plpgsql;

-- Add comments
COMMENT ON TABLE audit_logs IS 'Security event audit logs with 90-day retention policy';
COMMENT ON FUNCTION cleanup_old_audit_logs() IS 'Deletes audit logs older than 90 days. Run manually: SELECT cleanup_old_audit_logs();';

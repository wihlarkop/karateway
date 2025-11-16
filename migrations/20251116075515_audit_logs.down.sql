-- Drop cleanup function
DROP FUNCTION IF EXISTS cleanup_old_audit_logs();

-- Drop indexes
DROP INDEX IF EXISTS idx_audit_logs_severity;
DROP INDEX IF EXISTS idx_audit_logs_client_ip;
DROP INDEX IF EXISTS idx_audit_logs_event_category;
DROP INDEX IF EXISTS idx_audit_logs_event_type;
DROP INDEX IF EXISTS idx_audit_logs_created_at;

-- Drop audit_logs table
DROP TABLE IF EXISTS audit_logs;

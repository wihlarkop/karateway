# Karateway Database Schema

Complete database schema documentation for Karateway.

## Database: PostgreSQL 17+

All tables use UUID primary keys and include timestamp tracking (`created_at`, `updated_at`).

---

## Tables

### 1. backend_services

Stores backend services that the gateway proxies to.

```sql
CREATE TABLE backend_services (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    base_url VARCHAR(512) NOT NULL,
    health_check_url VARCHAR(512),
    health_check_interval_seconds INTEGER DEFAULT 30,
    timeout_ms INTEGER DEFAULT 30000,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**Indexes:**
- `idx_backend_services_is_active` ON `is_active`
- `idx_backend_services_name` ON `name`

**Constraints:**
- `name` must be unique
- `base_url` must be a valid URL format
- `health_check_interval_seconds` must be > 0
- `timeout_ms` must be > 0

---

### 2. api_routes

Defines routing rules for the gateway.

```sql
CREATE TABLE api_routes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    path_pattern VARCHAR(512) NOT NULL,
    method VARCHAR(10) NOT NULL CHECK (method IN ('GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'HEAD', 'OPTIONS')),
    backend_service_id UUID NOT NULL REFERENCES backend_services(id) ON DELETE CASCADE,
    strip_path_prefix BOOLEAN NOT NULL DEFAULT false,
    preserve_host_header BOOLEAN NOT NULL DEFAULT false,
    timeout_ms INTEGER,
    is_active BOOLEAN NOT NULL DEFAULT true,
    priority INTEGER NOT NULL DEFAULT 0,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**Indexes:**
- `idx_api_routes_is_active` ON `is_active`
- `idx_api_routes_path_pattern` ON `path_pattern`
- `idx_api_routes_backend_service_id` ON `backend_service_id`
- `idx_api_routes_priority` ON `priority DESC`

**Constraints:**
- `method` must be a valid HTTP method
- `priority` higher values = higher priority
- Foreign key to `backend_services`

**Metadata JSONB Fields (examples):**
```json
{
  "description": "User management endpoint",
  "tags": ["users", "api-v1"],
  "documentation_url": "https://docs.example.com/users"
}
```

---

### 3. rate_limits

Rate limiting configuration for routes or global.

```sql
CREATE TABLE rate_limits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    api_route_id UUID REFERENCES api_routes(id) ON DELETE CASCADE,
    identifier_type VARCHAR(50) NOT NULL CHECK (identifier_type IN ('ip', 'api_key', 'user_id', 'global')),
    max_requests INTEGER NOT NULL CHECK (max_requests > 0),
    window_seconds INTEGER NOT NULL CHECK (window_seconds > 0),
    burst_size INTEGER CHECK (burst_size > 0),
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**Indexes:**
- `idx_rate_limits_is_active` ON `is_active`
- `idx_rate_limits_api_route_id` ON `api_route_id`
- `idx_rate_limits_identifier_type` ON `identifier_type`

**Constraints:**
- `max_requests` must be > 0
- `window_seconds` must be > 0
- `api_route_id` NULL = global rate limit
- `identifier_type` determines how clients are identified

**Identifier Types:**
- `ip` - Rate limit by client IP address
- `api_key` - Rate limit by API key header
- `user_id` - Rate limit by authenticated user ID
- `global` - Global rate limit across all clients

---

### 4. whitelist_rules

Access control and authentication rules.

```sql
CREATE TABLE whitelist_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_name VARCHAR(255) NOT NULL,
    rule_type VARCHAR(50) NOT NULL CHECK (rule_type IN ('ip', 'api_key', 'jwt', 'custom')),
    api_route_id UUID REFERENCES api_routes(id) ON DELETE CASCADE,
    config JSONB NOT NULL DEFAULT '{}'::jsonb,
    is_active BOOLEAN NOT NULL DEFAULT true,
    priority INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**Indexes:**
- `idx_whitelist_rules_is_active` ON `is_active`
- `idx_whitelist_rules_api_route_id` ON `api_route_id`
- `idx_whitelist_rules_priority` ON `priority DESC`
- `idx_whitelist_rules_rule_type` ON `rule_type`

**Constraints:**
- `priority` higher values = checked first
- `api_route_id` NULL = global rule
- `config` stores rule-specific configuration

**Config JSONB Examples:**

IP Whitelist:
```json
{
  "allowed_ips": ["192.168.1.100", "10.0.0.0/8"],
  "denied_ips": ["192.168.1.50"]
}
```

API Key:
```json
{
  "header_name": "X-API-Key",
  "allowed_keys": ["key1", "key2"]
}
```

JWT:
```json
{
  "issuer": "https://auth.example.com",
  "audience": "api.example.com",
  "public_key": "-----BEGIN PUBLIC KEY-----..."
}
```

---

### 5. load_balancer_config

Load balancing configuration for backend services.

```sql
CREATE TABLE load_balancer_config (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    backend_service_id UUID NOT NULL REFERENCES backend_services(id) ON DELETE CASCADE,
    algorithm VARCHAR(50) NOT NULL CHECK (algorithm IN ('round_robin', 'least_connections', 'ip_hash', 'weighted')),
    health_check_enabled BOOLEAN NOT NULL DEFAULT true,
    health_check_interval_seconds INTEGER DEFAULT 30,
    health_check_timeout_ms INTEGER DEFAULT 5000,
    health_check_unhealthy_threshold INTEGER DEFAULT 3,
    health_check_healthy_threshold INTEGER DEFAULT 2,
    session_affinity_enabled BOOLEAN NOT NULL DEFAULT false,
    session_affinity_cookie_name VARCHAR(100),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**Indexes:**
- `idx_load_balancer_backend_service_id` ON `backend_service_id`

**Constraints:**
- One config per backend service
- `algorithm` must be a valid load balancing algorithm

---

### 6. audit_logs

Security event logging for rate limits, whitelist denials, etc.

```sql
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type VARCHAR(100) NOT NULL,
    event_category VARCHAR(50) NOT NULL CHECK (event_category IN ('authentication', 'rate_limit', 'whitelist', 'admin')),
    severity VARCHAR(20) NOT NULL CHECK (severity IN ('info', 'warning', 'critical')),
    request_method VARCHAR(10),
    request_path VARCHAR(512),
    client_ip VARCHAR(45),
    user_agent TEXT,
    api_route_id UUID REFERENCES api_routes(id) ON DELETE SET NULL,
    backend_service_id UUID REFERENCES backend_services(id) ON DELETE SET NULL,
    message TEXT NOT NULL,
    metadata JSONB DEFAULT '{}'::jsonb,
    status_code INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**Indexes:**
- `idx_audit_logs_created_at` ON `created_at DESC`
- `idx_audit_logs_event_category` ON `event_category`
- `idx_audit_logs_severity` ON `severity`
- `idx_audit_logs_client_ip` ON `client_ip`
- `idx_audit_logs_event_type` ON `event_type`

**Constraints:**
- `event_category` must be a valid category
- `severity` must be info, warning, or critical
- Immutable - no updates allowed

**Event Types:**
- `rate_limit_exceeded` - Client exceeded rate limit
- `whitelist_denied` - Access denied by whitelist
- `authentication_failed` - Auth attempt failed
- `authorization_denied` - Insufficient permissions
- `invalid_request` - Malformed request
- `backend_error` - Backend service error
- `configuration_changed` - Config updated

**Metadata JSONB Examples:**
```json
{
  "rate_limit": {
    "limit": 100,
    "window": 60,
    "current_count": 101
  },
  "rule_id": "uuid-here",
  "reason": "IP not in whitelist"
}
```

---

### 7. config_audit_log

Audit trail for all configuration changes.

```sql
CREATE TABLE config_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    table_name VARCHAR(100) NOT NULL,
    record_id UUID NOT NULL,
    operation VARCHAR(20) NOT NULL CHECK (operation IN ('INSERT', 'UPDATE', 'DELETE')),
    old_data JSONB,
    new_data JSONB,
    changed_by VARCHAR(255),
    changed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**Indexes:**
- `idx_config_audit_log_changed_at` ON `changed_at DESC`
- `idx_config_audit_log_table_name` ON `table_name`
- `idx_config_audit_log_record_id` ON `record_id`

**Constraints:**
- Immutable - no updates allowed
- `old_data` NULL for INSERT
- `new_data` NULL for DELETE

---

### 8. config_versions

Point-in-time snapshots of complete configuration.

```sql
CREATE TABLE config_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    version_name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    snapshot_data JSONB NOT NULL,
    created_by VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**Indexes:**
- `idx_config_versions_version_name` ON `version_name`
- `idx_config_versions_created_at` ON `created_at DESC`

**Constraints:**
- `version_name` must be unique
- Immutable - no updates allowed

**Snapshot Data Format:**
```json
{
  "backend_services": [...],
  "api_routes": [...],
  "rate_limits": [...],
  "whitelist_rules": [...]
}
```

---

### 9. gateway_metrics

Optional metrics storage for analytics.

```sql
CREATE TABLE gateway_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    metric_type VARCHAR(50) NOT NULL,
    api_route_id UUID REFERENCES api_routes(id) ON DELETE CASCADE,
    backend_service_id UUID REFERENCES backend_services(id) ON DELETE CASCADE,
    value NUMERIC NOT NULL,
    labels JSONB DEFAULT '{}'::jsonb,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**Indexes:**
- `idx_gateway_metrics_recorded_at` ON `recorded_at DESC`
- `idx_gateway_metrics_metric_type` ON `metric_type`
- `idx_gateway_metrics_api_route_id` ON `api_route_id`

**Metric Types:**
- `request_count` - Number of requests
- `request_duration_ms` - Request latency
- `error_count` - Number of errors
- `bytes_sent` - Bytes sent to client
- `bytes_received` - Bytes received from client

---

## Triggers

### 1. Update Timestamp Trigger

Automatically updates `updated_at` timestamp on record changes.

```sql
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Applied to all tables with updated_at column
CREATE TRIGGER update_backend_services_updated_at
    BEFORE UPDATE ON backend_services
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Similar triggers for: api_routes, rate_limits, whitelist_rules, load_balancer_config
```

### 2. Config Audit Trigger

Logs all configuration changes to `config_audit_log`.

```sql
CREATE OR REPLACE FUNCTION audit_config_changes()
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

-- Applied to config tables
CREATE TRIGGER audit_backend_services
    AFTER INSERT OR UPDATE OR DELETE ON backend_services
    FOR EACH ROW EXECUTE FUNCTION audit_config_changes();

-- Similar triggers for: api_routes, rate_limits, whitelist_rules
```

### 3. Config Update Notification

Sends PostgreSQL NOTIFY when configuration changes.

```sql
CREATE OR REPLACE FUNCTION notify_config_update()
RETURNS TRIGGER AS $$
BEGIN
    PERFORM pg_notify('config_update', TG_TABLE_NAME);
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Applied to all config tables
CREATE TRIGGER notify_backend_services_update
    AFTER INSERT OR UPDATE OR DELETE ON backend_services
    FOR EACH STATEMENT EXECUTE FUNCTION notify_config_update();

-- Similar triggers for: api_routes, rate_limits, whitelist_rules
```

---

## Functions

### 1. Create Config Snapshot

Creates a point-in-time snapshot of all configuration.

```sql
CREATE OR REPLACE FUNCTION create_config_snapshot(
    p_version_name VARCHAR,
    p_description TEXT,
    p_created_by VARCHAR
)
RETURNS UUID AS $$
DECLARE
    v_snapshot_id UUID;
    v_snapshot_data JSONB;
BEGIN
    -- Build snapshot data
    SELECT jsonb_build_object(
        'backend_services', (SELECT jsonb_agg(row_to_json(t.*)) FROM backend_services t),
        'api_routes', (SELECT jsonb_agg(row_to_json(t.*)) FROM api_routes t),
        'rate_limits', (SELECT jsonb_agg(row_to_json(t.*)) FROM rate_limits t),
        'whitelist_rules', (SELECT jsonb_agg(row_to_json(t.*)) FROM whitelist_rules t)
    ) INTO v_snapshot_data;

    -- Insert snapshot
    INSERT INTO config_versions (version_name, description, snapshot_data, created_by)
    VALUES (p_version_name, p_description, v_snapshot_data, p_created_by)
    RETURNING id INTO v_snapshot_id;

    RETURN v_snapshot_id;
END;
$$ LANGUAGE plpgsql;

-- Usage:
-- SELECT create_config_snapshot('v1.0.0', 'Initial release', 'admin');
```

---

## Views

### 1. Active Routes View

Shows all active routes with their backend service info.

```sql
CREATE VIEW v_active_routes AS
SELECT
    r.id,
    r.path_pattern,
    r.method,
    r.priority,
    s.id AS backend_service_id,
    s.name AS backend_service_name,
    s.base_url,
    s.is_active AS service_is_active
FROM api_routes r
JOIN backend_services s ON r.backend_service_id = s.id
WHERE r.is_active = true
ORDER BY r.priority DESC, r.created_at ASC;
```

### 2. Route Rate Limits View

Shows rate limits applied to each route.

```sql
CREATE VIEW v_route_rate_limits AS
SELECT
    r.id AS route_id,
    r.path_pattern,
    r.method,
    rl.id AS rate_limit_id,
    rl.name AS rate_limit_name,
    rl.identifier_type,
    rl.max_requests,
    rl.window_seconds,
    rl.is_active
FROM api_routes r
LEFT JOIN rate_limits rl ON r.id = rl.api_route_id
WHERE r.is_active = true;
```

---

## Indexes Strategy

### Performance Indexes

- All foreign keys are indexed
- `is_active` columns are indexed for quick filtering
- `created_at` and `updated_at` indexed for time-based queries
- `priority` columns indexed DESC for sorting

### JSONB Indexes (if needed for large datasets)

```sql
-- Index on metadata fields
CREATE INDEX idx_api_routes_metadata_gin ON api_routes USING GIN (metadata);
CREATE INDEX idx_whitelist_rules_config_gin ON whitelist_rules USING GIN (config);

-- Specific JSONB path indexes
CREATE INDEX idx_api_routes_metadata_tags ON api_routes USING GIN ((metadata->'tags'));
```

---

## Partitioning (for high-volume tables)

For tables with high write volume like `audit_logs` and `gateway_metrics`, consider partitioning by date:

```sql
-- Example: Partition audit_logs by month
CREATE TABLE audit_logs_2025_11 PARTITION OF audit_logs
    FOR VALUES FROM ('2025-11-01') TO ('2025-12-01');

CREATE TABLE audit_logs_2025_12 PARTITION OF audit_logs
    FOR VALUES FROM ('2025-12-01') TO ('2026-01-01');
```

---

## Data Retention

### Audit Logs Retention (90 days)

```sql
-- Delete audit logs older than 90 days
DELETE FROM audit_logs WHERE created_at < NOW() - INTERVAL '90 days';
```

### Metrics Retention (30 days)

```sql
-- Delete metrics older than 30 days
DELETE FROM gateway_metrics WHERE recorded_at < NOW() - INTERVAL '30 days';
```

---

## Backup Strategy

1. **Full Backup**: Daily at 2 AM
2. **WAL Archiving**: Continuous
3. **Retention**: 7 days for full backups, 30 days for WAL

```bash
# Daily backup script
pg_dump -U karateway karateway | gzip > /backups/karateway_$(date +%Y%m%d).sql.gz
```

---

## Migration Files

All schema changes are managed through SQLx migrations in the `migrations/` directory:

- `20251116075450_backend_services.up.sql`
- `20251116075452_api_routes.up.sql`
- `20251116075454_whitelist_rules.up.sql`
- `20251116075456_rate_limits.up.sql`
- `20251116075458_load_balancer_config.up.sql`
- `20251116075501_config_audit_log.up.sql`
- `20251116075503_config_versions.up.sql`
- `20251116075505_gateway_metrics.up.sql`
- `20251116075507_indexes.up.sql`
- `20251116075509_update_timestamp_triggers.up.sql`
- `20251116075511_audit_triggers.up.sql`
- `20251116075513_config_snapshot_functions.up.sql`
- `20251116075515_audit_logs.up.sql`

Apply migrations with:
```bash
sqlx migrate run
```

Revert with:
```bash
sqlx migrate revert
```

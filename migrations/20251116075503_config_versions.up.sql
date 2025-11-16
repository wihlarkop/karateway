-- Configuration Versions for Point-in-Time Snapshots
CREATE TABLE config_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    version_name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    config_snapshot JSONB NOT NULL,
    created_by VARCHAR(100),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

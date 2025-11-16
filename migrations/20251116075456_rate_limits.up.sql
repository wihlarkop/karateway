-- Rate Limiting Configuration
CREATE TABLE rate_limits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE,
    api_route_id UUID REFERENCES api_routes(id) ON DELETE CASCADE,
    max_requests INT NOT NULL,
    window_seconds INT NOT NULL,
    identifier_type VARCHAR(50) NOT NULL CHECK (identifier_type IN ('ip', 'api_key', 'user_id', 'global')),
    is_active BOOLEAN DEFAULT true,
    burst_size INT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

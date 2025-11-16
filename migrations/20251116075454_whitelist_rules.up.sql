-- Whitelist Rules for Authentication/Authorization
CREATE TABLE whitelist_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_name VARCHAR(100) NOT NULL UNIQUE,
    rule_type VARCHAR(50) NOT NULL CHECK (rule_type IN ('ip', 'api_key', 'jwt', 'custom')),
    api_route_id UUID REFERENCES api_routes(id) ON DELETE CASCADE,
    config JSONB NOT NULL,
    is_active BOOLEAN DEFAULT true,
    priority INT DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

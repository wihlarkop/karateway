-- API Routes Configuration
CREATE TABLE api_routes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    path_pattern VARCHAR(500) NOT NULL,
    method VARCHAR(10) NOT NULL CHECK (method IN ('GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'HEAD', 'OPTIONS')),
    backend_service_id UUID NOT NULL REFERENCES backend_services(id) ON DELETE CASCADE,
    strip_path_prefix BOOLEAN DEFAULT false,
    preserve_host_header BOOLEAN DEFAULT false,
    timeout_ms INT DEFAULT 30000,
    is_active BOOLEAN DEFAULT true,
    priority INT DEFAULT 0,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(path_pattern, method)
);

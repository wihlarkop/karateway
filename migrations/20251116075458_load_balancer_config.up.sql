-- Load Balancer Configuration
CREATE TABLE load_balancer_config (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    backend_service_id UUID NOT NULL REFERENCES backend_services(id) ON DELETE CASCADE,
    algorithm VARCHAR(50) NOT NULL CHECK (algorithm IN ('round_robin', 'least_conn', 'ip_hash', 'weighted')),
    health_check_enabled BOOLEAN DEFAULT true,
    config JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(backend_service_id)
);

-- Backend Services Registry
CREATE TABLE backend_services (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    base_url VARCHAR(500) NOT NULL,
    health_check_url VARCHAR(500),
    health_check_interval_seconds INT DEFAULT 30,
    timeout_ms INT DEFAULT 5000,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

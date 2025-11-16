-- Gateway Metrics (Optional - for storing metrics in DB)
CREATE TABLE gateway_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    timestamp TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    route_id UUID REFERENCES api_routes(id) ON DELETE SET NULL,
    method VARCHAR(10),
    path VARCHAR(500),
    status_code INT,
    response_time_ms FLOAT,
    backend_service_id UUID REFERENCES backend_services(id) ON DELETE SET NULL,
    error_message TEXT,
    metadata JSONB DEFAULT '{}'
);

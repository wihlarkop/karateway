# Karateway API Documentation

Complete API reference for the Karateway Admin API.

## Base URL

```
http://localhost:8081
```

## API Documentation

- **Swagger UI**: http://localhost:8081/swagger-ui
- **OpenAPI Spec**: http://localhost:8081/api-docs/openapi.json

---

## Backend Services

Manage backend services that the gateway proxies to.

### List Backend Services

```http
GET /api/services?page=1&limit=10&search=my-service
```

**Query Parameters:**
- `page` (optional): Page number (default: 1)
- `limit` (optional): Items per page (default: 10)
- `search` (optional): Search by service name

**Response:**
```json
{
  "data": [
    {
      "id": "123e4567-e89b-12d3-a456-426614174000",
      "name": "my-backend",
      "description": "My backend service",
      "base_url": "http://backend.example.com",
      "health_check_url": "http://backend.example.com/health",
      "health_check_interval_seconds": 30,
      "timeout_ms": 5000,
      "is_active": true,
      "created_at": "2025-11-16T10:00:00Z",
      "updated_at": "2025-11-16T10:00:00Z"
    }
  ],
  "success": true,
  "meta": {
    "page": 1,
    "limit": 10,
    "total_data": 1,
    "total_pages": 1
  },
  "status_code": 200,
  "timestamp": "2025-11-16T10:00:00Z"
}
```

### Get Single Backend Service

```http
GET /api/services/{id}
```

**Response:**
```json
{
  "data": {
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "name": "my-backend",
    "description": "My backend service",
    "base_url": "http://backend.example.com",
    "health_check_url": "http://backend.example.com/health",
    "health_check_interval_seconds": 30,
    "timeout_ms": 5000,
    "is_active": true,
    "created_at": "2025-11-16T10:00:00Z",
    "updated_at": "2025-11-16T10:00:00Z"
  },
  "success": true,
  "status_code": 200,
  "timestamp": "2025-11-16T10:00:00Z"
}
```

### Create Backend Service

```http
POST /api/services
Content-Type: application/json

{
  "name": "my-backend",
  "description": "My backend service",
  "base_url": "http://backend.example.com",
  "health_check_url": "http://backend.example.com/health",
  "health_check_interval_seconds": 30,
  "timeout_ms": 5000
}
```

**Response:**
```json
{
  "data": {
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "name": "my-backend",
    "description": "My backend service",
    "base_url": "http://backend.example.com",
    "health_check_url": "http://backend.example.com/health",
    "health_check_interval_seconds": 30,
    "timeout_ms": 5000,
    "is_active": true,
    "created_at": "2025-11-16T10:00:00Z",
    "updated_at": "2025-11-16T10:00:00Z"
  },
  "success": true,
  "message": "Backend service created successfully",
  "status_code": 201,
  "timestamp": "2025-11-16T10:00:00Z"
}
```

### Update Backend Service

```http
PUT /api/services/{id}
Content-Type: application/json

{
  "name": "updated-backend",
  "is_active": false
}
```

### Delete Backend Service

```http
DELETE /api/services/{id}
```

**Response:**
```json
{
  "success": true,
  "message": "Operation completed successfully",
  "status_code": 204,
  "timestamp": "2025-11-16T10:00:00Z"
}
```

### Get Service with Routes

```http
GET /api/services/{id}/routes
```

**Response:**
```json
{
  "data": {
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "name": "my-backend",
    "base_url": "http://backend.example.com",
    "routes": [
      {
        "id": "456e4567-e89b-12d3-a456-426614174111",
        "path_pattern": "/api/v1/users",
        "method": "GET",
        "backend_service_id": "123e4567-e89b-12d3-a456-426614174000",
        "is_active": true,
        "priority": 100
      }
    ]
  },
  "success": true,
  "status_code": 200,
  "timestamp": "2025-11-16T10:00:00Z"
}
```

---

## API Routes

Configure routing rules for the gateway.

### List API Routes

```http
GET /api/routes?page=1&limit=10&search=/api
```

**Response:**
```json
{
  "data": [
    {
      "id": "456e4567-e89b-12d3-a456-426614174111",
      "path_pattern": "/api/v1/users",
      "method": "GET",
      "backend_service_id": "123e4567-e89b-12d3-a456-426614174000",
      "strip_path_prefix": false,
      "preserve_host_header": false,
      "timeout_ms": 5000,
      "is_active": true,
      "priority": 100,
      "metadata": {},
      "created_at": "2025-11-16T10:00:00Z",
      "updated_at": "2025-11-16T10:00:00Z"
    }
  ],
  "success": true,
  "meta": {
    "page": 1,
    "limit": 10,
    "total_data": 1,
    "total_pages": 1
  },
  "status_code": 200,
  "timestamp": "2025-11-16T10:00:00Z"
}
```

### Create API Route

```http
POST /api/routes
Content-Type: application/json

{
  "path_pattern": "/api/v1/users",
  "method": "GET",
  "backend_service_id": "123e4567-e89b-12d3-a456-426614174000",
  "strip_path_prefix": false,
  "preserve_host_header": false,
  "timeout_ms": 5000,
  "priority": 100,
  "metadata": {
    "description": "User management endpoint"
  }
}
```

### Update API Route

```http
PUT /api/routes/{id}
Content-Type: application/json

{
  "is_active": false,
  "priority": 200
}
```

### Delete API Route

```http
DELETE /api/routes/{id}
```

---

## Rate Limits

Configure rate limiting policies.

### List Rate Limits

```http
GET /api/rate-limits?page=1&limit=10
```

**Response:**
```json
{
  "data": [
    {
      "id": "789e4567-e89b-12d3-a456-426614174222",
      "name": "Global API limit",
      "api_route_id": null,
      "identifier_type": "Ip",
      "max_requests": 100,
      "window_seconds": 60,
      "burst_size": 10,
      "is_active": true,
      "created_at": "2025-11-16T10:00:00Z",
      "updated_at": "2025-11-16T10:00:00Z"
    }
  ],
  "success": true,
  "status_code": 200,
  "timestamp": "2025-11-16T10:00:00Z"
}
```

### Create Rate Limit

```http
POST /api/rate-limits
Content-Type: application/json

{
  "name": "Per-route limit",
  "api_route_id": "456e4567-e89b-12d3-a456-426614174111",
  "identifier_type": "Ip",
  "max_requests": 50,
  "window_seconds": 60,
  "burst_size": 5
}
```

**Identifier Types:**
- `Ip` - Rate limit by client IP address
- `ApiKey` - Rate limit by API key
- `UserId` - Rate limit by user ID
- `Global` - Global rate limit

### Update Rate Limit

```http
PUT /api/rate-limits/{id}
Content-Type: application/json

{
  "max_requests": 200,
  "window_seconds": 120
}
```

### Delete Rate Limit

```http
DELETE /api/rate-limits/{id}
```

---

## Whitelist Rules

Manage access control rules.

### List Whitelist Rules

```http
GET /api/whitelist?page=1&limit=10
```

**Response:**
```json
{
  "data": [
    {
      "id": "abc12345-e89b-12d3-a456-426614174333",
      "rule_name": "Allow specific IPs",
      "rule_type": "Ip",
      "api_route_id": null,
      "config": {
        "allowed_ips": ["192.168.1.100", "10.0.0.0/8"]
      },
      "is_active": true,
      "priority": 100,
      "created_at": "2025-11-16T10:00:00Z",
      "updated_at": "2025-11-16T10:00:00Z"
    }
  ],
  "success": true,
  "status_code": 200,
  "timestamp": "2025-11-16T10:00:00Z"
}
```

### Create Whitelist Rule

```http
POST /api/whitelist
Content-Type: application/json

{
  "rule_name": "Allow specific IPs",
  "rule_type": "Ip",
  "api_route_id": null,
  "config": {
    "allowed_ips": ["192.168.1.100", "10.0.0.0/8"]
  },
  "priority": 100
}
```

**Rule Types:**
- `Ip` - IP-based whitelist
- `ApiKey` - API key validation
- `Jwt` - JWT token validation
- `Custom` - Custom validation logic

### Update Whitelist Rule

```http
PUT /api/whitelist/{id}
Content-Type: application/json

{
  "is_active": false
}
```

### Delete Whitelist Rule

```http
DELETE /api/whitelist/{id}
```

---

## Audit Logs

View security events and access logs.

### List Audit Logs

```http
GET /api/audit-logs?limit=50&offset=0&severity=critical
```

**Query Parameters:**
- `limit` (optional): Number of records (default: 50, max: 1000)
- `offset` (optional): Offset for pagination (default: 0)
- `event_type` (optional): Filter by event type
- `event_category` (optional): Filter by category (authentication, rate_limit, whitelist, admin)
- `severity` (optional): Filter by severity (info, warning, critical)
- `client_ip` (optional): Filter by client IP

**Response:**
```json
{
  "data": {
    "logs": [
      {
        "id": "def45678-e89b-12d3-a456-426614174444",
        "event_type": "rate_limit_exceeded",
        "event_category": "rate_limit",
        "severity": "warning",
        "request_method": "GET",
        "request_path": "/api/v1/users",
        "client_ip": "192.168.1.100",
        "user_agent": "Mozilla/5.0...",
        "api_route_id": "456e4567-e89b-12d3-a456-426614174111",
        "backend_service_id": null,
        "message": "Rate limit exceeded for IP 192.168.1.100",
        "metadata": {
          "limit": 100,
          "window": 60,
          "current_count": 101
        },
        "status_code": 429,
        "created_at": "2025-11-16T10:00:00Z"
      }
    ],
    "total": 1,
    "limit": 50,
    "offset": 0
  },
  "success": true,
  "status_code": 200,
  "timestamp": "2025-11-16T10:00:00Z"
}
```

---

## Service Health

Get real-time health status of backend services.

### Get Services Health

```http
GET /api/services/health?force_refresh=false
```

**Query Parameters:**
- `force_refresh` (optional): Force health check refresh (default: false)

**Response:**
```json
{
  "data": {
    "services": [
      {
        "id": "123e4567-e89b-12d3-a456-426614174000",
        "name": "my-backend",
        "base_url": "http://backend.example.com",
        "health_check_url": "http://backend.example.com/health",
        "is_healthy": true,
        "status_message": "200 OK"
      }
    ],
    "last_checked": "2025-11-16T10:00:00Z"
  },
  "success": true,
  "status_code": 200,
  "timestamp": "2025-11-16T10:00:00Z"
}
```

---

## Health Check

### Get API Health

```http
GET /health
```

**Response:**
```json
{
  "status": "healthy",
  "version": "1.0.0"
}
```

---

## Error Responses

All error responses follow this format:

```json
{
  "data": null,
  "message": "Error description",
  "success": false,
  "status_code": 400,
  "timestamp": "2025-11-16T10:00:00Z",
  "error_code": "BAD_REQUEST"
}
```

**Common Error Codes:**
- `400` - Bad Request
- `401` - Unauthorized
- `403` - Forbidden
- `404` - Not Found
- `409` - Conflict (duplicate resource)
- `500` - Internal Server Error

---

## Rate Limiting

The Admin API itself is not rate-limited, but you can configure rate limits for gateway routes.

---

## Authentication

Currently, the Admin API does not require authentication. In production, you should:
1. Put the Admin API behind a firewall
2. Use a reverse proxy with authentication (nginx, Caddy)
3. Enable VPN access only

---

## Examples

### Complete Workflow Example

```bash
# 1. Create a backend service
curl -X POST http://localhost:8081/api/services \
  -H "Content-Type: application/json" \
  -d '{
    "name": "httpbin",
    "base_url": "https://httpbin.org",
    "health_check_url": "https://httpbin.org/get"
  }'

# Response: Get the service ID from the response
# service_id="123e4567-e89b-12d3-a456-426614174000"

# 2. Create an API route
curl -X POST http://localhost:8081/api/routes \
  -H "Content-Type: application/json" \
  -d '{
    "path_pattern": "/httpbin",
    "method": "GET",
    "backend_service_id": "123e4567-e89b-12d3-a456-426614174000",
    "priority": 100
  }'

# Response: Get the route ID
# route_id="456e4567-e89b-12d3-a456-426614174111"

# 3. Add a rate limit
curl -X POST http://localhost:8081/api/rate-limits \
  -H "Content-Type: application/json" \
  -d '{
    "name": "httpbin rate limit",
    "api_route_id": "456e4567-e89b-12d3-a456-426614174111",
    "identifier_type": "Ip",
    "max_requests": 10,
    "window_seconds": 60
  }'

# 4. Test the gateway
curl http://localhost:8080/httpbin

# 5. Check audit logs
curl http://localhost:8081/api/audit-logs?limit=10
```

---

## WebSocket Support

Currently not supported. All endpoints are REST/HTTP only.

---

## Versioning

The API is currently at version 1.0.0. Breaking changes will increment the major version.

# Karateway - Dynamic API Gateway

> **Version:** 1.0.0
> **Status:** Production Ready

A high-performance, dynamic API Gateway built with Cloudflare's Pingora framework, featuring zero-downtime configuration updates, comprehensive security audit logging, and database-driven configuration management.

## Project Structure

```
karateway/
├── crates/
│   ├── gateway/           # Main Pingora gateway binary
│   ├── admin-api/         # Admin API (Axum) for management
│   ├── karateway-core/    # Core domain models and types
│   ├── config/            # Configuration management
│   └── metrics/           # Metrics collection
├── dashboard/             # Svelte admin dashboard
├── migrations/            # Database migrations (sqlx)
├── docker-compose.yml     # PostgreSQL & Redis setup
└── README.md
```

## Tech Stack

### Backend
- **Pingora** - High-performance proxy framework
- **Axum** - Web framework for Admin API
- **SQLx** - Type-safe SQL with compile-time checking
- **PostgreSQL** - Configuration storage
- **Redis** - Caching and rate limiting
- **Tokio** - Async runtime

### Frontend
- **Svelte 5** - Reactive UI framework
- **Vite** - Build tool
- **TailwindCSS v4** - Styling
- **shadcn-svelte** - UI components
- **TypeScript** - Type safety

## Prerequisites

- Rust 1.75+ (with cargo)
- Node.js 20+
- pnpm 10+
- Docker & Docker Compose
- PostgreSQL 17+ (via Docker)
- Redis 7+ (via Docker)
- sqlx-cli (install: `cargo install sqlx-cli --no-default-features --features postgres`)

## Quick Start

### Automated Setup (Recommended)

```bash
# Run the setup script
chmod +x setup.sh
./setup.sh
```

This will:
- ✅ Install sqlx-cli if not present
- ✅ Start Docker containers (PostgreSQL + Redis)
- ✅ Run database migrations
- ✅ Install frontend dependencies

### Manual Setup

#### 1. Clone and Setup Environment

```bash
# Copy environment file
cp .env.example .env

# Edit .env with your configuration if needed
# Default values work for local development
```

#### 2. Start Database Services

```bash
# Start PostgreSQL and Redis
docker-compose up -d

# Check services are healthy (both should show "Up")
docker-compose ps
```

#### 3. Run Database Migrations

```bash
# Install sqlx-cli (if not already installed)
cargo install sqlx-cli --no-default-features --features postgres

# Create database and run migrations
sqlx database create
sqlx migrate run
```

#### 4. Run Admin API

```bash
# Development mode (with auto-reload on code changes)
cargo watch -x 'run --bin admin-api'

# Or standard run
cargo run --bin admin-api

# Or release mode (faster, for production)
cargo run --bin admin-api --release
```

The Admin API will be available at:
- **API**: `http://localhost:8081`
- **Swagger UI**: `http://localhost:8081/swagger-ui`
- **OpenAPI JSON**: `http://localhost:8081/api-docs/openapi.json`

#### 5. (Optional) Setup Frontend Dashboard

```bash
cd dashboard

# Install dependencies
pnpm install

# Start dev server
pnpm dev
```

The dashboard will be available at `http://localhost:5173`

#### 6. Run Gateway

```bash
# Development mode
cargo run --bin gateway

# Or with debug logging
RUST_LOG=debug cargo run --bin gateway

# Production mode (recommended)
cargo run --bin gateway --release
```

The gateway will be available at:
- **HTTP**: `http://localhost:8080`
- **HTTPS**: `https://localhost:8443` (if TLS certificates are configured)

To generate self-signed certificates for local development:
```bash
mkdir -p certs
openssl req -x509 -newkey rsa:4096 -keyout certs/key.pem -out certs/cert.pem -days 365 -nodes -subj "/CN=localhost"
```

## Running the Project

### Development Mode

```bash
# Terminal 1: Start databases
docker-compose up

# Terminal 2: Run Admin API with auto-reload
cargo watch -x 'run --bin admin-api'

# Terminal 3: Run Gateway
RUST_LOG=debug cargo run --bin gateway

# Terminal 4 (optional): Run dashboard
cd dashboard && pnpm dev
```

### Production Mode

```bash
# Start databases in background
docker-compose up -d

# Build release binaries
cargo build --release

# Run Admin API
./target/release/admin-api &

# Run Gateway
./target/release/gateway &

# Build and serve dashboard
cd dashboard
pnpm build
pnpm preview
```

### Testing the API

Once the Admin API is running, you can:

1. **Visit Swagger UI**: http://localhost:8081/swagger-ui
2. **Create a backend service**:
   ```bash
   curl -X POST http://localhost:8081/api/services \
     -H "Content-Type: application/json" \
     -d '{
       "name": "my-service",
       "base_url": "http://example.com",
       "description": "Test service"
     }'
   ```
3. **List services**:
   ```bash
   curl http://localhost:8081/api/services
   ```

### Stopping Services

```bash
# Stop databases
docker-compose down

# Or keep data volumes
docker-compose stop

# Remove everything including volumes
docker-compose down -v
```

## Database Schema

The database schema includes:
- **backend_services** - Service registry
- **api_routes** - Route configuration
- **whitelist_rules** - Authentication/authorization
- **rate_limits** - Rate limiting configs
- **load_balancer_config** - Load balancing
- **audit_logs** - Security event logging (rate limits, whitelist denials, etc.)
- **config_audit_log** - Configuration change audit trail
- **config_versions** - Point-in-time snapshots
- **gateway_metrics** - Optional metrics storage

See `migrations/` for complete schema details.

## Configuration Management

### Creating a Config Snapshot

```sql
SELECT create_config_snapshot(
    'v1.0.0',           -- version name
    'Initial release',  -- description
    'admin'            -- created by
);
```

### Real-time Config Updates

The gateway listens to PostgreSQL `NOTIFY` events for zero-downtime config reloads:

```sql
-- Any change to config tables triggers pg_notify('config_update', ...)
INSERT INTO api_routes (path_pattern, method, backend_service_id)
VALUES ('/api/v1/users', 'GET', 'uuid-here');
-- Gateway automatically reloads!
```

## Security Audit Logging

Karateway includes comprehensive security audit logging for all gateway events:

### Features

- **Non-blocking Logging**: Audit logs are written asynchronously using background workers
- **Event Types**: Rate limit exceeded, whitelist denials, authentication failures, etc.
- **Severity Levels**: Info, Warning, Critical
- **Rich Context**: Request method, path, client IP, user agent, status codes
- **API Access**: Query audit logs via Admin API with filtering and pagination
- **Dashboard Viewer**: Real-time audit log viewing with auto-refresh in the admin dashboard

### Event Categories

- **Authentication**: Login attempts, token validation
- **Rate Limit**: Rate limit violations and throttling events
- **Whitelist**: Access control denials
- **Admin**: Configuration changes and administrative actions

### Viewing Audit Logs

**Via Admin API:**
```bash
# Get recent audit logs
curl http://localhost:8081/api/audit-logs?limit=50&offset=0

# Filter by severity
curl http://localhost:8081/api/audit-logs?severity=critical

# Filter by event category
curl http://localhost:8081/api/audit-logs?event_category=rate_limit
```

**Via Dashboard:**
Navigate to `http://localhost:5173/audit-logs` to view logs with:
- Pagination controls
- Auto-refresh toggle (10-second intervals)
- Color-coded severity badges
- Full request context display

## Development

### Adding a New Migration

```bash
sqlx migrate add <migration_name>
# Edit the generated file in migrations/
sqlx migrate run
```

### Frontend Development

```bash
cd dashboard
pnpm dev        # Start dev server
pnpm build      # Build for production
pnpm preview    # Preview production build
```

### Backend Development

```bash
# Run with hot reload (requires cargo-watch)
cargo watch -x 'run --bin gateway'

# Run tests
cargo test

# Format code
cargo fmt

# Lint
cargo clippy
```

## Admin Dashboard Features

The Svelte-based admin dashboard provides a modern UI for managing the gateway:

### Available Pages

- **Dashboard** (`/`) - Overview and statistics
- **Services** (`/services`) - Manage backend services
- **Routes** (`/routes`) - Configure API routes
- **Rate Limits** (`/rate-limits`) - Set up rate limiting policies
- **Whitelist** (`/whitelist`) - Manage access control rules
- **Audit Logs** (`/audit-logs`) - View security events with real-time updates

### Features

- **Real-time Data**: Live updates from the Admin API
- **Responsive Design**: Works on desktop and mobile
- **Dark Mode Support**: Automatic theme detection
- **Data Tables**: Sortable, searchable tables with pagination
- **Form Validation**: Client-side validation for all forms
- **Toast Notifications**: User-friendly success/error messages

## Documentation

- **[API Documentation](docs/API.md)** - Complete API reference with examples
- **[Deployment Guide](docs/DEPLOYMENT.md)** - Production deployment instructions
- **[Database Schema](Karateway-Database-Schema.md)** - Database design and models
- **[RFC](RFC-Karateway-v2.md)** - Architecture and design decisions

## API Endpoints

See **[docs/API.md](docs/API.md)** for complete API documentation with examples.

### Admin API (Port 8081)

- **Backend Services**: Create, list, update, delete backend services
- **API Routes**: Configure routing rules and path patterns
- **Rate Limits**: Set up rate limiting policies
- **Whitelist Rules**: Manage access control rules
- **Audit Logs**: View security events and access logs with filtering
- **Health Check**: `/health` endpoint for monitoring
- **Service Health**: Real-time health status of backend services

### Gateway (Port 8080 HTTP / 8443 HTTPS)

- **Dynamic Routing**: All configured routes are dynamically proxied based on database configuration
- **Rate Limiting**: Per-route and global rate limiting with Redis
- **Whitelist Validation**: IP and API key-based access control
- **Health Checking**: Automatic backend service health monitoring
- **Audit Logging**: Non-blocking security event logging to database
- **Zero-Downtime Reload**: Configuration updates without restarts

## Performance Targets

- **Throughput**: >100,000 RPS
- **Latency**: <25ms P99
- **Config Reload**: <1 second
- **Memory**: <500MB under load

## License

MIT

## Contributing

See RFC-Karateway-v2.md for architecture details and implementation plan.

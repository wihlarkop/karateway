# Karateway - Dynamic API Gateway

> **Version:** 1.0.0
> **Status:** Production Ready (Admin API + Dashboard) | Gateway In Development

A high-performance, dynamic API Gateway built with Cloudflare's Pingora framework, featuring zero-downtime configuration updates through database-driven configuration management.

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

#### 6. (Future) Run Gateway

```bash
# Gateway is not yet implemented
# Will be available in Phase 2
cargo run --bin gateway --release
```

## Running the Project

### Development Mode

```bash
# Terminal 1: Start databases
docker-compose up

# Terminal 2: Run Admin API with auto-reload
cargo watch -x 'run --bin admin-api'

# Terminal 3 (optional): Run dashboard
cd dashboard && pnpm dev
```

### Production Mode

```bash
# Start databases in background
docker-compose up -d

# Build and run Admin API
cargo build --release
./target/release/admin-api

# Build and serve dashboard (when ready)
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
- **config_audit_log** - Complete audit trail
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
- **Health Check**: `/health` endpoint for monitoring

### Gateway (Port 8080)

- All configured routes are dynamically proxied (coming in Phase 2)

## Performance Targets

- **Throughput**: >100,000 RPS
- **Latency**: <25ms P99
- **Config Reload**: <1 second
- **Memory**: <500MB under load

## License

MIT

## Contributing

See RFC-Karateway-v2.md for architecture details and implementation plan.

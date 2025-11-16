# Karateway Deployment Guide

Complete guide for deploying Karateway to production environments.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Environment Setup](#environment-setup)
- [Database Setup](#database-setup)
- [Building](#building)
- [Deployment Options](#deployment-options)
  - [Docker Deployment](#docker-deployment)
  - [Systemd Services](#systemd-services)
  - [Kubernetes](#kubernetes)
- [TLS/SSL Configuration](#tlsssl-configuration)
- [Monitoring](#monitoring)
- [Security Hardening](#security-hardening)
- [Backup and Recovery](#backup-and-recovery)

---

## Prerequisites

### System Requirements

**Minimum:**
- CPU: 2 cores
- RAM: 2GB
- Disk: 20GB SSD
- OS: Linux (Ubuntu 22.04+, RHEL 8+, or similar)

**Recommended (Production):**
- CPU: 4+ cores
- RAM: 8GB+
- Disk: 50GB+ SSD
- OS: Linux with latest updates

### Software Requirements

- **Rust**: 1.75+ (for building)
- **PostgreSQL**: 17+
- **Redis**: 7+
- **Node.js**: 20+ (for dashboard)
- **pnpm**: 10+ (for dashboard)
- **Docker** (optional, recommended)

---

## Environment Setup

### 1. Create Production Environment File

```bash
cp .env.example .env.production
```

### 2. Configure Environment Variables

```bash
# .env.production

# Database
DATABASE_URL=postgresql://karateway:STRONG_PASSWORD@db.internal:5432/karateway

# Redis
REDIS_URL=redis://:REDIS_PASSWORD@redis.internal:6379/0

# Admin API
ADMIN_API_HOST=0.0.0.0
ADMIN_API_PORT=8081

# Gateway
GATEWAY_HTTP_PORT=8080
GATEWAY_HTTPS_PORT=8443
GATEWAY_TLS_CERT=/etc/karateway/certs/cert.pem
GATEWAY_TLS_KEY=/etc/karateway/certs/key.pem

# Logging
RUST_LOG=info,karateway_gateway=info,karateway_admin_api=info
```

### 3. Set Secure Permissions

```bash
chmod 600 .env.production
chown karateway:karateway .env.production
```

---

## Database Setup

### 1. Install PostgreSQL

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install postgresql-17 postgresql-contrib

# RHEL/CentOS
sudo dnf install postgresql17-server postgresql17-contrib
sudo postgresql-17-setup initdb
sudo systemctl enable --now postgresql-17
```

### 2. Create Database and User

```sql
-- Connect as postgres user
sudo -u postgres psql

-- Create user
CREATE USER karateway WITH PASSWORD 'STRONG_PASSWORD';

-- Create database
CREATE DATABASE karateway OWNER karateway;

-- Grant privileges
GRANT ALL PRIVILEGES ON DATABASE karateway TO karateway;

-- Enable extensions
\c karateway
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
```

### 3. Configure PostgreSQL

Edit `/etc/postgresql/17/main/postgresql.conf`:

```ini
# Performance tuning
shared_buffers = 256MB
effective_cache_size = 1GB
maintenance_work_mem = 64MB
checkpoint_completion_target = 0.9
wal_buffers = 16MB
default_statistics_target = 100
random_page_cost = 1.1
effective_io_concurrency = 200
work_mem = 4MB
min_wal_size = 1GB
max_wal_size = 4GB
max_worker_processes = 4
max_parallel_workers_per_gather = 2
max_parallel_workers = 4

# Connection settings
max_connections = 100
listen_addresses = 'localhost'  # Or specific IP
```

Edit `/etc/postgresql/17/main/pg_hba.conf`:

```
# TYPE  DATABASE        USER            ADDRESS                 METHOD
local   all             postgres                                peer
local   all             karateway                               md5
host    all             karateway       127.0.0.1/32            md5
host    all             karateway       ::1/128                 md5
```

Restart PostgreSQL:

```bash
sudo systemctl restart postgresql
```

### 4. Run Migrations

```bash
# Install sqlx-cli
cargo install sqlx-cli --no-default-features --features postgres

# Run migrations
export DATABASE_URL=postgresql://karateway:PASSWORD@localhost:5432/karateway
sqlx migrate run
```

---

## Building

### 1. Build Release Binaries

```bash
# Build all binaries in release mode
cargo build --release

# Binaries will be in target/release/
ls -lh target/release/gateway
ls -lh target/release/admin-api
```

### 2. Strip Debug Symbols (Optional, reduces size)

```bash
strip target/release/gateway
strip target/release/admin-api
```

### 3. Build Dashboard

```bash
cd dashboard
pnpm install
pnpm build

# Built files will be in dashboard/dist/
```

---

## Deployment Options

### Docker Deployment

#### 1. Create Dockerfile

```dockerfile
# Dockerfile
FROM rust:1.75 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM ubuntu:22.04

RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/gateway /app/gateway
COPY --from=builder /app/target/release/admin-api /app/admin-api

RUN useradd -r -s /bin/false karateway && \
    chown -R karateway:karateway /app

USER karateway

CMD ["/app/gateway"]
```

#### 2. Build Docker Image

```bash
docker build -t karateway:1.0.0 .
```

#### 3. Docker Compose (Production)

```yaml
# docker-compose.prod.yml
version: '3.8'

services:
  postgres:
    image: postgres:17-alpine
    environment:
      POSTGRES_USER: karateway
      POSTGRES_PASSWORD: ${DB_PASSWORD}
      POSTGRES_DB: karateway
    volumes:
      - postgres_data:/var/lib/postgresql/data
    networks:
      - karateway_internal
    restart: unless-stopped

  redis:
    image: redis:7-alpine
    command: redis-server --requirepass ${REDIS_PASSWORD}
    volumes:
      - redis_data:/data
    networks:
      - karateway_internal
    restart: unless-stopped

  admin-api:
    image: karateway:1.0.0
    command: /app/admin-api
    environment:
      DATABASE_URL: postgresql://karateway:${DB_PASSWORD}@postgres:5432/karateway
      REDIS_URL: redis://:${REDIS_PASSWORD}@redis:6379/0
    ports:
      - "8081:8081"
    networks:
      - karateway_internal
    depends_on:
      - postgres
      - redis
    restart: unless-stopped

  gateway:
    image: karateway:1.0.0
    command: /app/gateway
    environment:
      DATABASE_URL: postgresql://karateway:${DB_PASSWORD}@postgres:5432/karateway
      REDIS_URL: redis://:${REDIS_PASSWORD}@redis:6379/0
    ports:
      - "8080:8080"
      - "8443:8443"
    volumes:
      - ./certs:/app/certs:ro
    networks:
      - karateway_internal
      - karateway_public
    depends_on:
      - postgres
      - redis
    restart: unless-stopped

volumes:
  postgres_data:
  redis_data:

networks:
  karateway_internal:
    driver: bridge
  karateway_public:
    driver: bridge
```

#### 4. Run with Docker Compose

```bash
# Create .env file with passwords
echo "DB_PASSWORD=your_secure_password" > .env
echo "REDIS_PASSWORD=your_redis_password" >> .env

# Start services
docker-compose -f docker-compose.prod.yml up -d

# Check logs
docker-compose -f docker-compose.prod.yml logs -f gateway
```

---

### Systemd Services

#### 1. Create Service User

```bash
sudo useradd -r -s /bin/false karateway
```

#### 2. Install Binaries

```bash
sudo mkdir -p /opt/karateway/bin
sudo cp target/release/gateway /opt/karateway/bin/
sudo cp target/release/admin-api /opt/karateway/bin/
sudo chown -R karateway:karateway /opt/karateway
sudo chmod +x /opt/karateway/bin/*
```

#### 3. Create Systemd Service Files

**/etc/systemd/system/karateway-gateway.service:**

```ini
[Unit]
Description=Karateway API Gateway
After=network.target postgresql.service redis.service
Wants=postgresql.service redis.service

[Service]
Type=simple
User=karateway
Group=karateway
WorkingDirectory=/opt/karateway
EnvironmentFile=/opt/karateway/.env.production
ExecStart=/opt/karateway/bin/gateway
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal
SyslogIdentifier=karateway-gateway

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/log/karateway

[Install]
WantedBy=multi-user.target
```

**/etc/systemd/system/karateway-admin-api.service:**

```ini
[Unit]
Description=Karateway Admin API
After=network.target postgresql.service redis.service
Wants=postgresql.service redis.service

[Service]
Type=simple
User=karateway
Group=karateway
WorkingDirectory=/opt/karateway
EnvironmentFile=/opt/karateway/.env.production
ExecStart=/opt/karateway/bin/admin-api
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal
SyslogIdentifier=karateway-admin-api

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/log/karateway

[Install]
WantedBy=multi-user.target
```

#### 4. Enable and Start Services

```bash
# Reload systemd
sudo systemctl daemon-reload

# Enable services
sudo systemctl enable karateway-gateway
sudo systemctl enable karateway-admin-api

# Start services
sudo systemctl start karateway-gateway
sudo systemctl start karateway-admin-api

# Check status
sudo systemctl status karateway-gateway
sudo systemctl status karateway-admin-api

# View logs
sudo journalctl -u karateway-gateway -f
sudo journalctl -u karateway-admin-api -f
```

---

### Kubernetes

See `k8s/` directory for Kubernetes manifests (coming soon).

Basic deployment structure:

```yaml
# gateway-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: karateway-gateway
spec:
  replicas: 3
  selector:
    matchLabels:
      app: karateway-gateway
  template:
    metadata:
      labels:
        app: karateway-gateway
    spec:
      containers:
      - name: gateway
        image: karateway:1.0.0
        ports:
        - containerPort: 8080
        - containerPort: 8443
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: karateway-secrets
              key: database-url
        - name: REDIS_URL
          valueFrom:
            secretKeyRef:
              name: karateway-secrets
              key: redis-url
```

---

## TLS/SSL Configuration

### 1. Generate Self-Signed Certificate (Testing)

```bash
mkdir -p /etc/karateway/certs
openssl req -x509 -newkey rsa:4096 \
  -keyout /etc/karateway/certs/key.pem \
  -out /etc/karateway/certs/cert.pem \
  -days 365 -nodes \
  -subj "/CN=gateway.example.com"
```

### 2. Let's Encrypt (Production)

```bash
# Install certbot
sudo apt install certbot

# Get certificate
sudo certbot certonly --standalone \
  -d gateway.example.com \
  --email admin@example.com \
  --agree-tos

# Certificates will be in:
# /etc/letsencrypt/live/gateway.example.com/fullchain.pem
# /etc/letsencrypt/live/gateway.example.com/privkey.pem

# Link to karateway directory
sudo ln -s /etc/letsencrypt/live/gateway.example.com/fullchain.pem /etc/karateway/certs/cert.pem
sudo ln -s /etc/letsencrypt/live/gateway.example.com/privkey.pem /etc/karateway/certs/key.pem
```

### 3. Auto-Renewal

```bash
# Test renewal
sudo certbot renew --dry-run

# Add cron job for auto-renewal
sudo crontab -e

# Add this line:
0 0 * * * certbot renew --quiet && systemctl restart karateway-gateway
```

---

## Monitoring

### 1. Logging

Logs are sent to journald. View with:

```bash
# Real-time logs
sudo journalctl -u karateway-gateway -f
sudo journalctl -u karateway-admin-api -f

# Last 100 lines
sudo journalctl -u karateway-gateway -n 100

# Since yesterday
sudo journalctl -u karateway-gateway --since yesterday
```

### 2. Prometheus Metrics

Karateway exposes Prometheus metrics on `/metrics` endpoint.

**prometheus.yml:**

```yaml
scrape_configs:
  - job_name: 'karateway-gateway'
    static_configs:
      - targets: ['localhost:8080']

  - job_name: 'karateway-admin-api'
    static_configs:
      - targets: ['localhost:8081']
```

### 3. Health Checks

```bash
# Gateway health
curl http://localhost:8080/health

# Admin API health
curl http://localhost:8081/health
```

---

## Security Hardening

### 1. Firewall Configuration

```bash
# UFW (Ubuntu)
sudo ufw allow 8080/tcp  # Gateway HTTP
sudo ufw allow 8443/tcp  # Gateway HTTPS
sudo ufw allow 8081/tcp from 10.0.0.0/8  # Admin API (internal only)
sudo ufw enable
```

### 2. Rate Limiting at OS Level

```bash
# Use fail2ban for additional protection
sudo apt install fail2ban

# Configure fail2ban for Karateway logs
```

### 3. Database Security

- Use strong passwords
- Enable SSL connections
- Restrict network access
- Regular backups
- Regular security updates

### 4. Redis Security

- Use password authentication
- Bind to localhost or internal network only
- Disable dangerous commands
- Enable persistence

---

## Backup and Recovery

### 1. Database Backup

```bash
# Create backup directory
sudo mkdir -p /var/backups/karateway

# Backup script
#!/bin/bash
BACKUP_DIR=/var/backups/karateway
DATE=$(date +%Y%m%d_%H%M%S)

pg_dump -U karateway karateway | gzip > $BACKUP_DIR/karateway_$DATE.sql.gz

# Keep only last 7 days
find $BACKUP_DIR -name "karateway_*.sql.gz" -mtime +7 -delete
```

### 2. Automated Backups

```bash
# Add to crontab
0 2 * * * /opt/karateway/scripts/backup.sh
```

### 3. Restore from Backup

```bash
# Stop services
sudo systemctl stop karateway-gateway karateway-admin-api

# Restore database
gunzip < /var/backups/karateway/karateway_20251116_020000.sql.gz | \
  psql -U karateway karateway

# Start services
sudo systemctl start karateway-gateway karateway-admin-api
```

---

## Scaling

### Horizontal Scaling

1. **Gateway**: Can run multiple instances behind a load balancer
2. **Admin API**: Can run multiple instances (use sticky sessions)
3. **Database**: Use PostgreSQL replication or managed service
4. **Redis**: Use Redis Cluster or managed service

### Load Balancer Configuration (nginx)

```nginx
upstream karateway_gateway {
    least_conn;
    server 10.0.1.10:8080;
    server 10.0.1.11:8080;
    server 10.0.1.12:8080;
}

server {
    listen 80;
    server_name api.example.com;

    location / {
        proxy_pass http://karateway_gateway;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

---

## Troubleshooting

### Gateway Not Starting

```bash
# Check logs
sudo journalctl -u karateway-gateway -n 100

# Check config
sudo -u karateway /opt/karateway/bin/gateway --help

# Test database connection
psql -U karateway -h localhost karateway
```

### High Memory Usage

```bash
# Check memory usage
ps aux | grep gateway

# Monitor in real-time
top -p $(pgrep gateway)

# Check for memory leaks
valgrind --leak-check=full /opt/karateway/bin/gateway
```

### Performance Issues

1. Check database query performance
2. Monitor Redis performance
3. Review rate limiting configuration
4. Check network latency to backends
5. Review gateway logs for errors

---

## Production Checklist

- [ ] Strong passwords for database and Redis
- [ ] TLS certificates configured
- [ ] Firewall rules in place
- [ ] Automated backups configured
- [ ] Monitoring and alerting set up
- [ ] Log rotation configured
- [ ] Security updates automated
- [ ] Rate limiting configured
- [ ] Health checks configured
- [ ] Load balancer configured (if multi-instance)
- [ ] Database properly tuned
- [ ] Redis persistence enabled
- [ ] Admin API access restricted
- [ ] Documentation updated
- [ ] Disaster recovery plan in place

---

## Support

For deployment issues:
1. Check the logs first
2. Review this documentation
3. Open an issue on GitHub
4. Contact support (if applicable)

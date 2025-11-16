#!/bin/bash
# Setup script for Karateway

set -e

echo "🚀 Starting Karateway Setup..."

# Check prerequisites
echo "📋 Checking prerequisites..."

if ! command -v docker &> /dev/null; then
    echo "❌ Docker is not installed. Please install Docker first."
    exit 1
fi

if ! command -v pnpm &> /dev/null; then
    echo "❌ pnpm is not installed. Please install pnpm first."
    exit 1
fi

if ! command -v sqlx &> /dev/null; then
    echo "⚠️  sqlx-cli is not installed. Installing..."
    cargo install sqlx-cli --no-default-features --features postgres
fi

# Setup environment
echo "🔧 Setting up environment..."
if [ ! -f .env ]; then
    cp .env.example .env
    echo "✅ Created .env file from .env.example"
else
    echo "ℹ️  .env file already exists"
fi

# Start Docker services
echo "🐳 Starting Docker services..."
docker-compose up -d

echo "⏳ Waiting for PostgreSQL to be ready..."
sleep 5

# Run migrations
echo "📊 Running database migrations..."
sqlx database create || echo "Database already exists"
sqlx migrate run

echo "✅ Database migrations completed"

# Setup frontend
echo "🎨 Setting up frontend..."
cd dashboard
pnpm install
cd ..

echo ""
echo "✨ Setup complete! ✨"
echo ""
echo "Next steps:"
echo "1. Start frontend: cd dashboard && pnpm dev"
echo "2. Build backend: cargo build"
echo "3. Run gateway: cargo run --bin gateway"
echo "4. Run admin API: cargo run --bin admin-api"
echo ""
echo "Happy coding! 🎉"

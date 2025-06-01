#!/bin/bash
set -euo pipefail

VERBOSE=${1:-false}  # Pass --verbose as first arg
if [[ "$VERBOSE" == "--verbose" ]]; then
    set -x
fi
: ${VERSION:?is required}

echo "🔥 Running cr8s smoke test..."
echo "Testing that the system boots and core functionality works"

# CI-friendly: Set timeout
TIMEOUT=120
export COMPOSE_HTTP_TIMEOUT=$TIMEOUT

# Cleanup first (CI environments may have stale containers)
echo "🧹 Cleaning up any existing containers..."
docker compose down -v --remove-orphans || true

# Ensure we cleanup on exit (important for CI)
trap 'exit_status=$?;
      echo "🧹 Cleanup on exit...";
      docker compose down -v --remove-orphans >/dev/null 2>&1;
      exit $exit_status' EXIT

# Build images (ensure we're testing the current code)
echo "🏗️  Building images..."
docker compose build

# Test: Start the full stack
echo "🚀 Starting full stack..."
docker compose up -d

# Wait for services to be healthy with timeout
echo "⏳ Waiting for services to be ready..."
for i in {1..30}; do
    if curl -sf http://127.0.0.1:8000/health > /dev/null 2>&1; then
        echo "✅ Services ready after ${i}0 seconds"
        break
    fi
    if [ $i -eq 30 ]; then
        echo "❌ Services failed to start within 5 minutes"
        echo "Server logs:"
        docker compose logs server
        exit 1
    fi
    sleep 10
done

# Test: Health check (smoke test essential)
echo "🏥 Testing health endpoint..."
if curl -sf http://127.0.0.1:8000/health; then
    echo "✅ Health check passed"
else
    echo "❌ Health check failed"
    docker compose logs server
    exit 1
fi

# Test: Database connectivity (smoke test essential)
echo "🗄️  Testing database connectivity..."
docker compose exec -T postgres psql -U postgres cr8s -c "SELECT 1;" > /dev/null
echo "✅ Database connectivity passed"

# Test: Redis connectivity (smoke test essential)
echo "🔴 Testing Redis connectivity..."
docker compose exec -T redis redis-cli ping | grep -q PONG
echo "✅ Redis connectivity passed"

# Initialize database schema
echo "📊 Initializing database schema..."
docker compose exec -T postgres psql -U postgres cr8s < scripts/sql/db-init.sql
echo "✅ Database schema initialized"

# Debug schema if verbose mode
if [[ "$VERBOSE" == "--verbose" ]]; then
    echo "🔍 Dumping schema for debugging..."
    docker compose exec -T postgres psql -U postgres cr8s -c "\dt"
    docker compose exec -T postgres psql -U postgres cr8s -c "\d role"
fi

echo "🧪 Loading default test data..."
docker compose exec -T postgres psql -U postgres cr8s < scripts/sql/load-defaults.sql
echo "✅ Default test data loaded"

# Test: Core CLI functionality
echo "🔧 Testing core CLI functionality..."
docker compose run --rm cli list-users

echo "👤 Testing user creation workflow..."
docker compose run --rm cli create-user \
       --username smoketest-$(date +%s) \
       --password testpass123 --roles Viewer

echo "📋 Testing data persistence..."
docker compose run --rm cli list-users | grep smoketest

echo "🎉 Smoke test completed successfully!"
echo "System is ready for deployment"

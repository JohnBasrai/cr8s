#!/bin/bash
set -e

echo "🧹 Cleaning up old containers and volumes..."
docker compose down -v

echo "🐘 Starting Postgres and Redis..."
docker compose up -d postgres redis

echo "⏳ Waiting for Postgres to become available..."
until docker compose exec -T postgres pg_isready -U postgres; do
    sleep 1
done

echo "📦 Running Diesel setup (create DB, run migrations)..."
docker compose run --rm app diesel setup

# Optional but often safe to rerun to ensure migrations are up to date
echo "📈 Re-applying migrations just in case..."
docker compose run --rm app diesel migration run

echo "👤 Seeding database with default admin user..."
docker compose run --rm app cargo run --bin cli -- users create admin@example.com password123 admin

echo "✅ Setup complete."

echo "🧩 Starting backend app container..."
docker compose up -d app

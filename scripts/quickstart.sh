#!/usr/bin/env bash
set -e

docker compose down -v
docker compose up -d postgres redis

# Wait for Postgres, then run migrations
docker compose run --rm app \
  bash -c 'until </dev/tcp/postgres/5432 >/dev/null 2>&1; do sleep 1; done && diesel setup'

# Launch Rocket and run tests
docker compose run --rm --service-ports app \
  bash -c "cargo run --bin server & sleep 3 && cargo test -- --test-threads=1"

docker compose down

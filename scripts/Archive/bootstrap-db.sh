#!/bin/bash
# cr8s/scripts/bootstrap-db.sh
set -euo pipefail

DB_CONTAINER="cr8s-postgres-1"

#ocker compose down
docker compose up -d postgres

cat "scripts/sql/db-init.sql" | \
    docker exec -i "${DB_CONTAINER}" psql -U postgres -d cr8s -h localhost -p 5432

DATABASE_URL="postgres://postgres:secret@localhost:5432/cr8s"
cargo sqlx prepare --database-url "${DATABASE_URL}"


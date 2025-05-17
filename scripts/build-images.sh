#!/bin/bash
set -euo pipefail

if [[ "${1:-}" == "--debug" ]]; then
    DEBUG_FLAG="--build-arg DEBUG=1 --debug"
else
    DEBUG_FLAG=""
fi

PLATFORM=$(uname)

if [[ "$PLATFORM" == "Linux" ]]; then
    NETWORK_FLAG="--network=host"
    DB_HOST=172.17.0.1
else
    NETWORK_FLAG=""
    DB_HOST=host.docker.internal
fi

echo "DB_HOST: $DB_HOST"

# Note for local (dev) builds do:
#  - docker compose up -d postgres redis
#  - scripts/build-images.sh [--debug]
# For CI workflow do:
#  - scripts/build-images.sh

VERSION=$(awk -F'"' '/^\s*version\s*=/ { print $2 }' Cargo.toml)
echo "🔨 Building cr8s-server and cr8s-cli images (version: ${VERSION})"
[ -z "$VERSION" ] && { echo "❌ Failed to extract version from Cargo.toml"; exit 1; }

docker buildx build $DEBUG_FLAG $NETWORK_FLAG \
    --build-arg CI="${CI:-false}" \
    --build-arg DATABASE_HOST="${DB_HOST}" \
    --build-arg REDIS_HOST="${DB_HOST}" \
    --add-host=host.docker.internal:host-gateway \
    --builder default \
    --file Dockerfile \
    --provenance=false \
    --sbom=false \
    \
    --output type=image,name=ghcr.io/johnbasrai/cr8s/cr8s-server:${VERSION},push=false \
    --target runtime-server \
    \
    --output type=image,name=ghcr.io/johnbasrai/cr8s/cr8s-cli:${VERSION},push=false \
    --target runtime-cli \
    .

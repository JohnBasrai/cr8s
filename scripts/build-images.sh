#!/bin/bash
set -euo pipefail

if [[ "${1:-}" == "--debug" ]]; then
    DEBUG_FLAG="--build-arg DEBUG=1"
else
    DEBUG_FLAG=""
fi

VERSION=$(awk -F'"' '/^\s*version\s*=/ { print $2 }' Cargo.toml)
echo "🔨 Building cr8s-server and cr8s-cli images (version: ${VERSION})"
[ -z "$VERSION" ] && { echo "❌ Failed to extract version from Cargo.toml"; exit 1; }

docker buildx build $DEBUG_FLAG \
    --build-arg CI="${CI:-false}" \
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

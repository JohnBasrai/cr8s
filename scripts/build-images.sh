#!/bin/bash
set -euo pipefail

# Parse debug flag
if [[ "${1:-}" == "--debug" ]]; then
    DEBUG_FLAG="--progress=plain"
else
    DEBUG_FLAG=""
fi

# Get version from Cargo.toml
VERSION=$(awk -F'"' '/^\s*version\s*=/ { print $2 ; exit 0 }' Cargo.toml)
echo "🔨 Building cr8s-server and cr8s-cli images (version: ${VERSION})"

if [ -z "${VERSION}" ]; then
    echo "❌ Could not extract version from Cargo.toml"
    exit 1
fi

# Build server image first (creates and caches builder stage)
echo "🏗️ Building server image..."
docker buildx build $DEBUG_FLAG \
    --build-arg CI="${CI:-false}" \
    --tag ghcr.io/johnbasrai/cr8s/cr8s-server:${VERSION} \
    --target runtime-server \
    --load \
    .

# Build CLI image (reuses cached builder stage)
echo "🏗️ Building CLI image..."
docker buildx build $DEBUG_FLAG \
    --build-arg CI="${CI:-false}" \
    --tag ghcr.io/johnbasrai/cr8s/cr8s-cli:${VERSION} \
    --target runtime-cli \
    --load \
    .

echo "✅ Both images built successfully"

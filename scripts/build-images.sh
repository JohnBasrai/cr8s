#!/bin/bash
set -euo pipefail

# Parse flags
DEBUG_FLAG=""
DEV_MODE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --debug)
            DEBUG_FLAG="--progress=plain"
            shift
            ;;
        --dev)
            DEV_MODE=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--debug] [--dev]"
            echo ""
            echo "Options:"
            echo "  --debug    Enable verbose build output"
            echo "  --dev      Create dev-tagged images for local testing"
            exit 1
            ;;
    esac
done

# Get version from Cargo.toml
export VERSION=$(awk -F'"' '/^\s*version\s*=/ { print $2 ; exit 0 }' Cargo.toml)
echo $VERSION > VERSION

if [ -z "${VERSION}" ]; then
    echo "❌ Could not extract version from Cargo.toml"
    exit 1
fi

# Determine tags based on mode
if [[ "$DEV_MODE" == true ]]; then
    # Get current branch for dev tagging
    BRANCH_NAME=$(git rev-parse --abbrev-ref HEAD)
    BRANCH_CLEAN=${BRANCH_NAME//\//-}  # Replace slashes with dashes
    COMMIT_SHORT=$(git rev-parse --short HEAD)
    DEV_TAG="${VERSION}-dev-${BRANCH_CLEAN}-${COMMIT_SHORT}"
    
    SERVER_TAG="ghcr.io/johnbasrai/cr8s/cr8s-server:${DEV_TAG}"
    CLI_TAG="ghcr.io/johnbasrai/cr8s/cr8s-cli:${DEV_TAG}"
    
    echo "🧪 Building cr8s DEV images (tag: ${DEV_TAG})"
    echo "📝 Use this in your .env file: CR8S_VERSION=latest"
else
    SERVER_TAG="ghcr.io/johnbasrai/cr8s/cr8s-server:${VERSION}"
    CLI_TAG="ghcr.io/johnbasrai/cr8s/cr8s-cli:${VERSION}"
    
    echo "🔨 Building cr8s RELEASE images (version: ${VERSION})"
fi

# Build server image first (creates and caches builder stage)
echo "🏗️ Building server image..."
docker buildx build $DEBUG_FLAG \
    --build-arg CI="${CI:-false}" \
    --tag ${SERVER_TAG} \
    --target runtime-server \
    --load \
    .

# Build CLI image (reuses cached builder stage)
echo "🏗️ Building CLI image..."
docker buildx build $DEBUG_FLAG \
    --build-arg CI="${CI:-false}" \
    --tag ${CLI_TAG} \
    --target runtime-cli \
    --load \
    .

echo "✅ Images built successfully:"
echo "   Server: ${SERVER_TAG}"
echo "   CLI: ${CLI_TAG}"

if [[ "$DEV_MODE" == true ]]; then
    docker tag ${SERVER_TAG} cr8s-server-dev:latest
    docker tag ${CLI_TAG} cr8s-cli-dev:latest
fi

if [[ "$DEV_MODE" == true ]]; then
    echo ""
    echo "🧪 DEV MODE: To test these images:"
    echo "   1. Update cr8s-fe/.env with: CR8S_VERSION=${DEV_TAG}"
    echo "   2. Run: ./scripts/quickstart.sh --no-cache"
    echo ""
    echo "💡 To push dev images to GHCR (optional):"
    echo "   docker push ${SERVER_TAG}"
    echo "   docker push ${CLI_TAG}"
fi

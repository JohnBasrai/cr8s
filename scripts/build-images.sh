#!/bin/bash
set -euo pipefail

# Parse flags
DEBUG_FLAG=""
DEV_MODE=false
progname="$(basename $0)"

# Source common functions
source "$(dirname "$0")/common.sh"

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
            echo "${progname}: Unknown option: $1"
            echo "Usage: ${progname} [--debug] [--dev]"
            echo ""
            echo "Options:"
            echo "  --debug    Enable verbose build output"
            echo "  --dev      Create dev-tagged images for local testing"
            exit 1
            ;;
    esac
done

# Get version from Cargo.toml
export VERSION=$(get-version)

if [ -z "${VERSION}" ]; then
    echo "${progname}:❌ Could not extract version from Cargo.toml"
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
    
    echo "${progname}:🧪 Building cr8s DEV images (tag: ${DEV_TAG})"
    echo "${progname}:📝 Use CR8S_VERSION=latest"
else
    SERVER_TAG="ghcr.io/johnbasrai/cr8s/cr8s-server:${VERSION}"
    CLI_TAG="ghcr.io/johnbasrai/cr8s/cr8s-cli:${VERSION}"
    
    echo "${progname}:🔨 Building cr8s RELEASE images (version: ${VERSION})"
fi

# Build server image first (creates and caches builder stage)
echo "${progname}:🏗️ Building server image..."
docker buildx build ${DEBUG_FLAG} \
    --build-arg CI="${CI:-false}" \
    --tag ${SERVER_TAG} \
    --target runtime-server \
    --load \
    .

# Build CLI image (reuses cached builder stage)
echo "${progname}:🏗️ Building CLI image..."
docker buildx build ${DEBUG_FLAG} \
    --build-arg CI="${CI:-false}" \
    --tag ${CLI_TAG} \
    --target runtime-cli \
    --load \
    .

echo "${progname}:✅ Images built successfully:"
echo "${progname}:   Server: ${SERVER_TAG}"
echo "${progname}:   CLI: ${CLI_TAG}"

if [[ "$DEV_MODE" == true ]]; then
    docker tag ${SERVER_TAG} cr8s-server-dev:latest
    docker tag ${CLI_TAG} cr8s-cli-dev:latest
fi

if [[ "$DEV_MODE" == true ]]; then
    echo ""
    echo "🧪 DEV MODE: Images ready for testing"
    echo "   Local tags : cr8s-server-dev:latest, cr8s-cli-dev:latest" 
    echo "   GHCR tags  : ${SERVER_TAG}, ${CLI_TAG}"
    echo ""
    echo "🧪 Next steps - Choose your workflow:"
    echo "   Integration testing : ./scripts/dev-test-setup.sh"
    echo "   Frontend testing    : Use with cr8s-fe (see README there)"
    echo "   Manual testing      : ./scripts/dev-test-setup.sh --no-tests"
    echo ""
    echo "💡 Optional: Push to GHCR for sharing"
    echo "   docker push ${SERVER_TAG} && docker push ${CLI_TAG}"
fi

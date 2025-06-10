#!/bin/bash
# -----------------------------------------------------------------------------
# build-verification-test.sh – Comprehensive system smoke test
#
# This script performs a comprehensive integration test:
# - Starts the complete application stack (server, database, Redis)
# - Validates service health and connectivity
# - Tests database schema initialization
# - Verifies core CLI functionality and user management workflows
# - Ensures data persistence across operations
#
# Intended for CI and local test environments to catch system integration
# issues early and validate that the full stack works end-to-end.
#
# Usage:
#   ./scripts/build-verification-test.sh [--debug | --verbose] [--dev]
#
# -----------------------------------------------------------------------------

set -euo pipefail

VERSION="${VERSION:-}"
VERBOSE=
progname="$(basename $0)"
RUN=
DRY_RUN=false
export CLI_IMAGE=${CLI_IMAGE:-ghcr.io/johnbasrai/cr8s/cr8s-cli:${VERSION}}
export SERVER_IMAGE=${SERVER_IMAGE:-ghcr.io/johnbasrai/cr8s/cr8s-server:${VERSION}}

usage() {
    echo ""
    echo "Usage: ${progname} [ --verbose | --debug ] [--dev] [--dry-run]"
    echo ""
    echo "Options:"
    echo "  --verbose  Enable verbose output"
    echo "  --dev      Use local dev-tagged images for testing"
    exit 0
}

while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run)
            DRY_RUN=true;
            RUN=echo
            shift
            ;;
        --verbose|--debug)
            set -x
            VERBOSE=true
            shift
            ;;
        --dev)
            export VERSION=latest
            export SERVER_IMAGE=cr8s-server-dev
            export CLI_IMAGE=cr8s-cli-dev
            DEV_MODE=true
            shift
            ;;
        -h|--help) usage ;;
        *)  echo "${progname}: Unknown option: $1"
            usage ;;
    esac
done

: ${VERSION:?is required}
: ${CLI_IMAGE:?is required}
: ${SERVER_IMAGE:?is required}

echo "VERSION      : ${VERSION}"
echo "CLI_IMAGE    : ${CLI_IMAGE}"
echo "SERVER_IMAGE : ${SERVER_IMAGE}"

echo "🔥 Running cr8s smoke test..."
echo "   Testing that the system boots and core functionality works"

# CI-friendly: Set timeout
TIMEOUT=120
export COMPOSE_HTTP_TIMEOUT=$TIMEOUT

# Cleanup first (CI environments may have stale containers)
echo "🧹 Cleaning up any existing containers..."
$RUN docker compose down -v --remove-orphans || true

# Ensure we cleanup on exit (important for CI)
trap 'exit_status=$?;
      echo "🧹 Cleanup on exit...";
      $RUN docker compose down -v --remove-orphans >/dev/null 2>&1;
      exit $exit_status' EXIT

# Test: Start the full stack
echo "🚀 Starting full stack..."
$RUN docker compose up -d

# Wait for services to be healthy with timeout
if [ "${DRY_RUN}" == "false" ] ; then
    echo "⏳ Waiting for services to be ready..."
    for i in {1..30}; do
        if curl -sf http://127.0.0.1:8000/cr8s/health > /dev/null 2>&1; then
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
fi

# Test: Health check (smoke test essential)
if [ "${DRY_RUN}" == "false" ] ; then
    echo "🏥 Testing health endpoint..."
    if curl -sf http://127.0.0.1:8000/cr8s/health; then
        echo "✅ Health check passed"
    else
        echo "❌ Health check failed"
        docker compose logs server
        exit 1
    fi
fi

# Test: Database connectivity (smoke test essential)
echo "🗄️  Testing database connectivity..."
$RUN docker compose exec -T postgres psql -U postgres cr8s -c "SELECT 1;" > /dev/null
echo "✅ Database connectivity passed"

# Test: Redis connectivity (smoke test essential)
echo "🔴 Testing Redis connectivity..."
$RUN docker compose exec -T redis redis-cli ping | grep -q PONG
echo "✅ Redis connectivity passed"

# Initialize database schema
echo "📊 Initializing database schema..."
$RUN docker compose run --rm cli load-schema
echo "✅ Database schema initialized"

# Debug schema if verbose mode
if [ "${RUN}" = "true" -a "$VERBOSE" == "true" ]; then
    echo "🔍 Dumping schema for debugging..."
    docker compose exec -T postgres psql -U postgres cr8s -c "\dt"
    docker compose exec -T postgres psql -U postgres cr8s -c "\d role"
fi

# Test: Core CLI functionality
echo "🔧 Testing core CLI functionality..."
$RUN docker compose run --rm cli list-users

echo "👤 Testing user creation workflow..."
$RUN docker compose run --rm cli create-user \
       --username smoketest-$(date +%s) \
       --password testpass123 --roles Viewer

echo "📋 Testing data persistence..."
$RUN docker compose run --rm cli list-users | grep smoketest

echo "🎉 Smoke test completed successfully!"
echo "System is ready for deployment"

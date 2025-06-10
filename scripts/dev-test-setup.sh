#!/bin/bash
# -----------------------------------------------------------------------------
# dev-test-setup.sh – Define local test helper functions for cr8s development.
#
# This script is meant to be sourced (not executed) to expose functions in the
# user's shell environment. It simplifies running integration tests by setting
# up aliases and workflows for:
#
# - Starting Postgres, Redis, and cr8s backend
# - Seeding the test database
# - Running CLI and HTTP API integration tests
# - Checking server health and logs
#
# Usage:
#   source scripts/dev-test-setup.sh
#
# Then invoke helpers:
#   start-services      # Bring up services and seed test user
#   run-tests           # Run CLI + server integration tests
#   check-server        # View logs, confirm health
#
# Full list of functions is documented in docs/development.md
# -----------------------------------------------------------------------------

set -euo pipefail

progname="$(basename "${BASH_SOURCE[0]}")"

# Source common functions  
source "$(dirname "${BASH_SOURCE[0]}")/common.sh"

if [ "${CI:-false}" = true ] ; then
    PS1=CI
    echo "$progname: 🧪 Setting up for CI workflow testing..."
    : ${CLI_IMAGE:?is required}
    : ${SERVER_IMAGE:?is required}
    : ${VERSION:?is required}
else
    # Set up dev environment (assumes images already built)
    echo "$progname: 🧪 Setting up development environment for integration testing..."
    setup-dev-env
fi

echo "$progname: 🐳 Docker images expected:"
echo "$progname:   SERVER_IMAGE = ${SERVER_IMAGE}"
echo "$progname:   CLI_IMAGE    = ${CLI_IMAGE}"
echo "$progname:   VERSION      = ${VERSION}"

# Set up development environment prompt with status
if [[ -z "${CR8S_DEV_ENV:-}" ]]; then
    export CR8S_DEV_ENV="stopped"
    export CR8S_ORIGINAL_PS1="${PS1:-ps1}"
    
    update-cr8s-prompt() {
        local status="${CR8S_DEV_ENV:-stopped}"
        export PS1="(cr8s-dev:${status}) ${CR8S_ORIGINAL_PS1}"
    }

    update-cr8s-prompt
    
    echo "$progname: 🎯 Development environment active"
    echo "$progname: 💡 Run 'deactivate-cr8s' to restore original prompt"
fi

# Define helper functions for the developer

start-services() {
    echo "🧹 Cleaning up any existing containers..."
    cleanup-docker

    echo "🚀 Starting docker services..."
    docker compose up -d
    
    echo "⏳ Waiting for services to be ready..."
    wait-for-services
    
    echo "📊 Loading database schema..."
    docker compose run --rm cli load-schema
    
    echo "👤 Creating test admin user (admin@example.com)..."
    docker compose run --rm cli create-user \
        --username admin@example.com \
        --password password123 \
        --roles admin,editor,viewer
    
    export CR8S_DEV_ENV="running"
    update-cr8s-prompt
    echo "✅ Services ready with test user!"
}

stop-services() {
    echo "🛑 Stopping docker services..."
    cleanup-docker
    export CR8S_DEV_ENV="stopped"
    update-cr8s-prompt
    echo "✅ Services stopped!"
}

# CLI Testing Functions
run-cli-tests() {
    echo "🧪 Running CLI integration tests..."
    cargo test --test cli_integration "$@"
}

# Server Testing Functions  
run-server-tests() {
    echo "🚀 Running server integration tests..."
    
    # Ensure services are running
    if ! curl -sf http://127.0.0.1:8000/cr8s/health > /dev/null 2>&1; then
        echo "❌ Server not responding. Starting services..."
        start-services
    fi
    
    cargo test --test server_integration "$@"
}

run-single-server-test() {
    local test_name="${1:-}"
    if [ -z "$test_name" ]; then
        echo "Usage: run-single-server-test <test_name>"
        echo "Available server tests:"
        cargo test --test server_integration -- --list | grep "test " | sed 's/test /  /'
        return 1
    fi
    
    echo "🧪 Running single server test: $test_name"
    
    # Ensure services are running
    if ! curl -sf http://127.0.0.1:8000/cr8s/health > /dev/null 2>&1; then
        echo "❌ Server not responding. Starting services..."
        start-services
    fi
    
    cargo test --test server_integration "$test_name" -- --nocapture
}

# Combined test runner
run-tests() {
    echo "🧪 Running all integration tests (CLI + Server)..."
    
    # Ensure services are running
    if ! curl -sf http://127.0.0.1:8000/cr8s/health > /dev/null 2>&1; then
        echo "❌ Server not responding. Starting services..."
        start-services
    fi
    
    echo "📋 Step 1: CLI tests..."
    run-cli-tests "$@"
    
    echo "📋 Step 2: Server tests..."
    run-server-tests "$@"
    
    echo "🎉 All integration tests completed!"
}

# Utility Functions
show-logs() {
    local service="${1:-server}"
    echo "📋 Showing logs for $service..."
    docker compose logs -f "$service"
}

restart-server() {
    echo "🔄 Restarting server..."
    export CR8S_DEV_ENV="restarting"
    update-cr8s-prompt
    docker compose restart server
    wait-for-services
    export CR8S_DEV_ENV="running"
    update-cr8s-prompt
    echo "✅ Server restarted!"
}

check-server() {
    echo "🏥 Checking server health and status..."
    
    echo "--- Basic Health Check ---"
    if curl -sf http://127.0.0.1:8000/cr8s/health; then
        echo " ✅ Health check passed"
    else
        echo " ❌ Health check failed"
    fi
    
    echo -e "\n--- Server Status ---"
    if docker compose ps server | grep -q "healthy"; then
        echo " ✅ Server container healthy"
    else
        echo " ⚠️  Server container status:"
        docker compose ps server
    fi
    
    echo -e "\n--- Recent Server Logs ---"
    docker compose logs --tail=10 server
}

# Docker compose aliases
alias dc='docker compose'
alias dcr='docker compose run --rm'

deactivate-cr8s() {
    if [[ -n "${CR8S_ORIGINAL_PS1:-}" ]]; then
        # Clean up services if they're running
        if [[ "${CR8S_DEV_ENV}" == "running" ]]; then
            echo "🛑 Stopping services before deactivating..."
            cleanup-docker
        fi
        
        # Restore original prompt
        export PS1="${CR8S_ORIGINAL_PS1}"
        unset CR8S_ORIGINAL_PS1
        unset CR8S_DEV_ENV
        unset -f update-cr8s-prompt \
              start-services \
              stop-services \
              restart-server \
              run-cli-tests \
              run-server-tests \
              run-single-server-test \
              run-tests \
              show-logs \
              check-server \
              deactivate-cr8s
        unalias dc dcr 2>/dev/null || true
        echo "✅ cr8s development environment deactivated"
    else
        echo "⚠️  No cr8s environment to deactivate"
    fi
}

# Show available commands
echo ""
echo "$progname: ✅ Environment ready! Available commands:"
echo ""
echo "$progname:🚀 Service Management:"
echo "$progname:   start-services       # Start postgres, redis, server + create test user"
echo "$progname:   stop-services        # Stop all services"
echo "$progname:   restart-server       # Restart just the server"
echo "$progname:   show-logs [service]  # Show service logs (default: server)"
echo "$progname:   check-server         # Check server health and logs"
echo ""
echo "$progname:🧪 Testing:"
echo "$progname:   run-tests            # Run all integration tests (CLI + Server)"
echo "$progname:   run-cli-tests        # Run CLI integration tests"
echo "$progname:   run-server-tests     # Run server integration tests"
echo "$progname:   run-single-server-test <name>  # Run specific server test"
echo ""
echo "$progname:🔧 Utilities:"
echo "$progname:   dc                   # Alias for 'docker compose'"
echo "$progname:   dcr                  # Alias for 'docker compose run --rm'"
echo "$progname:   deactivate-cr8s      # Exit development environment"
echo ""
echo "$progname:🚀 Quick start:"
echo "$progname:   start-services && run-tests"

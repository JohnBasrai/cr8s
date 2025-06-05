#!/bin/bash
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
    
    export CR8S_DEV_ENV="running"
    update-cr8s-prompt
    echo "✅ Services ready!"
}

stop-services() {
    echo "🛑 Stopping docker services..."
    cleanup-docker
    export CR8S_DEV_ENV="stopped"
    update-cr8s-prompt
    echo "✅ Services stopped!"
}

run-tests() {
    echo "🧪 Running CLI integration tests..."
    cargo test --test cli_integration "$@"
}

run-single-test() {
    local test_name="${1:-}"
    if [ -z "$test_name" ]; then
        echo "Usage: run-single-test <test_name>"
        echo "Available tests:"
        cargo test --test cli_integration -- --list | grep "test " | sed 's/test /  /'
        return 1
    fi
    
    echo "🧪 Running single test: $test_name"
    cargo test --test cli_integration "$test_name" -- --nocapture
}

show-logs() {
    local service="${1:-server}"
    echo "📋 Showing logs for $service..."
    docker compose logs -f "$service"
}

test-cli() {
    echo "🔧 Testing CLI command: $*"
    docker compose run --rm cli "$@"
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
              run-tests \
              run-single-test \
              test-cli \
              show-logs \
              deactivate-cr8s
        echo "✅ cr8s development environment deactivated"
    else
        echo "⚠️  No cr8s environment to deactivate"
    fi
}

# Show available commands
echo ""
echo "$progname: ✅ Environment ready! Available commands:"
echo "$progname:   start-services    # Start postgres, redis, server"
echo "$progname:   stop-services     # Stop all services"
echo "$progname:   run-tests         # Run all CLI integration tests"
echo "$progname:   run-single-test   # Run a specific test"
echo "$progname:   test-cli          # Run a CLI command directly"
echo "$progname:   show-logs         # Show service logs" 
echo "$progname:   restart-server    # Restart just the server"
echo "$progname:   deactivate-cr8s   # Exit development environment"
echo ""
echo "$progname: 🚀 Quick start:"
echo "$progname:   start-services && run-tests"


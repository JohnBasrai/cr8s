#!/bin/bash
# common.sh - Shared functions for cr8s build and test scripts

progname="${progname:=common.sh}"

# Extract version from Cargo.toml and write to VERSION file
get-version() {
    #
    local version
    version=$(awk -F'"' '/^\s*version\s*=/ { print $2 ; exit 0 }' Cargo.toml)
    
    if [ -z "${version}" ]; then
        echo "${progname}: ❌ Could not extract version from Cargo.toml" >&2
        return 1
    fi
    
    echo "${version}" > VERSION
    export VERSION="${version}"
    echo "${version}"
}

# Set up Docker environment variables for CI mode
setup-ci-env() {
    #
    local version="${1:-${VERSION}}"
    if [ -z "${version}" ]; then
        echo "${progname}: ❌ Version required for CI environment setup" >&2
        return 1
    fi
    
    export SERVER_IMAGE="ghcr.io/johnbasrai/cr8s/cr8s-server:${version}"
    export CLI_IMAGE="ghcr.io/johnbasrai/cr8s/cr8s-cli:${version}"
    export VERSION="${version}"
}

# Set up Docker environment variables for dev mode  
setup-dev-env() {
    #
    export SERVER_IMAGE="cr8s-server-dev:latest"
    export CLI_IMAGE="cr8s-cli-dev:latest" 
    export VERSION="latest"
    export RUST_LOG="${RUST_LOG:-debug}"
    export SERVER_DEBUG_ARGS="${SERVER_DEBUG_ARGS:---dump-state-traits --check}"
}

# Wait for docker compose services to be healthy
wait-for-services() {
    #
    local timeout="${1:-300}"  # 5 minutes default
    local interval="${2:-10}"  # 10 seconds default
    
    echo "${progname}: ⏳ Waiting for services to be ready (timeout: ${timeout}s)..."
    
    for ((i=1; i<=timeout/interval; i++)); do
        #
        if curl -sf http://127.0.0.1:8000/cr8s/health > /dev/null 2>&1; then
            echo "${progname}: ✅ Services ready after $((i * interval)) seconds"
            return 0
        fi
        
        if [ $i -eq $((timeout/interval)) ]; then
            echo "${progname}: ❌ Services failed to start within ${timeout} seconds" >&2
            return 1
        fi
        
        sleep ${interval}
    done
}

# Cleanup function for trap handlers
cleanup-docker() {
    echo "${progname}: 🧹 Cleaning up docker containers..."
    docker compose down -v --remove-orphans >/dev/null 2>&1 || true
}

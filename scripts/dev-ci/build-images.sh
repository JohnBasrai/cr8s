#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<EOF
Usage: ${0##*/} [OPTIONS] [TARGET...]

Build and optionally push cr8s containers using Docker BuildKit.

Options:
    -d, --dry-run, --debug     Show build commands without executing
    -v, --verbose              Enable verbose shell output (set -x)
    --prune                    Remove build cache before running buildx
        --push                 Push images to GHCR instead of loading locally
                               (automatically enabled if CI=true)
    -s, --sign                 Placeholder: signing not supported yet
    -D, --diesel-migrations    Run diesel setup, migration, and format (dev/CI)
        --full-lint            Run full lint suite only (no build)
    -h, --help                 Show this help message and exit

Targets:
    cr8s-server                REST API backend (Rocket)
    cr8s-cli                   Admin CLI tool
    cr8s-cli-seeder            CLI-based DB seeder
    cr8s-test-runner           Dev container running cargo test

If no targets are specified, all of the above are built.
EOF
    exit 0
}

VERSION="${CR8S_VERSION:-}"
if [[ -z "$VERSION" ]]; then
    VERSION=$(awk -F '"' '/^version =/ { print $2 }' Cargo.toml)
fi

CACHE_DIR="${CACHE_DIR:-$HOME/.cache/buildx}"
DB_HOST=172.17.0.1
BUILDER_NAME=cr8s-builder

DOCKER_PRUNE=false
PUSH_MODE=false
DRY_RUN=false
DIESEL_MIGRATIONS=false
FULL_LINT=false

# Parse CLI arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --push)
            PUSH_MODE=true
            echo "📤 Push mode enabled — images will be pushed to GHCR"
            ;;
        -d|--dry-run|--debug)
            DRY_RUN=true
            echo "🧪 Dry run mode — build commands will be shown but not executed"
            ;;
        -v|--verbose)
            DEBUG_MODE="--verbose" ; set -x ; _v=-v
            ;;
        -s|--sign)
            echo "Signing not supported yet" >&2
            exit 1
            ;;
        -D|--diesel-migrations)
            DIESEL_MIGRATIONS=true
            echo "📦 Diesel migrations will be executed before build"
            ;;
        --prune) DOCKER_PRUNE=true;;
        --full-lint)
            FULL_LINT=true
            echo "🔎 Running full lint suite (fmt, clippy, audit, outdated, deny)"
            ;;
        -h|--help)
            usage
            ;;
        -* )
            echo "❌ Unknown parameter passed: $1" >&2
            usage
            ;;
        * )
            break
            ;;
    esac
    shift 1
done

# Prevent unsupported combination
if $PUSH_MODE && $FULL_LINT; then
    echo "❌ Cannot use --push and --full-lint together" >&2
    exit 1
fi

# Handle full lint mode early
if $FULL_LINT; then
    docker run --rm \
        -v "$(pwd)":/app \
        -w /app \
        ghcr.io/johnbasrai/cr8s/rust-dev:$VERSION \
        bash -c "cargo fmt --check && \
                 cargo clippy --all-targets --all-features -- -D warnings && \
                 cargo audit || true && \
                 cargo outdated || true && \
                 cargo deny check || true"
    exit 0
fi

# If no positional targets given, default to all
if [[ $# -eq 0 ]]; then
    set -- cr8s-server cr8s-cli cr8s-cli-seeder cr8s-test-runner
fi

# Autodetect CI mode
if [[ "${CI:-}" == "true" ]]; then
    echo "🤖 CI environment detected — enabling push mode"
    PUSH_MODE=true
fi

mkdir -p "$CACHE_DIR"

# Generate Rocket.toml if needed.
if [[ ! -f Rocket.toml ]]; then
    scripts/dev-ci/generate-rocket-toml.sh > Rocket.toml
fi

if grep '%{' Rocket.toml > /dev/null; then
    echo "❌ Substitution incomplete in Rocket.toml!" >&2
    exit 1
fi

# Optional: run diesel setup and migration before build
if $DIESEL_MIGRATIONS; then
    echo "🚀 Running diesel setup, migration, and formatting..."
    diesel setup
    diesel migration run
    rustfmt src/schema.rs
fi

# 🧱 Ensure builder exists
if ! docker buildx inspect "$BUILDER_NAME" > /dev/null 2>&1; then
    echo "🔧 Creating buildx builder: $BUILDER_NAME"
    docker buildx create --name "$BUILDER_NAME" --driver docker-container --use
else
    docker buildx use "$BUILDER_NAME"
fi

echo "🧹 Using builder: $BUILDER_NAME"
if [[ "${DOCKER_PRUNE:-false}" == true ]] ; then
    echo "🧹 Pruning BuildKit cache for builder: $BUILDER_NAME"
    docker buildx prune --builder --all --force
    exit $?
fi

build_target() {
    local target=$1
    local tag=$2

    echo "🛠️  Building $target → $tag"
    local build_cmd=(
        docker buildx build
        --builder "$BUILDER_NAME"
        --network=host
        --build-arg DATABASE_HOST=$DB_HOST
        --build-arg REDIS_HOST=$DB_HOST
        --cache-from=type=local,src=$CACHE_DIR
        --cache-to=type=local,dest=$CACHE_DIR
        --file Dockerfile
        --target "$target"
        $($PUSH_MODE && echo --push || echo --load)
        --tag "$tag"
        .
    )

    if $DRY_RUN; then
        echo "🔍 DRY RUN: ${build_cmd[*]}"
    else
        "${build_cmd[@]}"
    fi
}

for t in "$@"; do
    case $t in
        cr8s-server)        build_target cr8s-server       ghcr.io/johnbasrai/cr8s/cr8s-server:$VERSION ;;
        cr8s-cli)           build_target cr8s-cli          ghcr.io/johnbasrai/cr8s/cr8s-cli:$VERSION    ;;
        cr8s-cli-seeder)    build_target cr8s-cli-seeder   cr8s-cli-seeder:dev                           ;;
        cr8s-test-runner)   build_target cr8s-test-runner  cr8s-test-runner:dev                          ;;
        *)
            echo "❌ Unknown target: $t" >&2
            usage
            ;;
    esac
    echo
done

# Optionally clean up untagged layers if not in dry-run mode
if ! $DRY_RUN; then
    echo "🧽 Removing dangling images..."
    docker image prune --force
fi

#!/bin/bash
set -euo pipefail

typeset -rx base_dir="$(cd "$( dirname "${BASH_SOURCE[0]}/" )/" >/dev/null 2>&1 && pwd )"
. ${base_dir}/.cr8s_env
export CONTAINER="cr8s-dev-${CONTAINER_TAG}"

echo "🛑 Stopping ${CONTAINER}..."

docker rm -f ${CONTAINER} 2>/dev/null || echo "Container ${CONTAINER} not running or already removed."

echo "🛑 Stopping docker-compose services..."
docker compose down --remove-orphans

echo "✅ All services stopped."

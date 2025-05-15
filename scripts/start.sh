#!/bin/bash
set -euo pipefail

typeset -rx base_dir="$(cd "$( dirname "${BASH_SOURCE[0]}/" )/.." >/dev/null 2>&1 && pwd )"
cd ${base_dir}
. ./scripts/.cr8s_env
export CONTAINER="cr8s-dev-${CONTAINER_TAG}"

echo "🚀 Starting cr8s-dev ${CONTAINER}..."

if [ -n "$(docker ps -q --filter name=${CONTAINER})" ] ; then
    echo "$0: Error: Container ${CONTAINER} in already running..."
    exit 1
fi

docker network inspect cr8s-net &>/dev/null || docker network create cr8s-net

docker run --detach --network=cr8s-net \
  --name "${CONTAINER}" \
  -p 8000:8000 \
  -u "${CR8S_USER_TAG}" \
  -v "$PWD:$PWD" \
  -w "$PWD" \
  -e CARGO_HOME="$CARGO_HOME" \
  -e DATABASE_URL="$DATABASE_URL" \
  ghcr.io/johnbasrai/cr8s/rust-dev:v1.83 \
  sleep infinity

ensure_running() {
    who=$1
    if ! docker compose ps ${who} | grep -q 'Up'; then
        echo "🚀 Starting ${who}..."
        docker compose up -d ${who}
    else
        echo "✅ ${who} already running"
    fi
}
ensure_running redis
ensure_running postgres


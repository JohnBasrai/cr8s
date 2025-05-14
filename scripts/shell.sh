#!/bin/bash
set -euo pipefail

typeset -rx base_dir="$(cd "$( dirname "${BASH_SOURCE[0]}/" )/" >/dev/null 2>&1 && pwd )"

. ${base_dir}/.cr8s_env

CONTAINER="cr8s-dev-${CONTAINER_TAG}"

if [ -z "$(docker ps -q --filter name=${CONTAINER})" ] ; then
    echo "$0: Error: Container ${CONTAINER} in not running..."
    exit 1
fi

docker exec -u "${CR8S_USER_TAG}" -it "$CONTAINER" \
  env CARGO_HOME="$CARGO_HOME" DATABASE_URL="$DATABASE_URL" bash

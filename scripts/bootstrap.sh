#!/bin/bash
set -euo pipefail

typeset -rx base_dir="$(cd "$( dirname "${BASH_SOURCE[0]}/" )/.." >/dev/null 2>&1 && pwd )"
. ${base_dir}/scripts/.cr8s_env
pwd

CONTAINER="cr8s-dev-${CONTAINER_TAG}"

if [ -z "$(docker ps -q --filter name=${CONTAINER})" ] ; then
    echo "$0: Error: Container ${CONTAINER} in not running..."
    exit 1
fi

SENTINAL_FILE=/tmp/_BOOTSTRAP_SENTINAL

if docker exec -it ${CONTAINER} test -f ${SENTINAL_FILE}; then
    echo "$0: Container ${CONTAINER} is already bootstrap'd"
    exit 0
fi

echo "📦 1. Preparing writable Cargo cache..."
docker exec -u root "${CONTAINER}" bash -c "
  mkdir -p $CARGO_HOME
  cd /usr/local/cargo
  tar -cf - . | tar -xf - -C $CARGO_HOME
  rm -rf $CARGO_HOME/registry/src
  chown -R $(id -u):$(id -g) $CARGO_HOME
  touch ${SENTINAL_FILE}
  "

echo "📜 2. Running Diesel migrations..."
pwd; ls -la diesel.toml
docker exec "${CONTAINER}" diesel setup
docker exec "${CONTAINER}" diesel migration run

echo "👤 3. Seeding default users..."
docker exec "${CONTAINER}" bash -c "
  export CARGO_HOME=$CARGO_HOME
  cargo run --bin cli -- users create test_admin password123 admin
"

docker exec "${CONTAINER}" bash -c "
  export CARGO_HOME=$CARGO_HOME
  cargo run --bin cli -- users create test_viewer password123 viewer
"

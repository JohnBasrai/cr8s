#!/usr/bin/env bash
set -euo pipefail

DB_HOST=${DB_HOST:-localhost}

if [[ ! -f Rocket.toml.template ]] ; then
    echo "⚠️  Rocket.toml.template not found; skipping generation."
    exit 1
fi

sed Rocket.toml.template \
  -e "s|%{REDIS_HOST}%|${DB_HOST}|g" \
  -e "s|%{DATABASE_HOST}%|${DB_HOST}|g"

echo "✅ Rocket.toml substitutions complete."

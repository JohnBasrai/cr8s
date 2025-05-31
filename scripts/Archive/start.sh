#!/bin/bash
set -euo pipefail

# Start required services for local dev or cr8s-fe integration
docker compose up -d postgres redis server

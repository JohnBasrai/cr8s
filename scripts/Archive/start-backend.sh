#!/bin/bash
set -euo pipefail

# Launch the backend server in the background (daemon mode) and wait for it to become ready.
# This is called from the Dockerfile in the build container, which requires a running backend
# for unit tests. It uses the freshly built release version of the server, assuming a successful
# build path — i.e., compile errors will surface before clippy warnings.

# Use server just built in Dockerfile
printenv | grep REDIS
target/release/server > /tmp/server.log 2>&1 &

sleep 3
ps aux
tail -n 50 /tmp/server.log

echo "⏳ Waiting for Rocket to start..."
for i in {1..20}; do
    CODE=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8000/health || true)
    if [[ "$CODE" == "200" ]]; then
        echo "✅ Rocket is ready (HTTP 200 on /health)"

        # Optional extra check: Redis-specific
        PING_RESPONSE=$(curl -s http://localhost:8000/ping)
        if [[ "$PING_RESPONSE" == "PONG" ]]; then
            echo "✅ Redis ping succeeded"
        else
            echo "❌ Redis ping failed (got: $PING_RESPONSE)"
            cat /tmp/server.log
            exit 1
        fi
        exit 0
    fi
    echo "⏳ [$i/20] Got HTTP $CODE from /health, waiting..."
    sleep 1
done

echo "❌ Rocket did not start responding with HTTP 200 in time"

echo "🪵 --- BEGIN /tmp/server.log ---"
cat /tmp/server.log || echo "⚠️ No log output found"
echo "🪵 --- END /tmp/server.log ---"

exit 1

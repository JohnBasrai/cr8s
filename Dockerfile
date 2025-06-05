# syntax=docker/dockerfile:1.4
FROM ghcr.io/johnbasrai/cr8s/rust-dev:1.83.0-rev5 AS builder
ARG DEBUG=0
ARG CI=false
ENV ROCKET_LOG=debug
WORKDIR /app

USER root

# Optimize build caching by copying dependency files first
# This allows Docker to cache the expensive `cargo fetch` step
# when only source code changes (not dependencies)
# ✅ If Files unchanged → CACHED (if you change Cargo.* you want to fetch)
COPY Cargo.toml Cargo.lock ./

# ✅ If Previous layers are cached → this one is CACHED (skips download!)
RUN cargo fetch

# Copy rest of app
# ❌ If source changed → REBUILDS from here (not changed => CACHED)
# Instead of COPY . .   We copy just src
COPY src/ ./src/
COPY Rocket.toml.template ./
# Don't copy logs, target/, .git/, etc.

RUN /bin/sh -c 'echo "👀 Lint checks..." >&2'
RUN cargo fmt --check
RUN cargo clippy --release --all-targets --all-features -- -D warnings

# Security audit false positive - ✅ Confirmed: We do not using MySQL, so the RSA
# vulnerability doesn't affect cr8s
RUN cargo audit --ignore RUSTSEC-2023-0071 || cargo outdated || true
RUN cargo test --release --all-targets --all-features -- --nocapture
RUN /bin/sh -c 'echo "🛠️ Build binaries..." >&2'
RUN cargo build --release --bin server --bin cli
RUN strip target/release/server target/release/cli

########################
# 🚀 Server image
########################
FROM ghcr.io/johnbasrai/cr8s/rust-runtime:0.1.3 AS runtime-server
USER root
ARG REDIS_HOST=redis
ARG DATABASE_HOST=postgres
RUN /bin/sh -c 'echo "🔨 Building runtime SERVER container" >&2'
WORKDIR /app
COPY Rocket.toml.template /app/Rocket.toml
RUN sed -i "s|%{REDIS_HOST}%|${REDIS_HOST}|g; s|%{DATABASE_HOST}%|${DATABASE_HOST}|g" /app/Rocket.toml

RUN cat /app/Rocket.toml  # Show the final config (remove && false)
COPY --from=builder /app/target/release/server /app/server
USER appuser
ENTRYPOINT ["./server"]

########################
# 🛠 CLI image
########################
FROM ghcr.io/johnbasrai/cr8s/rust-runtime:0.1.3 AS runtime-cli
USER root
RUN /bin/sh -c 'echo "🔨 Building runtime CLI container" >&2'
WORKDIR /app
COPY --from=builder /app/target/release/cli /app/cli
COPY --chown=appuser:appuser scripts/sql/db-init.sql /app/
USER appuser
ENTRYPOINT ["./cli"]

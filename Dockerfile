
# syntax=docker/dockerfile:1.4
FROM ghcr.io/johnbasrai/cr8s/rust-dev:1.83.0-rev4 AS builder

ARG DEBUG=0
ARG CI=false
ENV ROCKET_LOG=debug

WORKDIR /app

# Optimize build caching
COPY Cargo.toml Cargo.lock ./
COPY src/bin ./src/bin
USER root
RUN cargo fetch

# Copy rest of app
COPY . .

RUN /bin/sh -c 'echo "👀 Lint checks..." >&2'
RUN cargo fmt --check
RUN cargo clippy --release --all-targets --all-features -- -D warnings
RUN cargo audit || cargo outdated || true

RUN /bin/sh -c 'echo "🧪 Testing trait-based architecture (database-free)..." >&2'
RUN cargo test --release --bin cli --bin server
RUN cargo test --all-targets --all-features -- --nocapture

RUN /bin/sh -c 'echo "🛠️ Build binaries..." >&2'
RUN cargo build --release --bin server --bin cli
RUN strip target/release/server target/release/cli

# Stop here - no runtime images needed for this issue
# We will readd them in a later issue.

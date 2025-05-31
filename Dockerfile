# syntax=docker/dockerfile:1.4
FROM ghcr.io/johnbasrai/cr8s/rust-dev:1.83.0-rev5 AS builder
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

########################
# 🚀 Server image
########################
FROM ghcr.io/johnbasrai/cr8s/rust-runtime:0.1.3 AS runtime-server
RUN /bin/sh -c 'echo "🔨 Building runtime SERVER container" >&2'
WORKDIR /app
COPY .Rocket.toml.template /app/Rocket.toml
RUN sed -ie "s|%{REDIS_HOST}%|${REDIS_HOST}|g" \
         -e "s|%{DATABASE_HOST}%|${DATABASE_HOST}|g" /app/Rocket.toml
RUN cat /app/Rocket.toml && false
COPY --from=builder /app/target/release/server /app/server
USER appuser
ENTRYPOINT ["./server"]

########################
# 🛠 CLI image
########################
FROM ghcr.io/johnbasrai/cr8s/rust-runtime:0.3.1 AS runtime-cli
RUN /bin/sh -c 'echo "🔨 Building runtime CLI container" >&2'
WORKDIR /app
COPY --from=builder /app/target/release/cli /app/cli
USER appuser
ENTRYPOINT ["./cli"]

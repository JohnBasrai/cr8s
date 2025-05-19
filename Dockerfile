# syntax=docker/dockerfile:1.4

########################
# 🔧 Build stage
########################
ARG DEV_VERSION=1.83.0-rev3
ARG RUN_VERSION=0.1.3

FROM ghcr.io/johnbasrai/cr8s/rust-dev:${DEV_VERSION} AS cr8s-builder
ARG DEV_VERSION  # Must declare DEV_VERSION before/after FROM (Docker idiosyncrasy)

ARG DEBUG=0
ARG CI=false
ARG DATABASE_HOST=localhost
ARG REDIS_HOST=localhost

ENV DATABASE_URL=postgres://postgres:secret@${DATABASE_HOST}:5432/cr8s
ENV REDIS_URL=redis://${REDIS_HOST}:6379
ENV ROCKET_LOG=debug

# The echo of date makes it uncacheable so we can see the output on every one.
RUN if [ "$CI" = "true" ]; then sh -c 'echo "Running in CI : $(date)"'; fi

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src/bin ./src/bin
USER root
RUN cargo fetch

COPY . .

RUN diesel setup
RUN diesel migration run
RUN rustfmt src/schema.rs

RUN cargo fmt --check
RUN cargo clippy --all-targets --all-features -- -D warnings
RUN cargo audit || cargo outdated || true

RUN cargo build --release --bin server
RUN cargo build --release --bin cli

RUN target/release/cli roles init-defaults

RUN chmod +x scripts/start-backend.sh && scripts/start-backend.sh && \
    cargo test --release --all-targets --all-features || \
    pkill server || true

RUN pkill target/release/server || true
RUN strip target/release/server target/release/cli

########################
# 🚀 Server image
########################
FROM ghcr.io/johnbasrai/cr8s/rust-runtime:${RUN_VERSION} AS cr8s-server
ARG RUN_VERSION # see comment at top of file.

WORKDIR /app
COPY Rocket.toml .
COPY --from=cr8s-builder /app/target/release/server /app/server

USER appuser
ENTRYPOINT ["./server"]

########################
# 🛠 CLI image
########################
FROM ghcr.io/johnbasrai/cr8s/rust-runtime:${RUN_VERSION} AS cr8s-cli
ARG RUN_VERSION # see comment at top of file.

WORKDIR /app
COPY --from=cr8s-builder /app/target/release/cli /app/cli

USER appuser
ENTRYPOINT ["./cli"]

########################
# 🌱 Seeder container (based on CLI)
########################
FROM cr8s-cli AS cr8s-cli-seeder
ENTRYPOINT ["cli", "roles", "init-defaults"]

########################
# 🧪 Test runner container (based on dev)
########################
FROM ghcr.io/johnbasrai/cr8s/rust-dev:${DEV_VERSION} AS cr8s-test-runner

WORKDIR /app
COPY . .
ENV DATABASE_URL=postgres://postgres:secret@postgres:5432/cr8s
CMD ["cargo", "test", "--workspace", "--all-targets", "--all-features", "--locked"]

# Dockerfile
ARG PROFILE=release
FROM ghcr.io/johnbasrai/cr8s/rust-dev:1.83

WORKDIR /app
COPY . .

RUN cargo build --${PROFILE}

# Launch the Rocket backend (can override in Compose or CLI)
CMD ["./target/${PROFILE}/server"]

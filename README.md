# cr8s

Sample full‑stack **Rust** web service demonstrating clean architecture, trait-based design, SQLx/PostgreSQL, Redis, Docker, and automated CI.

---

## ✨ What's inside?

| Layer | Tech | Purpose |
|-------|------|---------|
| HTTP  | **Rocket 0.5** | Async web framework |
| DB    | **SQLx** + **PostgreSQL** | Async SQL with runtime verification |
| Cache | **Redis** | Session / ephemeral storage |
| Admin | CLI binary (`cargo run --bin cli`) | User management, DB setup, and admin utilities |
| Tests | `tokio`, `reqwest` | CLI & HTTP API integration tests with authentication |
| Dev   | **Docker Compose** | One‑command reproducible stack |
| CI    | **GitHub Actions** | Lint → migrate → build → test |

---

## 🏗️ Architecture

- **Domain-Driven Design** - Business logic separated from infrastructure
- **Trait-Based Abstractions** - Clean boundaries between layers
- **Repository Pattern** - Database access abstracted behind traits
- **Dependency Injection** - Components use trait objects, not concrete types
- **Multi-layered Testing** - Unit tests + integration tests for CLI & HTTP API

---

## 🛠️ Prerequisites

```text
Docker ≥ 24.x           # Engine
Docker Compose v2       # Already bundled with modern Docker
```

---

## 🚀 Getting Started

### Development Mode (WIP)

```bash
docker compose up -d postgres redis
cargo run                      # backend starts on :8000
```

### Initialize Database

See [docs/CR8S - Database Schema.md](docs/CR8S%20-%20Database%20Schema.md) for database setup instructions.

### With Frontend (`cr8s-fe`)

See the **cr8s-fe** repository for full-stack development instructions.

---

## 🧪 Testing

### Run All Tests
```bash
# Set up development environment
source scripts/dev-test-setup.sh

# Start services and run all tests
start-services && run-tests
```

### Individual Test Suites
```bash
run-cli-tests        # Test CLI commands
run-server-tests     # Test HTTP API endpoints
```

See [docs/development.md](docs/development.md) for detailed testing workflows.

---

## 📂 Project layout

```text
cr8s/
├── Cargo.toml                 # workspace + crate metadata
├── Rocket.toml.template       # config template with env substitution
├── Dockerfile                 # backend container (tests & prod)
├── docker-compose.yml         # Postgres + Redis + Rocket
│
├── src/                       # application code
│   ├── bin/                   # cli.rs, server.rs entry-points
│   ├── domain/                # business logic traits & models
│   ├── repository/            # SQLx implementations & database layer
│   ├── rocket_routes/         # HTTP handlers & REST API
│   ├── auth.rs                # authentication & password handling
│   ├── mail/                  # email service implementation
│   └── tests/                 # unit tests & architectural validation
│
├── tests/                     # integration tests
│   ├── cli_integration.rs     # CLI command testing
│   └── server_integration.rs  # HTTP API testing
│
├── templates/email/           # Tera email templates
├── scripts
│   ├── build-images.sh        # Builds cr8s server and cli Docker images
│   ├── build-verification-test.sh # Comprehensive system smoke test
│   └── sql
│       └── db-init.sql
└── docs/                      # Docker tips & native workflow
```

---

## Development

See [docs/development.md](docs/development.md) for detailed information about:
- CLI argument testing (with/without database)
- Route state analysis
- Local development setup

---

## 🧪 Continuous Integration

GitHub Actions runs the CI pipeline inside the `cr8s-dev` container, ensuring full parity with local development.

Non-gating advisory checks (e.g., `cargo audit`, `cargo outdated`) are also included for visibility.

---

MIT © 2025 John Basrai

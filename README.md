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
| Tests | `tokio`, `reqwest`, `sqlx` | Async/await integration tests with role-based auth |
| Dev   | **Docker Compose** | One‑command reproducible stack |
| CI    | **GitHub Actions** | Lint → migrate → build → test |

---

## 🏗️ Architecture

- **Domain-Driven Design** - Business logic separated from infrastructure
- **Trait-Based Abstractions** - Clean boundaries between layers
- **Repository Pattern** - Database access abstracted behind traits
- **Dependency Injection** - Components use trait objects, not concrete types
- **Clean Testing** - Comprehensive test coverage with mock implementations

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

### Initialize database schema

```bash
# Run using local cargo, prebuilt Docker, or Compose
cargo run --bin cli -- load-schema
docker compose run cli --rm load-schema
```

This executes `scripts/sql/db-init.sql` and inserts default roles.
Use `CR8S_DB_INIT_SQL=/path/to/alt.sql` to override the default file.

### With Frontend (`cr8s-fe`)

See the **cr8s-fe** repository for full-stack development instructions.

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
│   ├── auth/                  # authentication & password handling
│   ├── mail/                  # email service implementation
│   ├── mock/                  # test mocks & stubs
│   └── tests/                 # integration tests
│
├── templates/email/           # Tera email templates
├── scripts/                   # development & deployment scripts
│   ├── quickstart-dev.sh      # one-shot dev bootstrap
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
```

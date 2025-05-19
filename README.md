# cr8s

Sample full‑stack **Rust** web service demonstrating Rocket, Diesel/PostgreSQL, Redis, Docker, and automated CI.

---

## ✨ What’s inside?

| Layer | Tech | Purpose |
|-------|------|---------|
| HTTP  | **Rocket 0.5** | Async web framework |
| DB    | **Diesel v2** + **PostgreSQL** | Relational data model & migrations |
| Cache | **Redis** | Session / ephemeral storage |
| Admin | CLI binary (`cargo run --bin cli`) | Manage users & seed data |
| Tests | `tokio`, `reqwest`, `diesel_async` | Async/await integration tests with role-based auth and Diesel-backed setup |
| Dev   | **Docker Compose** | One‑command reproducible stack |
| CI    | **GitHub Actions** | Lint → migrate → build → test |

---

## 🛠️ Prerequisites

```text
Docker ≥ 24.x           # Engine
Docker Compose v2       # Already bundled with modern Docker
```
---

## 🚀 Quick Start Options

> 🐳 All development, build, and test workflows are containerized.  
> See [`docs/docker-usage.md`](docs/docker-usage.md) for developer tips, image usage, and CI details.

### 🧪 Development Mode (contributing to `cr8s`)

To contribute to `cr8s`, you’ll primarily work from source using containerized services.

```bash
docker compose down                   # Stop and clean up any existing services
docker compose up -d postgres redis   # Start Postgres and Redis
```

> 🧹 To remove cache volumes or reset the local DB, run:
> `docker compose down -v`

```bash
scripts/dev/build-images.sh [--debug] # Build all containers, seed DB, run tests
```

This builds:

* `cr8s-server`
* `cr8s-cli`
* `cli-seeder` (runs `roles init-defaults`)
* `test-runner` (executes all unit/integration tests)

All workflows match the CI pipeline. For details, see [`docs/docker-usage.md`](docs/docker-usage.md).

> 💡 To skip containers and run the backend directly from source:
>
> ```bash
> cargo run --bin server
> ```

---

## 🏃 Runtime Mode (used by `cr8s-fe` or external consumers)

Use this mode when running a precompiled backend container (e.g. in `cr8s-fe/ci` or for E2E tests). It uses a minimal runtime image that contains the release build of `cr8s`.

```bash
docker compose up -d postgres redis server
```

This will:
- Start Postgres, Redis and the backend server
- Expose the API on [http://localhost:8000](http://localhost:8000)

> This container does **not** run `diesel` migrations or seed the DB. External systems (like `cr8s-fe`) are responsible for that.

---

## 📂 Project layout

```text
.
├── Cargo.toml
├── Dockerfile
├── docker-compose.yml
├── README.md
├── Rocket.toml.template
├── src/
│   ├── bin/              # Entry points: cli.rs, server.rs
│   ├── rocket_routes/    # HTTP handlers
│   ├── tests/            # Integration tests
│   ├── lib.rs            # Shared core
│   └── schema.rs         # Diesel schema (generated)
├── templates/
│   └── email/
├── scripts/
│   ├── dev/              # Build/test utilities
│   └── run/              # Start/stop orchestration
└── docs/
    └── docker-usage.md
```

---

## 🧪 Continuous Integration

CI is run via GitHub Actions using the same containers built for development and production.

- Tests and linting are performed using the `test-runner` container
- Diesel migrations and database seeding use the built `cr8s-cli` binary
- Postgres and Redis are started as GitHub Actions `services`

> 🐳 Want to understand how this project is built and tested?  
> - See [`docs/docker-usage.md`](docs/docker-usage.md) for a high-level overview  
> - See [`rust.yml`](.github/workflows/rust.yml) for the exact CI pipeline

---

MIT © 2025 John Basrai

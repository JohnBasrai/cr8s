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
| Tests | `tokio`, `reqwest` | Integration tests hitting live server |
| Dev   | **Docker Compose** | One‑command reproducible stack |
| CI    | **GitHub Actions** | Lint → migrate → build → test |

---

## 🛠️ Prerequisites

```text
Docker ≥ 24.x           # Engine
Docker Compose v2       # Already bundled with modern Docker
```

> 📝 Prefer running everything natively? Check the community‑supported instructions in [`docs/native-workflow.md`](docs/native-workflow.md).

---

## 🚀 Quick start (Docker‑first workflow)

```bash
# 1. Clone & build
$ git clone https://github.com/JohnBasrai/cr8s.git && cd cr8s
$ docker compose build              # compiles Rust into the image

# 2. Launch backing services (Postgres & Redis)
$ docker compose up -d postgres redis

# 3. Initialize database (idempotent)
$ docker compose run --rm app diesel setup

# 4. Run test‑suite via dedicated runner (stops stack when done)
$ docker compose up --build --abort-on-container-exit test_runner

# 5. Tear everything down
$ docker compose down
```

*The `test_runner` service starts its own container, waits for the app server, and executes `cargo test -- --test-threads=1`. Logs from the server and the tests stay neatly separated.*

* **No host env‑vars needed** – the `app` service already sets `DATABASE_URL` and `ROCKET_DATABASES` in *docker-compose.yml*, so Diesel, Rocket, and the test runner all see the right values automatically.
* **Integrated server spin‑up** – the `test_runner` container starts Rocket on `127.0.0.1:8000` and the tests reach it via `reqwest`, exactly like in CI.
* **Expected result** – when the suite finishes you should see something like `ok. 2 passed; 0 failed` (or however many tests exist).

Need more Docker tips (stream logs, cURL pokes, DB maintenance)? See [`docs/docker-usage.md`](docs/docker-usage.md).

---

## 📂 Project layout

```
├── src/
│   ├── bin/
│   │   ├── server.rs        # Rocket entry‑point
│   │   └── cli.rs           # Maintenance CLI
│   └── lib.rs               # Shared domain logic
├── tests/                   # Integration tests
├── migrations/              # Diesel SQL migrations
├── Dockerfile               # Application image
├── docker-compose.yml       # Dev/CI stack definition
├── docs/
│   ├── docker-usage.md      # Extra Docker commands (optional)
│   └── native-workflow.md   # Community‑supported native setup
└── .github/workflows/
    └── rust.yml             # CI pipeline
```

---

## 🧪 Continuous Integration

1) GitHub Actions spins up Postgres & Redis service containers
2) installs `diesel_cli`
3) runs `diesel setup`
4) and executes the test runner

Clippy and rustfmt are gated with `-D warnings`, so the main branch stays clean.

---

MIT © 2025 John Basrai

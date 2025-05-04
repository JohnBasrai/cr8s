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

# 4. Run the test-suite (server + tests in same container)
$ docker compose run --rm --service-ports app \
    bash -c 'cargo run --bin server & sleep 3 && cargo test -- --test-threads=1'

# 5. Tear everything down
$ docker compose down
```

* **No host env‑vars needed** – the `app` service already sets `DATABASE_URL` and `ROCKET_DATABASES` in *docker-compose.yml*, so Diesel, Rocket, and the test runner all see the right values automatically.
* **Integrated server spin-up** – the test command starts Rocket (`cargo run --bin server & sleep 5`) on `127.0.0.1:8000`; the tests hit it via `reqwest`, exactly like in CI.
* **Expected result** – when the suite finishes you should see something like `ok. 2 passed; 0 failed` (or however many tests exist).

Need more Docker tips (stream logs, cURL pokes, DB maintenance)? See [`docs/docker-usage.md`](docs/docker-usage.md).

---

## 📂 Project layout

```
├── Cargo.toml
├── CHANGELOG.md
├── diesel.toml
├── docker-compose.yml
├── Dockerfile
├── docs
│   ├── docker-usage.md
│   └── native-workflow.md
├── LICENSE
├── README.md
├── src
│   ├── auth.rs
│   ├── bin
│   │   ├── cli.rs
│   │   └── server.rs
│   ├── commands.rs
│   ├── lib.rs
│   ├── mail.rs
│   ├── models.rs
│   ├── repositories.rs
│   ├── rocket_routes
│   │   ├── authorization.rs
│   │   ├── crates.rs
│   │   ├── mod.rs
│   │   └── rustaceans.rs
│   └── schema.rs
├── templates
│   └── email
│       └── digest.html
└── tests
    ├── authorization.rs
    ├── common
    │   └── mod.rs
    ├── crates.rs
    └── rustaceans.rs
```

---

## 🧪 Continuous Integration

1) GitHub Actions spins up Postgres & Redis service containers
2) installs `diesel_cli`
3) runs `diesel setup`
4) and runs cargo test against the live server.

Clippy and rustfmt are gated with `-D warnings`, so the main branch stays clean.

---

MIT © 2025 John Basrai

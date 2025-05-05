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

## 🚀 Quick start (Docker-first workflow)

```bash
# 0 .  Clone & build the image once
git clone https://github.com/JohnBasrai/cr8s.git && cd cr8s
docker compose build              # compiles the Rust workspace into the app image

# 1 .  Run the helper script – it does the rest in one shot
./scripts/quickstart.sh
````

`quickstart.sh` executes the same steps you would run manually:

1. `docker compose down -v` – start clean (containers + volumes)
2. `docker compose up -d postgres redis` – bring up Postgres & Redis
3. Wait until Postgres accepts TCP connections, then run
   `diesel setup` – create the database & apply migrations
4. Launch the Rocket server and run the integration-test suite
5. `docker compose down` – tear everything back down

### What to expect

* **Exit status** – returns `0` when every step succeeds (Bash’s `set -e` will surface any error with a non-zero code).
* **No host env-vars needed** – the `app` service injects `DATABASE_URL` and `ROCKET_DATABASES`, so Diesel, Rocket, and the tests “just work.”
* **Successful run** – you’ll see something like `result: ok. 6 passed; 0 failed;`, and your shell prompt will return with exit code 0.

Need more Docker tips (stream logs, cURL pokes, DB maintenance)? See [`docs/docker-usage.md`](docs/docker-usage.md).

---

<details>
<summary><strong>Local development (use with <code>cr8s-fe</code> frontend)</strong></summary>

```bash
cp Rocket.toml.sample Rocket.toml   # dev-only defaults
cargo run --release                 # backend starts on :8000
````

*(For the full two-terminal walkthrough—including the frontend steps—see the **cr8s-fe** README.)*

</details>


## 📂 Project layout

```text
cr8s/
├── Cargo.toml                 # workspace + crate metadata
├── Rocket.toml.sample         # dev-friendly DB urls
├── Dockerfile                 # backend container (tests & prod)
├── docker-compose.yml         # Postgres + Redis + Rocket
│
├── src/                       # application code
│   ├── bin/                   # cli.rs , server.rs entry-points
│   ├── rocket_routes/         # REST/HTTP handlers
│   ├── models.rs              # Diesel models
│   ├── schema.rs              # Diesel schema (generated)
│   └── lib.rs                 # library root (commands, auth, etc.)
│
├── templates/                 # Tera e-mail templates
├── migrations/                # Diesel SQL migrations
├── tests/                     # integration tests (HTTP & DB)
│
├── scripts/quickstart.sh      # one-shot dev bootstrap
└── docs/                      # Docker tips & native workflow
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

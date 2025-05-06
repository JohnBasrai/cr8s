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
# 0. Clone & build the image once
git clone https://github.com/JohnBasrai/cr8s.git && cd cr8s
docker compose build              # compiles the Rust workspace into the app image

# 1. Run the helper script – it does the rest in one shot
./scripts/quickstart.sh
```

`quickstart.sh` performs:

1. `docker compose down -v` – clean up containers and volumes
2. `docker compose up -d postgres redis` – launch DB dependencies
3. Waits for Postgres to accept connections
4. Runs `diesel setup` and `diesel migration run` to initialize the database
5. Creates a default admin user via the CLI:
   `cargo run --bin cli -- users create admin@example.com password123 admin`
6. Starts the backend app container (Rocket on port 8000)
7. ✅ Done! The backend is now ready for use with the frontend
> ⚠️ **Note for Linux users:**  
> When using Docker with volume mounts, you may find that the `target/` directory becomes owned by `root`.  
> This is because the container runs as root and writes to the mounted volume.  
> If needed, you can clean it up with:
>
> ```bash
> sudo rm -rf target/
> ```
>
> This doesn't affect the runtime or correctness, but may interfere with local tools that expect to write to `target/`.

---

## Local development (use with <code>cr8s-fe</code> frontend)
```bash
cargo run                      # backend starts on :8000
```
*(For the full two-terminal walkthrough—including the frontend steps—see the **cr8s-fe** README.)*


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

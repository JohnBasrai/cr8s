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

## 🚀 Quick Start Options

### 🧪 Development Mode (contributing to `cr8s`)

Use this mode when developing or debugging `cr8s` itself. It launches Postgres, Redis, and a fully-featured development container (`cr8s-dev`) with the full Rust toolchain, Diesel CLI, and project source code mounted.

```bash
# One command to set up and seed the database
./scripts/quickstart-dev.sh

# Then start the backend manually (optional)
docker compose exec dev cargo run
```

---

### 🏃 Runtime Mode (used by `cr8s-fe` or external consumers)

Use this mode when running a precompiled backend container (e.g. in `cr8s-fe/ci` or for E2E tests). It uses a minimal runtime image that contains the release build of `cr8s`.

```bash
docker compose up -d postgres redis backend
```

This will:
- Start Postgres and Redis services
- Launch the `backend` container using `ghcr.io/johnbasrai/cr8s/rust-runtime:latest`
- Expose the API on [http://localhost:8000](http://localhost:8000)

> This container does **not** run `diesel` migrations or seed the DB. External systems (like `cr8s-fe`) are responsible for that.

---

## Local development (use with <code>cr8s-fe</code> frontend)

```bash
cargo run                      # backend starts on :8000
```

*(For the full two-terminal walkthrough—including the frontend steps—see the **cr8s-fe** README.)*

---

### 🛠 Advanced: Editor-Friendly Dev Container (Emacs / VS Code)

If you're using Emacs or VS Code and need precise file path alignment for error navigation or stack traces, you can launch the `cr8s-dev` container interactively:

```bash
./scripts/start-rust-dev
```

<details>
<summary>📘 Why this helps (click to expand)</summary>

This script:

- Mounts your current directory into the container at the **same absolute path**
  (`-v "$PWD:$PWD" -w "$PWD"`)
  - This makes compiler errors and backtraces use real host paths, so:
    - ✅ Both Emacs and VS Code can follow file paths when parsing compilation output
- Launches an interactive Bash shell using the `cr8s-dev` container
- Ensures `cargo`, `diesel`, and `rustfmt` have full access to the workspace
- Uses your host UID and GID (via `-u $(id -u):$(id -g)`) to ensure any files created 
  inside the container (e.g., `./target/`) are not owned by root.

However, if your host UID does not match a named user in the container (like `johnb`), you may see this in the shell prompt:

```
I have no name!@0803495724cd:cr8s $ 
```
This is harmless — all tools still work. It simply means the UID exists but has no matching entry in /etc/passwd. You can safely ignore it.

---

#### 🧪 Emacs Example

```emacs
M-x compile RET cargo build --release
```

This allows Emacs to highlight compiler errors and navigate to the correct files.

#### 🧪 VS Code Use

Use with [Remote Containers](https://code.visualstudio.com/docs/remote/containers) or terminal-based workflows. Editor features like go-to-definition, error overlays, and task runners will behave as expected.

</details>

---

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

## 🧪 Continuous Integration

GitHub Actions runs the CI pipeline inside the `cr8s-dev` container, ensuring full parity with local development.

1. Spins up Postgres and Redis as service containers
2. Runs Diesel migrations using the built-in `diesel` CLI
3. Seeds the database with a default admin user using the CLI binary
4. Lints with `cargo fmt` and `cargo clippy` (gated via `-D warnings`)
5. Runs `cargo test` against the live backend

Non-gating advisory checks (`cargo audit`, `cargo outdated`) are also included for visibility.

---

MIT © 2025 John Basrai

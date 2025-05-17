# Changelog

All notable changes to **cr8s** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## \[Unreleased]
---

## [v0.3.1] – 2025-05-17
### Added
- `Rocket.toml.template` with `%{VAR}%` placeholders for dynamic build-time config
- `scripts/build-images.sh` for dev + CI container builds
- `scripts/start-backend.sh` for launching backend server inside Docker build
- Redis `/ping` route for CI and readiness testing
- Docker Compose healthchecks for Redis and cr8s-server (/health)

### Changed
- `Rocket.toml` is now generated at build time and ignored by Git
- CI now builds, tests, and verifies images with live server integration
- `docker-compose.yml` updated to support clean dependency start via `depends_on`
- `start.sh` and `stop.sh` simplified to use `docker compose up/down`

### Fixed
- `DATABASE_URL` and `REDIS_URL` substitution bugs in CI
- Redis misconfig with `connection_info` replaced by working `url =` form
- Integration tests now pass consistently with backend running during `cargo test`

### Removed
- Deprecated `.Rocket.toml.template` (inlined into `Rocket.toml.template`)
- Obsolete cr8s-dev container logic from `start.sh`, and `stop.sh`
- Deleted obsolete `shell.sh`

---
## [0.3.0] – 2025-05-14

### Added
- /health endpoint with Redis check, returns 200/"OK" 
- Migrated integration tests under `src/tests/` for tighter crate-level access and test visibility
- `test_utils::common` helpers for:
  - database setup, role assignment, user login
  - reusable `unique_username()` generator
- Role-based test clients using `Editor`, `Viewer`, `Admin` variants
- Added `ensure_status!` macro for consistent API test assertions
- Enabled `tracing` logs in Rocket server and test startup
- Introduced `test_support.rs` with helpers:
  - `establish_test_connection`, `insert_test_user`, `assign_role`
- `scripts/start-rust-dev`: launches interactive `cr8s-dev` container with host path mapping and correct user permissions
  - Supports Emacs `M-x compile`, VS Code navigation, and safe `./target` ownership
- `README.md`: new expandable documentation for editor-integrated development using `start-rust-dev`
- `quickstart-dev.sh`: one-shot setup script for spinning up Postgres, Redis, and `cr8s-dev`
  - Runs `diesel setup`, `diesel migration run`, and seeds a default admin user via CLI
- `backend` service in `docker-compose.yml` using prebuilt `cr8s/rust-runtime` image (for `cr8s-fe`)
- `dev` service in `docker-compose.yml` using `Dockerfile.dev` for internal development and CI
- `restart: unless-stopped` and Postgres healthchecks added to improve service resilience
- Non-gating CI steps for advisory checks:
  - `cargo audit` to detect known security vulnerabilities
  - `cargo outdated` to identify outdated dependencies
  - These steps do not block PRs but provide visibility into project health

### Changed
- CI workflow now waits for Rocket server readiness using curl -sf /health instead of HTML grep.
- Added redis = "0.25" explicitly to Cargo.toml to enable use of cmd("PING").
- Replaced CLI-based test setup with direct Diesel calls
- All tests use unique users per role to prevent conflicts in suite runs
- Removed `tests/` directory and old `common/mod.rs`, `test_support.rs`, `authorization.rs`
- Upgraded testing infrastructure to use async/await pattern
- Improved error handling with contextualized messages
- Enhanced test reliability with proper cleanup and setup
- Fixed module structure and imports for easier maintenance
- Added macros for common testing patterns
- Converted assertions to use ensure! for better diagnostics
- Renamed table `users_roles` → `user_roles` in: SQL schema & updated code to match
- Removed unused `ctor` setup and `Command::new("cli")` logic from tests
- Updated `NewUser` model usage to match schema
- Replaced CLI-based user setup in tests with direct Diesel calls
- Removed legacy `app` service from `docker-compose.yml`.
- Simplified environment config: replaced deprecated `ROCKET_PROFILE` with `ROCKET_ENV`.
- Removed unused `CR8S_APP_MODE` toggle pending re-evaluation.
- Standardized Postgres connection details to match `Rocket.toml` and runtime 
  container expectations.
- Internal workflows and CI are now expected to use the `dev` container instead of the
  legacy `app` service.
- Seeding logic is centralized into `quickstart-dev.sh` to ensure consistent DB
  initialization across environments.

### Removed
- Legacy test support module `test_support.rs`
- Commented/unused test CLI login helpers

---
## \[0.2.1] - 2025-05-06

### Added

- Added Docker volume ownership warning to README
- quickstart.sh now copies Rocket.toml if missing

### Changed
- Renamed Rocket.toml.sample → .Rocket.toml.template

---

## \[0.2.0] - 2025-05-04

### Added

- **GitHub Actions** workflow (`.github/workflows/rust.yml`) with Postgres + Redis
  service containers, Diesel migrations, lint, build, and test steps.
- Docker‑first quick‑start and expanded docs in `README.md`.
- `docs/docker-usage.md`: cheat‑sheet of common Docker Compose commands.
- `docs/native-workflow.md`: step‑by‑step native (non‑Docker) setup guide.
- `CR8S_APP_MODE` environment variable to toggle debug/release in dev
- Full error logging in `server_error()` for better diagnostics without
  leaking internal errors to clients


### Changed

- `docker-compose.yml`: removed deprecated `version:` key. Tests now run
  in the same `app` container.
- Enabled Tokio **macros** and **rt‑multi‑thread** features in `Cargo.toml`.
- Replaced hand‑written `ToString` with `Display` for `RoleCode`.
- Ran `cargo fmt` and fixed assorted Clippy warnings.
- Login route now returns **401 Unauthorized** on bad credentials.
- `quickstart.sh` now waits for Postgres, runs migrations, seeds admin user,
  and starts the backend
- `README.md` updated with clearer Docker-first quickstart instructions


### Fixed

- CI reproducibility: no host env‑vars required; tests run against the
  in‑container server on `127.0.0.1:8000`.
- Unified database hostnames across `Rocket.toml` and `docker-compose.yml`
  to support Dockerized development
- Fixed service resolution failures between containers (e.g. Redis, Postgres)
  via health checks and `depends_on`

### Cleanup
- Removed stale feature branches
- Tagged backup of `feat/rocket-toml-dev-defaults` before deletion

### Security

- Added `.cargo/audit.toml` to temporarily ignore **RUSTSEC‑2024‑0365** (Diesel
  2.1 advisory). Will upgrade to Diesel 2.2 when `rocket_db_pools` and
  `diesel‑async` publish compatible releases.
  [#4]: https://github.com/JohnBasrai/cr8s/issues/4


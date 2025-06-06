# Changelog

All notable changes to **cr8s** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## \[Unreleased]

## [0.5.1] - 2025-06-06

### Added
- **Root Landing Page**: Added missing root endpoint (`/`) to resolve 404 errors when accessing the server base URL
  - Provides basic server information and API endpoint discovery
  - Improves developer experience for API exploration

### Fixed
- **Docker Compose Health Check**: Corrected server health check endpoint in `docker-compose.yml`
  - Fixed invalid health check path that was preventing proper container health monitoring
  - Server health checks now properly validate service availability during startup

---

## [0.5.0] - 2025-06-06

### Added
- **Server Integration Testing**: Complete HTTP API testing suite with 4 comprehensive tests
  - Login API validation (mirrors Playwright authentication flow in cr8s-fe)
  - Rustacean (author) creation testing with authentication
  - Crate creation workflow testing
  - Authentication guard integration testing (validates full Rocket guard flow)
- **Enhanced Development Workflow**: Updated `dev-test-setup.sh` with server testing functions
  - `run-server-tests` - Run HTTP API integration tests
  - `run-single-server-test` - Run specific server test
  - `run-tests` - Run both CLI and server tests
  - `check-server` - Server health and log utilities
  - Docker compose aliases (`dc`, `dcr`) for faster workflows
- **Guard Integration Testing**: Comprehensive authentication/authorization flow validation
  - Bearer token extraction and validation
  - Redis session management testing
  - Database role lookup verification
  - Consistent auth enforcement across endpoints
- **Documentation Overhaul**: 
  - Updated architecture documentation with testing strategy
  - Enhanced database schema documentation
  - Improved development workflow documentation
  - Removed outdated mock system references

### Changed
- **Testing Architecture**: Multi-layered approach with complementary unit and integration tests
- **Documentation Strategy**: Eliminated redundancy between README and detailed docs
- **Development Experience**: Streamlined commands and better error handling

### Removed
- **Mock Infrastructure**: Removed unused `src/mock/` directory and references
- **Redundant Documentation**: Cleaned up duplicate information across docs

### Fixed
- **Local CLI Commands**: Corrected environment variable requirements in documentation
- **Schema Documentation**: Fixed missing `CR8S_DB_INIT_SQL` path requirement
- **Architecture References**: Updated outdated module structure references

### Infrastructure
- **Integration Test Coverage**: Server tests now validate the same API flows that Playwright tests depend on
- **Development Environment**: Enhanced with server testing capabilities and better health checks
- **Documentation Accuracy**: All docs now reflect current codebase structure

---

## [0.4.8] - 2025-06-05

### Added
- **Integration Testing Framework**: Added comprehensive CLI integration tests (`cli_integration.rs`) that validate complete user management workflows against live Docker services
- **Development Environment Setup**: New `dev-test-setup.sh` script providing interactive development environment with helper functions for testing and debugging
- **CI Integration Testing**: Enhanced GitHub Actions workflow to run full integration tests as part of the build pipeline, ensuring CLI commands work correctly in containerized environments

### Improved
- **Build Script Enhancement**: Updated `build-images.sh` with better dev mode support and clearer output formatting
- **Development Workflow**: Streamlined local development with easy-to-use commands for starting services, running tests, and debugging
- **Quality Assurance**: Added end-to-end validation covering schema management, user operations, role validation, and cleanup verification

### Technical Details
- Integration tests cover complete CLI workflow: `load-schema`, `create-user`, `list-users`, `user-exists`, `delete-user-by-name`, and error handling
- Development environment provides shell functions: `start-services`, `stop-services`, `run-tests`, `run-single-test`, `test-cli`, `show-logs`, `restart-server`
- CI pipeline now validates both unit tests and integration tests before publishing container images
- Test execution time optimized to ~14 seconds through efficient Docker layer caching

### Developer Experience
- Added visual prompt indicators showing development environment status (`cr8s-dev:running`, `cr8s-dev:stopped`)
- Comprehensive error handling and cleanup in both local and CI environments
- Clear documentation and helper text for all available development commands

---

## [0.4.7] - 2025-06-04

### Added
- New CLI command: `load-schema`
  - Loads database schema and default roles from a SQL file (`scripts/sql/db-init.sql`)
  - Path can be overridden with the `CR8S_DB_INIT_SQL` environment variable
  - Automatically initializes the database connection pool before execution
  - Replaces the deprecated `init-default-roles` command

---

## [0.4.6] - 2025-06-04

### Fixed
- **BREAKING**: Mount API routes under `/cr8s` prefix (requires frontend updates)

### Added  
- Enhanced development workflow with `--dev` build mode for faster iterations
- Improved server debugging and startup inspection capabilities

---
## [0.4.5] - 2025-06-04

### Fixed
- Fixed EditorUser guard failing with SQLx type mismatch error
- POST/PUT requests to /cr8s/rustaceans now work properly for Editor users
- Role authorization now correctly fetches all user roles from database

---
## [0.4.4] - 2025-06-03

### Fixed
- **CRITICAL:** Fixed EditorUser guard authorization logic that was preventing editor operations
  - `is_editor()` method now properly queries user roles instead of always returning false
  - Resolves 403 Forbidden errors when creating/editing rustaceans and crates via cr8s-fe
  - Restores role-based access control functionality lost during Diesel→SQLx migration

### Added
- Comprehensive unit tests for role-based authorization (8 new test cases)
- Test-friendly default implementations for administrative methods in `AppUserTableTrait`
- Macro-based test utilities to reduce boilerplate in guard testing

### Improved
- Enhanced CLI role parsing with shortcuts (a/e/v for admin/editor/viewer)
- Better error messages and case-insensitive role handling
- Cleaner test architecture with focused mocks and proper error handling

### Technical Details
- Fixed async method signatures in `GuardedAppUser::is_editor()` and `is_admin()`
- Added proper trait bounds for `AppUserTableTraitPtr` in test contexts
- Improved guard implementation to use repository pattern for role lookups

---
## [0.4.3] - 2025-06-02

### Fixed
- **Database Schema**: Removed incorrect `user_id` column references from author table operations
- **API Endpoints**: Fixed 500 Internal Server Error on `/rustaceans` endpoint
- **Repository Layer**: Corrected `AuthorRepo` SQL queries to match actual database schema
- **Domain Model**: Aligned author domain model with intended one-way relationship design (`app_user.author_id` → `author.id`)

### Technical Details
- Removed `user_id` field from `AuthorRow` struct and all related SQL queries
- Fixed `find_multiple`, `find`, `create`, `update` methods in `AuthorTableTrait` implementation
- Maintained correct schema design where authors can exist independently of user accounts
- Eliminated circular reference between `app_user` and `author` tables

---
## [v0.4.2] – 2025-05-31

### Added
- **Documentation**: Container Usage Guide for deployment and development workflows
- **Documentation**: Development guide (development.md) with setup and contribution instructions
- **Security**: cargo audit integration with documented RUSTSEC-2023-0071 exception (MySQL RSA vulnerability not applicable to PostgreSQL/Redis stack)
- **CI/Development**: build-verification-test.sh script for automated testing in CI and local development

### Changed
- **Docker Build Optimization**: Dramatically improved build cache performance by separating dependency and source file copying, reducing incremental build times from minutes to seconds when only source code changes
- **Dockerfile Structure**: Reorganized build layers for optimal caching - dependency fetching now only runs when Cargo.toml/Cargo.lock change
- **Project Organization**: Archived legacy scripts to archive/ directory (no longer in use)
- **Code Quality**: Fixed multiple EMBP (Explicit Module Boundary Pattern) violations for better module organization

### Fixed
- **Database**: Resolved SQLx database logic issues
- **Server**: Fixed server startup issues and improved reliability  
- **CLI**: Multiple bug fixes and stability improvements
- **Build Performance**: Eliminated redundant Docker file copies and improved layer ordering

---
## [v0.4.1] – 2025-05-29

### Added
- Domain trait abstractions for repository pattern implementation (#22)
- Comprehensive binary test coverage: 27 tests total (19 CLI + 8 server)
- RedisCacheContext trait with async Redis operations and connection retry logic
- Server binary diagnostic functions for Rocket route/state analysis
- CLI support for --help without requiring DATABASE_URL environment variable
- Unified HealthTrait abstraction for /health endpoint
- EMBP (Explicit Module Boundary Pattern) applied where possible, more
  work remains

### Changed
- Added EMBP (Explicit Module Boundary Pattern) to architectural models
- Updated architecture documentation to reflect SQLx migration
- Corrected repository layer documentation from Diesel to SQLx references
- Enhanced separation of concerns section with detailed module responsibilities
- **BREAKING**: Complete migration from Diesel to SQLx with trait-based repositories
- Refactored CLI binary to use domain traits instead of direct database calls
- Restructured server binary: main.rs (entry point) + server.rs (implementation)
- Improved error handling with anyhow::Result throughout CLI and server binaries
- Enhanced user feedback with emoji status indicators and descriptive error messages
- Updated module structure using standard Rust patterns instead of EMBP for binaries

### Fixed
- Module import resolution issues in server binary ("too many leading super keywords")
- CLI argument parsing for negative user IDs with allow_hyphen_values
- Redis pool accessor functions for health check services
- Database connection dependency in CLI help commands

### Removed
- Direct Diesel database dependencies in CLI commands
- Legacy Redis-specific health check implementation
- init_default_roles seeding logic (replaced with "not supported" error for compatibility)

### Technical Debt
- Prepared foundation for MockUserRepo and MockCacheContext implementation
- Ready for database-free testing as outlined in GitHub issue #22

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


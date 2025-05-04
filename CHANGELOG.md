# Changelog
All notable changes to **cr8s** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **GitHub Actions** workflow (`.github/workflows/rust.yml`) with Postgres + Redis
  service containers, Diesel migrations, lint, build, and test steps.
- Docker-first quick-start and expanded docs in `README.md`.
- `docs/native-workflow.md` (community-supported native setup).

### Changed
- `docker-compose.yml`: removed deprecated `version:` key and prepared
  `test_runner` service for cleaner test logging.
- Enabled `tokio` **macros** and **rt-multi-thread** features in `Cargo.toml`.
- Replaced hand-written `ToString` with `Display` for `RoleCode`.
- Ran `cargo fmt` and fixed assorted Clippy warnings.
- Login route now returns **401 Unauthorized** for invalid credentials.

### Fixed
- CI reproducibility: no host env-vars required; tests run against the
  in-container server on `127.0.0.1:8000`.

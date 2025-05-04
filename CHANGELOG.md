# Changelog

All notable changes to **cr8s** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## \[Unreleased]

*No changes yet*

---

## \[0.2.0] - 2025-05-04

### Added

* **GitHub Actions** workflow (`.github/workflows/rust.yml`) with Postgres + Redis
  service containers, Diesel migrations, lint, build, and test steps.
* Docker‑first quick‑start and expanded docs in `README.md`.
* `docs/docker-usage.md`: cheat‑sheet of common Docker Compose commands.
* `docs/native-workflow.md`: step‑by‑step native (non‑Docker) setup guide.

### Changed

* `docker-compose.yml`: removed deprecated `version:` key. Tests now run
  in the same `app` container.
* Enabled Tokio **macros** and **rt‑multi‑thread** features in `Cargo.toml`.
* Replaced hand‑written `ToString` with `Display` for `RoleCode`.
* Ran `cargo fmt` and fixed assorted Clippy warnings.
* Login route now returns **401 Unauthorized** on bad credentials.

### Fixed

* CI reproducibility: no host env‑vars required; tests run against the
  in‑container server on `127.0.0.1:8000`.

### Security

* Added `.cargo/audit.toml` to temporarily ignore **RUSTSEC‑2024‑0365** (Diesel
  2.1 advisory). Will upgrade to Diesel 2.2 when `rocket_db_pools` and
  `diesel‑async` publish compatible releases.
  [#4]: https://github.com/JohnBasrai/cr8s/issues/4


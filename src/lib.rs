//src/lib.rs
//! # cr8s – Core Library
//!
//! This crate defines the core logic shared by both the Rocket server and CLI binaries.
//! It acts as the public interface for business logic, domain models, and application services.
//!
//! ## Structure
//! - `domain/`     – Domain models and trait-based business logic
//! - `repository/` – SQLx-backed implementations for Postgres and Redis
//! - `auth.rs`     – Password hashing and authentication helpers
//! - `mail/`       – Email formatting and delivery logic
//!
//! ## Consumers
//! - `bin/server/` – Rocket-based HTTP API
//! - `bin/cli/`    – CLI entry point for user management and admin tools
//!
//! For architectural guidance, see `docs/development.md`

pub mod auth;

// --- Domain traits and types (pure interfaces) ---
pub mod domain;

// --- Mail delivery mechanisms ---
pub mod mail;

// --- Repository: Diesel-backed data layer ---
pub mod repository;

// Contains Diesel-backed implementations and models.
pub mod rocket_routes;

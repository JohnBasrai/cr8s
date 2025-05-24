//src/lib.rs

//! CR8S core library crate.
//!
//! This file wires together the module tree and exposes the public API surface for integration,
//! tests, and binary crates. It enforces the architectural layering between domain, service,
//! and infrastructure.

// --- Auth and password hashing strategies ---
pub mod auth;

// --- Domain traits and types (pure interfaces) ---
pub mod domain;

// --- Mail delivery mechanisms ---
pub mod mail;

// --- Mocks
pub mod mock;

// --- Repository: Diesel-backed data layer ---
pub mod repository;

// Contains Diesel-backed implementations and models.
pub mod rocket_routes;

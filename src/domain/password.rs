//! Domain-layer abstraction for password-based authentication.
//!
//! This module defines the `PasswordHasherTrait` and related types.
//!
//! - Interface-only: contains no concrete hashing logic or dependencies like
//!   `argon2`, `rand`, etc.
//! - Designed for testability: can be mocked with in-memory or static implementations.
//! - Intended for use across boundaries: CLI handlers, HTTP routes, or internal services.

use anyhow::Result;

/// Trait for password-based authentication behavior.
pub trait PasswordHasherTrait: Send + Sync {
    // ---
    /// Hash a plaintext password.
    fn hash_password(&self, password: &str) -> Result<String>;

    /// Verify a candidate password against a stored hash.
    fn verify_password(&self, hashed: &str, candidate: &str) -> Result<()>;

    /// Generate a random session token (e.g. for login state).
    fn generate_session_token(&self) -> String;
}

/// Shared pointer type for injecting a PasswordHasherTrait object.
pub type PasswordHasherTraitPtr = std::sync::Arc<dyn PasswordHasherTrait>;

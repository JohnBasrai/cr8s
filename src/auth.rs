//! Authentication utilities for password hashing and verification.
//!
//! Provides a default implementation of the `PasswordHasherTrait`
//! and exposes a constructor for use in both CLI and server contexts.

use crate::domain::{PasswordHasherTrait, PasswordHasherTraitPtr};
use anyhow::Result;
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use rand::prelude::*;
use std::sync::Arc;

struct Argon2PasswordHasher;

/// Return a default password hasher implementation using Argon2id.
pub fn create_password_hasher() -> Result<PasswordHasherTraitPtr> {
    // ---
    Ok(Arc::new(Argon2PasswordHasher {}))
}

const SESSION_TOKEN_LEN: usize = 128;

impl PasswordHasherTrait for Argon2PasswordHasher {
    // ---

    fn hash_password(&self, password: &str) -> Result<String> {
        let salt = SaltString::generate(OsRng);
        let hashed = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(anyhow::Error::msg)?
            .to_string();
        Ok(hashed)
    }

    // ---

    fn verify_password(&self, hashed: &str, candidate: &str) -> Result<()> {
        let parsed_hash = PasswordHash::new(hashed).map_err(anyhow::Error::msg)?;
        Argon2::default()
            .verify_password(candidate.as_bytes(), &parsed_hash)
            .map_err(anyhow::Error::msg)?;
        Ok(())
    }

    // ---

    fn generate_session_token(&self) -> String {
        // ---
        rand::rng()
            .sample_iter(&rand::distr::Alphanumeric)
            .take(SESSION_TOKEN_LEN)
            .map(char::from)
            .collect()
    }
}

//! Unified trait for cache-backed session/context access.
//!
//! This trait abstracts over Redis or any other ephemeral key-value store
//! used for authentication, sessions, or token-based state.
//!
//! Implementations should live in `repository/` and be injected here
//! via factory functions like `create_cache_context()`.

use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

/// Abstract trait over session/token cache behavior.
#[async_trait]
pub trait CacheContextTrait: Send + Sync {
    /// Validate a session token and return the associated user ID, or `None` if invalid.
    async fn get_user_id_by_session_token(&self, token: &str) -> Result<Option<i32>>;

    /// Write a new session token for a user (or update existing one).
    async fn set_user_session_token(&self, user_id: i32, token: &str) -> Result<()>;

    /// Delete a session token (e.g., logout), return `true` if found else `false.
    async fn clear_session_token(&self, token: &str) -> Result<bool>;
}

/// Shared trait object pointer for any cache context implementation.
pub type CacheContextTraitPtr = Arc<dyn CacheContextTrait>;

// --- Factory re-export

pub use crate::repository::create_cache_context;

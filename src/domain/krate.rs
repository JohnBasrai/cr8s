// src/domain/krate.rs
//! Domain trait and summary struct for crate (package) metadata access.
//!
//! This trait abstracts over the storage mechanism for retrieving crate summaries.
//! Used in services like `digest_send()` to decouple from the database backend.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// A simplified projection of a crate used in digest emails or summaries.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CrateSummary {
    pub name: String,
    pub version: String,
}

#[async_trait]
pub trait CrateTableTrait: Send + Sync {
    // ---
    /// Fetch a list of crates up to max limit.
    async fn find_multiple(&self, limit: i64) -> Result<Vec<Crate>>;

    /// Find a single crate by its ID.
    async fn find(&self, id: i32) -> Result<Crate>;

    /// Create a new crate.
    async fn create(&self, new: NewCrate) -> Result<Crate>;

    /// Update an existing crate by ID.
    async fn update(&self, id: i32, current_version: i32, updated: NewCrate) -> Result<Crate>;

    /// Delete a crate by ID.
    async fn delete(&self, id: i32) -> Result<()>;

    /// Return crate summaries modified within the last N hours.
    async fn find_since(&self, hours_since: i32) -> Result<Vec<CrateSummary>>;
}

/// Shared pointer to trait object (dyn dispatch).
pub type CrateTableTraitPtr = std::sync::Arc<dyn CrateTableTrait>;
pub use crate::repository::create_crate_repo;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Crate {
    pub id: i32,
    pub author_id: i32,
    pub code: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub row_version: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NewCrate {
    pub author_id: i32,
    pub code: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub row_version: i32,
}

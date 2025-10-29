// src/domain/author.rs

//! Domain-facing interface and types for authors.
//!
//! Authors are linked to published crates and may optionally be linked to app users.

use anyhow::Result;
use async_trait::async_trait;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Author {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub created_at: NaiveDateTime,
    pub row_version: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NewAuthor {
    pub name: String,
    pub email: String,
}

// Trait alias for shared trait objects
pub type AuthorTableTraitPtr = Arc<dyn AuthorTableTrait + Send + Sync>;

/// Domain-level trait to abstract all author operations.
#[async_trait]
pub trait AuthorTableTrait: Send + Sync {
    // ---
    async fn find(&self, id: i32) -> Result<Author>;
    async fn find_multiple(&self, limit: i64) -> Result<Vec<Author>>;
    async fn create(&self, author: NewAuthor) -> Result<Author>;
    async fn update(&self, id: i32, current_version: i32, author: Author) -> Result<Author>;
    async fn delete(&self, id: i32) -> Result<()>;
}

pub use crate::repository::create_author_repo;

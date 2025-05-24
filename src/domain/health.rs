// domain/health.rs
//! Unified trait for infrastructure health checks.
//!
//! Enables components to expose a consistent `health_check()` method, regardless of
//! underlying system or service.  A consistent liveness API via `health_check()`,
//! regardless of backend.
//!
//! Factory functions are re-exported from the repository layer and are
//! backend-agnostic.

use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

/// A unified trait for infrastructure health checks.
#[async_trait]
pub trait HealthTrait: Send + Sync {
    async fn health_check(&self) -> Result<()>;
}

/// Type alias for any backend that implements HealthTrait.
pub type HealthTraitPtr = Arc<dyn HealthTrait>;

// --- Factory re-exports

pub use crate::repository::{
    // ---
    create_cache_health_service,
    create_database_health_service,
};

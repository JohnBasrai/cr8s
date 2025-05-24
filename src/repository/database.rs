// repository/database.rs
//! Internal SQLx Postgres connection pool for use inside `repository/` only.

use anyhow::{anyhow, Result};
use once_cell::sync::OnceCell;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration as tokio_duration;

static POOL: OnceCell<PgPool> = OnceCell::new();

/// Initialize the DB connection pool with retry logic.
///
/// Respects env vars:
/// - `CR8S_DB_RETRY_COUNT` (default: 50)
/// - `CR8S_DB_RETRY_DELAY_SECS` (default: 1)
pub async fn init_database_with_retry_from_env() -> Result<()> {
    // ---
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    tracing::info!("🚨 Rocket attaching to database at: {:?}", url);

    let retry_max = crate::get_env_with_default!(u32, "CR8S_DB_RETRY_COUNT", 50);
    let delay_sec = crate::get_env_with_default!(u64, "CR8S_DB_RETRY_DELAY_SECS", 1);

    for attempt in 1..=retry_max {
        // ---
        match PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(tokio_duration::from_secs(delay_sec))
            .connect(&url)
            .await
        {
            Ok(pool) => {
                POOL.set(pool)
                    .map_err(|_| anyhow!("Pool already initialized"))?;
                return Ok(());
            }
            Err(e) if attempt == retry_max => {
                return Err(anyhow!(
                    "Failed to connect to DB after {} retries: {}",
                    retry_max,
                    e
                ));
            }
            Err(_) => {
                let backoff_secs =
                    tokio_duration::from_secs(std::cmp::min(2u64.pow(attempt - 1), 8));
                tracing::warn!(
                    "DB not ready (attempt {}/{}) — retrying in {}s...",
                    attempt,
                    retry_max,
                    backoff_secs.as_secs()
                );
                tokio::time::sleep(backoff_secs).await;
            }
        }
    }
    unreachable!("Exhausted retries should already have returned above")
}

pub(crate) fn get_pool() -> &'static PgPool {
    // ---
    POOL.get()
        .expect("Pool not initialized. Call init_pool_with_retry() first.")
}

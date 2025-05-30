// repository/redis_cache.rs
//! Internal Redis connection pool for use inside `repository/` only.

use crate::domain::{CacheContextTrait, CacheContextTraitPtr};
use anyhow::{anyhow, Context, Result};
use deadpool_redis::{redis::AsyncCommands, Connection, Pool}; // from explicit `redis = "0.25.4"` dependency
use once_cell::sync::OnceCell;
use std::time::Duration;
use tokio::time::sleep;

static REDIS_POOL: OnceCell<Pool> = OnceCell::new();

/// Internal accessor for Redis pool (must be initialized first).
pub fn get_redis_pool() -> &'static Pool {
    // ---
    REDIS_POOL
        .get()
        .expect("Redis pool not initialized. Call init_redis_cache_with_retry_from_env() first.")
}

/// Internal accessor for a Redis connection (requires pool to be initialized).
pub async fn get_redis_connection() -> Result<Connection> {
    // ---
    let pool = REDIS_POOL
        .get()
        .context("Redis pool not initialized. Call init_redis_with_retry() first.")?;

    pool.get()
        .await
        .context("Failed to get Redis connection from pool")
}

struct RedisCacheContext;
impl Default for RedisCacheContext {
    // ---
    fn default() -> Self {
        RedisCacheContext
    }
}

#[async_trait::async_trait]
impl CacheContextTrait for RedisCacheContext {
    // --
    /// Validate a session token and return the associated user ID, or `None` if invalid.
    async fn get_user_id_by_session_token(&self, token: &str) -> Result<Option<i32>> {
        // ---

        let mut conn = get_redis_connection().await?;

        let val: Option<i32> = conn
            .get(token)
            .await
            .context("failed to get session token from Redis")?;

        Ok(val)
    }

    /// Write a new session token for a user (or update existing one).
    async fn set_user_session_token(&self, user_id: i32, token: &str) -> Result<()> {
        // ---

        let mut conn = get_redis_connection().await?;

        match conn.set::<_, _, ()>(token, user_id).await {
            Ok(_) => Ok(()),
            Err(err) => Err(anyhow!("failed to set session token in Redis:{err}")),
        }
    }

    /// Delete a session token (e.g., logout), return `true` if found else `false`.
    async fn clear_session_token(&self, token: &str) -> Result<bool> {
        // ---
        let mut conn = get_redis_connection().await?;

        let delete_count: u64 = conn
            .del(token)
            .await
            .context("failed to delete session token from Redis")?;

        Ok(delete_count > 0)
    }
}

pub fn create_cache_context() -> CacheContextTraitPtr {
    // ---
    std::sync::Arc::new(RedisCacheContext)
}

/// Attempt to create and verify a Redis connection pool with retry logic.
pub async fn init_redis_cache_with_retry_from_env() -> Result<()> {
    // ---
    let redis_url = get_env_with_default!(String, "REDIS_URL", "redis://127.0.0.1/".to_owned());
    let retry_max = get_env_with_default!(u32, "CR8S_REDIS_RETRY_COUNT", 50);
    let delay_secs = get_env_with_default!(u64, "CR8S_REDIS_RETRY_DELAY_SECS", 1);
    let delay_secs = Duration::from_secs(delay_secs);

    // Don't use this direct access method after calling this function once. Use
    // `get_redis_connection()` instead. We are using it here with reverse polarity,
    // i.e. existing pool being the error case here.
    //
    if let Some(_pool) = REDIS_POOL.get() {
        // --
        tracing::error!("🚨 init_redis_cache_with_retry_from_env:: called again incorrectly");
        return Err(anyhow!("Redis pool already initialized"));
    }

    tracing::info!("🚨 Attaching to redis at: {redis_url:?}");

    for attempt in 1..=retry_max {
        // ---

        match try_create_redis_pool(&redis_url) {
            // ---
            Ok(pool) => {
                if verify_redis_pool(&pool).await.is_ok() {
                    // ---
                    // Handle the race conditional if this method is called
                    // concurrently by two threads.  Main is not supposed to do this
                    // but we need to protect against it anyway.
                    REDIS_POOL
                        .set(pool)
                        .map_err(|_| anyhow!("Redis pool already initialized"))?;
                    return Ok(());
                }
            }
            Err(e) if attempt == retry_max => {
                return Err(anyhow!(
                    "Failed to create Redis pool after {retry_max} retries: {e}"
                ));
            }
            Err(_) => {}
        }

        tracing::warn!(
            "Redis not ready (attempt {attempt}/{retry_max}) — retrying in {}s...",
            delay_secs.as_secs()
        );
        sleep(delay_secs).await;
    }

    Err(anyhow!("Exhausted retries but Redis is not available"))
}

/// Attempts to create a Redis connection pool from the given URL string.
///
/// This function performs all necessary steps to configure and initialize a
/// `deadpool`-based Redis pool:
/// - Validates the URL using `Config::from_url`
/// - Constructs a `PoolBuilder`
/// - Attaches the appropriate async runtime (Tokio 1.x)
/// - Builds and returns the final connection pool
///
/// All intermediate steps are wrapped with `.with_context(...)` to provide precise
/// error messages for each of the possible errors when user-supplied configuration
/// is invalid.
///
/// # Arguments
///
/// * `redis_url` - A Redis connection string (e.g. `"redis://127.0.0.1/"`)
///
/// # Errors
///
/// Returns a detailed error if any step in pool creation fails.
///
/// This is typically used inside a retry loop to tolerate transient failures at
/// application startup.
fn try_create_redis_pool(redis_url: &str) -> Result<Pool> {
    // ---
    let cfg = deadpool_redis::Config::from_url(redis_url.to_owned());

    cfg.builder()
        .with_context(|| format!("Failed to create builder for URL: {redis_url}"))?
        .runtime(deadpool_redis::Runtime::Tokio1)
        .build()
        .with_context(|| format!("Failed to build Redis pool for URL: {redis_url}"))
}

/// Basic connection check: get/set/delete a dummy redis key.
#[allow(clippy::let_unit_value)]
async fn verify_redis_pool(pool: &Pool) -> Result<()> {
    // ---
    let mut conn = pool
        .get()
        .await
        .context("verify_redis_pool: failed to get Redis connection")?;

    let token = "__cr8s_redis_ping";

    // Try to set the key
    conn.set(token, "ok")
        .await
        .context("verify_redis_pool: failed to SET test key")
}

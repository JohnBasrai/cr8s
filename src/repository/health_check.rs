use crate::domain::HealthTrait;
use crate::domain::HealthTraitPtr;
use crate::repository::get_pool;
use anyhow::Result;
use async_trait::async_trait;
//use redis::AsyncCommands; // ✅ From your own `redis = "=0.25.4"` dep
use rocket_db_pools::deadpool_redis::Pool as RedisPool;
use sqlx::PgPool;
use std::sync::Arc;

// ------------- Database health check implementation --------------------

pub struct DatabaseHealthService {
    pool: PgPool,
}

impl DatabaseHealthService {
    pub fn new(pool: PgPool) -> Self {
        // ---
        Self { pool }
    }
}

#[async_trait]
impl HealthTrait for DatabaseHealthService {
    // ---
    async fn health_check(&self) -> Result<()> {
        // ---
        sqlx::query("SELECT 1").execute(&self.pool).await?;
        Ok(())
    }
}

pub fn create_database_health_service() -> Result<HealthTraitPtr> {
    // ---
    Ok(Arc::new(DatabaseHealthService::new(get_pool().clone())))
}

// ------------- Cache health check implementation --------------------

pub struct CacheHealthService {
    pool: RedisPool,
}

impl CacheHealthService {
    pub fn new(pool: RedisPool) -> Self {
        // ---
        Self { pool }
    }
}

#[async_trait]
impl HealthTrait for CacheHealthService {
    // ---
    async fn health_check(&self) -> Result<()> {
        // ---
        let mut conn = self.pool.get().await?;
        let pong: String = redis::cmd("PING").query_async(&mut *conn).await?;

        if pong == "PONG" {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Unexpected ping response: {pong}"))
        }
    }
}

pub fn create_cache_health_service() -> Result<HealthTraitPtr> {
    // ---
    let pool = super::get_redis_pool().clone();
    Ok(Arc::new(CacheHealthService::new(pool)))
}

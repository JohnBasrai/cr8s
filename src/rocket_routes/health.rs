use crate::rocket_routes::CacheConn;
use redis::cmd;
use rocket::get;
use rocket::http::Status;
use rocket_db_pools::Connection;

#[get("/health")]
pub async fn health(mut redis: Connection<CacheConn>) -> (Status, &'static str) {
    // ---
    tracing::debug!("🐾 Health check...");

    let result: redis::RedisResult<String> = cmd("PING").query_async(redis.as_mut()).await;

    match result {
        Ok(_) => (Status::Ok, "OK"),
        Err(e) => {
            tracing::warn!("Redis ping failed: {e}");
            (Status::ServiceUnavailable, "Unavailable")
        }
    }
}

#[get("/ping")]
pub async fn redis_ping(mut redis: Connection<CacheConn>) -> (Status, String) {
    match cmd("PING").query_async(redis.as_mut()).await {
        Ok(resp) => (Status::Ok, resp),
        Err(e) => {
            tracing::warn!("Redis ping failed: {e}");
            (Status::ServiceUnavailable, "Unavailable".into())
        }
    }
}

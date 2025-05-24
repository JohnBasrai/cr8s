// rocket_routes/health.rs
use crate::domain::HealthTraitPtr;
use rocket::get;
use rocket::http::Status;
use rocket::State;

// REST /health endpoint
#[get("/health")]
pub async fn health_endpoint(cache: &State<HealthTraitPtr>) -> Status {
    // ---
    if cache.inner().health_check().await.is_ok() {
        Status::Ok
    } else {
        Status::ServiceUnavailable
    }
}

#[cfg(test)]
mod tests {
    // ---
    use super::*;
    use anyhow::{ensure, Result};
    use rocket::http::Status;

    #[tokio::test]
    async fn test_check_cache_health_ok() -> Result<()> {
        // ---
        let result = health_endpoint().await;
        ensure!(result == Status::Ok);
        Ok(())
    }

    #[tokio::test]
    async fn test_check_cache_health_fails() -> Result<()> {
        // ---
        let result = health_endpoint().await;
        ensure!(result, Status::ServiceUnavailable);
        Ok(())
    }
}

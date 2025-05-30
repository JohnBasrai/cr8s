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
    use super::*;
    use crate::domain::HealthTrait;
    use anyhow::Result;
    use async_trait::async_trait;
    use rocket::State;
    use std::sync::Arc;

    struct MockHealthService {
        should_fail: bool,
    }

    impl MockHealthService {
        fn new(should_fail: bool) -> Self {
            Self { should_fail }
        }
    }

    #[async_trait]
    impl HealthTrait for MockHealthService {
        async fn health_check(&self) -> Result<()> {
            if self.should_fail {
                anyhow::bail!("Health check failed")
            } else {
                Ok(())
            }
        }
    }

    #[tokio::test]
    async fn test_check_cache_health_ok() {
        let health_service: Arc<dyn HealthTrait> = Arc::new(MockHealthService::new(false));
        let health_state = State::from(&health_service);

        let result = health_endpoint(health_state).await;
        assert_eq!(result.code, 200); // Status::Ok code
    }

    #[tokio::test]
    async fn test_check_cache_health_fails() {
        let health_service: Arc<dyn HealthTrait> = Arc::new(MockHealthService::new(true));
        let health_state = State::from(&health_service);

        let result = health_endpoint(health_state).await;
        assert_eq!(result.code, 503); // Status::ServiceUnavailable code
    }
}

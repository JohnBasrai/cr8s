use crate::domain::{PasswordHasherTrait, PasswordHasherTraitPtr};
use anyhow::Result;
use std::sync::Arc;

pub struct MockPasswordHasher;

impl PasswordHasherTrait for MockPasswordHasher {
    // ---
    fn hash_password(&self, password: &str) -> Result<String> {
        Ok(format!("mock_hash({})", password))
    }

    fn verify_password(&self, hashed: &str, candidate: &str) -> Result<()> {
        if hashed == format!("mock_hash({})", candidate) {
            Ok(())
        } else {
            Err(anyhow::anyhow!("mock verification failed"))
        }
    }

    fn generate_session_token(&self) -> String {
        "mock-session-token".into()
    }
}

pub fn create_mock_password_hasher() -> Result<PasswordHasherTraitPtr> {
    Ok(Arc::new(MockPasswordHasher))
}

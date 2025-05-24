use anyhow::Result;

pub struct MockPasswordHasher;

impl MockPasswordHasher {
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

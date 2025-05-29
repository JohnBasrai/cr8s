use super::{server_error, GuardedAppUser};
use crate::domain::{
    //
    authenticate_user,
    AppUserTableTraitPtr,
    CacheContextTraitPtr,
    Credentials,
};
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::serde::json::{json, Json, Value};
use rocket::State;

#[rocket::post("/login", format = "json", data = "<credentials>")]
pub async fn login(
    app_user_repo: &State<AppUserTableTraitPtr>,
    cache_context: &State<CacheContextTraitPtr>,
    credentials: Json<Credentials>,
) -> Result<Value, Custom<Value>> {
    // --
    let (user, session_id) =
        match authenticate_user(app_user_repo.inner().clone(), credentials.into_inner()).await {
            // ---
            Ok(result) => result,
            Err(err) => {
                tracing::warn!("❌ Login failed: {err}");
                return Err(Custom(
                    Status::Unauthorized,
                    json!("error: Invalid credentials"),
                ));
            }
        };

    cache_context
        .set_user_session_token(user.id, &session_id)
        .await
        .map_err(server_error)?;

    Ok(json!({ "token": session_id }))
}

#[rocket::get("/me")]
pub fn me(user: GuardedAppUser) -> Value {
    // ---
    tracing::info!("✅ Authenticated user: {:?}", user);
    json!(user)
}

#[cfg(test)]
mod tests {
    // ---
    use super::*;
    use crate::domain::{
        //
        AppUser as DomainAppUser,
        AppUser,
        AppUserTableTrait,
        AppUserWithRoleCodes,
        CacheContextTrait,
        Credentials,
        RoleCode,
    };

    use anyhow::Result;
    use async_trait::async_trait;
    use chrono::Utc;
    use rocket::State;
    use std::collections::HashMap;
    use std::sync::Arc;

    struct MockAppUserRepo {
        users: HashMap<String, DomainAppUser>,
    }

    impl MockAppUserRepo {
        // ---

        fn with_user(username: &str, password: &str) -> Self {
            // ---

            let mut users = HashMap::new();

            // Hash the password like the real system would
            let hasher = crate::auth::create_password_hasher().unwrap();
            let hashed_password = hasher.hash_password(password).unwrap();

            users.insert(
                username.to_string(),
                DomainAppUser {
                    id: 1,
                    username: username.to_string(),
                    password: hashed_password,
                    created_at: Utc::now().naive_utc(),
                },
            );
            Self { users }
        }
    }

    #[async_trait]
    impl AppUserTableTrait for MockAppUserRepo {
        // --
        async fn find_with_roles(&self) -> Result<Vec<AppUserWithRoleCodes>> {
            // ---
            unreachable!()
        }

        async fn find(&self, _id: i32) -> Result<AppUser> {
            // ---
            unreachable!()
        }

        async fn find_by_username(&self, username: &str) -> Result<DomainAppUser> {
            // ---
            self.users
                .get(username)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("not found"))
        }

        async fn create(
            // ---
            &self,
            _new_user: crate::domain::NewUser,
            _roles: Vec<RoleCode>,
        ) -> Result<DomainAppUser> {
            unreachable!()
        }

        async fn find_roles_by_user(&self, _user: &DomainAppUser) -> Result<Vec<RoleCode>> {
            // ---
            unreachable!()
        }

        async fn delete_by_id(&self, _user_id: i32) -> Result<()> {
            // ---
            unreachable!()
        }

        async fn delete_by_username(&self, _username: &str) -> Result<()> {
            // ---
            unreachable!()
        }
    }

    struct MockCacheContext {
        expected_token: String,
        expected_user_id: i32,
        fail: bool,
    }

    impl MockCacheContext {
        fn new(expected_token: &str, expected_user_id: i32, fail: bool) -> Self {
            Self {
                expected_token: expected_token.to_string(),
                expected_user_id,
                fail,
            }
        }
    }

    #[async_trait]
    impl CacheContextTrait for MockCacheContext {
        // ---
        async fn get_user_id_by_session_token(&self, _token: &str) -> Result<Option<i32>> {
            // ---
            unreachable!()
        }

        async fn set_user_session_token(&self, user_id: i32, token: &str) -> Result<()> {
            // ---
            assert_eq!(user_id, self.expected_user_id, "user_id mismatch");
            if !self.expected_token.is_empty() {
                assert_eq!(token, self.expected_token, "token mismatch");
            }
            if self.fail {
                anyhow::bail!("simulated Redis failure");
            }
            Ok(())
        }

        async fn clear_session_token(&self, _token: &str) -> Result<bool> {
            // ---
            unreachable!()
        }
    }

    #[tokio::test]
    async fn test_login_success() {
        // ---
        let repo: Arc<dyn AppUserTableTrait + Send + Sync> =
            Arc::new(MockAppUserRepo::with_user("alice", "password"));

        // Accept any tokena
        let cache: Arc<dyn CacheContextTrait> = Arc::new(MockCacheContext::new("", 1, false));

        let repo_state = State::from(&repo);
        let cache_state = State::from(&cache);

        let creds = Credentials {
            username: "alice".into(),
            password: "password".into(),
        };

        // Debug: Check what user is stored
        let stored_user = repo.find_by_username("alice").await.unwrap();
        println!("Stored user: {:?}", stored_user);
        println!("Stored password: {}", stored_user.password);
        println!("Input password: {}", creds.password);

        let result = login(repo_state, cache_state, Json(creds)).await;

        match result {
            Ok(value) => {
                assert!(value["token"].is_string()); // Just verify a token was returned
                assert!(!value["token"].as_str().unwrap().is_empty());
            }
            Err(e) => panic!("Expected success but got error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_login_invalid_password() {
        // ---
        let repo: Arc<dyn AppUserTableTrait + Send + Sync> =
            Arc::new(MockAppUserRepo::with_user("alice", "correct"));
        let cache: Arc<dyn CacheContextTrait> = Arc::new(MockCacheContext::new("x", 1, false));
        let repo_state = State::from(&repo);
        let cache_state = State::from(&cache);

        let creds = Credentials {
            username: "alice".into(),
            password: "wrong".into(),
        };

        let result = login(repo_state, cache_state, Json(creds)).await;

        // Check if it's an Unauthorized error without using Status::Unauthorized in pattern
        match result {
            Err(Custom(status, _)) => {
                assert_eq!(status.code, 401); // Unauthorized status code
            }
            Ok(_) => panic!("Expected unauthorized error but got success"),
        }
    }

    #[test]
    fn test_me_returns_user_json() {
        let user = GuardedAppUser(DomainAppUser {
            id: 1,
            username: "test@example.com".into(),
            password: "test_password".into(),
            created_at: Utc::now().naive_utc(),
        });

        let result = me(user);

        assert_eq!(result["id"], 1);
        assert_eq!(result["username"], "test@example.com");
    }
}

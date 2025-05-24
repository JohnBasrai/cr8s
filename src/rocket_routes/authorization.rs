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
    use crate::domain::{AppUser as DomainAppUser, CacheContextTrait, Credentials, RoleCode};
    use anyhow::Result;
    use async_trait::async_trait;
    use chrono::Utc;
    use std::collections::HashMap;
    use std::sync::Arc;

    struct MockAppUserRepo {
        users: HashMap<String, DomainAppUser>,
    }

    impl MockAppUserRepo {
        // ---
        fn with_user(username: &str, password: &str) -> Self {
            let mut users = HashMap::new();
            users.insert(
                username.to_string(),
                DomainAppUser {
                    id: 1,
                    username: username.to_string(),
                    password: password.to_string(),
                    created_at: Utc::now().naive_utc(),
                },
            );
            Self { users }
        }
    }

    #[async_trait]
    impl AppUserTableTrait for MockAppUserRepo {
        // ---
        async fn find_by_username(&self, username: &str) -> Result<DomainAppUser> {
            self.users
                .get(username)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("not found"))
        }

        async fn create(
            &self,
            _new_user: crate::domain::NewUser,
            _roles: Vec<RoleCode>,
        ) -> Result<DomainAppUser> {
            unreachable!()
        }

        async fn find_roles_by_user(&self, _user: &DomainAppUser) -> Result<Vec<RoleCode>> {
            unreachable!()
        }

        async fn delete_by_id(&self, _user_id: i32) -> Result<()> {
            unreachable!()
        }

        async fn delete_by_username(&self, _username: &str) -> Result<()> {
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
            unreachable!()
        }

        async fn set_user_session_token(&self, user_id: i32, token: &str) -> Result<()> {
            assert_eq!(user_id, self.expected_user_id, "user_id mismatch");
            assert_eq!(token, self.expected_token, "token mismatch");
            if self.fail {
                anyhow::bail!("simulated Redis failure");
            }
            Ok(())
        }

        async fn clear_session_token(&self, _token: &str) -> Result<bool> {
            unreachable!()
        }
    }

    #[tokio::test]
    async fn test_login_success() -> Result<()> {
        // ---
        let repo = Arc::new(MockAppUserRepo::with_user("alice", "password"));
        let cache = Arc::new(MockCacheContext::new("mock-token", 1, false));
        let creds = Credentials {
            username: "alice".into(),
            password: "password".into(),
        };

        let result = login(repo.into(), cache.into(), Json(creds)).await?;
        assert_eq!(result["token"], "mock-token");
        Ok(())
    }

    #[tokio::test]
    async fn test_login_invalid_password() -> Result<()> {
        // ---
        let repo = Arc::new(MockAppUserRepo::with_user("alice", "correct"));
        let cache = Arc::new(MockCacheContext::new("x", 1, false));
        let creds = Credentials {
            username: "alice".into(),
            password: "wrong".into(),
        };

        let result = login(repo.into(), cache.into(), Json(creds)).await;
        assert!(matches!(result, Err(Custom(Status::Unauthorized, _))));
        Ok(())
    }

    #[test]
    fn test_me_returns_user_json() {
        // ---
        let user = crate::domain::DomainAppUser {
            id: 1,
            username: "test@example.com".into(),
        };

        let result = me(user);

        assert_eq!(result["id"], 1);
        assert_eq!(result["username"], "test@example.com");
    }
}

use crate::domain::{AppUser, AppUserTableTraitPtr, CacheContextTraitPtr, RoleCode};
use rocket::http::Status;
use rocket::outcome::Outcome;
use rocket::request::FromRequest;
use rocket::{Request, State};

// ---

#[derive(Debug, serde::Serialize)]
pub struct GuardedAppUser(pub AppUser);

#[derive(Debug, serde::Serialize)]
pub struct EditorUser(pub GuardedAppUser);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for GuardedAppUser {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        // Extract token from Authorization header
        let token = request
            .headers()
            .get_one("Authorization")
            .and_then(|auth| auth.strip_prefix("Bearer "))
            .map(|s| s.to_string());

        let token = match token {
            Some(t) => t,
            None => return Outcome::Error((Status::Unauthorized, ())),
        };

        // Get cache and user repo from Rocket state - FIX: Remove unwrap()
        let cache: &State<CacheContextTraitPtr> = match request.guard().await {
            Outcome::Success(cache) => cache,
            _ => {
                tracing::debug!("Failed to get cache from Rocket state");
                return Outcome::Error((Status::InternalServerError, ()));
            }
        };

        let user_repo: &State<AppUserTableTraitPtr> = match request.guard().await {
            Outcome::Success(repo) => repo,
            _ => {
                tracing::debug!("Failed to get user repo from Rocket state");
                return Outcome::Error((Status::InternalServerError, ()));
            }
        };

        // Validate session token
        let user_id = match cache.inner().get_user_id_by_session_token(&token).await {
            Ok(Some(id)) => id,
            _ => return Outcome::Error((Status::Unauthorized, ())),
        };

        // Fetch user from DB
        let user = match user_repo.inner().find(user_id).await {
            Ok(user) => user,
            Err(_) => return Outcome::Error((Status::Unauthorized, ())),
        };

        Outcome::Success(GuardedAppUser(user))
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for EditorUser {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        // First get the authenticated user
        let user = match req.guard::<GuardedAppUser>().await {
            Outcome::Success(user) => user,
            Outcome::Error(e) => return Outcome::Error(e),
            Outcome::Forward(f) => return Outcome::Forward(f),
        };

        // Then get the user repository from managed state
        let user_repo: &State<AppUserTableTraitPtr> = match req.guard().await {
            Outcome::Success(repo) => repo,
            _ => {
                tracing::debug!("EditorUser: Failed to get user repo from state");
                return Outcome::Error((Status::InternalServerError, ()));
            }
        };

        // Check permissions
        match user.is_editor(user_repo.inner()).await {
            Ok(true) => Outcome::Success(EditorUser(user)),
            Ok(false) => {
                tracing::debug!(
                    "EditorUser: User {} lacks editor privileges",
                    user.0.username
                );
                Outcome::Error((Status::Forbidden, ()))
            }
            Err(e) => {
                tracing::debug!(
                    "EditorUser: Role check failed for {}: {:?}",
                    user.0.username,
                    e
                );
                Outcome::Error((Status::InternalServerError, ()))
            }
        }
    }
}

impl GuardedAppUser {
    pub async fn is_editor(&self, user_repo: &AppUserTableTraitPtr) -> anyhow::Result<bool> {
        let roles = user_repo.find_roles_by_user(&self.0).await?;
        Ok(roles
            .iter()
            .any(|role| matches!(role, RoleCode::Admin | RoleCode::Editor)))
    }

    pub async fn is_admin(&self, user_repo: &AppUserTableTraitPtr) -> anyhow::Result<bool> {
        let roles = user_repo.find_roles_by_user(&self.0).await?;
        Ok(roles.iter().any(|role| matches!(role, RoleCode::Admin)))
    }
}

#[cfg(test)]
mod tests {
    // ---
    // Unit tests for role-based authorization logic (is_editor, is_admin).
    //
    // NOT COVERED (intentionally - would require integration tests):
    // - Full Rocket guard flow (GuardedAppUser::from_request, EditorUser::from_request)
    // - Token validation and session management
    // - Database role lookups in real repository implementations
    //
    // For full end-to-end testing of guards, see integration tests in tests/ directory.
    // ---

    use super::*;
    use crate::domain::{AppUser, AppUserTableTrait, NewUser, RoleCode};
    use anyhow::Result;
    use async_trait::async_trait;
    use chrono::Utc;
    use std::sync::Arc;

    // Much simpler mock - only implement what we need!
    struct MockAppUserRepo {
        user_roles: Vec<RoleCode>,
    }

    impl MockAppUserRepo {
        fn with_roles(roles: Vec<RoleCode>) -> Self {
            Self { user_roles: roles }
        }
    }

    #[async_trait]
    impl AppUserTableTrait for MockAppUserRepo {
        // --
        // Only implement the one method we actually use in guard tests
        async fn find_roles_by_user(&self, _user: &AppUser) -> Result<Vec<RoleCode>> {
            Ok(self.user_roles.clone())
        }

        // All other methods use the trait defaults, so we don't need to implement them!

        async fn create(&self, _new_user: NewUser, _role_codes: Vec<RoleCode>) -> Result<AppUser> {
            unreachable!("Not used in guard tests")
        }

        async fn find(&self, _id: i32) -> Result<AppUser> {
            unreachable!("Not used in guard tests")
        }

        async fn find_by_username(&self, _username: &str) -> Result<AppUser> {
            unreachable!("Not used in guard tests")
        }
    }

    // Macro to create a mock repo with specific roles
    macro_rules! mock_repo {
        ($($role:expr),*) => {
            Arc::new(MockAppUserRepo::with_roles(vec![$($role),*]))
        };
    }

    fn create_test_user() -> GuardedAppUser {
        GuardedAppUser(AppUser {
            id: 1,
            username: "test_user".to_string(),
            password: "hashed_password".to_string(),
            created_at: Utc::now().naive_utc(),
        })
    }

    // Tests begin here ---

    #[tokio::test]
    async fn test_is_editor_with_admin_role() -> anyhow::Result<()> {
        let user = create_test_user();
        let repo: AppUserTableTraitPtr = mock_repo!(RoleCode::Admin);

        let result = user.is_editor(&repo).await?;
        anyhow::ensure!(result, "Admin should have editor privileges");
        Ok(())
    }

    #[tokio::test]
    async fn test_is_editor_with_editor_role() -> anyhow::Result<()> {
        let user = create_test_user();
        let repo: AppUserTableTraitPtr = mock_repo!(RoleCode::Editor);

        let result = user.is_editor(&repo).await?;
        anyhow::ensure!(result, "Editor should have editor privileges");
        Ok(())
    }

    #[tokio::test]
    async fn test_is_editor_with_viewer_role() -> anyhow::Result<()> {
        let user = create_test_user();
        let repo: AppUserTableTraitPtr = mock_repo!(RoleCode::Viewer);

        let result = user.is_editor(&repo).await?;
        anyhow::ensure!(!result, "Viewer should not have editor privileges");
        Ok(())
    }

    #[tokio::test]
    async fn test_is_editor_with_multiple_roles() -> anyhow::Result<()> {
        let user = create_test_user();
        let repo: AppUserTableTraitPtr = mock_repo!(RoleCode::Viewer, RoleCode::Editor);

        let result = user.is_editor(&repo).await?;
        anyhow::ensure!(
            result,
            "User with Editor among multiple roles should have editor privileges"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_is_editor_with_no_roles() -> anyhow::Result<()> {
        let user = create_test_user();
        let repo: AppUserTableTraitPtr = mock_repo!();

        let result = user.is_editor(&repo).await?;
        anyhow::ensure!(
            !result,
            "User with no roles should not have editor privileges"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_is_admin_with_admin_role() -> anyhow::Result<()> {
        let user = create_test_user();
        let repo: AppUserTableTraitPtr = mock_repo!(RoleCode::Admin);

        let result = user.is_admin(&repo).await?;
        anyhow::ensure!(result, "Admin should have admin privileges");
        Ok(())
    }

    #[tokio::test]
    async fn test_is_admin_with_editor_role() -> anyhow::Result<()> {
        let user = create_test_user();
        let repo: AppUserTableTraitPtr = mock_repo!(RoleCode::Editor);

        let result = user.is_admin(&repo).await?;
        anyhow::ensure!(!result, "Editor should not have admin privileges");
        Ok(())
    }

    #[tokio::test]
    async fn test_is_admin_with_viewer_role() -> anyhow::Result<()> {
        let user = create_test_user();
        let repo: AppUserTableTraitPtr = mock_repo!(RoleCode::Viewer);

        let result = user.is_admin(&repo).await?;
        anyhow::ensure!(!result, "Viewer should not have admin privileges");
        Ok(())
    }
}

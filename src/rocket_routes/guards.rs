use crate::domain::{AppUser, AppUserTableTraitPtr, CacheContextTraitPtr};
use rocket::http::Status;
use rocket::outcome::Outcome;
use rocket::request::FromRequest;
use rocket::{Request, State};

// ---

#[derive(Debug, serde::Serialize)]
pub struct GuardedAppUser(pub AppUser);

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

        // Get cache and user repo from Rocket state
        let cache: &State<CacheContextTraitPtr> = request.guard().await.unwrap();
        let user_repo: &State<AppUserTableTraitPtr> = request.guard().await.unwrap();

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

impl GuardedAppUser {
    // ---
    pub fn is_editor(&self) -> bool {
        // Note: AppUser doesn't have a role field based on the error
        // You may need to add role checking logic here or modify AppUser struct
        // For now, returning false as placeholder
        false
    }

    pub fn is_admin(&self) -> bool {
        // Note: AppUser doesn't have a role field based on the error
        // You may need to add role checking logic here or modify AppUser struct
        // For now, returning false as placeholder
        false
    }
}

#[derive(Debug, serde::Serialize)]
pub struct EditorUser(pub GuardedAppUser);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for EditorUser {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        match req.guard::<GuardedAppUser>().await {
            Outcome::Success(user) => {
                if user.is_editor() {
                    Outcome::Success(EditorUser(user))
                } else {
                    Outcome::Error((Status::Forbidden, ()))
                }
            }
            Outcome::Error(e) => Outcome::Error(e),
            Outcome::Forward(f) => Outcome::Forward(f),
        }
    }
}

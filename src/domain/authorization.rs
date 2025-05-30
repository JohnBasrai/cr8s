// src/domain/auth.rs
use super::{AppUser, AppUserTableTraitPtr};
use anyhow::Result;
use serde::Deserialize;

/// Domain model for login credentials used in authentication.
#[derive(Debug, Deserialize, Clone)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

impl Credentials {
    // ---
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
        }
    }
}

pub use crate::auth::create_password_hasher;

/// Authenticates a user and returns a session token.
///
/// The session token is a temporary identifier for a logged-in user. It is stored in Redis
/// and used to authorize future requests.
///
/// Returns `Ok((user, token))` on success, or `Err(LoginError)` if authentication fails.
pub async fn authenticate_user(
    repo: AppUserTableTraitPtr,
    creds: Credentials,
) -> Result<(AppUser, String), LoginError> {
    // ---
    let user_tag = "[user redacted]";
    let user = repo
        .find_by_username(&creds.username)
        .await
        .map_err(|err| {
            tracing::warn!("Unknown user {user_tag}: {err}");
            LoginError::InvalidCredentials
        })?;

    let hasher = crate::auth::create_password_hasher().map_err(|err| {
        tracing::warn!("create_password_hasher failed: {user_tag}: {err}");
        LoginError::InvalidCredentials
    })?;

    hasher
        .verify_password(&user.password, &creds.password)
        .map_err(|err| {
            tracing::warn!("auth failure or {user_tag}: {err}");
            LoginError::InvalidCredentials
        })?;

    let token = hasher.generate_session_token();

    Ok((user, token))
}

#[derive(Debug, thiserror::Error)]
pub enum LoginError {
    #[error("invalid username or password")]
    InvalidCredentials,

    #[error("internal error: {0}")]
    Internal(String),
}

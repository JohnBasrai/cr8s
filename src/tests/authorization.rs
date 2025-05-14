use super::test_utils::common::unique_username;
use crate::auth::hash_password;
use crate::ensure_status;
use crate::models::{NewUser, RoleCode};
use crate::tests::test_utils::common::*;
use anyhow::{Context, Result};
use diesel_async::AsyncPgConnection;
use reqwest::{Client, StatusCode};
use serde_json::json;

/// Creates the user with the specified role using Diesel, not the CLI.
///
#[allow(dead_code)]
pub async fn create_user(
    conn: &mut AsyncPgConnection,
    username: &str,
    password_hash: &str,
    role: RoleCode, // <- Note, use enum
) -> Result<()> {
    // ---
    let new_user = NewUser {
        username: username.to_string(),
        password: password_hash.to_string(),
    };

    let user = insert_test_user(conn, &new_user).await?;
    assign_role(conn, user.id, role)
        .await
        .context("failed to assign role")?;
    Ok(())
}

/// Verifies that a user can log in with valid credentials and access the `/me` endpoint
#[tokio::test]
async fn test_login_and_me_returns_user_details() -> anyhow::Result<()> {
    // ---
    let mut conn: AsyncPgConnection = ensure_test_db_ready().await?;

    let raw_password = "password123";
    let username = unique_username("test-login-me-details");

    let password_hash = hash_password(raw_password.to_string())
        .map_err(|e| anyhow::anyhow!("Password hashing failed: {e}"))?;

    let user = NewUser {
        username: username.clone(),
        password: password_hash,
    };

    let _ = insert_test_user(&mut conn, &user).await;

    let client = Client::new();
    let login_response = client
        .post(format!("{}/login", APP_HOST))
        .json(&json!({ "username": username, "password": raw_password }))
        .send()
        .await
        .context("Login request failed")?;

    ensure_status!(login_response, StatusCode::OK, "Login failed");

    let login_json: serde_json::Value = login_response
        .json()
        .await
        .context("Failed to parse login response JSON")?;

    let token = login_json["token"]
        .as_str()
        .context("Missing 'token' field in response")?;

    let me_response = client
        .get(format!("{}/me", APP_HOST))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .context("GET /me request failed")?;

    ensure_status!(me_response, StatusCode::OK, "GET /me failed");

    Ok(())
}

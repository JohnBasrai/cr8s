use anyhow::{ensure, Context, Result};
use common::*;
use cr8s::models::NewUser;
use cr8s::test_support::{assign_role, insert_test_user};
use diesel_async::AsyncPgConnection;
use reqwest::{Client, StatusCode};
use serde_json::json;

use common::APP_HOST;
use cr8s::test_support::establish_test_connection;

/// Creates the user with the specified role using Diesel, not the CLI.
///
pub async fn create_user(
    conn: &mut AsyncPgConnection,
    username: &str,
    password_hash: &str,
    role: &str,
) -> Result<()> {
    // ---
    let new_user = NewUser {
        username: username.to_string(),
        password: password_hash.to_string(),
    };

    let user = insert_test_user(conn, &new_user).await?;
    assign_role(conn, user.id, role)
        .await
        .context("failed to assign Admin role")?;

    Ok(())
}

/// Tests that a newly created admin user can successfully log in via the API.
///
#[tokio::test]
async fn test_user_with_admin_role_can_access_admin_route() -> Result<()> {
    // ---
    let mut conn = establish_test_connection().await.context("db connection")?;

    let user = NewUser {
        username: "admin@example.com".into(),
        password: "hashed_pw".into(),
    };

    let _ = insert_test_user(&mut conn, &user)
        .await
        .context("insert user");

    // Now proceed with login check via HTTP or direct call

    let client = Client::new();
    let response = client
        .post(format!("{}/login", APP_HOST))
        .json(&json!({ "username": user.username, "password": user.password }))
        .send()
        .await
        .context("Login request failed")?;

    ensure!(
        response.status() == StatusCode::OK,
        "Login failed with status: {}",
        response.status()
    );
    Ok(())
}

/// Tests that a logged-in viewer user can access their identity via `GET /me`.
///
#[tokio::test]
async fn test_me_reponse_has_correct_user_details() -> Result<()> {
    // ---
    let username = "test_me_user";
    let password = "password123";

    let mut conn = establish_test_connection().await.context("DB connection")?;

    create_user(&mut conn, username, password, "viewer")
        .await
        .context(format!("Error creating test user:{username}"))?;

    let client = Client::new();
    let login = client
        .post(format!("{}/login", APP_HOST))
        .json(&json!({ "username": username, "password": password }))
        .send()
        .await
        .context("Login request failed")?;

    ensure!(
        login.status() == StatusCode::OK,
        "Login failed with status: {}",
        login.status()
    );

    let cookie = login
        .headers()
        .get("set-cookie")
        .context("Missing set-cookie header")?
        .to_str()
        .context("set-cookie header is not valid UTF-8")?;

    let me = client
        .get(format!("{}/me", APP_HOST))
        .header("cookie", cookie)
        .send()
        .await
        .context("GET /me request failed")?;

    ensure!(
        me.status() == StatusCode::OK,
        "/me endpoint returned unexpected status: {}",
        me.status()
    );

    #[derive(serde::Deserialize, Debug)]
    struct MeResponse {
        username: String,
        roles: Vec<String>,
    }

    let json: MeResponse = me
        .json()
        .await
        .context("Failed to parse JSON from /me response")?;

    ensure!(
        json.username == username,
        "Username mismatch: expected '{}', got '{}'",
        username,
        json.username
    );

    ensure!(
        json.roles.contains(&"viewer".to_string()),
        "Expected roles to include 'viewer', got: {:?}",
        json.roles
    );

    Ok(())
}

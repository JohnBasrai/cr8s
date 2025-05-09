use reqwest::{blocking::Client, StatusCode};
use serde_json::json;
use std::process::Command;

pub mod common;

use crate::common::APP_HOST;

const DB_URL: &str = "postgres://postgres:secret@localhost:5432/cr8s";

/// Deletes a user from the database using the CLI (if they already exist),
/// then re-creates the user with the specified role.
fn create_user(username: &str, password: &str, role: &str) {
    // Try deleting the user if it exists
    let delete_output = Command::new("target/debug/cli")
        .args(["users", "delete", username])
        .env("DATABASE_URL", DB_URL)
        .output()
        .expect("Failed to run CLI delete command");

    eprintln!(
        "[delete {}] stdout: {}",
        username,
        String::from_utf8_lossy(&delete_output.stdout)
    );
    eprintln!(
        "[delete {}] stderr: {}",
        username,
        String::from_utf8_lossy(&delete_output.stderr)
    );

    // Now create the user fresh
    let create_output = Command::new("target/debug/cli")
        .args(["users", "create", username, password, role])
        .env("DATABASE_URL", DB_URL)
        .output()
        .expect("Failed to run CLI create command");

    eprintln!(
        "[create {}] stdout: {}",
        username,
        String::from_utf8_lossy(&create_output.stdout)
    );
    eprintln!(
        "[create {}] stderr: {}",
        username,
        String::from_utf8_lossy(&create_output.stderr)
    );

    if !create_output.status.success() {
        panic!("User creation failed: {}", username);
    }
}

/// Tests that a newly created admin user can successfully log in via the API.
///
/// This validates the basic authentication flow (`POST /login`)
/// using a user that is created specifically for this test.
#[test]
fn test_login() {
    create_user("test_admin", "password123", "admin");

    let client = Client::new();
    let response = client
        .post(format!("{}/login", APP_HOST))
        .json(&json!({
            "username": "test_admin",
            "password": "password123",
        }))
        .send()
        .expect("Request failed");

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Login failed with status: {}\nResponse: {}",
        response.status(),
        response.text().unwrap_or_default()
    );
}

/// Tests that a logged-in viewer user can access their identity via `GET /me`.
///
/// This ensures that:
/// - Session or cookie-based authentication is working
/// - The `/me` endpoint returns a correctly structured JSON object
/// - The user's identity and role are accurate in the response
#[test]
fn test_me() {
    create_user("test_viewer", "password123", "viewer");

    let client = Client::new();
    let login = client
        .post(format!("{}/login", APP_HOST))
        .json(&json!({
            "username": "test_viewer",
            "password": "password123",
        }))
        .send()
        .expect("Login request failed");

    assert_eq!(
        login.status(),
        StatusCode::OK,
        "Login failed: {}",
        login.text().unwrap_or_default()
    );

    // Preserve cookies/session
    let cookie = login
        .headers()
        .get("set-cookie")
        .expect("Missing set-cookie")
        .to_str()
        .unwrap();

    let me = client
        .get(format!("{}/me", APP_HOST))
        .header("cookie", cookie)
        .send()
        .expect("GET /me request failed");

    assert_eq!(
        me.status(),
        StatusCode::OK,
        "/me failed: {}",
        me.text().unwrap_or_default()
    );

    #[derive(serde::Deserialize, Debug)]
    struct MeResponse {
        username: String,
        roles: Vec<String>,
    }

    let json: MeResponse = me.json().expect("Failed to parse JSON from /me response");

    assert_eq!(
        json.username, "test_viewer",
        "Expected username to be 'test_viewer', got: {}",
        json.username
    );
    assert!(
        json.roles.contains(&"viewer".to_string()),
        "Expected roles to include 'viewer', got: {:?}",
        json.roles
    );
} // test_me

/// This test verifies that the `admin@example.com` user seeded by `bootstrap.sh`
/// is able to log in successfully.
///
/// # Purpose
/// - Ensures the default admin account created during local bootstrap is valid
/// - Provides coverage for human-oriented sandbox data (not test-generated data)
///
/// # Behavior
/// - Runs a real login request using the known credentials
/// - Fails if the user was not seeded or credentials have changed
///
/// # Notes
/// - This test is ignored by default and must be run manually or by a special CI job:
///   ```bash
///   cargo test -- --ignored
///   ```
/// - It should not be relied on for regression testing
/// - It is intended to catch configuration drift in development tooling
#[test]
#[ignore]
fn bootstrap_admin_can_login() {
    let client = Client::new();
    let response = client
        .post(format!("{}/login", APP_HOST))
        .json(&json!({
            "username": "admin@example.com",
            "password": "password123",
        }))
        .send()
        .expect("Login request failed");

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Expected bootstrap admin@example.com to log in, got: {}",
        response.text().unwrap_or_default()
    );
}

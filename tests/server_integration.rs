//! Integration tests for cr8s Server HTTP API
//!
//! These tests run against a live docker compose stack with:
//! - postgres database
//! - redis cache  
//! - cr8s server (HTTP API)
//!
//! Prerequisites:
//! 1. Build dev images: `./scripts/build-images.sh --dev`
//! 2. Set up environment: `./scripts/dev-test-setup.sh`
//! 3. Run tests: `cargo test --test server_integration`
//!
//! Tests mirror the Playwright test workflows and run sequentially.

use anyhow::{ensure, Context, Result};
use reqwest::{Client, StatusCode};
use serde_json::{json, Value};
use tokio::time::{sleep, Duration};

/// Base URL for the cr8s server API
const BASE_URL: &str = "http://127.0.0.1:8000";

/// HTTP client for making requests
fn http_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client")
}

/// Helper to make HTTP requests with optional authentication
async fn make_request(
    client: &Client,
    method: &str,
    path: &str,
    body: Option<Value>,
    auth_token: Option<&str>,
) -> Result<(StatusCode, Value)> {
    // ---

    let url = format!("{BASE_URL}{path}");

    let mut request = match method.to_uppercase().as_str() {
        "GET" => client.get(&url),
        "POST" => client.post(&url),
        "PUT" => client.put(&url),
        "DELETE" => client.delete(&url),
        _ => return Err(anyhow::anyhow!("Unsupported HTTP method: {}", method)),
    };

    // Add authentication header if token provided
    if let Some(token) = auth_token {
        request = request.header("Authorization", format!("Bearer {token}"));
    }

    // Add JSON body if provided
    if let Some(body) = body {
        request = request.json(&body);
    }

    let response = request
        .send()
        .await
        .context("Failed to send HTTP request")?;

    let status = response.status();
    let body_text = response
        .text()
        .await
        .context("Failed to read response body")?;

    // Try to parse as JSON, fall back to string if not valid JSON
    let body_json =
        serde_json::from_str(&body_text).unwrap_or_else(|_| json!({"raw_response": body_text}));

    Ok((status, body_json))
}

/// Helper to login and get auth token
async fn login_as_admin(client: &Client) -> Result<String> {
    // ---

    let login_body = json!({
        "username": "admin@example.com",
        "password": "password123"
    });

    let (status, response) =
        make_request(client, "POST", "/cr8s/login", Some(login_body), None).await?;

    ensure!(
        status == StatusCode::OK,
        "Login failed with status {}: {}",
        status,
        serde_json::to_string_pretty(&response)?
    );

    let token = response
        .get("token")
        .and_then(|t| t.as_str())
        .ok_or_else(|| anyhow::anyhow!("No token in login response: {}", response))?;

    Ok(token.to_string())
}

/// Wait for server to be ready
async fn wait_for_server_ready() -> Result<()> {
    // ---

    let client = http_client();
    let max_attempts = 15;
    let delay = Duration::from_secs(2);

    for attempt in 1..=max_attempts {
        match make_request(&client, "GET", "/cr8s/health", None, None).await {
            Ok((status, _)) if status.is_success() => {
                println!("✅ Server ready after {attempt} attempts");
                return Ok(());
            }
            _ => {
                if attempt == max_attempts {
                    return Err(anyhow::anyhow!(
                        "Server failed to become ready after {} attempts",
                        max_attempts
                    ));
                }
                println!("⏳ Server not ready, attempt {attempt}/{max_attempts}");
                sleep(delay).await;
            }
        }
    }

    unreachable!()
}

#[tokio::test]
async fn test_login_api() -> Result<()> {
    // ---
    // Tests direct API login flow that Playwright depends on
    // Validates: POST /cr8s/login endpoint, credential validation, token generation

    println!("🔐 Testing login API (mirrors Playwright login)");

    let client = http_client();
    wait_for_server_ready().await?;

    // Test successful login with admin credentials
    println!("📋 Step 1: Login with admin credentials");
    let login_body = json!({
        "username": "admin@example.com",
        "password": "password123"
    });

    let (status, response) =
        make_request(&client, "POST", "/cr8s/login", Some(login_body), None).await?;

    ensure!(
        status == StatusCode::OK,
        "Login failed with status {}: {}",
        status,
        serde_json::to_string_pretty(&response)?
    );

    // Verify token is returned
    let token = response
        .get("token")
        .and_then(|t| t.as_str())
        .ok_or_else(|| anyhow::anyhow!("No token in login response"))?;

    ensure!(!token.is_empty(), "Token is empty");
    println!(
        "✅ Login successful, token received: {}...",
        &token[..20.min(token.len())]
    );

    // Test invalid credentials
    println!("📋 Step 2: Testing invalid credentials");
    let invalid_login = json!({
        "username": "admin@example.com",
        "password": "wrongpassword"
    });

    let (status, _) =
        make_request(&client, "POST", "/cr8s/login", Some(invalid_login), None).await?;
    ensure!(
        status == StatusCode::UNAUTHORIZED,
        "Expected 401 for invalid credentials, got {}",
        status
    );

    println!("✅ Login API test passed!");
    Ok(())
}

#[tokio::test]
async fn test_create_rustacean_api() -> Result<()> {
    // ---
    // Tests authenticated author creation that Playwright depends on
    // Validates: POST /cr8s/rustaceans with Bearer token, author persistence, list verification

    println!("👤 Testing rustacean creation API (mirrors Playwright rustacean test)");

    let client = http_client();
    wait_for_server_ready().await?;

    // Step 1: Login to get token
    println!("📋 Step 1: Login as admin");
    let token = login_as_admin(&client).await?;

    // Step 2: Create a rustacean (author)
    println!("📋 Step 2: Create rustacean");
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let rustacean_body = json!({
        "name": "Playwright Test",
        "email": format!("test-{}@example.com", timestamp)
    });

    let (status, response) = make_request(
        &client,
        "POST",
        "/cr8s/rustaceans",
        Some(rustacean_body),
        Some(&token),
    )
    .await?;

    ensure!(
        status == StatusCode::CREATED,
        "Rustacean creation failed with status {}: {}",
        status,
        serde_json::to_string_pretty(&response)?
    );

    // Verify the response contains expected fields
    let rustacean_id = response
        .get("id")
        .and_then(|id| id.as_i64())
        .ok_or_else(|| anyhow::anyhow!("No id in rustacean response"))?;

    let rustacean_name = response
        .get("name")
        .and_then(|name| name.as_str())
        .ok_or_else(|| anyhow::anyhow!("No name in rustacean response"))?;

    ensure!(rustacean_name == "Playwright Test", "Name mismatch");
    ensure!(rustacean_id > 0, "Invalid rustacean ID");

    println!(
        "✅ Rustacean created successfully: ID {rustacean_id}, Name: {rustacean_name}"
    );

    // Step 3: Verify rustacean appears in list
    println!("📋 Step 3: Verify rustacean in list");
    let (status, response) =
        make_request(&client, "GET", "/cr8s/rustaceans", None, Some(&token)).await?;

    ensure!(status == StatusCode::OK, "Failed to list rustaceans");

    let rustaceans = response
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("Expected array of rustaceans"))?;

    let found = rustaceans
        .iter()
        .any(|r| r.get("name").and_then(|n| n.as_str()) == Some("Playwright Test"));

    ensure!(found, "Created rustacean not found in list");

    println!("✅ Rustacean creation API test passed!");
    Ok(())
}

#[tokio::test]
async fn test_create_crate_api() -> Result<()> {
    // ---
    // Tests complete crate creation workflow that Playwright depends on
    // Validates: author creation → crate creation → persistence verification (full workflow)
    println!("📦 Testing crate creation API (mirrors Playwright crate test)");

    let client = http_client();
    wait_for_server_ready().await?;

    // Step 1: Login to get token
    println!("📋 Step 1: Login as admin");
    let token = login_as_admin(&client).await?;

    // Step 2: Create an author first (like Playwright test does)
    println!("📋 Step 2: Create author for crate");
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let rustacean_body = json!({
        "name": "Playwright Test",
        "email": format!("test-crate-{}@example.com", timestamp)
    });

    let (status, response) = make_request(
        &client,
        "POST",
        "/cr8s/rustaceans",
        Some(rustacean_body),
        Some(&token),
    )
    .await?;

    ensure!(
        status == StatusCode::CREATED,
        "Author creation failed with status {}: {}",
        status,
        serde_json::to_string_pretty(&response)?
    );

    let author_id = response
        .get("id")
        .and_then(|id| id.as_i64())
        .ok_or_else(|| anyhow::anyhow!("No id in author response"))?;

    println!("✅ Author created for crate: ID {author_id}");

    // Step 3: Create a crate using the author we just created
    println!("📋 Step 3: Create crate");
    let crate_body = json!({
        "code": format!("{}", timestamp % 1_000_000),
        "name": format!("crate-{}", timestamp),
        "version": "1.0.0",
        "author_id": author_id,
        "description": format!("Created by test run at {}", timestamp)
    });

    let (status, response) = make_request(
        &client,
        "POST",
        "/cr8s/crates",
        Some(crate_body),
        Some(&token),
    )
    .await?;

    ensure!(
        status == StatusCode::OK,
        "Crate creation failed with status {}: {}",
        status,
        serde_json::to_string_pretty(&response)?
    );

    // Verify the response contains expected fields
    let crate_id = response
        .get("id")
        .and_then(|id| id.as_i64())
        .ok_or_else(|| anyhow::anyhow!("No id in crate response"))?;

    let crate_name = response
        .get("name")
        .and_then(|name| name.as_str())
        .ok_or_else(|| anyhow::anyhow!("No name in crate response"))?;

    ensure!(crate_name.starts_with("crate-"), "Name format mismatch");
    ensure!(crate_id > 0, "Invalid crate ID");

    println!(
        "✅ Crate created successfully: ID {crate_id}, Name: {crate_name}"
    );

    // Step 4: Verify crate appears in list
    println!("📋 Step 4: Verify crate in list");
    let (status, response) =
        make_request(&client, "GET", "/cr8s/crates", None, Some(&token)).await?;

    ensure!(status == StatusCode::OK, "Failed to list crates");

    let crates = response
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("Expected array of crates"))?;

    let found = crates
        .iter()
        .any(|c| c.get("name").and_then(|n| n.as_str()) == Some(crate_name));

    ensure!(found, "Created crate not found in list");

    println!("✅ Crate creation API test passed!");
    Ok(())
}

/// **Guard Integration Test**
///
/// Tests the complete Rocket guard system that unit tests in `rocket_routes/guards.rs`
/// intentionally skip. This validates the full HTTP authentication/authorization flow:
///
/// **Unit tests in `rocket_routes/guards.rs` cover:**
/// - Role logic only (`is_editor()`, `is_admin()` business rules)
/// - Using focused mocks, no HTTP/Rocket integration
///
/// **This integration test covers what unit tests skip:**
/// - Full Rocket guard flow (`GuardedAppUser::from_request`, `EditorUser::from_request`)  
/// - Bearer token extraction from HTTP headers → Redis session lookup → database user retrieval
/// - Real database role lookups with live PostgreSQL
/// - Session persistence across multiple HTTP requests
/// - Consistent authentication enforcement across all protected endpoints
///
/// **Together, unit + integration tests provide complete guard coverage.**
#[tokio::test]
async fn test_authentication_guard_workflow() -> Result<()> {
    // ---
    println!("🔐 Testing authentication guard workflow");

    let client = http_client();
    wait_for_server_ready().await?;

    // Test 1: No Authorization header = 401 Unauthorized
    println!("📋 Step 1: Testing no authentication");
    let (status, _) = make_request(&client, "GET", "/cr8s/rustaceans", None, None).await?;
    ensure!(
        status == StatusCode::UNAUTHORIZED,
        "Expected 401 for no auth, got {}",
        status
    );

    // Test 2: Invalid token = 401 Unauthorized
    println!("📋 Step 2: Testing invalid token");
    let (status, _) = make_request(
        &client,
        "GET",
        "/cr8s/rustaceans",
        None,
        Some("invalid-token"),
    )
    .await?;
    ensure!(
        status == StatusCode::UNAUTHORIZED,
        "Expected 401 for invalid token, got {}",
        status
    );

    // Test 3: Malformed Authorization header = 401 Unauthorized
    println!("📋 Step 3: Testing malformed auth header");
    let (status, _) = make_request(
        &client,
        "GET",
        "/cr8s/rustaceans",
        None,
        Some("not-bearer-format"),
    )
    .await?;
    ensure!(
        status == StatusCode::UNAUTHORIZED,
        "Expected 401 for malformed auth, got {}",
        status
    );

    // Test 4: Valid admin token = 200 OK (GuardedAppUser works)
    println!("📋 Step 4: Testing valid admin authentication");
    let admin_token = login_as_admin(&client).await?;
    let (status, response) =
        make_request(&client, "GET", "/cr8s/rustaceans", None, Some(&admin_token)).await?;
    ensure!(
        status == StatusCode::OK,
        "Expected 200 for valid admin token, got {}",
        status
    );

    // Verify we get an array response (basic validation)
    let rustaceans = response
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("Expected array response from /cr8s/rustaceans"))?;
    println!("✅ Admin can list rustaceans: {} items", rustaceans.len());

    // Test 5: Valid token can access editor endpoints (EditorUser guard)
    println!("📋 Step 5: Testing editor role authorization");

    // Create a rustacean (requires EditorUser guard)
    let rustacean_body = json!({
        "name": "Guard Test Author",
        "email": "guard-test@example.com"
    });

    let (status, response) = make_request(
        &client,
        "POST",
        "/cr8s/rustaceans",
        Some(rustacean_body),
        Some(&admin_token),
    )
    .await?;

    ensure!(
        status == StatusCode::CREATED,
        "Expected 201 for rustacean creation, got {}",
        status
    );

    let author_id = response
        .get("id")
        .and_then(|id| id.as_i64())
        .ok_or_else(|| anyhow::anyhow!("No id in rustacean response"))?;

    println!("✅ Admin can create rustaceans: ID {author_id}");

    // Test 6: Valid token can access crate endpoints
    println!("📋 Step 6: Testing crate creation authorization");

    let crate_body = json!({
        "code": "guard-test",
        "name": "Guard Test Crate",
        "version": "1.0.0",
        "author_id": author_id,
        "description": "Created during guard testing"
    });

    let (status, crate_response) = make_request(
        &client,
        "POST",
        "/cr8s/crates",
        Some(crate_body),
        Some(&admin_token),
    )
    .await?;

    ensure!(
        status == StatusCode::OK,
        "Expected 200 for crate creation, got {}",
        status
    );

    let crate_id = crate_response
        .get("id")
        .and_then(|id| id.as_i64())
        .ok_or_else(|| anyhow::anyhow!("No id in crate response"))?;

    println!("✅ Admin can create crates: ID {crate_id}");

    // Test 7: Token validation persists across requests
    println!("📋 Step 7: Testing token persistence");
    let (status, _) = make_request(&client, "GET", "/cr8s/me", None, Some(&admin_token)).await?;
    ensure!(
        status == StatusCode::OK,
        "Expected 200 for /cr8s/me with valid token, got {}",
        status
    );

    println!("✅ Token validation persists across requests");

    // Test 8: Different endpoints consistently enforce authentication
    println!("📋 Step 8: Testing consistent auth enforcement");

    let protected_endpoints = vec!["/cr8s/rustaceans", "/cr8s/crates", "/cr8s/me"];

    for endpoint in protected_endpoints {
        let (status, _) = make_request(&client, "GET", endpoint, None, None).await?;
        ensure!(
            status == StatusCode::UNAUTHORIZED,
            "Endpoint {} should require auth, got {}",
            endpoint,
            status
        );
    }

    println!("✅ All protected endpoints enforce authentication");

    println!("✅ Authentication guard workflow test passed!");
    Ok(())
}

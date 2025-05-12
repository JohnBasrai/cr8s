use anyhow::{ensure, Context, Result as AnyhowResult};
use cr8s::models::NewUser;
use cr8s::test_support::{assign_role, establish_test_connection, insert_test_user};
use reqwest::header;
use reqwest::{Client, ClientBuilder, StatusCode};
use serde_json::{json, Value};
use std::process::Command;

// Make the macros module public so it's available to all test files
#[macro_use]
pub mod macros;

pub static APP_HOST: &str = "http://127.0.0.1:8000";

pub async fn create_test_crate(client: &Client, rustacean: &Value) -> AnyhowResult<Value> {
    // ---
    let response = client
        .post(format!("{}/crates", APP_HOST))
        .json(&json!({
            "rustacean_id": rustacean["id"],
            "code": "foo",
            "name": "Foo crate",
            "version": "0.1",
            "description": "Foo crate description"
        }))
        .send()
        .await
        .context("failed to send POST /crates request")?;

    ensure_status!(response, StatusCode::CREATED);

    let json = response
        .json()
        .await
        .context("failed to parse JSON from crate creation response")?;

    Ok(json)
}

pub async fn delete_test_rustacean(client: &Client, rustacean: Value) -> AnyhowResult<()> {
    // ---
    let response = client
        .delete(format!("{}/rustaceans/{}", APP_HOST, rustacean["id"]))
        .send()
        .await
        .context("failed to send DELETE /rustaceans request")?;

    ensure_status!(response, StatusCode::NO_CONTENT);
    Ok(())
}

pub async fn delete_test_crate(client: &Client, a_crate: Value) -> AnyhowResult<()> {
    // ---
    let response = client
        .delete(format!("{}/crates/{}", APP_HOST, a_crate["id"]))
        .send()
        .await
        .context("failed to send DELETE /crates request")?;

    ensure_status!(response, StatusCode::NO_CONTENT);
    Ok(())
}

pub async fn get_logged_in_client(username: &str, role: &str) -> AnyhowResult<Client> {
    // ---
    use crate::test_support::{assign_role, establish_test_connection, insert_test_user};
    use anyhow::{ensure, Context};

    let mut conn = establish_test_connection()
        .await
        .context("failed to get DB connection for test login")?;

    let user = insert_test_user(
        &mut conn,
        &NewUser {
            username: username.to_string(),
            password: "1234".into(),
        },
    )
    .await
    .context("failed to insert test user")?;

    assign_role(&mut conn, user.id, &role.to_string())
        .await
        .context("failed to assign role to test user")?;

    let response = Client::new()
        .post(format!("{}/login", APP_HOST))
        .json(&json!({
            "username": username,
            "password": "1234",
        }))
        .send()
        .await
        .context("failed to log in test user")?;

    ensure_status!(response, StatusCode::OK, "login failed");

    let json: Value = response
        .json()
        .await
        .context("failed to parse login response JSON")?;

    let token = json
        .get("token")
        .and_then(|v| v.as_str())
        .context("login response missing token")?;

    let header_value = format!("Bearer {}", token);
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        header::HeaderValue::from_str(&header_value).context("invalid token header value")?,
    );

    let client = ClientBuilder::new()
        .default_headers(headers)
        .build()
        .context("failed to build authorized client")?;

    Ok(client)
}

pub async fn get_client_with_logged_in_viewer() -> AnyhowResult<Client> {
    get_logged_in_client("test_viewer", "Viewer").await
}

pub async fn get_client_with_logged_in_admin() -> AnyhowResult<Client> {
    get_logged_in_client("admin@example.com", "Admin").await
}

pub async fn create_test_rustacean(client: &Client) -> AnyhowResult<Value> {
    // ---
    let response = client
        .post(format!("{}/rustaceans", APP_HOST))
        .json(&json!({
            "name": "Foo bar",
            "email": "foo@bar.com"
        }))
        .send()
        .await
        .context("failed to send POST /rustaceans")?;

    ensure_status!(response, StatusCode::CREATED);

    let json = response
        .json()
        .await
        .context("failed to parse JSON from rustacean creation response")?;

    Ok(json)
}

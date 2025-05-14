use crate::auth::hash_password;
use crate::models::{NewUser, RoleCode, User};
use crate::schema::users;
use anyhow::{Context, Result as AnyhowResult};
use diesel_async::AsyncPgConnection;
use diesel_async::RunQueryDsl as AsyncRunQueryDsl;
use reqwest::header;
use reqwest::{Client, ClientBuilder, StatusCode};
use serde_json::{json, Value};

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

pub async fn get_logged_in_client(
    username: &str,
    raw_password: &str,
    role: RoleCode,
) -> AnyhowResult<Client> {
    // ---
    let mut conn = ensure_test_db_ready()
        .await
        .context("failed to get DB connection for test login")?;

    println!("get_logged_in_client: password:{raw_password}");
    let password_hash = match hash_password(raw_password.to_string()) {
        Ok(password) => password,
        Err(err) => {
            return Err(anyhow::anyhow!(
                "get_logged_in_client: failed to hash password:{err}"
            ));
        }
    };
    println!("get_logged_in_client: hash_password:{password_hash}");

    let user = insert_test_user(
        &mut conn,
        &NewUser {
            username: username.to_string(),
            password: password_hash.clone(), // insert hashed_password
        },
    )
    .await
    .context("failed to insert test user")?;

    assign_role(&mut conn, user.id, role)
        .await
        .context("failed to assign role to test user")?;

    println!("get_logged_in_client: /login {username} / hash_password:{password_hash}");
    let response = Client::new()
        .post(format!("{}/login", APP_HOST))
        .json(&json!({
            "username": username,
            "password": raw_password,
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

pub async fn ensure_test_db_ready() -> AnyhowResult<AsyncPgConnection> {
    // --
    use diesel_async::AsyncConnection;
    let url = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    eprintln!("🔍 DATABASE_URL = {}", url); // added

    if !url.contains("cr8s") {
        anyhow::bail!("Expected DATABASE_URL to point to 'cr8s'. Found: {}", url);
    }

    let conn = AsyncPgConnection::establish(&url).await?;

    Ok(conn)
}

// This test_support module provides support for Diesel-based integration tests without
// relying on the CLI:

/// Inserts a test user into the database using the provided connection.
///
/// # Arguments
/// * `conn` - A mutable reference to an asynchronous Postgres connection.
/// * `new_user` - A reference to a `NewUser` struct with the test user's data.
///
/// # Returns
/// Returns the inserted `User` on success, or an error if the insert fails.
pub async fn insert_test_user(
    conn: &mut AsyncPgConnection,
    new_user: &NewUser,
) -> AnyhowResult<User> {
    use diesel::insert_into;

    let username = new_user.username.clone();

    let user = AsyncRunQueryDsl::get_result(insert_into(users::table).values(new_user), conn)
        .await
        .context(format!("Failed to insert test user:{username}"))?;

    Ok(user)
}

/// Finds the role ID by code and assigns it to the user in the `user_roles` table.
pub async fn assign_role(
    conn: &mut AsyncPgConnection,
    user_id_value: i32,
    role_code: RoleCode,
) -> AnyhowResult<()> {
    // ---
    use crate::schema::roles::dsl::{code, id as role_id_column, roles};
    use crate::schema::user_roles::dsl::{
        role_id as role_id_col, user_id as user_id_col, user_roles,
    };
    use diesel::insert_into;
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;

    // 🔍 Step 1: Resolve role ID from role code
    let role_id: i32 = roles
        .select(role_id_column)
        .filter(code.eq(role_code))
        .first(conn)
        .await
        .context("Failed to find role by code")?;

    // ✅ Step 2: Insert into user_roles with resolved IDs
    insert_into(user_roles)
        .values((user_id_col.eq(user_id_value), role_id_col.eq(role_id)))
        .execute(conn)
        .await
        .context("Failed to insert into user_roles")?;

    Ok(())
}

/// Generates a unique username for test isolation
pub fn unique_username(base: &str) -> String {
    format!("{}-{}", base, uuid::Uuid::new_v4())
}

use super::test_utils::common::*;
use crate::ensure_status;
use crate::models::RoleCode;
use anyhow::{ensure, Context, Result};
use reqwest::StatusCode;
use serde_json::{json, Value};

#[tokio::test]
async fn test_get_crates() -> anyhow::Result<()> {
    use anyhow::{ensure, Context};
    use reqwest::StatusCode;
    use serde_json::Value;

    // Setup
    let password = "passwd-tgc";
    let username = unique_username("user-tgc");
    let client = get_logged_in_client(&username, password, RoleCode::Editor).await?;
    let rustacean = create_test_rustacean(&client).await?;
    let a_crate = create_test_crate(&client, &rustacean).await?;
    let b_crate = create_test_crate(&client, &rustacean).await?;

    // Test
    let response = client
        .get(format!("{}/crates", APP_HOST))
        .send()
        .await
        .context("failed to send GET /crates request")?;

    ensure_status!(response, StatusCode::OK);

    let json: Value = response.json().await.context("expected a JSON payload")?;

    let array = json
        .as_array()
        .context("expected top-level response to be a JSON array")?;

    ensure!(
        array.contains(&a_crate),
        "response JSON does not contain expected crate: {a_crate:?}"
    );

    ensure!(
        array.contains(&b_crate),
        "response JSON does not contain expected crate: {b_crate:?}"
    );

    // Cleanup
    delete_test_crate(&client, a_crate).await?;
    delete_test_crate(&client, b_crate).await?;
    delete_test_rustacean(&client, rustacean).await?;

    Ok(())
}

#[tokio::test]
async fn test_get_crates_not_loggedin_fails() -> anyhow::Result<()> {
    use anyhow::Context;
    use reqwest::StatusCode;

    // Setup
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/crates", APP_HOST))
        .send()
        .await
        .context("failed to send GET /crates request as unauthenticated user")?;

    ensure_status!(response, StatusCode::UNAUTHORIZED);

    Ok(())
}

#[tokio::test]
async fn test_view_crate() -> Result<()> {
    // ---
    // Setup
    let username = unique_username("user-tvc");
    let client = get_logged_in_client(&username, "passwd-tvc", RoleCode::Editor).await?;
    let rustacean = create_test_rustacean(&client).await?;
    let a_crate = create_test_crate(&client, &rustacean).await?;

    // Test
    let response = client
        .get(format!("{}/crates/{}", APP_HOST, a_crate["id"]))
        .send()
        .await
        .context("failed to send GET /crates/{id} request")?;

    ensure!(
        response.status() == StatusCode::OK,
        "expected 200 OK, got {}",
        response.status()
    );

    let actual: Value = response
        .json()
        .await
        .context("failed to parse JSON response from /crates/{id}")?;

    let expected = json!({
        "id": a_crate["id"],
        "code": "foo",
        "name": "Foo crate",
        "version": "0.1",
        "description": "Foo crate description",
        "rustacean_id": rustacean["id"],
        "created_at": actual["created_at"], // we accept what the server gave us
    });

    ensure!(
        actual == expected,
        "crate response did not match expected:\nexpected: {}\nactual: {}",
        expected,
        actual
    );

    // Cleanup
    delete_test_crate(&client, actual).await?;
    delete_test_rustacean(&client, rustacean).await?;

    Ok(())
}

#[tokio::test]
async fn test_update_crate() -> anyhow::Result<()> {
    use anyhow::{ensure, Context};
    use reqwest::StatusCode;
    use serde_json::json;

    // Setup
    let username = unique_username("user-tuc");
    let client = get_logged_in_client(&username, "passwd-tuc", RoleCode::Editor).await?;
    let rustacean = create_test_rustacean(&client).await?;
    let mut a_crate = create_test_crate(&client, &rustacean).await?;

    // Test: update crate with new metadata
    let response = client
        .put(format!("{}/crates/{}", APP_HOST, a_crate["id"]))
        .json(&json!({
            "code": "fooz",
            "name": "Fooz crate",
            "version": "0.2",
            "description": "Fooz crate description",
            "rustacean_id": rustacean["id"],
        }))
        .send()
        .await
        .context("failed to send first PUT /crates/{id} request")?;

    ensure!(
        response.status() == StatusCode::OK,
        "expected 200 OK from first update, got {}",
        response.status()
    );

    a_crate = response
        .json()
        .await
        .context("failed to parse first update response JSON")?;

    let expected = json!({
        "id": a_crate["id"],
        "code": "fooz",
        "name": "Fooz crate",
        "version": "0.2",
        "description": "Fooz crate description",
        "rustacean_id": rustacean["id"],
        "created_at": a_crate["created_at"],
    });

    ensure!(
        a_crate == expected,
        "first update did not match expected:\nexpected: {}\nactual: {}",
        expected,
        a_crate
    );

    // Test: update with long description and switch author
    let rustacean2 = create_test_rustacean(&client).await?;
    let long_description = r#"\
        Lorem ipsum dolor sit amet, consectetur adipiscing elit. Pellentesque gravida aliquet \n\
        arcu, non maximus urna iaculis et. Nam eu ante eu dolor volutpat maximus. Sed tincidunt \n\
        pretium elementum. Quisque rutrum ex id sem luctus rhoncus ac ultrices lacus. Ut \n\
        vulputate magna facilisis dignissim porttitor. Nulla vitae pretium neque. Vestibulum \n\
        rutrum semper justo, ut mattis diam. Curabitur a tempus felis. Pellentesque sit amet \n\
        pharetra nunc. Curabitur est nunc, tincidunt sit amet arcu sed, bibendum accumsan \n\
        ligula. Maecenas eu dolor sed mi viverra congue. Phasellus vel dignissim lacus, vel \n\
        tempor velit. Vestibulum vulputate sapien nisi, ac ullamcorper enim sodales vitae. \n\
        Aliquam erat volutpat. Etiam tincidunt aliquet velit ac vulputate. Aenean et augue \n\
        dolor.
"#;

    let response = client
        .put(format!("{}/crates/{}", APP_HOST, a_crate["id"]))
        .json(&json!({
            "code": "fooz",
            "name": "Fooz crate",
            "version": "0.2",
            "description": long_description,
            "rustacean_id": rustacean2["id"],
        }))
        .send()
        .await
        .context("failed to send second PUT /crates/{id} request")?;

    ensure_status!(response, StatusCode::OK, "from second update");

    a_crate = response
        .json()
        .await
        .context("failed to parse second update response JSON")?;

    let expected = json!({
        "id": a_crate["id"],
        "code": "fooz",
        "name": "Fooz crate",
        "version": "0.2",
        "description": long_description,
        "rustacean_id": rustacean2["id"],
        "created_at": a_crate["created_at"],
    });

    ensure!(
        a_crate == expected,
        "second update did not match expected:\nexpected: {}\nactual: {}",
        expected,
        a_crate
    );

    let desc = a_crate["description"]
        .as_str()
        .context("description field was not a string")?;

    ensure!(
        desc == long_description,
        "description mismatch: expected:\n{}\ngot:\n{}",
        long_description,
        desc
    );

    ensure!(
        !desc.contains('\r'),
        "description contains unexpected carriage return characters (\\r): {:?}",
        desc
    );

    // Cleanup
    delete_test_crate(&client, a_crate).await?;
    delete_test_rustacean(&client, rustacean).await?;
    delete_test_rustacean(&client, rustacean2).await?;

    Ok(())
}

#[tokio::test]
async fn test_delete_crate() -> anyhow::Result<()> {
    // Setup
    let password = "passwd-tdc";
    let username = unique_username("user-tdc");

    println!("👤 Creating user: {username}");
    println!("🔐 Logging in as: {username} with password: {password}");

    let client = get_logged_in_client(&username, password, RoleCode::Editor).await?;
    let rustacean = create_test_rustacean(&client).await?;
    let a_crate = create_test_crate(&client, &rustacean).await?;

    // Test
    let response = client
        .delete(format!("{}/crates/{}", APP_HOST, a_crate["id"]))
        .send()
        .await
        .context("failed to send DELETE /crates/{id} request")?;

    ensure_status!(response, StatusCode::NO_CONTENT);

    // Cleanup
    delete_test_rustacean(&client, rustacean).await?;

    Ok(())
}

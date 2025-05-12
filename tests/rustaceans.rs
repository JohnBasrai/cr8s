use anyhow::{ensure, Context, Result};
use common::*;
use reqwest::{Client, StatusCode};
use serde_json::{json, Value};

#[tokio::test]
async fn test_get_rustaceans() -> anyhow::Result<()> {
    // ---
    let client_adm = common::get_client_with_logged_in_admin().await?;
    let rustacean1 = common::create_test_rustacean(&client_adm).await?;
    let rustacean2 = common::create_test_rustacean(&client_adm).await?;

    // Test
    let client_view = common::get_client_with_logged_in_viewer().await?;
    let response = client_view
        .get(format!("{}/rustaceans", common::APP_HOST))
        .send()
        .await
        .context("failed to send GET /rustaceans")?;

    ensure_status!(response, StatusCode::OK);

    let json: Value = response
        .json()
        .await
        .context("failed to parse response JSON from GET /rustaceans")?;

    let array = json
        .as_array()
        .context("expected JSON array in GET /rustaceans response")?;

    ensure!(
        array.contains(&rustacean1),
        "response JSON missing rustacean1: {rustacean1:?}"
    );
    ensure!(
        array.contains(&rustacean2),
        "response JSON missing rustacean2: {rustacean2:?}"
    );

    // Cleanup
    common::delete_test_rustacean(&client_adm, rustacean1).await?;
    common::delete_test_rustacean(&client_adm, rustacean2).await?;

    Ok(())
}

#[tokio::test]
async fn test_get_rustaceans_not_loggedin_fails() -> anyhow::Result<()> {
    // ---
    let client = Client::new();

    let response = client
        .get(format!("{}/rustaceans", common::APP_HOST))
        .send()
        .await
        .context("failed to send GET /rustaceans without login")?;

    ensure_status!(response, StatusCode::UNAUTHORIZED);

    Ok(())
}

#[tokio::test]
async fn test_create_rustacean() -> anyhow::Result<()> {
    // ---
    let client = common::get_client_with_logged_in_admin().await?;

    let response = client
        .post(format!("{}/rustaceans", common::APP_HOST))
        .json(&json!({
            "name": "Foo bar",
            "email": "foo@bar.com"
        }))
        .send()
        .await
        .context("failed to send POST /rustaceans")?;

    ensure_status!(response, StatusCode::CREATED);

    let rustacean: Value = response
        .json()
        .await
        .context("failed to parse JSON from /rustaceans response")?;

    let expected = json!({
        "id": rustacean["id"],
        "name": "Foo bar",
        "email": "foo@bar.com",
        "created_at": rustacean["created_at"],
    });

    ensure!(
        rustacean == expected,
        "unexpected rustacean response: {rustacean:?}"
    );

    common::delete_test_rustacean(&client, rustacean).await?;
    Ok(())
}

#[tokio::test]
async fn test_view_rustacean() -> anyhow::Result<()> {
    // ---
    let client = common::get_client_with_logged_in_admin().await?;
    let rustacean = common::create_test_rustacean(&client).await?;

    let client_view = common::get_client_with_logged_in_viewer().await?;
    let response = client_view
        .get(format!(
            "{}/rustaceans/{}",
            common::APP_HOST,
            rustacean["id"]
        ))
        .send()
        .await
        .context("failed to send GET /rustaceans/{id}")?;

    ensure_status!(response, StatusCode::OK);

    let fetched: Value = response
        .json()
        .await
        .context("failed to parse JSON from GET /rustaceans/{id}")?;

    let expected = json!({
        "id": rustacean["id"],
        "name": "Foo bar",
        "email": "foo@bar.com",
        "created_at": rustacean["created_at"],
    });

    ensure!(
        fetched == expected,
        "unexpected rustacean response: {fetched:?}"
    );

    common::delete_test_rustacean(&client, fetched).await?;
    Ok(())
}

#[tokio::test]
async fn test_update_rustacean() -> anyhow::Result<()> {
    // ---
    let client = common::get_client_with_logged_in_admin().await?;
    let rustacean = common::create_test_rustacean(&client).await?;

    let response = client
        .put(format!(
            "{}/rustaceans/{}",
            common::APP_HOST,
            rustacean["id"]
        ))
        .json(&json!({
            "name": "Fooz bar",
            "email": "fooz@bar.com"
        }))
        .send()
        .await
        .context("failed to send PUT /rustaceans/{id}")?;

    ensure_status!(response, StatusCode::OK);

    let updated: Value = response
        .json()
        .await
        .context("failed to parse JSON from update response")?;

    let expected = json!({
        "id": rustacean["id"],
        "name": "Fooz bar",
        "email": "fooz@bar.com",
        "created_at": rustacean["created_at"],
    });

    ensure!(
        updated == expected,
        "unexpected rustacean update result: {updated:?}"
    );

    common::delete_test_rustacean(&client, updated).await?;
    Ok(())
}

#[tokio::test]
async fn test_delete_rustacean() -> anyhow::Result<()> {
    // ---
    let client = common::get_client_with_logged_in_admin().await?;
    let rustacean = common::create_test_rustacean(&client).await?;

    let response = client
        .delete(format!(
            "{}/rustaceans/{}",
            common::APP_HOST,
            rustacean["id"]
        ))
        .send()
        .await
        .context("failed to send DELETE /rustaceans/{id}")?;

    ensure_status!(response, StatusCode::NO_CONTENT);

    Ok(())
}

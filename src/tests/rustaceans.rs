use crate::ensure_status;
use crate::models::RoleCode;
use crate::tests::test_utils::common::*;
use anyhow::{ensure, Context};
use reqwest::{Client, StatusCode};
use serde_json::{json, Value};

#[tokio::test]
async fn test_get_rustaceans() -> anyhow::Result<()> {
    // ---
    let password = "passwd-tgr";
    let username = unique_username("user-tgr-admin");
    let client_adm = get_logged_in_client(&username, password, RoleCode::Admin).await?;
    let rustacean1 = create_test_rustacean(&client_adm).await?;
    let rustacean2 = create_test_rustacean(&client_adm).await?;

    // test
    let username = unique_username("user-tgr-viewer");
    let client_view = get_logged_in_client(&username, password, RoleCode::Viewer).await?;
    let response = client_view
        .get(format!("{}/rustaceans", APP_HOST))
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
    delete_test_rustacean(&client_adm, rustacean1).await?;
    delete_test_rustacean(&client_adm, rustacean2).await?;

    Ok(())
}

#[tokio::test]
async fn test_get_rustaceans_not_loggedin_fails() -> anyhow::Result<()> {
    // ---
    let client = Client::new();

    let response = client
        .get(format!("{}/rustaceans", APP_HOST))
        .send()
        .await
        .context("failed to send GET /rustaceans without login")?;

    ensure_status!(response, StatusCode::UNAUTHORIZED);

    Ok(())
}

#[tokio::test]
async fn test_create_rustacean() -> anyhow::Result<()> {
    // ---
    let password = "passwd-tcr";
    let username = unique_username("user-tcr");
    let client = get_logged_in_client(&username, password, RoleCode::Editor).await?;

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

    delete_test_rustacean(&client, rustacean).await?;
    Ok(())
}

#[tokio::test]
async fn test_view_rustacean() -> anyhow::Result<()> {
    // ---
    let password = "passwd-tvr";

    let (editor_name, viewer_name) = (unique_username("user-tvr-e"), unique_username("user-tvr-v"));

    let client = get_logged_in_client(&editor_name, password, RoleCode::Editor).await?;
    let rustacean = create_test_rustacean(&client).await?;

    let client_view = get_logged_in_client(&viewer_name, password, RoleCode::Viewer).await?;
    let response = client_view
        .get(format!("{}/rustaceans/{}", APP_HOST, rustacean["id"]))
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

    delete_test_rustacean(&client, fetched).await?;
    Ok(())
}

#[tokio::test]
async fn test_update_rustacean() -> anyhow::Result<()> {
    // ---
    let username = unique_username("user-tur");
    let client = get_logged_in_client(&username, "passwd-tur", RoleCode::Editor).await?;
    let rustacean = create_test_rustacean(&client).await?;

    let response = client
        .put(format!("{}/rustaceans/{}", APP_HOST, rustacean["id"]))
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

    delete_test_rustacean(&client, updated).await?;
    Ok(())
}

#[tokio::test]
async fn test_delete_rustacean() -> anyhow::Result<()> {
    // ---
    let username = unique_username("user-tdr");
    let client = get_logged_in_client(&username, "passwd-tdr", RoleCode::Editor).await?;
    let rustacean = create_test_rustacean(&client).await?;

    let response = client
        .delete(format!("{}/rustaceans/{}", APP_HOST, rustacean["id"]))
        .send()
        .await
        .context("failed to send DELETE /rustaceans/{id}")?;

    ensure_status!(response, StatusCode::NO_CONTENT);

    Ok(())
}

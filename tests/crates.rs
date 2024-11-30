use reqwest::StatusCode;
use serde_json::{json, Value};

pub mod common;


#[test]
fn test_get_crates() {
    // Setup
    let client = common::get_client_with_logged_in_admin();
    let rustacean = common::create_test_rustacean(&client);
    let a_crate = common::create_test_crate(&client, &rustacean);
    let b_crate = common::create_test_crate(&client, &rustacean);

    // Test
    let client = common::get_client_with_logged_in_viewer();
    let response = client.get(format!("{}/crates", common::APP_HOST)).send().unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let json: Value = response.json().unwrap();
    assert!(json.as_array().unwrap().contains(&a_crate));
    assert!(json.as_array().unwrap().contains(&b_crate));

    // Cleanup
    let client = common::get_client_with_logged_in_admin();
    common::delete_test_crate(&client, a_crate);
    common::delete_test_crate(&client, b_crate);
    common::delete_test_rustacean(&client, rustacean);
}

#[test]
fn test_create_crate() {
    // Setup
    let client = common::get_client_with_logged_in_admin();
    let rustacean = common::create_test_rustacean(&client);

    // Test
    let response = client.post(format!("{}/crates", common::APP_HOST))
        .json(&json!({
            "rustacean_id": rustacean["id"],
            "code": "foo",
            "name": "Foo crate",
            "version": "0.1",
            "description": "Foo crate description"
        }))
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let a_crate: Value = response.json().unwrap();
    assert_eq!(a_crate, json!({
        "id": a_crate["id"],
        "code": "foo",
        "name": "Foo crate",
        "version": "0.1",
        "description": "Foo crate description",
        "rustacean_id": rustacean["id"],
        "created_at": a_crate["created_at"],
    }));

    // Cleanup
    common::delete_test_crate(&client, a_crate);
    common::delete_test_rustacean(&client, rustacean);
}

#[test]
fn test_view_crate() {
    // Setup
    let client = common::get_client_with_logged_in_admin();
    let rustacean = common::create_test_rustacean(&client);
    let a_crate = common::create_test_crate(&client, &rustacean);

    // Test
    let client = common::get_client_with_logged_in_viewer();
    let response = client.get(format!("{}/crates/{}", common::APP_HOST, a_crate["id"]))
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let a_crate: Value = response.json().unwrap();
    assert_eq!(a_crate, json!({
        "id": a_crate["id"],
        "code": "foo",
        "name": "Foo crate",
        "version": "0.1",
        "description": "Foo crate description",
        "rustacean_id": rustacean["id"],
        "created_at": a_crate["created_at"],
    }));

    // Cleanup
    let client = common::get_client_with_logged_in_admin();
    common::delete_test_crate(&client, a_crate);
    common::delete_test_rustacean(&client, rustacean);
}

#[test]
fn test_update_crate() {
    // Setup
    let client = common::get_client_with_logged_in_admin();
    let rustacean = common::create_test_rustacean(&client);
    let a_crate = common::create_test_crate(&client, &rustacean);

    // Test
    let response = client.put(format!("{}/crates/{}", common::APP_HOST, a_crate["id"]))
        .json(&json!({
            "code": "fooz",
            "name": "Fooz crate",
            "version": "0.2",
            "description": "Fooz crate description",
            "rustacean_id": rustacean["id"],
        }))
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let a_crate: Value = response.json().unwrap();
    assert_eq!(a_crate, json!({
        "id": a_crate["id"],
        "code": "fooz",
        "name": "Fooz crate",
        "version": "0.2",
        "description": "Fooz crate description",
        "rustacean_id": rustacean["id"],
        "created_at": a_crate["created_at"],
    }));

    // Test author-switching for a crate and a very long description.
    let rustacean2 = common::create_test_rustacean(&client);
    let response = client.put(format!("{}/crates/{}", common::APP_HOST, a_crate["id"]))
        .json(&json!({
            "code": "fooz",
            "name": "Fooz crate",
            "version": "0.2",
            "description": "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Pellentesque gravida aliquet arcu, non maximus urna iaculis et. Nam eu ante eu dolor volutpat maximus. Sed tincidunt pretium elementum. Quisque rutrum ex id sem luctus rhoncus ac ultrices lacus. Ut vulputate magna facilisis dignissim porttitor. Nulla vitae pretium neque. Vestibulum rutrum semper justo, ut mattis diam. Curabitur a tempus felis. Pellentesque sit amet pharetra nunc. Curabitur est nunc, tincidunt sit amet arcu sed, bibendum accumsan ligula. Maecenas eu dolor sed mi viverra congue. Phasellus vel dignissim lacus, vel tempor velit. Vestibulum vulputate sapien nisi, ac ullamcorper enim sodales vitae. Aliquam erat volutpat. Etiam tincidunt aliquet velit ac vulputate. Aenean et augue dolor.",
            "rustacean_id": rustacean2["id"],
        }))
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let a_crate: Value = response.json().unwrap();
    assert_eq!(a_crate, json!({
        "id": a_crate["id"],
        "code": "fooz",
        "name": "Fooz crate",
        "version": "0.2",
        "description": "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Pellentesque gravida aliquet arcu, non maximus urna iaculis et. Nam eu ante eu dolor volutpat maximus. Sed tincidunt pretium elementum. Quisque rutrum ex id sem luctus rhoncus ac ultrices lacus. Ut vulputate magna facilisis dignissim porttitor. Nulla vitae pretium neque. Vestibulum rutrum semper justo, ut mattis diam. Curabitur a tempus felis. Pellentesque sit amet pharetra nunc. Curabitur est nunc, tincidunt sit amet arcu sed, bibendum accumsan ligula. Maecenas eu dolor sed mi viverra congue. Phasellus vel dignissim lacus, vel tempor velit. Vestibulum vulputate sapien nisi, ac ullamcorper enim sodales vitae. Aliquam erat volutpat. Etiam tincidunt aliquet velit ac vulputate. Aenean et augue dolor.",
        "rustacean_id": rustacean2["id"],
        "created_at": a_crate["created_at"],
    }));

    // Cleanup
    common::delete_test_crate(&client, a_crate);
    common::delete_test_rustacean(&client, rustacean);
}

#[test]
fn test_delete_crate() {
    // Setup
    let client = common::get_client_with_logged_in_admin();
    let rustacean = common::create_test_rustacean(&client);
    let a_crate = common::create_test_crate(&client, &rustacean);

    // Test
    let response = client.delete(format!("{}/crates/{}", common::APP_HOST, a_crate["id"]))
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Cleanup
    common::delete_test_rustacean(&client, rustacean);
}

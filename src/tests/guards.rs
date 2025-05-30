#[rocket::async_test]
async fn editor_user_can_access_protected_route() {
    let client = rocket::build()
        .mount("/", routes![your_protected_route])
        .manage(mock_cache_context())
        .manage(mock_app_user_repo())
        .ignite()
        .await
        .unwrap();

    let req = client
        .get("/rustaceans")
        .header(Authorization::bearer("valid-token"));
    let response = req.dispatch().await;
    assert_eq!(response.status(), Status::Ok);
}

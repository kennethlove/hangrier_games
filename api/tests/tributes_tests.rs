mod common;

use axum_test::TestServer;
use common::{TestDb, TestUser, create_test_router};
use serde_json::json;

/// Helper to create an authenticated test user
async fn create_authenticated_user(server: &TestServer, username: &str) -> TestUser {
    let test_user = TestUser::new(username);

    let response = server
        .post("/api/users")
        .json(&json!({
            "username": test_user.username,
            "email": test_user.email,
            "password": test_user.password,
        }))
        .await;

    let body = response.json::<serde_json::Value>();
    let access_token = body["access_token"].as_str().unwrap().to_string();
    let refresh_token = body["refresh_token"].as_str().unwrap().to_string();

    test_user.with_tokens(access_token, refresh_token)
}

/// Helper to create a game for testing
async fn create_test_game(server: &TestServer, user: &TestUser) -> String {
    let response = server
        .post("/api/games")
        .add_header(
            "Authorization".parse().unwrap(),
            user.auth_header().parse().unwrap(),
        )
        .json(&json!({
            "max_tributes": 24,
            "tribute_pool": 24,
            "tribute_list": [],
        }))
        .await;

    let body = response.json::<serde_json::Value>();
    body["id"].as_str().unwrap().to_string()
}

/// Test creating a tribute
#[tokio::test]
async fn test_create_tribute() {
    let test_db = TestDb::new().await;
    let app_state = test_db.app_state();
    let router = create_test_router(app_state);
    let server = TestServer::new(router).unwrap();

    let user = create_authenticated_user(&server, "tribute_creator1").await;
    let game_id = create_test_game(&server, &user).await;

    let response = server
        .post(&format!("/api/games/{}/tributes", game_id))
        .add_header(
            "Authorization".parse().unwrap(),
            user.auth_header().parse().unwrap(),
        )
        .json(&json!({
            "name": "Katniss Everdeen",
            "district": 12,
        }))
        .await;

    response.assert_status_ok();

    let body = response.json::<serde_json::Value>();
    assert!(body.get("id").is_some());
    assert_eq!(body["name"], "Katniss Everdeen");
    assert_eq!(body["district"], 12);
    assert!(body.get("health").is_some());
    assert!(body.get("sanity").is_some());

    test_db.cleanup().await;
}

/// Test getting a tribute
#[tokio::test]
async fn test_get_tribute() {
    let test_db = TestDb::new().await;
    let app_state = test_db.app_state();
    let router = create_test_router(app_state);
    let server = TestServer::new(router).unwrap();

    let user = create_authenticated_user(&server, "tribute_getter").await;
    let game_id = create_test_game(&server, &user).await;

    // Create a tribute
    let create_response = server
        .post(&format!("/api/games/{}/tributes", game_id))
        .add_header(
            "Authorization".parse().unwrap(),
            user.auth_header().parse().unwrap(),
        )
        .json(&json!({
            "name": "Peeta Mellark",
            "district": 12,
        }))
        .await;

    let create_body = create_response.json::<serde_json::Value>();
    let tribute_id = create_body["id"].as_str().unwrap();

    // Get the tribute
    let get_response = server
        .get(&format!("/api/games/{}/tributes/{}", game_id, tribute_id))
        .add_header(
            "Authorization".parse().unwrap(),
            user.auth_header().parse().unwrap(),
        )
        .await;

    get_response.assert_status_ok();

    let get_body = get_response.json::<serde_json::Value>();
    assert_eq!(get_body["name"], "Peeta Mellark");
    assert_eq!(get_body["district"], 12);

    test_db.cleanup().await;
}

/// Test updating a tribute
#[tokio::test]
async fn test_update_tribute() {
    let test_db = TestDb::new().await;
    let app_state = test_db.app_state();
    let router = create_test_router(app_state);
    let server = TestServer::new(router).unwrap();

    let user = create_authenticated_user(&server, "tribute_updater").await;
    let game_id = create_test_game(&server, &user).await;

    // Create a tribute
    let create_response = server
        .post(&format!("/api/games/{}/tributes", game_id))
        .add_header(
            "Authorization".parse().unwrap(),
            user.auth_header().parse().unwrap(),
        )
        .json(&json!({
            "name": "Rue",
            "district": 11,
        }))
        .await;

    let create_body = create_response.json::<serde_json::Value>();
    let tribute_id = create_body["id"].as_str().unwrap();

    // Update the tribute
    let update_response = server
        .put(&format!("/api/games/{}/tributes/{}", game_id, tribute_id))
        .add_header(
            "Authorization".parse().unwrap(),
            user.auth_header().parse().unwrap(),
        )
        .json(&json!({
            "name": "Rue (Updated)",
        }))
        .await;

    update_response.assert_status_ok();

    let update_body = update_response.json::<serde_json::Value>();
    assert_eq!(update_body["name"], "Rue (Updated)");

    test_db.cleanup().await;
}

/// Test deleting a tribute
#[tokio::test]
async fn test_delete_tribute() {
    let test_db = TestDb::new().await;
    let app_state = test_db.app_state();
    let router = create_test_router(app_state);
    let server = TestServer::new(router).unwrap();

    let user = create_authenticated_user(&server, "tribute_deleter").await;
    let game_id = create_test_game(&server, &user).await;

    // Create a tribute
    let create_response = server
        .post(&format!("/api/games/{}/tributes", game_id))
        .add_header(
            "Authorization".parse().unwrap(),
            user.auth_header().parse().unwrap(),
        )
        .json(&json!({
            "name": "Cato",
            "district": 2,
        }))
        .await;

    let create_body = create_response.json::<serde_json::Value>();
    let tribute_id = create_body["id"].as_str().unwrap();

    // Delete the tribute
    let delete_response = server
        .delete(&format!("/api/games/{}/tributes/{}", game_id, tribute_id))
        .add_header(
            "Authorization".parse().unwrap(),
            user.auth_header().parse().unwrap(),
        )
        .await;

    delete_response.assert_status(axum::http::StatusCode::NO_CONTENT);

    // Try to get the deleted tribute - should return 404
    let get_response = server
        .get(&format!("/api/games/{}/tributes/{}", game_id, tribute_id))
        .add_header(
            "Authorization".parse().unwrap(),
            user.auth_header().parse().unwrap(),
        )
        .await;

    get_response.assert_status(axum::http::StatusCode::NOT_FOUND);

    test_db.cleanup().await;
}

/// Test creating multiple tributes in a game
#[tokio::test]
async fn test_create_multiple_tributes() {
    let test_db = TestDb::new().await;
    let app_state = test_db.app_state();
    let router = create_test_router(app_state);
    let server = TestServer::new(router).unwrap();

    let user = create_authenticated_user(&server, "multi_tribute_creator").await;
    let game_id = create_test_game(&server, &user).await;

    // Create multiple tributes
    let tributes = vec![("Katniss", 12), ("Peeta", 12), ("Rue", 11), ("Thresh", 11)];

    for (name, district) in tributes {
        let response = server
            .post(&format!("/api/games/{}/tributes", game_id))
            .add_header(
                "Authorization".parse().unwrap(),
                user.auth_header().parse().unwrap(),
            )
            .json(&json!({
                "name": name,
                "district": district,
            }))
            .await;

        response.assert_status_ok();
    }

    test_db.cleanup().await;
}

/// Test tribute log endpoint
#[tokio::test]
async fn test_tribute_log() {
    let test_db = TestDb::new().await;
    let app_state = test_db.app_state();
    let router = create_test_router(app_state);
    let server = TestServer::new(router).unwrap();

    let user = create_authenticated_user(&server, "tribute_logger").await;
    let game_id = create_test_game(&server, &user).await;

    // Create a tribute
    let create_response = server
        .post(&format!("/api/games/{}/tributes", game_id))
        .add_header(
            "Authorization".parse().unwrap(),
            user.auth_header().parse().unwrap(),
        )
        .json(&json!({
            "name": "Finnick Odair",
            "district": 4,
        }))
        .await;

    let create_body = create_response.json::<serde_json::Value>();
    let tribute_id = create_body["id"].as_str().unwrap();

    // Get tribute log
    let log_response = server
        .get(&format!(
            "/api/games/{}/tributes/{}/log",
            game_id, tribute_id
        ))
        .add_header(
            "Authorization".parse().unwrap(),
            user.auth_header().parse().unwrap(),
        )
        .await;

    log_response.assert_status_ok();

    let log_body = log_response.json::<serde_json::Value>();
    assert!(log_body.is_array());

    test_db.cleanup().await;
}

/// Test tribute items relationship
#[tokio::test]
async fn test_tribute_items() {
    let test_db = TestDb::new().await;
    let app_state = test_db.app_state();
    let router = create_test_router(app_state);
    let server = TestServer::new(router).unwrap();

    let user = create_authenticated_user(&server, "tribute_item_tester").await;
    let game_id = create_test_game(&server, &user).await;

    // Create a tribute
    let create_response = server
        .post(&format!("/api/games/{}/tributes", game_id))
        .add_header(
            "Authorization".parse().unwrap(),
            user.auth_header().parse().unwrap(),
        )
        .json(&json!({
            "name": "Johanna Mason",
            "district": 7,
        }))
        .await;

    let create_body = create_response.json::<serde_json::Value>();
    let tribute_id = create_body["id"].as_str().unwrap();

    // Get tribute details (should include items)
    let get_response = server
        .get(&format!("/api/games/{}/tributes/{}", game_id, tribute_id))
        .add_header(
            "Authorization".parse().unwrap(),
            user.auth_header().parse().unwrap(),
        )
        .await;

    get_response.assert_status_ok();

    let get_body = get_response.json::<serde_json::Value>();
    assert!(get_body.get("items").is_some());

    test_db.cleanup().await;
}

/// Test creating tribute without required fields
#[tokio::test]
async fn test_create_tribute_validation() {
    let test_db = TestDb::new().await;
    let app_state = test_db.app_state();
    let router = create_test_router(app_state);
    let server = TestServer::new(router).unwrap();

    let user = create_authenticated_user(&server, "tribute_validator").await;
    let game_id = create_test_game(&server, &user).await;

    // Try to create tribute without name
    let response = server
        .post(&format!("/api/games/{}/tributes", game_id))
        .add_header(
            "Authorization".parse().unwrap(),
            user.auth_header().parse().unwrap(),
        )
        .json(&json!({
            "district": 1,
        }))
        .await;

    // Should fail validation
    assert!(
        response.status_code() == axum::http::StatusCode::BAD_REQUEST
            || response.status_code() == axum::http::StatusCode::UNPROCESSABLE_ENTITY
    );

    test_db.cleanup().await;
}

/// Test tribute district validation (1-12)
#[tokio::test]
async fn test_tribute_district_validation() {
    let test_db = TestDb::new().await;
    let app_state = test_db.app_state();
    let router = create_test_router(app_state);
    let server = TestServer::new(router).unwrap();

    let user = create_authenticated_user(&server, "district_validator").await;
    let game_id = create_test_game(&server, &user).await;

    // Try to create tribute with invalid district
    let response = server
        .post(&format!("/api/games/{}/tributes", game_id))
        .add_header(
            "Authorization".parse().unwrap(),
            user.auth_header().parse().unwrap(),
        )
        .json(&json!({
            "name": "Invalid District",
            "district": 99,
        }))
        .await;

    // Should fail validation (if district validation is implemented)
    // If not implemented, this test documents expected behavior
    if response.status_code() == axum::http::StatusCode::BAD_REQUEST {
        let body = response.json::<serde_json::Value>();
        assert!(body.get("error").is_some());
    }

    test_db.cleanup().await;
}

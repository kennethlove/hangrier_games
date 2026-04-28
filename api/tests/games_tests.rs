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

/// Test creating a game
#[tokio::test]
async fn test_create_game() {
    let test_db = TestDb::new().await;
    let app_state = test_db.app_state();
    let router = create_test_router(app_state);
    let server = TestServer::new(router);

    let user = create_authenticated_user(&server, "game_creator1").await;

    let response = server
        .post("/api/games")
        .add_header("Authorization", user.auth_header())
        .json(&json!({
            "name": "Test Game",
        }))
        .await;

    response.assert_status_ok();

    let body = response.json::<serde_json::Value>();
    assert!(body.get("identifier").is_some());
    assert_eq!(body["name"], "Test Game");
    assert_eq!(body["status"], "NotStarted");

    test_db.cleanup().await;
}

/// Test listing games with pagination
#[tokio::test]
async fn test_list_games() {
    let test_db = TestDb::new().await;
    let app_state = test_db.app_state();
    let router = create_test_router(app_state);
    let server = TestServer::new(router);

    let user = create_authenticated_user(&server, "game_lister").await;

    // Create a few games
    for _ in 0..3 {
        server
            .post("/api/games")
            .add_header("Authorization", user.auth_header())
            .json(&json!({
                "max_tributes": 24,
                "tribute_pool": 24,
                "tribute_list": [],
            }))
            .await
            .assert_status_ok();
    }

    // List games
    let response = server
        .get("/api/games")
        .add_header("Authorization", user.auth_header())
        .await;

    response.assert_status_ok();

    let body = response.json::<serde_json::Value>();
    assert!(body.get("games").is_some());
    assert!(body.get("pagination").is_some());

    let games = body["games"].as_array().unwrap();
    assert!(games.len() >= 3);

    test_db.cleanup().await;
}

/// Test getting a specific game
#[tokio::test]
async fn test_get_game() {
    let test_db = TestDb::new().await;
    let app_state = test_db.app_state();
    let router = create_test_router(app_state);
    let server = TestServer::new(router);

    let user = create_authenticated_user(&server, "game_getter").await;

    // Create a game
    let create_response = server
        .post("/api/games")
        .add_header("Authorization", user.auth_header())
        .json(&json!({
            "max_tributes": 24,
            "tribute_pool": 24,
            "tribute_list": [],
        }))
        .await;

    let create_body = create_response.json::<serde_json::Value>();
    let game_id = create_body["identifier"].as_str().unwrap();

    // Get the game
    let get_response = server
        .get(&format!("/api/games/{}", game_id))
        .add_header("Authorization", user.auth_header())
        .await;

    get_response.assert_status_ok();

    let get_body = get_response.json::<serde_json::Value>();
    assert_eq!(get_body["identifier"].as_str().unwrap(), game_id);

    test_db.cleanup().await;
}

/// Test updating a game
#[tokio::test]
async fn test_update_game() {
    let test_db = TestDb::new().await;
    let app_state = test_db.app_state();
    let router = create_test_router(app_state);
    let server = TestServer::new(router);

    let user = create_authenticated_user(&server, "game_updater").await;

    // Create a game
    let create_response = server
        .post("/api/games")
        .add_header("Authorization", user.auth_header())
        .json(&json!({
            "max_tributes": 24,
            "tribute_pool": 24,
            "tribute_list": [],
        }))
        .await;

    let create_body = create_response.json::<serde_json::Value>();
    let game_id = create_body["identifier"].as_str().unwrap();

    // Update the game
    let update_response = server
        .put(&format!("/api/games/{}", game_id))
        .add_header("Authorization", user.auth_header())
        .json(&json!({
            "identifier": game_id,
            "name": "Renamed Game",
            "private": false,
        }))
        .await;

    update_response.assert_status_ok();

    let update_body = update_response.json::<serde_json::Value>();
    assert_eq!(update_body["name"], "Renamed Game");
    assert_eq!(update_body["private"], false);

    test_db.cleanup().await;
}

/// Test deleting a game
#[tokio::test]
async fn test_delete_game() {
    let test_db = TestDb::new().await;
    let app_state = test_db.app_state();
    let router = create_test_router(app_state);
    let server = TestServer::new(router);

    let user = create_authenticated_user(&server, "game_deleter").await;

    // Create a game
    let create_response = server
        .post("/api/games")
        .add_header("Authorization", user.auth_header())
        .json(&json!({
            "max_tributes": 24,
            "tribute_pool": 24,
            "tribute_list": [],
        }))
        .await;

    let create_body = create_response.json::<serde_json::Value>();
    let game_id = create_body["identifier"].as_str().unwrap();

    // Delete the game
    let delete_response = server
        .delete(&format!("/api/games/{}", game_id))
        .add_header("Authorization", user.auth_header())
        .await;

    delete_response.assert_status(axum::http::StatusCode::NO_CONTENT);

    // Try to get the deleted game - should return 404
    let get_response = server
        .get(&format!("/api/games/{}", game_id))
        .add_header("Authorization", user.auth_header())
        .await;

    get_response.assert_status(axum::http::StatusCode::NOT_FOUND);

    test_db.cleanup().await;
}

/// Test game display endpoint
#[tokio::test]
async fn test_game_display() {
    let test_db = TestDb::new().await;
    let app_state = test_db.app_state();
    let router = create_test_router(app_state);
    let server = TestServer::new(router);

    let user = create_authenticated_user(&server, "game_displayer").await;

    // Create a game
    let create_response = server
        .post("/api/games")
        .add_header("Authorization", user.auth_header())
        .json(&json!({
            "max_tributes": 24,
            "tribute_pool": 24,
            "tribute_list": [],
        }))
        .await;

    let create_body = create_response.json::<serde_json::Value>();
    let game_id = create_body["identifier"].as_str().unwrap();

    // Get the game display
    let display_response = server
        .get(&format!("/api/games/{}/display", game_id))
        .add_header("Authorization", user.auth_header())
        .await;

    display_response.assert_status_ok();

    let display_body = display_response.json::<serde_json::Value>();
    assert!(display_body.get("identifier").is_some());
    assert!(display_body.get("status").is_some());

    test_db.cleanup().await;
}

/// Test game areas endpoint
#[tokio::test]
async fn test_game_areas() {
    let test_db = TestDb::new().await;
    let app_state = test_db.app_state();
    let router = create_test_router(app_state);
    let server = TestServer::new(router);

    let user = create_authenticated_user(&server, "area_viewer").await;

    // Create a game
    let create_response = server
        .post("/api/games")
        .add_header("Authorization", user.auth_header())
        .json(&json!({
            "max_tributes": 24,
            "tribute_pool": 24,
            "tribute_list": [],
        }))
        .await;

    let create_body = create_response.json::<serde_json::Value>();
    let game_id = create_body["identifier"].as_str().unwrap();

    // Get game areas
    let areas_response = server
        .get(&format!("/api/games/{}/areas", game_id))
        .add_header("Authorization", user.auth_header())
        .await;

    areas_response.assert_status_ok();

    let areas_body = areas_response.json::<serde_json::Value>();
    let areas = areas_body.as_array().unwrap();

    // Should have 5 areas (Cornucopia + 4 cardinal directions)
    assert_eq!(areas.len(), 5);

    test_db.cleanup().await;
}

/// Test publishing a game
#[tokio::test]
async fn test_publish_game() {
    let test_db = TestDb::new().await;
    let app_state = test_db.app_state();
    let router = create_test_router(app_state);
    let server = TestServer::new(router);

    let user = create_authenticated_user(&server, "game_publisher").await;

    // Create a game
    let create_response = server
        .post("/api/games")
        .add_header("Authorization", user.auth_header())
        .json(&json!({
            "max_tributes": 24,
            "tribute_pool": 24,
            "tribute_list": [],
        }))
        .await;

    let create_body = create_response.json::<serde_json::Value>();
    let game_id = create_body["identifier"].as_str().unwrap();

    // Publish the game
    let publish_response = server
        .put(&format!("/api/games/{}/publish", game_id))
        .add_header("Authorization", user.auth_header())
        .await;

    publish_response.assert_status_ok();

    let publish_body = publish_response.json::<serde_json::Value>();
    assert_eq!(publish_body["published"], true);

    test_db.cleanup().await;
}

/// Test unpublishing a game
#[tokio::test]
async fn test_unpublish_game() {
    let test_db = TestDb::new().await;
    let app_state = test_db.app_state();
    let router = create_test_router(app_state);
    let server = TestServer::new(router);

    let user = create_authenticated_user(&server, "game_unpublisher").await;

    // Create and publish a game
    let create_response = server
        .post("/api/games")
        .add_header("Authorization", user.auth_header())
        .json(&json!({
            "max_tributes": 24,
            "tribute_pool": 24,
            "tribute_list": [],
        }))
        .await;

    let create_body = create_response.json::<serde_json::Value>();
    let game_id = create_body["identifier"].as_str().unwrap();

    server
        .put(&format!("/api/games/{}/publish", game_id))
        .add_header("Authorization", user.auth_header())
        .await
        .assert_status_ok();

    // Unpublish the game
    let unpublish_response = server
        .put(&format!("/api/games/{}/unpublish", game_id))
        .add_header("Authorization", user.auth_header())
        .await;

    unpublish_response.assert_status_ok();

    let unpublish_body = unpublish_response.json::<serde_json::Value>();
    assert_eq!(unpublish_body["published"], false);

    test_db.cleanup().await;
}

/// Test unauthorized access to games
#[tokio::test]
async fn test_unauthorized_game_access() {
    let test_db = TestDb::new().await;
    let app_state = test_db.app_state();
    let router = create_test_router(app_state);
    let server = TestServer::new(router);

    // Try to create a game without authentication
    let response = server
        .post("/api/games")
        .json(&json!({
            "max_tributes": 24,
            "tribute_pool": 24,
            "tribute_list": [],
        }))
        .await;

    response.assert_status(axum::http::StatusCode::UNAUTHORIZED);

    test_db.cleanup().await;
}

/// Test that an unstarted game returns at least the current period summary (Day 0/Day).
#[tokio::test]
async fn timeline_summary_includes_current_period_even_when_empty() {
    let test_db = TestDb::new().await;
    let app_state = test_db.app_state();
    let router = create_test_router(app_state);
    let server = TestServer::new(router);

    let user = create_authenticated_user(&server, "timeline_user_1").await;

    // Create a game (status = NotStarted, day = 0)
    let create_response = server
        .post("/api/games")
        .add_header("Authorization", user.auth_header())
        .json(&json!({
            "max_tributes": 24,
            "tribute_pool": 24,
            "tribute_list": [],
        }))
        .await;
    create_response.assert_status_ok();
    let game_id = create_response.json::<serde_json::Value>()["identifier"]
        .as_str()
        .unwrap()
        .to_string();

    // Hit the timeline-summary endpoint
    let response = server
        .get(&format!("/api/games/{}/timeline-summary", game_id))
        .add_header("Authorization", user.auth_header())
        .await;

    response.assert_status_ok();
    let body = response.json::<serde_json::Value>();
    let summaries = body.as_array().expect("response should be an array");

    // summarize_periods always seeds at least the current period, even with no messages.
    assert!(
        !summaries.is_empty(),
        "expected at least one PeriodSummary for the current period"
    );

    let current = &summaries[0];
    assert_eq!(current["day"], 0, "current period day should be 0");
    assert_eq!(current["phase"], "day", "current period phase should be Day");
    assert_eq!(
        current["message_count"], 0,
        "current period should have zero messages for an unstarted game"
    );

    test_db.cleanup().await;
}

/// Test the endpoint returns 404 for a game that does not exist.
#[tokio::test]
async fn timeline_summary_returns_404_for_missing_game() {
    let test_db = TestDb::new().await;
    let app_state = test_db.app_state();
    let router = create_test_router(app_state);
    let server = TestServer::new(router);

    let user = create_authenticated_user(&server, "timeline_user_2").await;

    let bogus_id = uuid::Uuid::new_v4();
    let response = server
        .get(&format!("/api/games/{}/timeline-summary", bogus_id))
        .add_header("Authorization", user.auth_header())
        .await;

    response.assert_status(axum::http::StatusCode::NOT_FOUND);

    test_db.cleanup().await;
}

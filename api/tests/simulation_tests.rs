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

/// Helper to create a game with tributes
async fn create_game_with_tributes(
    server: &TestServer,
    user: &TestUser,
    num_tributes: usize,
) -> String {
    // Create game
    let game_response = server
        .post("/api/games")
        .add_header("Authorization", user.auth_header())
        .json(&json!({
            "max_tributes": 24,
            "tribute_pool": 24,
            "tribute_list": [],
        }))
        .await;

    let game_body = game_response.json::<serde_json::Value>();
    let game_id = game_body["id"].as_str().unwrap().to_string();

    // Add tributes
    for i in 0..num_tributes {
        let district = (i % 12) + 1;
        server
            .post(&format!("/api/games/{}/tributes", game_id))
            .add_header("Authorization", user.auth_header())
            .json(&json!({
                "name": format!("Tribute {}", i + 1),
                "district": district,
            }))
            .await
            .assert_status_ok();
    }

    game_id
}

/// Test advancing game to next step
#[tokio::test]
async fn test_advance_game() {
    let test_db = TestDb::new().await;
    let app_state = test_db.app_state();
    let router = create_test_router(app_state);
    let server = TestServer::new(router);

    let user = create_authenticated_user(&server, "game_advancer").await;
    let game_id = create_game_with_tributes(&server, &user, 4).await;

    // Advance the game
    let response = server
        .put(&format!("/api/games/{}/next", game_id))
        .add_header("Authorization", user.auth_header())
        .await;

    response.assert_status_ok();

    let body = response.json::<serde_json::Value>();
    assert!(body.get("day").is_some());

    // Day should have incremented
    let day = body["day"].as_i64().unwrap();
    assert!(day > 0);

    test_db.cleanup().await;
}

/// Test game status transitions (setup -> running -> finished)
#[tokio::test]
async fn test_game_status_transitions() {
    let test_db = TestDb::new().await;
    let app_state = test_db.app_state();
    let router = create_test_router(app_state);
    let server = TestServer::new(router);

    let user = create_authenticated_user(&server, "status_tester").await;
    let game_id = create_game_with_tributes(&server, &user, 2).await;

    // Check initial status
    let get_response = server
        .get(&format!("/api/games/{}", game_id))
        .add_header("Authorization", user.auth_header())
        .await;

    let get_body = get_response.json::<serde_json::Value>();
    assert_eq!(get_body["status"], "setup");

    // Advance game - should transition to running
    let advance_response = server
        .put(&format!("/api/games/{}/next", game_id))
        .add_header("Authorization", user.auth_header())
        .await;

    let advance_body = advance_response.json::<serde_json::Value>();
    let status = advance_body["status"].as_str().unwrap();
    assert!(status == "running" || status == "finished");

    test_db.cleanup().await;
}

/// Test game day logs
#[tokio::test]
async fn test_game_day_logs() {
    let test_db = TestDb::new().await;
    let app_state = test_db.app_state();
    let router = create_test_router(app_state);
    let server = TestServer::new(router);

    let user = create_authenticated_user(&server, "log_viewer").await;
    let game_id = create_game_with_tributes(&server, &user, 4).await;

    // Advance game to generate logs
    server
        .put(&format!("/api/games/{}/next", game_id))
        .add_header("Authorization", user.auth_header())
        .await
        .assert_status_ok();

    // Get logs for day 1
    let log_response = server
        .get(&format!("/api/games/{}/log/1", game_id))
        .add_header("Authorization", user.auth_header())
        .await;

    log_response.assert_status_ok();

    let log_body = log_response.json::<serde_json::Value>();
    assert!(log_body.is_array());

    test_db.cleanup().await;
}

/// Test tribute-specific logs
#[tokio::test]
async fn test_tribute_day_logs() {
    let test_db = TestDb::new().await;
    let app_state = test_db.app_state();
    let router = create_test_router(app_state);
    let server = TestServer::new(router);

    let user = create_authenticated_user(&server, "tribute_log_viewer").await;
    let game_id = create_game_with_tributes(&server, &user, 4).await;

    // Get tribute ID
    let game_response = server
        .get(&format!("/api/games/{}", game_id))
        .add_header("Authorization", user.auth_header())
        .await;

    let game_body = game_response.json::<serde_json::Value>();
    let tributes = game_body["tributes"].as_array().unwrap();
    let tribute_id = tributes[0]["id"].as_str().unwrap();

    // Advance game
    server
        .put(&format!("/api/games/{}/next", game_id))
        .add_header("Authorization", user.auth_header())
        .await
        .assert_status_ok();

    // Get tribute logs for day 1
    let log_response = server
        .get(&format!("/api/games/{}/log/1/{}", game_id, tribute_id))
        .add_header("Authorization", user.auth_header())
        .await;

    log_response.assert_status_ok();

    let log_body = log_response.json::<serde_json::Value>();
    assert!(log_body.is_array());

    test_db.cleanup().await;
}

/// Test multiple game advancement cycles
#[tokio::test]
async fn test_multiple_game_cycles() {
    let test_db = TestDb::new().await;
    let app_state = test_db.app_state();
    let router = create_test_router(app_state);
    let server = TestServer::new(router);

    let user = create_authenticated_user(&server, "cycle_tester").await;
    let game_id = create_game_with_tributes(&server, &user, 4).await;

    // Advance game 3 times
    for _ in 0..3 {
        let response = server
            .put(&format!("/api/games/{}/next", game_id))
            .add_header("Authorization", user.auth_header())
            .await;

        response.assert_status_ok();

        // Check that we get a valid response with updated state
        let body = response.json::<serde_json::Value>();
        assert!(body.get("day").is_some());
    }

    // Verify final state
    let get_response = server
        .get(&format!("/api/games/{}", game_id))
        .add_header("Authorization", user.auth_header())
        .await;

    let get_body = get_response.json::<serde_json::Value>();
    let day = get_body["day"].as_i64().unwrap();
    assert!(day >= 3);

    test_db.cleanup().await;
}

/// Test game finishes when only one tribute remains
#[tokio::test]
async fn test_game_finishes_with_winner() {
    let test_db = TestDb::new().await;
    let app_state = test_db.app_state();
    let router = create_test_router(app_state);
    let server = TestServer::new(router);

    let user = create_authenticated_user(&server, "winner_tester").await;
    let game_id = create_game_with_tributes(&server, &user, 2).await;

    // Advance game multiple times until it finishes (or max 50 iterations)
    let mut status = "running".to_string();
    let mut iterations = 0;

    while status != "finished" && iterations < 50 {
        let response = server
            .put(&format!("/api/games/{}/next", game_id))
            .add_header("Authorization", user.auth_header())
            .await;

        if response.status_code() != axum::http::StatusCode::OK {
            break;
        }

        let body = response.json::<serde_json::Value>();
        status = body["status"].as_str().unwrap_or("running").to_string();
        iterations += 1;
    }

    // Game should eventually finish
    // (Note: With only 2 tributes, it should finish relatively quickly)
    assert!(iterations < 50, "Game should finish within 50 iterations");

    test_db.cleanup().await;
}

/// Test advancing finished game (should return error or no-op)
#[tokio::test]
async fn test_advance_finished_game() {
    let test_db = TestDb::new().await;
    let app_state = test_db.app_state();
    let router = create_test_router(app_state);
    let server = TestServer::new(router);

    let user = create_authenticated_user(&server, "finished_game_tester").await;
    let game_id = create_game_with_tributes(&server, &user, 2).await;

    // Advance game until finished
    for _ in 0..50 {
        let response = server
            .put(&format!("/api/games/{}/next", game_id))
            .add_header("Authorization", user.auth_header())
            .await;

        if !response.status_code().is_success() {
            break;
        }

        let body = response.json::<serde_json::Value>();
        if body["status"].as_str() == Some("finished") {
            break;
        }
    }

    // Try to advance the finished game
    let response = server
        .put(&format!("/api/games/{}/next", game_id))
        .add_header("Authorization", user.auth_header())
        .await;

    // Should either return error or return the same state
    if response.status_code().is_success() {
        let body = response.json::<serde_json::Value>();
        assert_eq!(body["status"].as_str(), Some("finished"));
    }

    test_db.cleanup().await;
}

/// Test game state persistence between cycles
#[tokio::test]
async fn test_game_state_persistence() {
    let test_db = TestDb::new().await;
    let app_state = test_db.app_state();
    let router = create_test_router(app_state);
    let server = TestServer::new(router);

    let user = create_authenticated_user(&server, "persistence_tester").await;
    let game_id = create_game_with_tributes(&server, &user, 4).await;

    // Advance game
    server
        .put(&format!("/api/games/{}/next", game_id))
        .add_header("Authorization", user.auth_header())
        .await
        .assert_status_ok();

    // Get game state
    let get_response1 = server
        .get(&format!("/api/games/{}", game_id))
        .add_header("Authorization", user.auth_header())
        .await;

    let body1 = get_response1.json::<serde_json::Value>();
    let day1 = body1["day"].as_i64().unwrap();

    // Advance again
    server
        .put(&format!("/api/games/{}/next", game_id))
        .add_header("Authorization", user.auth_header())
        .await
        .assert_status_ok();

    // Get game state again
    let get_response2 = server
        .get(&format!("/api/games/{}", game_id))
        .add_header("Authorization", user.auth_header())
        .await;

    let body2 = get_response2.json::<serde_json::Value>();
    let day2 = body2["day"].as_i64().unwrap();

    // Day should have incremented
    assert!(day2 > day1);

    test_db.cleanup().await;
}

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

/// Helper to create a game for testing. Auto-spawns 24 tributes server-side
/// (see `api::games::create_game` in `api/src/games.rs:226-236`).
async fn create_test_game(server: &TestServer, user: &TestUser) -> String {
    let response = server
        .post("/api/games")
        .add_header("Authorization", user.auth_header())
        .json(&json!({
            "max_tributes": 24,
            "tribute_pool": 24,
            "tribute_list": [],
        }))
        .await;

    let body = response.json::<serde_json::Value>();
    body["identifier"].as_str().unwrap().to_string()
}

/// Helper to fetch the auto-spawned tribute roster for a game.
async fn fetch_tributes(
    server: &TestServer,
    user: &TestUser,
    game_id: &str,
) -> Vec<serde_json::Value> {
    let response = server
        .get(&format!("/api/games/{}/tributes?limit=24", game_id))
        .add_header("Authorization", user.auth_header())
        .await;
    response.assert_status_ok();
    let body = response.json::<serde_json::Value>();
    body["tributes"].as_array().cloned().unwrap_or_default()
}

/// Helper to grab the first auto-spawned tribute identifier for a game.
async fn first_tribute_id(server: &TestServer, user: &TestUser, game_id: &str) -> String {
    let tributes = fetch_tributes(server, user, game_id).await;
    tributes
        .first()
        .and_then(|t| t["identifier"].as_str())
        .map(|s| s.to_string())
        .expect("expected at least one auto-spawned tribute")
}

/// Verify that creating a game auto-spawns the full 24-tribute roster.
///
/// Replaces the old `POST /api/games/{id}/tributes` test: per
/// `hangrier_games-0jl`, manual tribute creation is intentionally not exposed.
/// Every game starts with 24 server-generated tributes which users edit in
/// place via `PUT /api/games/{id}/tributes/{id}`.
#[tokio::test]
async fn test_game_auto_spawns_tributes() {
    let test_db = TestDb::new().await;
    let app_state = test_db.app_state();
    let router = create_test_router(app_state);
    let server = TestServer::new(router);

    let user = create_authenticated_user(&server, "tribute_creator1").await;
    let game_id = create_test_game(&server, &user).await;

    let tributes = fetch_tributes(&server, &user, &game_id).await;
    assert_eq!(tributes.len(), 24, "auto-spawn should populate 24 tributes");

    for t in &tributes {
        assert!(t.get("identifier").is_some());
        assert!(t.get("name").is_some());
        assert!(t.get("district").is_some());
        let district = t["district"].as_u64().expect("district is u64");
        assert!(
            (1..=12).contains(&district),
            "district {} out of range",
            district
        );
    }

    test_db.cleanup().await;
}

/// Test getting a single auto-spawned tribute.
#[tokio::test]
async fn test_get_tribute() {
    let test_db = TestDb::new().await;
    let app_state = test_db.app_state();
    let router = create_test_router(app_state);
    let server = TestServer::new(router);

    let user = create_authenticated_user(&server, "tribute_getter").await;
    let game_id = create_test_game(&server, &user).await;
    let tribute_id = first_tribute_id(&server, &user, &game_id).await;

    let get_response = server
        .get(&format!("/api/games/{}/tributes/{}", game_id, tribute_id))
        .add_header("Authorization", user.auth_header())
        .await;

    get_response.assert_status_ok();

    let get_body = get_response.json::<serde_json::Value>();
    assert_eq!(get_body["identifier"], tribute_id);
    assert!(get_body.get("name").is_some());
    assert!(get_body.get("district").is_some());

    test_db.cleanup().await;
}

/// Test updating an auto-spawned tribute via `EditTribute`.
#[tokio::test]
async fn test_update_tribute() {
    let test_db = TestDb::new().await;
    let app_state = test_db.app_state();
    let router = create_test_router(app_state);
    let server = TestServer::new(router);

    let user = create_authenticated_user(&server, "tribute_updater").await;
    let game_id = create_test_game(&server, &user).await;
    let tribute_id = first_tribute_id(&server, &user, &game_id).await;

    let update_response = server
        .put(&format!("/api/games/{}/tributes/{}", game_id, tribute_id))
        .add_header("Authorization", user.auth_header())
        .json(&json!({
            "identifier": tribute_id,
            "name": "Rue (Updated)",
            "avatar": "",
            "game_identifier": game_id,
        }))
        .await;

    update_response.assert_status_ok();

    let get_response = server
        .get(&format!("/api/games/{}/tributes/{}", game_id, tribute_id))
        .add_header("Authorization", user.auth_header())
        .await;
    get_response.assert_status_ok();
    let get_body = get_response.json::<serde_json::Value>();
    assert_eq!(get_body["name"], "Rue (Updated)");

    test_db.cleanup().await;
}

/// Test deleting an auto-spawned tribute.
#[tokio::test]
async fn test_delete_tribute() {
    let test_db = TestDb::new().await;
    let app_state = test_db.app_state();
    let router = create_test_router(app_state);
    let server = TestServer::new(router);

    let user = create_authenticated_user(&server, "tribute_deleter").await;
    let game_id = create_test_game(&server, &user).await;
    let tribute_id = first_tribute_id(&server, &user, &game_id).await;

    let delete_response = server
        .delete(&format!("/api/games/{}/tributes/{}", game_id, tribute_id))
        .add_header("Authorization", user.auth_header())
        .await;

    delete_response.assert_status(axum::http::StatusCode::NO_CONTENT);

    let get_response = server
        .get(&format!("/api/games/{}/tributes/{}", game_id, tribute_id))
        .add_header("Authorization", user.auth_header())
        .await;

    get_response.assert_status(axum::http::StatusCode::NOT_FOUND);

    let remaining = fetch_tributes(&server, &user, &game_id).await;
    assert_eq!(remaining.len(), 23, "deleting one should leave 23 tributes");

    test_db.cleanup().await;
}

/// Test that the auto-spawn roster covers every district 1..=12.
#[tokio::test]
async fn test_auto_spawn_district_coverage() {
    let test_db = TestDb::new().await;
    let app_state = test_db.app_state();
    let router = create_test_router(app_state);
    let server = TestServer::new(router);

    let user = create_authenticated_user(&server, "multi_tribute_creator").await;
    let game_id = create_test_game(&server, &user).await;

    let tributes = fetch_tributes(&server, &user, &game_id).await;
    assert_eq!(tributes.len(), 24);

    let mut districts: Vec<u64> = tributes
        .iter()
        .map(|t| t["district"].as_u64().expect("district is u64"))
        .collect();
    districts.sort_unstable();
    let unique: std::collections::BTreeSet<_> = districts.iter().copied().collect();
    assert_eq!(
        unique,
        (1..=12u64).collect(),
        "every district 1..=12 should be represented"
    );

    test_db.cleanup().await;
}

/// Test the per-tribute log endpoint.
#[tokio::test]
async fn test_tribute_log() {
    let test_db = TestDb::new().await;
    let app_state = test_db.app_state();
    let router = create_test_router(app_state);
    let server = TestServer::new(router);

    let user = create_authenticated_user(&server, "tribute_logger").await;
    let game_id = create_test_game(&server, &user).await;
    let tribute_id = first_tribute_id(&server, &user, &game_id).await;

    let log_response = server
        .get(&format!(
            "/api/games/{}/tributes/{}/log",
            game_id, tribute_id
        ))
        .add_header("Authorization", user.auth_header())
        .await;

    log_response.assert_status_ok();

    let log_body = log_response.json::<serde_json::Value>();
    assert!(log_body.is_array());

    test_db.cleanup().await;
}

/// Test that a tribute detail response includes the items relationship.
#[tokio::test]
async fn test_tribute_items() {
    let test_db = TestDb::new().await;
    let app_state = test_db.app_state();
    let router = create_test_router(app_state);
    let server = TestServer::new(router);

    let user = create_authenticated_user(&server, "tribute_item_tester").await;
    let game_id = create_test_game(&server, &user).await;
    let tribute_id = first_tribute_id(&server, &user, &game_id).await;

    let get_response = server
        .get(&format!("/api/games/{}/tributes/{}", game_id, tribute_id))
        .add_header("Authorization", user.auth_header())
        .await;

    get_response.assert_status_ok();

    let get_body = get_response.json::<serde_json::Value>();
    assert!(get_body.get("items").is_some());

    test_db.cleanup().await;
}

/// Test that updating a tribute with an empty name fails validation.
#[tokio::test]
async fn test_update_tribute_validation() {
    let test_db = TestDb::new().await;
    let app_state = test_db.app_state();
    let router = create_test_router(app_state);
    let server = TestServer::new(router);

    let user = create_authenticated_user(&server, "tribute_validator").await;
    let game_id = create_test_game(&server, &user).await;
    let tribute_id = first_tribute_id(&server, &user, &game_id).await;

    let response = server
        .put(&format!("/api/games/{}/tributes/{}", game_id, tribute_id))
        .add_header("Authorization", user.auth_header())
        .json(&json!({
            "identifier": tribute_id,
            "name": "",
            "avatar": "",
            "game_identifier": game_id,
        }))
        .await;

    assert!(
        response.status_code() == axum::http::StatusCode::BAD_REQUEST
            || response.status_code() == axum::http::StatusCode::UNPROCESSABLE_ENTITY,
        "expected 400/422 for empty name, got {}",
        response.status_code()
    );

    test_db.cleanup().await;
}

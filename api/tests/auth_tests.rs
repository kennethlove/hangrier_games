mod common;

use axum_test::TestServer;
use common::{TestDb, TestUser, create_test_router};
use serde_json::json;

/// Test user registration (signup)
#[tokio::test]
async fn test_user_registration() {
    let test_db = TestDb::new().await;
    let app_state = test_db.app_state();
    let router = create_test_router(app_state);
    let server = TestServer::new(router);

    let test_user = TestUser::new("testuser1");

    let response = server
        .post("/api/users")
        .json(&json!({
            "display_name": test_user.username,
            "email": test_user.email,
            "password": test_user.password,
        }))
        .await;

    response.assert_status(axum::http::StatusCode::CREATED);

    let body = response.json::<serde_json::Value>();
    assert_eq!(body.get("status").and_then(|v| v.as_str()), Some("created"));
    assert!(body.get("message").is_some());

    test_db.cleanup().await;
}

/// Test user authentication (signin)
#[tokio::test]
async fn test_user_authentication() {
    let test_db = TestDb::new().await;
    let app_state = test_db.app_state();
    let router = create_test_router(app_state);
    let server = TestServer::new(router);

    let test_user = TestUser::new("testuser2");

    // First, register the user
    let reg_response = server
        .post("/api/users")
        .json(&json!({
            "display_name": test_user.username,
            "email": test_user.email,
            "password": test_user.password,
        }))
        .await;
    reg_response.assert_status(axum::http::StatusCode::CREATED);

    // Manually verify email in test database
    test_db.verify_email(&test_user.email).await;

    // Then, authenticate
    let auth_response = server
        .post("/api/users/authenticate")
        .json(&json!({
            "email": test_user.email,
            "password": test_user.password,
        }))
        .await;

    auth_response.assert_status_ok();

    let body = auth_response.json::<serde_json::Value>();
    assert!(body.get("access_token").is_some());
    assert!(body.get("refresh_token").is_some());

    test_db.cleanup().await;
}

/// Test authentication with wrong password
#[tokio::test]
async fn test_authentication_wrong_password() {
    let test_db = TestDb::new().await;
    let app_state = test_db.app_state();
    let router = create_test_router(app_state);
    let server = TestServer::new(router);

    let test_user = TestUser::new("testuser3");

    // Register the user
    server
        .post("/api/users")
        .json(&json!({
            "display_name": test_user.username,
            "email": test_user.email,
            "password": test_user.password,
        }))
        .await
        .assert_status(axum::http::StatusCode::CREATED);

    test_db.verify_email(&test_user.email).await;

    // Try to authenticate with wrong password
    let auth_response = server
        .post("/api/users/authenticate")
        .json(&json!({
            "email": test_user.email,
            "password": "WrongPassword123!",
        }))
        .await;

    auth_response.assert_status(axum::http::StatusCode::UNAUTHORIZED);

    test_db.cleanup().await;
}

/// Test token refresh
#[tokio::test]
async fn test_token_refresh() {
    let test_db = TestDb::new().await;
    let app_state = test_db.app_state();
    let router = create_test_router(app_state);
    let server = TestServer::new(router);

    let test_user = TestUser::new("testuser4");

    // Register user
    let reg_response = server
        .post("/api/users")
        .json(&json!({
            "display_name": test_user.username,
            "email": test_user.email,
            "password": test_user.password,
        }))
        .await;
    reg_response.assert_status(axum::http::StatusCode::CREATED);

    test_db.verify_email(&test_user.email).await;

    // Authenticate to get tokens
    let auth_response = server
        .post("/api/users/authenticate")
        .json(&json!({
            "email": test_user.email,
            "password": test_user.password,
        }))
        .await;
    auth_response.assert_status_ok();

    let auth_body = auth_response.json::<serde_json::Value>();
    let refresh_token = auth_body["refresh_token"].as_str().unwrap();

    // Use refresh token to get new tokens
    let refresh_response = server
        .post("/api/auth/refresh")
        .json(&json!({
            "refresh_token": refresh_token,
        }))
        .await;

    refresh_response.assert_status_ok();

    let refresh_body = refresh_response.json::<serde_json::Value>();
    assert!(refresh_body.get("access_token").is_some());
    assert!(refresh_body.get("refresh_token").is_some());

    // New refresh token should be different (token rotation)
    let new_refresh_token = refresh_body["refresh_token"].as_str().unwrap();
    assert_ne!(refresh_token, new_refresh_token);

    test_db.cleanup().await;
}

/// Test logout
#[tokio::test]
async fn test_logout() {
    let test_db = TestDb::new().await;
    let app_state = test_db.app_state();
    let router = create_test_router(app_state);
    let server = TestServer::new(router);

    let test_user = TestUser::new("testuser5");

    // Register user
    let reg_response = server
        .post("/api/users")
        .json(&json!({
            "display_name": test_user.username,
            "email": test_user.email,
            "password": test_user.password,
        }))
        .await;
    reg_response.assert_status(axum::http::StatusCode::CREATED);

    test_db.verify_email(&test_user.email).await;

    // Authenticate to get tokens
    let auth_response = server
        .post("/api/users/authenticate")
        .json(&json!({
            "email": test_user.email,
            "password": test_user.password,
        }))
        .await;
    auth_response.assert_status_ok();

    let auth_body = auth_response.json::<serde_json::Value>();
    let refresh_token = auth_body["refresh_token"].as_str().unwrap();

    // Logout (revoke refresh token)
    let logout_response = server
        .post("/api/auth/logout")
        .json(&json!({
            "refresh_token": refresh_token,
        }))
        .await;

    logout_response.assert_status(axum::http::StatusCode::NO_CONTENT);

    // Try to use the revoked refresh token - should fail
    let refresh_response = server
        .post("/api/auth/refresh")
        .json(&json!({
            "refresh_token": refresh_token,
        }))
        .await;

    refresh_response.assert_status(axum::http::StatusCode::UNAUTHORIZED);

    test_db.cleanup().await;
}

/// Test duplicate username registration
#[tokio::test]
async fn test_duplicate_username() {
    let test_db = TestDb::new().await;
    let app_state = test_db.app_state();
    let router = create_test_router(app_state);
    let server = TestServer::new(router);

    let test_user = TestUser::new("duplicate_user");

    // Register user first time - should succeed
    server
        .post("/api/users")
        .json(&json!({
            "display_name": test_user.username,
            "email": test_user.email,
            "password": test_user.password,
        }))
        .await
        .assert_status(axum::http::StatusCode::CREATED);

    // Try to register same display name with different email — should succeed
    // (display names are no longer unique, only email is unique)
    let response = server
        .post("/api/users")
        .json(&json!({
            "display_name": test_user.username,
            "email": "different@test.com",
            "password": test_user.password,
        }))
        .await;

    assert_eq!(response.status_code(), axum::http::StatusCode::CREATED);

    test_db.cleanup().await;
}

/// Test session endpoint (requires authentication)
#[tokio::test]
async fn test_session_endpoint() {
    let test_db = TestDb::new().await;
    let app_state = test_db.app_state();
    let router = create_test_router(app_state);
    let server = TestServer::new(router);

    let test_user = TestUser::new("testuser6");

    // Register user
    let reg_response = server
        .post("/api/users")
        .json(&json!({
            "display_name": test_user.username,
            "email": test_user.email,
            "password": test_user.password,
        }))
        .await;
    reg_response.assert_status(axum::http::StatusCode::CREATED);

    test_db.verify_email(&test_user.email).await;

    // Authenticate to get tokens
    let auth_response = server
        .post("/api/users/authenticate")
        .json(&json!({
            "email": test_user.email,
            "password": test_user.password,
        }))
        .await;
    auth_response.assert_status_ok();

    let auth_body = auth_response.json::<serde_json::Value>();
    let access_token = auth_body["access_token"].as_str().unwrap();

    // Call session endpoint with token
    let session_response = server
        .get("/api/users/session")
        .add_header("Authorization", format!("Bearer {}", access_token))
        .await;

    session_response.assert_status_ok();

    test_db.cleanup().await;
}

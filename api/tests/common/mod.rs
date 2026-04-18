use api::AppState;
use axum::Router;
use std::sync::Arc;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::opt::auth::Root;
use surrealdb_migrations::MigrationRunner;

/// Test configuration for SurrealDB
pub struct TestDb {
    pub db: Arc<Surreal<Any>>,
}

impl TestDb {
    /// Create a new test database connection
    pub async fn new() -> Self {
        // Connect to test SurrealDB instance
        let db = Arc::new(
            surrealdb::engine::any::connect("ws://localhost:8000")
                .await
                .expect("Failed to connect to test database"),
        );

        // Sign in as root
        db.signin(Root {
            username: "root",
            password: "root",
        })
        .await
        .expect("Failed to authenticate to test database");

        // Use test namespace and database
        let test_db_name = format!(
            "test_{}",
            uuid::Uuid::new_v4().to_string().replace('-', "_")
        );
        db.use_ns("hangry-games-test")
            .use_db(&test_db_name)
            .await
            .expect("Failed to use test database");

        // Run migrations
        MigrationRunner::new(&db)
            .up()
            .await
            .expect("Failed to apply migrations");

        TestDb { db }
    }

    /// Get the AppState for testing
    pub fn app_state(&self) -> AppState {
        use api::storage::LocalStorage;
        use api::websocket::GameBroadcaster;

        AppState {
            db: self.db.clone(),
            storage: Arc::new(LocalStorage::new("test_uploads", "/uploads")),
            broadcaster: Arc::new(GameBroadcaster::default()),
        }
    }

    /// Clean up the test database
    pub async fn cleanup(&self) {
        // Note: In a real test setup, you might want to drop the entire database
        // For now, we'll just clear the tables
        let _: Result<Vec<serde_json::Value>, _> = self.db.query("REMOVE DATABASE $this").await;
    }
}

/// Create a test router with the API routes
pub fn create_test_router(state: AppState) -> Router {
    use api::auth::AUTH_ROUTER;
    use api::games::GAMES_ROUTER;
    use api::users::USERS_ROUTER;
    use api::websocket::websocket_handler;
    use axum::middleware;
    use axum::routing::get;

    let api_routes = Router::new()
        .nest(
            "/games",
            GAMES_ROUTER
                .clone()
                .layer(middleware::from_fn_with_state(state.clone(), surreal_jwt)),
        )
        .nest("/users", USERS_ROUTER.clone())
        .nest("/auth", AUTH_ROUTER.clone());

    Router::new()
        .nest("/api", api_routes)
        .route("/ws", get(websocket_handler))
        .route(
            "/health",
            axum::routing::get(|| async { axum::Json(serde_json::json!({"status": "ok"})) }),
        )
        .with_state(state)
}

/// JWT authentication middleware for tests
async fn surreal_jwt(
    axum::extract::State(state): axum::extract::State<AppState>,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    use axum::http::StatusCode;
    use axum::http::header::AUTHORIZATION;
    use axum::response::IntoResponse;
    use base64_url::decode;
    use surrealdb::opt::auth::Jwt;
    use time::OffsetDateTime;

    let token = match request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
    {
        Some(token) => token,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };

    let token_parts: Vec<&str> = token.split('.').collect();
    if token_parts.len() != 3 {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let payload_base64 = token_parts[1].trim_start_matches("=");
    let payload_bytes = decode(payload_base64).map_err(|_| ()).unwrap_or_default();
    let payload_str = String::from_utf8(payload_bytes).unwrap_or_default();
    let payload: serde_json::Value = serde_json::from_str(&payload_str).unwrap_or_default();

    let exp = payload["exp"].as_u64().unwrap_or_default();
    let now = OffsetDateTime::now_utc().unix_timestamp() as u64;
    if exp < now {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let jwt = Jwt::from(token);
    match state.db.authenticate(jwt).await {
        Ok(_) => next.run(request).await,
        Err(_) => StatusCode::UNAUTHORIZED.into_response(),
    }
}

/// Test user credentials
pub struct TestUser {
    pub username: String,
    pub email: String,
    pub password: String,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
}

impl TestUser {
    pub fn new(username: &str) -> Self {
        Self {
            username: username.to_string(),
            email: format!("{}@test.com", username),
            password: "TestPass123!".to_string(),
            access_token: None,
            refresh_token: None,
        }
    }

    pub fn with_tokens(mut self, access_token: String, refresh_token: String) -> Self {
        self.access_token = Some(access_token);
        self.refresh_token = Some(refresh_token);
        self
    }

    pub fn auth_header(&self) -> String {
        format!("Bearer {}", self.access_token.as_ref().unwrap())
    }
}

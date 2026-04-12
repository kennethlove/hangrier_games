extern crate core;

use api::AppState;
use api::auth::AUTH_ROUTER;
use api::games::GAMES_ROUTER;
use api::users::USERS_ROUTER;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::http::header::{
    ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, ACCESS_CONTROL_ALLOW_METHODS,
    ACCESS_CONTROL_ALLOW_ORIGIN, ACCESS_CONTROL_MAX_AGE, AUTHORIZATION, CACHE_CONTROL,
    CONTENT_TYPE, EXPIRES,
};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::{Json, Router, middleware};
use base64_url::decode;
use serde_json::Value;
use std::collections::hash_map::DefaultHasher;
use std::env;
use std::hash::{Hash, Hasher};
use std::string::String;
use std::sync::Arc;
use std::sync::LazyLock;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::opt::auth::{Jwt, Root};
use surrealdb_migrations::MigrationRunner;
use time::OffsetDateTime;
use tower::ServiceBuilder;
use tower_governor::GovernorLayer;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::key_extractor::KeyExtractor;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub static DATABASE: LazyLock<Arc<Surreal<Any>>> = LazyLock::new(|| Arc::new(Surreal::init()));

fn initialize_logging() {
    // a layer that logs events to stdout
    let stdout_log = tracing_subscriber::fmt::layer().pretty();

    // a layer that logs events to a file, using the JSON format
    // let file = File::create("debug_log.json").expect("Failed to create log file");
    // let debug_log = tracing_subscriber::fmt::layer()
    //     .with_writer(Arc::new(file))
    //     .json();

    let _log_targets = tracing_subscriber::filter::Targets::new()
        .with_target("api::game", tracing::Level::INFO)
        .with_target("api::tribute", tracing::Level::INFO);

    let production = env::var("PRODUCTION").unwrap_or("true".to_string());
    let tracing_level = if production == "true" {
        "info"
    } else {
        "debug"
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}={tracing_level},tower_http={tracing_level},surrealdb={tracing_level},surrealdb_client={tracing_level}",
                        env!("CARGO_CRATE_NAME")).into()
            })
        )
        // only log INFO and above to stdout unless the span or event
        // has the `api` target prefix.
        // .with(stdout_log.with_filter(log_targets))
        .with(stdout_log)
        // log everything enabled by the global filter to `debug_log.json`.
        // .with(debug_log)
        // configure a global filter for the whole subscriber stack. This will
        // control what spans and events are recorded by both the `debug_log`
        // and the `stdout_log` layers, and `stdout_log` will *also* be
        // filtered by its per-layer filter.
        // .with(
        //     tracing_subscriber::filter::Targets::default()
        //         .with_target("api", tracing::Level::INFO)
        // ).with(
        //     HangryGamesLogLayer
        // ).init();
        .init();
    // .with(
    //     HangryGamesLogLayer
    //         .with_filter(log_targets)
    // )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    initialize_logging();

    let surreal_host =
        env::var("SURREAL_HOST").map_err(|_| "SURREAL_HOST environment variable not set")?;
    let db = Arc::new(
        surrealdb::engine::any::connect(surreal_host)
            .await
            .map_err(|e| format!("Failed to connect to database: {}", e))?,
    );
    tracing::debug!("connected to SurrealDB");

    let surreal_user =
        env::var("SURREAL_USER").map_err(|_| "SURREAL_USER environment variable not set")?;
    let surreal_pass =
        env::var("SURREAL_PASS").map_err(|_| "SURREAL_PASS environment variable not set")?;

    db.signin(Root {
        username: &surreal_user,
        password: &surreal_pass,
    })
    .await
    .map_err(|e| format!("Failed to authenticate to database: {}", e))?;
    tracing::debug!("authenticated to SurrealDB");

    db.use_ns("hangry-games")
        .use_db("games")
        .await
        .map_err(|e| format!("Failed to use database: {}", e))?;
    tracing::debug!("Using 'hangry-games' namespace and 'games' database");

    MigrationRunner::new(&db)
        .up()
        .await
        .map_err(|e| format!("Failed to apply migrations: {}", e))?;
    tracing::debug!("Applied migrations");

    let allowed_origins = vec!["http://localhost:8080", "http://127.0.0.1:8080"];

    // These are safe to unwrap as they are static HTTP method strings that are guaranteed valid
    let cors_layer = CorsLayer::new()
        .allow_methods(vec![
            "DELETE".parse()?,
            "GET".parse()?,
            "HEAD".parse()?,
            "OPTIONS".parse()?,
            "POST".parse()?,
            "PUT".parse()?,
        ])
        .allow_origin(
            allowed_origins
                .iter()
                .map(|o| o.parse())
                .collect::<Result<Vec<_>, _>>()?,
        )
        .allow_credentials(true)
        .allow_headers([
            ACCEPT,
            ACCEPT_ENCODING,
            ACCEPT_LANGUAGE,
            ACCESS_CONTROL_ALLOW_METHODS,
            ACCESS_CONTROL_ALLOW_ORIGIN,
            ACCESS_CONTROL_MAX_AGE,
            AUTHORIZATION,
            CACHE_CONTROL,
            CONTENT_TYPE,
            EXPIRES,
        ]);

    let governor_config = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(2) // Allow 1 request every 0.5 seconds (2 per second, ~120 per minute)
            .burst_size(50)
            .key_extractor(CompoundKeyExtractor)
            .finish()
            .ok_or("Failed to build GovernorConfig")?,
    );

    let app_state = AppState { db: db.clone() };

    let api_routes = Router::new()
        .nest(
            "/games",
            GAMES_ROUTER.clone().layer(middleware::from_fn_with_state(
                app_state.clone(),
                surreal_jwt,
            )),
        )
        .nest("/users", USERS_ROUTER.clone())
        .nest("/auth", AUTH_ROUTER.clone());

    let router = Router::new()
        .nest("/api", api_routes)
        .route(
            "/",
            axum::routing::get(move || async { Json(env!("CARGO_PKG_VERSION")) }),
        )
        .route(
            "/health",
            axum::routing::get(|State(state): State<AppState>| async move {
                let db_status = match state.db.health().await {
                    Ok(_) => "connected",
                    Err(_) => "disconnected",
                };
                Json(serde_json::json!({
                    "status": "ok",
                    "version": env!("CARGO_PKG_VERSION"),
                    "db": db_status
                }))
            }),
        )
        .with_state(app_state)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(GovernorLayer::new(governor_config))
                .layer(cors_layer)
                .into_inner(),
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .map_err(|e| format!("Failed to bind to 0.0.0.0:3000: {}", e))?;
    let local_addr = listener
        .local_addr()
        .map_err(|e| format!("Failed to get local address: {}", e))?;
    tracing::info!("listening on {}", local_addr);

    axum::serve(listener, router)
        .await
        .map_err(|e| format!("Server error: {}", e))?;

    Ok(())
}

async fn surreal_jwt(State(state): State<AppState>, request: Request, next: Next) -> Response {
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
    let payload: Value = serde_json::from_str(&payload_str).unwrap_or_default();

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

/// Rate limiting key extractor that uses IP + user_id for JWT-protected routes
/// and IP only for public routes.
#[derive(Clone, Copy, Debug)]
struct CompoundKeyExtractor;

impl KeyExtractor for CompoundKeyExtractor {
    type Key = u64;

    fn extract<T>(
        &self,
        request: &axum::http::Request<T>,
    ) -> Result<Self::Key, tower_governor::GovernorError> {
        // Extract IP from remote_addr
        let ip = request
            .extensions()
            .get::<axum::extract::connect_info::ConnectInfo<std::net::SocketAddr>>()
            .map(|ci| ci.0.ip().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        // Try to extract user_id from JWT payload in Authorization header
        let user_id = request
            .headers()
            .get(AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .and_then(|token| {
                let token_parts: Vec<&str> = token.split('.').collect();
                if token_parts.len() == 3 {
                    let payload_base64 = token_parts[1].trim_start_matches('=');
                    decode(payload_base64)
                        .ok()
                        .and_then(|bytes| String::from_utf8(bytes).ok())
                } else {
                    None
                }
            })
            .and_then(|payload_str| serde_json::from_str::<Value>(&payload_str).ok())
            .and_then(|payload| payload["sub"].as_str().map(String::from))
            .unwrap_or_default();

        // Combine IP and user_id into a single key using hash
        let mut hasher = DefaultHasher::new();
        ip.hash(&mut hasher);
        user_id.hash(&mut hasher);
        Ok(hasher.finish())
    }
}

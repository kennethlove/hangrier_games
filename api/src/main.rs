extern crate core;

use api::auth::AUTH_ROUTER;
use api::cleanup::start_cleanup_scheduler;
use api::cookies::{CSRF_COOKIE, SESSION_COOKIE, generate_csrf_token, read_cookie};
use api::email::{
    generate_verification_token, send_verification_email, validate_verification_token,
};
use api::games::GAMES_ROUTER;
use api::templates::AuthState;
use api::templates::auth;
use api::templates::game_detail;
use api::templates::pages;
use api::users::{USERS_PROTECTED_ROUTER, USERS_PUBLIC_ROUTER};
use api::{AppState, AuthDb};
use axum::Form;
use axum::extract::{Query, Request, State};
use axum::http::StatusCode;
use axum::http::header::{
    ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, ACCESS_CONTROL_ALLOW_METHODS,
    ACCESS_CONTROL_ALLOW_ORIGIN, ACCESS_CONTROL_MAX_AGE, AUTHORIZATION, CACHE_CONTROL,
    CONTENT_TYPE, EXPIRES, HeaderName,
};
use axum::middleware::Next;
use axum::response::Html;
use axum::response::{IntoResponse, Redirect, Response};
use axum::{Json, Router, middleware};
use base64_url::decode;
use serde_json::Value;
use shared::{EmailRegistrationUser, ListDisplayGame, UserSession};
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::env;
use std::hash::{Hash, Hasher};
use std::string::String;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::Mutex;
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
use tower_http::normalize_path::NormalizePathLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use validator::Validate;

pub static DATABASE: LazyLock<Arc<Surreal<Any>>> = LazyLock::new(|| Arc::new(Surreal::init()));

/// Cooldown cache for resend verification requests.
/// Key: "resend:<email>", Value: Instant of last send.
static RESEND_COOLDOWN: LazyLock<Mutex<HashMap<String, std::time::Instant>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn initialize_logging() {
    // a layer that logs events to stdout
    let stdout_log = tracing_subscriber::fmt::layer().pretty();

    // a layer that logs events to a file, using the JSON format
    // let file = File::create("debug_log.json").expect("Failed to create log file");
    // let debug_log = tracing_subscriber::fmt::layer()
    //     .with_writer(Arc::new(file))
    //     .json();

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
        .with(stdout_log)
        .init()
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

    let surreal_namespace =
        env::var("APP_SURREAL_NAMESPACE").unwrap_or_else(|_| "hangry-games".to_string());
    let surreal_database = env::var("APP_SURREAL_DATABASE").unwrap_or_else(|_| "games".to_string());

    db.use_ns(&surreal_namespace)
        .use_db(&surreal_database)
        .await
        .map_err(|e| format!("Failed to use database: {}", e))?;
    tracing::debug!(
        "Using '{}' namespace and '{}' database",
        surreal_namespace,
        surreal_database
    );

    // Try migration runner first. If migration state is corrupted, drop
    // the tracking table and re-run from scratch.
    let mut migration_ok = MigrationRunner::new(&db).up().await.is_ok();
    if !migration_ok {
        tracing::warn!("Migration runner failed; resetting state and retrying...");
        // Migration state corrupted; drop the tracking table and retry.
        let _ = db
            .query("REMOVE TABLE IF EXISTS _surrealdb_migrations;")
            .await;
        migration_ok = MigrationRunner::new(&db).up().await.is_ok();
        if migration_ok {
            tracing::info!("Migration runner succeeded after resetting state");
        }
    }

    if !migration_ok {
        // Last resort: apply critical schemas directly so auth still works.
        tracing::warn!("Migration still failing; applying critical schemas directly");
        let schema_paths = ["../schemas/users.surql", "../schemas/refresh_tokens.surql"];
        let _ = db
            .use_ns(&surreal_namespace)
            .use_db(&surreal_database)
            .await;
        for rel_path in &schema_paths {
            let abs_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel_path);
            match tokio::fs::read_to_string(&abs_path).await {
                Ok(sql) => {
                    if let Err(sqlerr) = db.query(sql).await {
                        tracing::warn!("Direct-schema apply failed for {}: {sqlerr}", rel_path);
                    } else {
                        tracing::info!("Applied schema: {rel_path}");
                    }
                }
                Err(read_err) => tracing::warn!("Cannot read schema {rel_path}: {read_err}"),
            }
        }
    }

    // CORS Configuration
    let env_mode = env::var("ENV").unwrap_or_else(|_| "production".to_string());
    let is_production = env_mode == "production";

    let allowed_origins_str = env::var("ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:8080,http://127.0.0.1:8080".to_string());

    let allowed_origins: Vec<String> = allowed_origins_str
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    // Validate CORS configuration in production
    if is_production {
        for origin in &allowed_origins {
            if origin == "*" || origin.contains("*") {
                return Err(format!(
                    "Wildcard CORS origins are not allowed in production. Found: {}. \
                    Please set ALLOWED_ORIGINS to specific domains.",
                    origin
                )
                .into());
            }
        }
        tracing::info!(
            "Production CORS validation passed. Allowed origins: {:?}",
            allowed_origins
        );
    } else {
        tracing::debug!("Development mode. Allowed origins: {:?}", allowed_origins);
    }

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

    // Rate Limiting Configuration
    let rate_limit_per_second = env::var("RATE_LIMIT_PER_SECOND")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(2); // Default: 2 requests per second (~120 per minute)

    let rate_limit_burst = env::var("RATE_LIMIT_BURST")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(50); // Default: burst size of 50

    tracing::info!(
        "Rate limiting configured: {} req/sec, burst size: {}",
        rate_limit_per_second,
        rate_limit_burst
    );

    let governor_config = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(rate_limit_per_second)
            .burst_size(rate_limit_burst)
            .key_extractor(CompoundKeyExtractor)
            .finish()
            .ok_or("Failed to build GovernorConfig")?,
    );

    // Initialize storage backend
    use api::storage::LocalStorage;
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let storage_path = env::var("STORAGE_PATH").unwrap_or_else(|_| "uploads".to_string());
    let storage = Arc::new(LocalStorage::new(&storage_path, "/uploads"));
    storage.init().await?;
    tracing::info!("Storage initialized at: {}", storage_path);

    // Initialize WebSocket broadcaster
    use api::websocket::GameBroadcaster;
    let broadcaster = Arc::new(GameBroadcaster::default());
    tracing::info!("WebSocket broadcaster initialized");

    let app_state = AppState {
        db: db.clone(),
        storage,
        broadcaster,
        namespace: surreal_namespace,
        database: surreal_database,
    };

    // Start cleanup scheduler for refresh tokens
    let _cleanup_scheduler = start_cleanup_scheduler(app_state.clone())
        .await
        .map_err(|e| format!("Failed to start cleanup scheduler: {}", e))?;
    tracing::info!("Cleanup scheduler initialized");

    let api_routes = Router::new()
        .route(
            "/version",
            axum::routing::get(|| async { Json(env!("CARGO_PKG_VERSION")) }),
        )
        .nest(
            "/games",
            GAMES_ROUTER.clone().layer(middleware::from_fn_with_state(
                app_state.clone(),
                surreal_jwt,
            )),
        )
        .nest("/users", USERS_PUBLIC_ROUTER.clone())
        .nest(
            "/users",
            USERS_PROTECTED_ROUTER
                .clone()
                .layer(middleware::from_fn_with_state(
                    app_state.clone(),
                    surreal_jwt,
                )),
        )
        .nest("/auth", AUTH_ROUTER.clone());

    let router = Router::new()
        .nest("/api", api_routes)
        .nest_service(
            "/uploads",
            tower_http::services::ServeDir::new(&storage_path),
        )
        .nest_service(
            "/assets",
            tower_http::services::ServeDir::new(
                std::path::PathBuf::from(&manifest_dir)
                    .join("assets")
                    .join("dist"),
            ),
        )
        .nest_service(
            "/icons",
            tower_http::services::ServeDir::new(
                std::path::PathBuf::from(&manifest_dir)
                    .join("assets")
                    .join("icons"),
            ),
        )
        .route(
            "/ws",
            axum::routing::get(api::websocket::websocket_handler).layer(
                middleware::from_fn_with_state(app_state.clone(), surreal_jwt),
            ),
        )
        // TODO: Move to GAMES_ROUTER?
        .route(
            "/api/games/{game_id}/events",
            axum::routing::get(api::sse::sse_handler).layer(middleware::from_fn_with_state(
                app_state.clone(),
                surreal_jwt,
            )),
        )
        .route("/", axum::routing::get(home_handler))
        .route("/account", axum::routing::get(account_handler))
        .route("/auth", axum::routing::get(auth_handler))
        .route("/auth/login", axum::routing::post(login_post_handler))
        .route("/auth/logout", axum::routing::post(logout_handler))
        .route("/auth/register", axum::routing::post(register_post_handler))
        .route("/auth/check-email", axum::routing::get(check_email_handler))
        .route(
            "/auth/verify-email",
            axum::routing::get(verify_email_handler),
        )
        .route(
            "/auth/resend-verification",
            axum::routing::post(resend_verification_handler),
        )
        .route(
            "/auth/email-verified",
            axum::routing::get(email_verified_handler),
        )
        .route("/games", axum::routing::get(games_list_handler))
        .route("/games/{id}", axum::routing::get(game_detail_handler))
        .route("/games/{id}/areas", axum::routing::get(game_areas_handler))
        .route("/games/{id}/log", axum::routing::get(game_log_handler))
        .route(
            "/games/{id}/tributes",
            axum::routing::get(game_tributes_handler),
        )
        .route(
            "/games/new",
            axum::routing::get(create_game_handler).post(create_game_post_handler),
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
        .route(
            "/dev/verify-email",
            axum::routing::post(dev_verify_email_handler),
        )
        .with_state(app_state)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(middleware::from_fn(add_rate_limit_headers))
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

    // Wrap the router in NormalizePathLayer so e.g. `/api/users/` is rewritten
    // to `/api/users` BEFORE route matching. Axum's per-route layers can't do
    // this — the layer must sit outside the Router.
    use axum::ServiceExt;
    use tower::Layer;
    let app = NormalizePathLayer::trim_trailing_slash().layer(router);

    axum::serve(listener, ServiceExt::<Request>::into_make_service(app))
        .await
        .map_err(|e| format!("Server error: {}", e))?;

    Ok(())
}

async fn surreal_jwt(State(state): State<AppState>, request: Request, next: Next) -> Response {
    // Prefer the HttpOnly `hg_session` cookie (browsers attach it on every
    // same-site request, including the WebSocket upgrade). Fall back to
    // `Authorization: Bearer …` so non-browser clients (tests, scripts) still
    // work.
    let token = api::cookies::read_cookie(request.headers(), api::cookies::SESSION_COOKIE)
        .map(|s| s.to_owned())
        .or_else(|| {
            request
                .headers()
                .get(AUTHORIZATION)
                .and_then(|h| h.to_str().ok())
                .and_then(|h| h.strip_prefix("Bearer "))
                .map(|s| s.to_owned())
        });
    let token = match token {
        Some(t) if !t.is_empty() => t,
        _ => return StatusCode::UNAUTHORIZED.into_response(),
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

    let jwt = Jwt::from(token.as_str());
    // Per-request session: clone the shared connection (independent
    // session state, same underlying socket per SurrealDB Rust SDK 2.x
    // multi-tenancy) and authenticate the clone. The original
    // root-authenticated `state.db` is untouched, so concurrent requests
    // can no longer race on `$auth`. The clone is injected as a request
    // extension so handlers (extractor `AuthDb`) see it. See bd
    // hangrier_games-c3ct (replaces the global `auth_lock` from c853).
    let user_db = (*state.db).clone();
    if user_db
        .use_ns(&state.namespace)
        .use_db(&state.database)
        .await
        .is_err()
    {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    if user_db.authenticate(jwt).await.is_err() {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let mut request = request;
    request.extensions_mut().insert(AuthDb(user_db));
    next.run(request).await
}

/// Middleware to add rate limit headers to responses
async fn add_rate_limit_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;

    // Read rate limit config from environment (same as main config)
    let rate_limit_per_second = env::var("RATE_LIMIT_PER_SECOND")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(2);
    let rate_limit_burst = env::var("RATE_LIMIT_BURST")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(50);

    // Calculate the limit per minute for header
    let limit_per_minute = rate_limit_per_second * 60;

    // Add rate limit headers
    let headers = response.headers_mut();
    headers.insert(
        HeaderName::from_static("x-ratelimit-limit"),
        limit_per_minute
            .to_string()
            .parse()
            .unwrap_or_else(|_| "120".parse().unwrap()),
    );

    // Note: We can't easily get the remaining count from tower-governor without more complex integration
    // For now, we'll document the limit. A future enhancement could track this more precisely.
    headers.insert(
        HeaderName::from_static("x-ratelimit-burst"),
        rate_limit_burst
            .to_string()
            .parse()
            .unwrap_or_else(|_| "50".parse().unwrap()),
    );

    response
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

// ── HTMX page handlers ──────────────────────────────────────────────

/// Render a Maud markup as an HTML response with the CSRF cookie attached.
fn html_with_csrf(markup: maud::Markup, csrf: &str) -> Response {
    let mut response = Html(markup).into_response();
    api::cookies::set_csrf_cookie(&mut response, csrf);
    response
}

async fn home_handler(headers: axum::http::HeaderMap) -> Response {
    let (auth, csrf) = extract_auth(&headers);
    html_with_csrf(pages::home_page(auth), &csrf)
}

#[derive(serde::Deserialize)]
struct GamesListQuery {
    #[serde(default = "default_limit")]
    limit: u32,
    #[serde(default)]
    offset: u32,
    #[serde(default)]
    status: Option<String>,
}

fn default_limit() -> u32 {
    10
}

async fn games_list_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Query(params): Query<GamesListQuery>,
) -> Response {
    let (auth, csrf) = extract_auth(&headers);
    let limit = params.limit.min(100);
    let offset = params.offset.min(10000);

    let result = state
        .db
        .query("SELECT * FROM fn::get_list_games($limit, $offset)")
        .bind(("limit", limit))
        .bind(("offset", offset))
        .await;

    let all_games: Vec<ListDisplayGame> = match result {
        Ok(mut result) => result.take(0).unwrap_or_default(),
        Err(_) => vec![],
    };

    let games = filter_games_by_status(&all_games, params.status.as_deref());
    let total = all_games.len() as u32;
    let has_more = (offset + limit) < total;
    let pagination = shared::PaginationMetadata {
        total,
        limit,
        offset,
        has_more,
    };
    let paginated = shared::PaginatedGames { games, pagination };
    let stats = api::templates::pages::GameStats::from_games(&all_games);
    let active_filter = params.status.as_deref().unwrap_or("");

    html_with_csrf(
        pages::games_list_page(auth, &paginated, &stats, active_filter),
        &csrf,
    )
}

fn filter_games_by_status(games: &[ListDisplayGame], status: Option<&str>) -> Vec<ListDisplayGame> {
    match status {
        Some("running") => games
            .iter()
            .filter(|g| g.status == shared::GameStatus::InProgress)
            .cloned()
            .collect(),
        Some("waiting") => games
            .iter()
            .filter(|g| g.status == shared::GameStatus::NotStarted)
            .cloned()
            .collect(),
        Some("finished") => games
            .iter()
            .filter(|g| g.status == shared::GameStatus::Finished)
            .cloned()
            .collect(),
        Some(_) | None => games.to_vec(),
    }
}

// ── Game detail HTMX handlers ─────────────────────────────────────────

async fn game_detail_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(game_identifier): axum::extract::Path<uuid::Uuid>,
) -> Response {
    let (auth, csrf) = extract_auth(&headers);
    let identifier = game_identifier.to_string();
    let result = state
        .db
        .query("SELECT * FROM fn::get_display_game($identifier);")
        .bind(("identifier", identifier.clone()))
        .await;

    let game = match result {
        Ok(mut result) => {
            let game: Option<shared::DisplayGame> = result.take(0).unwrap_or_default();
            game
        }
        Err(_) => None,
    };

    match game {
        Some(game) => html_with_csrf(game_detail::game_detail_page(auth, &game), &csrf),
        None => html_with_csrf(
            pages::not_found_page(auth, "The game you're looking for doesn't exist."),
            &csrf,
        ),
    }
}

async fn game_tributes_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(game_identifier): axum::extract::Path<uuid::Uuid>,
) -> Response {
    let (auth, csrf) = extract_auth(&headers);
    let identifier = game_identifier.to_string();
    let result = state
        .db
        .query("SELECT * FROM fn::get_tributes_by_game($identifier);")
        .bind(("identifier", identifier.clone()))
        .await;

    let tributes = match result {
        Ok(mut result) => {
            let tributes: Vec<Vec<game::tributes::Tribute>> =
                result.take("tributes").unwrap_or_default();
            tributes.into_iter().next().unwrap_or_default()
        }
        Err(_) => vec![],
    };

    html_with_csrf(
        game_detail::tributes_page(auth, &identifier, &tributes),
        &csrf,
    )
}

async fn game_areas_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(game_identifier): axum::extract::Path<uuid::Uuid>,
) -> Response {
    let (auth, csrf) = extract_auth(&headers);
    let identifier = game_identifier.to_string();
    let result = state
        .db
        .query(
            r#"
SELECT (
    SELECT *, ->items->item[*] AS items
    FROM ->areas->area
) AS areas FROM game WHERE identifier = $identifier;
"#,
        )
        .bind(("identifier", identifier.clone()))
        .await;

    let areas = match result {
        Ok(mut result) => {
            let areas: Vec<Vec<game::areas::AreaDetails>> =
                result.take("areas").unwrap_or_default();
            areas.into_iter().next().unwrap_or_default()
        }
        Err(_) => vec![],
    };

    html_with_csrf(game_detail::areas_page(auth, &identifier, &areas), &csrf)
}

async fn game_log_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(game_identifier): axum::extract::Path<uuid::Uuid>,
) -> Response {
    let (auth, csrf) = extract_auth(&headers);
    let identifier = game_identifier.to_string();
    let result = state
        .db
        .query(
            r#"SELECT * FROM message
            WHERE string::starts_with(subject, $identifier)
            ORDER BY game_day, phase, tick, emit_index;"#,
        )
        .bind(("identifier", identifier.clone()))
        .await;

    let messages = match result {
        Ok(mut logs) => {
            let rows: Vec<api::games::GameLog> = logs.take(0).unwrap_or_default();
            rows.into_iter()
                .map(shared::messages::GameMessage::from)
                .collect()
        }
        Err(_) => vec![],
    };

    html_with_csrf(game_detail::log_page(auth, &identifier, &messages), &csrf)
}

// ── CSRF validation ─────────────────────────────────────────────────

fn validate_csrf(headers: &axum::http::HeaderMap, form_token: &str) -> bool {
    let cookie_token = read_cookie(headers, CSRF_COOKIE);
    cookie_token.is_some_and(|t| t == form_token)
}

// ── Auth form types ─────────────────────────────────────────────────

#[derive(serde::Deserialize)]
struct AuthTabQuery {
    tab: Option<String>,
    error: Option<String>,
}

#[derive(serde::Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
    #[serde(default)]
    csrf_token: String,
}

#[derive(serde::Deserialize)]
struct RegisterRequest {
    display_name: String,
    email: String,
    password: String,
    confirm_password: String,
    #[serde(default)]
    csrf_token: String,
}

#[derive(serde::Deserialize)]
struct VerifyQuery {
    token: String,
}

#[derive(serde::Deserialize)]
struct CheckEmailQuery {
    address: Option<String>,
}

#[derive(serde::Deserialize)]
struct ResendVerificationRequest {
    email: String,
    #[serde(default)]
    csrf_token: String,
}

#[derive(serde::Deserialize)]
struct CreateGameRequest {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    private: Option<String>,
    #[serde(default)]
    csrf_token: String,
}

// ── Auth page handlers ──────────────────────────────────────────────

/// GET /auth — render unified auth form with CSRF token.
async fn auth_handler(
    headers: axum::http::HeaderMap,
    Query(params): Query<AuthTabQuery>,
) -> impl IntoResponse {
    let (auth, csrf) = extract_auth(&headers);
    if auth.is_authenticated() {
        return Redirect::to("/games").into_response();
    }
    let body = auth::auth_page_with_csrf(
        auth,
        &csrf,
        params.error.as_deref(),
        auth::AuthTab::from_query(params.tab.as_deref()),
    );
    html_with_csrf(body, &csrf)
}

/// POST /login — authenticate user.
async fn login_post_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Form(form): Form<LoginRequest>,
) -> impl IntoResponse {
    handle_login_post(&state, &headers, form.email, form.password, form.csrf_token).await
}

/// POST /register — create new user account.
async fn register_post_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Form(form): Form<RegisterRequest>,
) -> impl IntoResponse {
    handle_register_post(
        &state,
        &headers,
        form.display_name,
        form.email,
        form.password,
        form.confirm_password,
        form.csrf_token,
    )
    .await
}

/// Redirect to an auth tab with an error message in the query string.
/// Uses `url::form_urlencoded` for robust query construction and
/// percent-encoding. Handles paths that already contain query strings.
fn redirect_with_error(path: &str, tab: &str, error: &str) -> Response {
    let query = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("tab", tab)
        .append_pair("error", error)
        .finish();
    let separator = if path.contains('?') { "&" } else { "?" };
    Redirect::to(&format!("{}{}{}", path, separator, query)).into_response()
}
/// Handle login POST logic.
async fn handle_login_post(
    state: &AppState,
    headers: &axum::http::HeaderMap,
    login: String,
    password: String,
    csrf_token: String,
) -> Response {
    if !validate_csrf(headers, &csrf_token) {
        return redirect_with_error("/auth", "login", "Invalid form submission");
    }

    let user_db = (*state.db).clone();
    if user_db
        .use_ns(&state.namespace)
        .use_db(&state.database)
        .await
        .is_err()
    {
        return redirect_with_error("/auth", "login", "Database error");
    }

    // Pass as $email so the scope matches email OR username columns.
    // The scope checks: (email = $email OR username = $email OR username = $username)
    // $username is unset here, so the third clause is a no-op.
    let result = user_db
        .signin(surrealdb::opt::auth::Record {
            access: "user",
            namespace: &state.namespace,
            database: &state.database,
            params: serde_json::json!({
                "email": login.clone(),
                "password": password,
            }),
        })
        .await;

    match result {
        Ok(_auth_result) => {
            use api::auth::{
                RefreshToken, TokenResponse, generate_access_token, store_refresh_token,
            };
            use api::cookies::{set_refresh_cookie, set_session_cookie};
            use serde::Deserialize as SerdeDeserialize;

            // The user_db connection is now authenticated. Query $auth to
            // resolve the user id and display name (username column) from
            // the database rather than parsing the raw SurrealDB JWT.
            #[derive(SerdeDeserialize)]
            struct AuthRow {
                id: surrealdb::sql::Thing,
                username: String,
                email_verified: Option<bool>,
            }

            let mut resp = match user_db
                .query("SELECT id, username, email_verified FROM $auth")
                .await
            {
                Ok(r) => r,
                Err(_) => return redirect_with_error("/auth", "login", "Authentication error"),
            };
            let row: Option<AuthRow> = match resp.take(0) {
                Ok(r) => r,
                Err(_) => return redirect_with_error("/auth", "login", "Authentication error"),
            };
            let AuthRow {
                id: user_id,
                username: display_name,
                email_verified,
            } = match row {
                Some(r) => r,
                None => return redirect_with_error("/auth", "login", "Authentication error"),
            };

            if !email_verified.unwrap_or(false) {
                return redirect_with_error(
                    "/auth",
                    "login",
                    "Please verify your email before signing in",
                );
            }

            // Mint our own JWT carrying `sub: <username>` so display paths
            // (extract_auth_state) read the username directly without a DB
            // round-trip. The token reuses SurrealDB's HS512 key and claim
            // shape, so the SurrealDB SDK still accepts it for record auth.
            let access_token = match generate_access_token(
                &user_id,
                &display_name,
                &state.namespace,
                &state.database,
            ) {
                Ok(t) => t,
                Err(_) => return redirect_with_error("/auth", "login", "Authentication error"),
            };

            let refresh_token = RefreshToken::new(user_id, display_name);
            if store_refresh_token(&user_db, &refresh_token).await.is_err() {
                return redirect_with_error("/auth", "login", "Session error");
            }

            let pair = TokenResponse {
                access_token,
                refresh_token: refresh_token.token,
            };

            let mut response = Redirect::to("/account").into_response();
            set_session_cookie(&mut response, &pair.access_token);
            set_refresh_cookie(&mut response, &pair.refresh_token);
            response
        }
        Err(_) => redirect_with_error("/auth", "login", "Invalid email or username"),
    }
}

/// Handle register POST logic — creates user via SurrealDB signup, sends verification
/// email, and redirects to check-email page (user is NOT auto-signed-in).
async fn handle_register_post(
    state: &AppState,
    headers: &axum::http::HeaderMap,
    display_name: String,
    email: String,
    password: String,
    confirm_password: String,
    csrf_token: String,
) -> Response {
    if !validate_csrf(headers, &csrf_token) {
        return redirect_with_error("/auth", "register", "Invalid form submission");
    }

    if password != confirm_password {
        return redirect_with_error("/auth", "register", "Passwords do not match");
    }

    // Validate the registration inputs
    let reg_user = EmailRegistrationUser {
        display_name: display_name.clone(),
        email: email.clone(),
        password: password.clone(),
    };
    if let Err(e) = reg_user.validate() {
        return redirect_with_error("/auth", "register", &e.to_string());
    }

    let user_db = (*state.db).clone();
    if user_db
        .use_ns(&state.namespace)
        .use_db(&state.database)
        .await
        .is_err()
    {
        return redirect_with_error("/auth", "register", "Database error");
    }

    // Signup via SurrealDB access scope
    // The scope uses `$display_name ?? $username` for backward compat,
    // and `$email` for the email field.
    let result = user_db
        .signup(surrealdb::opt::auth::Record {
            access: "user",
            namespace: &state.namespace,
            database: &state.database,
            params: reg_user,
        })
        .await;

    match result {
        Ok(_auth_result) => {
            // Registration succeeded — send verification email async.
            // We use fire-and-forget (tokio::spawn) so the user is redirected
            // immediately even if the email service is briefly down.
            let email_for_token = email.clone();
            tokio::spawn(async move {
                match generate_verification_token(&email_for_token) {
                    Ok(token) => {
                        if let Err(e) = send_verification_email(&email_for_token, &token).await {
                            tracing::error!(
                                "Failed to send verification email to {}: {}",
                                email_for_token,
                                e
                            );
                        }
                    }
                    Err(e) => {
                        tracing::error!(
                            "Failed to generate verification token for {}: {}",
                            email_for_token,
                            e
                        );
                    }
                }
            });

            // Redirect to check-email page
            Redirect::to(&format!(
                "/auth/check-email?address={}",
                urlencoding(&email)
            ))
            .into_response()
        }
        Err(e) => {
            // SurrealDB signup error — common causes:
            // - unique_email index violation (already registered)
            // - schema validation failure
            let combined = format!("{e} {e:?}").to_lowercase();
            if combined.contains("unique_email") || combined.contains("already exists") {
                return redirect_with_error(
                    "/auth",
                    "register",
                    "An account with this email already exists",
                );
            }
            tracing::warn!("Registration failed with unrecognized error: {}", e);
            redirect_with_error("/auth", "register", "Registration failed")
        }
    }
}

/// GET /auth/check-email — interstitial page shown after registration.
async fn check_email_handler(
    headers: axum::http::HeaderMap,
    Query(params): Query<CheckEmailQuery>,
) -> impl IntoResponse {
    let (auth, csrf) = extract_auth(&headers);
    let body = auth::check_email_page(auth, params.address.as_deref(), &csrf);
    html_with_csrf(body, &csrf)
}

/// GET /auth/verify-email?token=... — verify email address.
async fn verify_email_handler(
    State(state): State<AppState>,
    Query(params): Query<VerifyQuery>,
) -> Response {
    // Validate the verification token
    let email = match validate_verification_token(&params.token) {
        Ok(email) => email,
        Err(_) => {
            return Redirect::to("/auth?tab=login&error=Invalid+or+expired+verification+link")
                .into_response();
        }
    };

    // Mark email as verified in the database
    let user_db = (*state.db).clone();
    if user_db
        .use_ns(&state.namespace)
        .use_db(&state.database)
        .await
        .is_err()
    {
        return Redirect::to("/auth?tab=login&error=Database+error").into_response();
    }

    let result = user_db
        .query("UPDATE user SET email_verified = true WHERE email = $email")
        .bind(("email", email.clone()))
        .await;

    match result {
        Ok(_) => Redirect::to("/auth/email-verified").into_response(),
        Err(e) => {
            tracing::error!("Failed to verify email {}: {}", email, e);
            Redirect::to("/auth?tab=login&error=Verification+failed.+Please+try+again.")
                .into_response()
        }
    }
}

/// Dev-only: bypass email verification without Mailpit.
/// Only active when ENV=development.
/// POST /dev/verify-email with Form { email: "..." }
async fn dev_verify_email_handler(
    State(state): State<AppState>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    let email = match form.get("email") {
        Some(e) => e.trim().to_lowercase(),
        None => return Redirect::to("/auth?tab=login&error=Missing+email").into_response(),
    };

    let env_mode = std::env::var("ENV").unwrap_or_else(|_| "production".to_string());
    if env_mode != "development" {
        return Redirect::to("/").into_response();
    }

    if state
        .db
        .use_ns(&state.namespace)
        .use_db(&state.database)
        .await
        .is_err()
    {
        return Redirect::to("/auth?tab=login&error=Database+error").into_response();
    }

    match state
        .db
        .query("UPDATE user SET email_verified = true WHERE email = $email")
        .bind(("email", email.clone()))
        .await
    {
        Ok(_) => Redirect::to("/auth?tab=login&error=Email+verified!+You+can+now+sign+in.")
            .into_response(),
        Err(e) => {
            tracing::error!("Dev verify failed for {}: {}", email, e);
            Redirect::to("/auth?tab=login&error=Verification+failed.+Try+again.").into_response()
        }
    }
}

/// POST /auth/resend-verification — resend verification email.
async fn resend_verification_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Form(form): Form<ResendVerificationRequest>,
) -> impl IntoResponse {
    if !validate_csrf(&headers, &form.csrf_token) {
        return Redirect::to("/auth?tab=login").into_response();
    }

    // Rate limit: check cooldown cache before proceeding
    let now = std::time::Instant::now();
    let cooldown_key = format!("resend:{}", form.email.to_lowercase());
    if let Some(last_sent) = RESEND_COOLDOWN.lock().unwrap().get(&cooldown_key)
        && now.duration_since(*last_sent).as_secs() < 60
    {
        return Redirect::to(&format!(
            "/auth/check-email?address={}&error={}",
            urlencoding(&form.email),
            urlencoding("Please wait 60 seconds before requesting another email.")
        ))
        .into_response();
    }

    let user_db = (*state.db).clone();
    if user_db
        .use_ns(&state.namespace)
        .use_db(&state.database)
        .await
        .is_err()
    {
        return Redirect::to(&format!(
            "/auth/check-email?address={}&error={}",
            urlencoding(&form.email),
            urlencoding("Something went wrong. Please try again.")
        ))
        .into_response();
    }

    // Check user exists and email is not already verified
    let result = user_db
        .query("SELECT email_verified FROM user WHERE email = $email LIMIT 1")
        .bind(("email", form.email.clone()))
        .await;

    match result {
        Ok(mut resp) => {
            #[derive(serde::Deserialize)]
            struct EmailRow {
                email_verified: Option<bool>,
            }
            let row: Option<EmailRow> = resp.take(0).unwrap_or(None);
            match row {
                Some(r) if r.email_verified.unwrap_or(false) => {
                    Redirect::to("/auth?tab=login&error=Email+already+verified").into_response()
                }
                Some(_) => {
                    // Not verified yet — send new token
                    let email_for_token = form.email.clone();
                    tokio::spawn(async move {
                        match api::email::generate_verification_token(&email_for_token) {
                            Ok(token) => {
                                if let Err(e) =
                                    api::email::send_verification_email(&email_for_token, &token)
                                        .await
                                {
                                    tracing::error!("Resend failed for {}: {}", email_for_token, e);
                                }
                            }
                            Err(e) => {
                                tracing::error!(
                                    "Token generation failed for {}: {}",
                                    email_for_token,
                                    e
                                );
                            }
                        }
                    });

                    // Set cooldown
                    RESEND_COOLDOWN.lock().unwrap().insert(cooldown_key, now);

                    Redirect::to(&format!(
                        "/auth/check-email?address={}",
                        urlencoding(&form.email)
                    ))
                    .into_response()
                }
                None => {
                    // No user found with this email
                    // Don't reveal existence — redirect to check-email anyway
                    Redirect::to(&format!(
                        "/auth/check-email?address={}",
                        urlencoding(&form.email)
                    ))
                    .into_response()
                }
            }
        }
        Err(_) => Redirect::to(&format!(
            "/auth/check-email?address={}&error={}",
            urlencoding(&form.email),
            urlencoding("Something went wrong. Please try again.")
        ))
        .into_response(),
    }
}

/// GET /auth/email-verified — confirmation page after email verification.
async fn email_verified_handler(headers: axum::http::HeaderMap) -> impl IntoResponse {
    let (auth, csrf) = extract_auth(&headers);
    let body = auth::email_verified_page(auth);
    html_with_csrf(body, &csrf)
}

/// Simple URL-encoding for query parameter values.
/// Replaces spaces with `%20` and keeps most alphanumeric/`-_.~` chars literal.
fn urlencoding(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => result.push(c),
            ' ' => result.push_str("%20"),
            _ => {
                for b in c.to_string().bytes() {
                    result.push_str(&format!("%{:02X}", b));
                }
            }
        }
    }
    result
}

/// POST /logout — revoke refresh token, clear cookies, redirect to /auth.
async fn logout_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Form(form): Form<LogoutRequest>,
) -> impl IntoResponse {
    if !validate_csrf(&headers, &form.csrf_token) {
        return Redirect::to("/auth?tab=login").into_response();
    }

    let refresh = read_cookie(&headers, api::cookies::REFRESH_COOKIE).map(|s| s.to_owned());

    if let Some(token) = refresh {
        let user_db = (*state.db).clone();
        if user_db
            .use_ns(&state.namespace)
            .use_db(&state.database)
            .await
            .is_ok()
        {
            let _ = api::auth::revoke_refresh_token(&user_db, &token).await;
        }
    }

    let mut response = Redirect::to("/auth?tab=login").into_response();
    api::cookies::clear_auth_cookies(&mut response);
    api::cookies::clear_csrf_cookie(&mut response);
    response
}

#[derive(serde::Deserialize)]
struct LogoutRequest {
    #[serde(default)]
    csrf_token: String,
}

/// GET /account — account dashboard (requires auth).
async fn account_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Response {
    let (auth, csrf) = extract_auth(&headers);
    let session = match require_auth(&state, &headers).await {
        Ok(s) => s,
        Err(_) => return Redirect::to("/auth?tab=login").into_response(),
    };

    let user_db = (*state.db).clone();
    if user_db
        .use_ns(&state.namespace)
        .use_db(&state.database)
        .await
        .is_err()
    {
        return Redirect::to("/auth?tab=login").into_response();
    }

    let games: Vec<ListDisplayGame> = user_db
        .query("SELECT * FROM fn::get_list_games(100, 0)")
        .await
        .ok()
        .and_then(|mut r| r.take(0).ok())
        .unwrap_or_default();

    html_with_csrf(auth::account_page(auth, &session, &games), &csrf)
}

/// GET /games/new — create game form (requires auth).
async fn create_game_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Response {
    let (auth, csrf) = extract_auth(&headers);
    if require_auth(&state, &headers).await.is_err() {
        return Redirect::to("/auth").into_response();
    }

    let body = auth::create_game_page_with_csrf(auth, &csrf);
    html_with_csrf(body, &csrf)
}

/// POST /games/new — create game, redirect to /games/{id}.
async fn create_game_post_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Form(form): Form<CreateGameRequest>,
) -> Response {
    if !validate_csrf(&headers, &form.csrf_token) {
        return Redirect::to("/auth").into_response();
    }

    let token = match read_cookie(&headers, SESSION_COOKIE) {
        Some(t) => t.to_owned(),
        None => return Redirect::to("/auth").into_response(),
    };

    let user_db = match authenticate_db(&state, &token).await {
        Ok(db) => db,
        Err(redirect) => return redirect.into_response(),
    };

    use game::games::Game;
    let game = Game::default();
    let game_identifier = uuid::Uuid::new_v4().to_string();
    let game_name = form.name.filter(|n| !n.is_empty()).unwrap_or(game.name);
    let is_private = form.private.is_some_and(|v| v == "true");

    let new_game = Game {
        identifier: game_identifier.clone(),
        name: game_name,
        status: shared::GameStatus::NotStarted,
        day: None,
        tributes: vec![],
        areas: vec![],
        private: is_private,
        config: Default::default(),
        messages: vec![],
        alliance_events: vec![],
        ..Default::default()
    };

    use surrealdb::RecordId;

    let game_rid = RecordId::from(("game", game_identifier.as_str()));
    let body = match serde_json::to_value(&new_game) {
        Ok(b) => b,
        Err(_) => return Redirect::to("/games/new").into_response(),
    };

    if user_db
        .query("UPSERT $rid CONTENT $body")
        .bind(("rid", game_rid.clone()))
        .bind(("body", body))
        .await
        .is_err()
    {
        return Redirect::to("/games/new").into_response();
    }

    // Create tributes
    let tribute_futures = (0..24)
        .map(|idx| api::tributes::create_tribute(None, &game_identifier, &user_db, idx % 12));
    let tribute_results = futures::future::join_all(tribute_futures).await;
    if tribute_results.into_iter().any(|r| r.is_err()) {
        return Redirect::to("/games/new").into_response();
    }

    // Create areas
    use game::areas::Area;
    use strum::IntoEnumIterator;
    let base_item_count = shared::ItemQuantity::default().base_item_count();
    let area_futures = Area::iter()
        .map(|area| api::games::create_area(&game_identifier, area, base_item_count, &user_db));
    let area_results = futures::future::join_all(area_futures).await;
    if area_results.into_iter().any(|r| r.is_err()) {
        return Redirect::to("/games/new").into_response();
    }

    Redirect::to(&format!("/games/{game_identifier}")).into_response()
}

// ── Auth helpers ────────────────────────────────────────────────────

/// Extract authentication state from request cookies, paired with the CSRF token.
///
/// The CSRF token is read from the existing `hg_csrf` cookie when present and
/// only minted fresh when no cookie exists. This keeps the token stable across
/// tabs and page loads so a form rendered on one page still validates after
/// the user navigates elsewhere and back.
fn extract_auth(headers: &axum::http::HeaderMap) -> (AuthState, String) {
    let csrf = read_cookie(headers, CSRF_COOKIE)
        .map(|s| s.to_owned())
        .unwrap_or_else(generate_csrf_token);
    let auth = extract_auth_state(headers, &csrf);
    (auth, csrf)
}

fn extract_auth_state(headers: &axum::http::HeaderMap, csrf: &str) -> AuthState {
    let token = match read_cookie(headers, SESSION_COOKIE) {
        Some(t) => t.to_owned(),
        None => return AuthState::guest(csrf),
    };

    let token_parts: Vec<&str> = token.split('.').collect();
    if token_parts.len() != 3 {
        return AuthState::guest(csrf);
    }

    let payload_base64 = token_parts[1].trim_start_matches('=');
    let payload_bytes = match base64_url::decode(payload_base64) {
        Ok(b) => b,
        Err(_) => return AuthState::guest(csrf),
    };

    let payload_str = match String::from_utf8(payload_bytes) {
        Ok(s) => s,
        Err(_) => return AuthState::guest(csrf),
    };

    let payload: Value = match serde_json::from_str(&payload_str) {
        Ok(v) => v,
        Err(_) => return AuthState::guest(csrf),
    };

    let exp = payload.get("exp").and_then(|v| v.as_u64()).unwrap_or(0);
    let now = OffsetDateTime::now_utc().unix_timestamp() as u64;
    if exp < now {
        return AuthState::guest(csrf);
    }

    let id = payload.get("ID").and_then(|v| v.as_str()).map(String::from);
    let username = payload
        .get("sub")
        .and_then(|v| v.as_str())
        .map(String::from);

    match (id, username) {
        (Some(id), Some(name)) if !name.is_empty() => AuthState::authenticated(id, name, csrf),
        _ => {
            // No `sub` claim — this is a raw SurrealDB-issued JWT (only
            // carries `ID: user:<record-id>`). We refuse to surface the
            // record id as a display name. The cookie will be replaced
            // with a `sub`-bearing token on the next login or refresh.
            tracing::warn!(
                "session JWT missing `sub` or `ID` claim; treating as guest for display"
            );
            AuthState::guest(csrf)
        }
    }
}

/// Check session cookie exists and is not expired.
///
/// For SurrealDB-issued JWTs (which lack a `sub` claim) this queries the
/// database to resolve the real username from the user record.
async fn require_auth(
    state: &AppState,
    headers: &axum::http::HeaderMap,
) -> Result<UserSession, ()> {
    let token = read_cookie(headers, SESSION_COOKIE).ok_or(())?.to_owned();

    let token_parts: Vec<&str> = token.split('.').collect();
    if token_parts.len() != 3 {
        return Err(());
    }
    let payload_base64 = token_parts[1].trim_start_matches('=');
    let payload_bytes = base64_url::decode(payload_base64).map_err(|_| ())?;
    let payload_str = String::from_utf8(payload_bytes).map_err(|_| ())?;
    let payload: Value = serde_json::from_str(&payload_str).map_err(|_| ())?;

    let exp = payload.get("exp").and_then(|v| v.as_u64()).unwrap_or(0);
    let now = OffsetDateTime::now_utc().unix_timestamp() as u64;
    if exp < now {
        return Err(());
    }

    // Prefer "sub" (own-issued tokens). SurrealDB-issued JWTs don't carry
    // "sub" or the actual username — only "ID":"user:<record-id>".
    if let Some(username) = payload.get("sub").and_then(|v| v.as_str()) {
        // Own-issued token has username inline — no DB query needed.
        let id = payload
            .get("id")
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_default();
        return Ok(UserSession {
            id,
            username: username.to_owned(),
        });
    }

    // SurrealDB-issued JWT — authenticate the connection and query $auth to
    // resolve the real username from the database.
    let user_db = (*state.db).clone();
    if user_db
        .use_ns(&state.namespace)
        .use_db(&state.database)
        .await
        .is_err()
    {
        return Err(());
    }
    if user_db
        .authenticate(surrealdb::opt::auth::Jwt::from(token.as_str()))
        .await
        .is_err()
    {
        return Err(());
    }

    #[derive(serde::Deserialize)]
    struct AuthRow {
        id: surrealdb::sql::Thing,
        username: String,
    }

    let mut response = match user_db.query("SELECT id, username FROM $auth").await {
        Ok(r) => r,
        Err(_) => return Err(()),
    };
    let row: Option<AuthRow> = match response.take(0) {
        Ok(r) => r,
        Err(_) => return Err(()),
    };
    let row = match row {
        Some(r) => r,
        None => return Err(()),
    };

    Ok(UserSession {
        id: row.id.to_string(),
        username: row.username,
    })
}

/// Clone the shared DB and authenticate with the given JWT.
async fn authenticate_db(
    state: &AppState,
    token: &str,
) -> Result<surrealdb::Surreal<Any>, Redirect> {
    let user_db = (*state.db).clone();
    if user_db
        .use_ns(&state.namespace)
        .use_db(&state.database)
        .await
        .is_err()
    {
        return Err(Redirect::to("/auth"));
    }
    if user_db.authenticate(Jwt::from(token)).await.is_err() {
        return Err(Redirect::to("/auth"));
    }
    Ok(user_db)
}

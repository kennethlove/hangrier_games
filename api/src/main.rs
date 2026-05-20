extern crate core;

use api::auth::AUTH_ROUTER;
use api::cleanup::start_cleanup_scheduler;
use api::cookies::{CSRF_COOKIE, SESSION_COOKIE, generate_csrf_token, read_cookie};
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
use shared::{ListDisplayGame, RegistrationUser, UserSession};
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
use tower_http::normalize_path::NormalizePathLayer;
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

    MigrationRunner::new(&db)
        .up()
        .await
        .map_err(|e| format!("Failed to apply migrations: {}", e))?;
    tracing::debug!("Applied migrations");

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
        .route(
            "/api/games/{game_id}/events",
            axum::routing::get(api::sse::sse_handler).layer(middleware::from_fn_with_state(
                app_state.clone(),
                surreal_jwt,
            )),
        )
        .route("/", axum::routing::get(home_handler))
        .route("/games", axum::routing::get(games_list_handler))
        .route("/games/{id}", axum::routing::get(game_detail_handler))
        .route(
            "/games/{id}/tributes",
            axum::routing::get(game_tributes_handler),
        )
        .route("/games/{id}/areas", axum::routing::get(game_areas_handler))
        .route("/games/{id}/log", axum::routing::get(game_log_handler))
        .route("/auth", axum::routing::get(auth_handler))
        .route("/login", axum::routing::post(login_post_handler))
        .route("/register", axum::routing::post(register_post_handler))
        .route("/logout", axum::routing::post(logout_handler))
        .route("/account", axum::routing::get(account_handler))
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

async fn home_handler(headers: axum::http::HeaderMap) -> Html<maud::Markup> {
    let auth = extract_auth(&headers);
    Html(pages::home_page(auth))
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
) -> Html<maud::Markup> {
    let auth = extract_auth(&headers);
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

    Html(pages::games_list_page(
        auth,
        &paginated,
        &stats,
        active_filter,
    ))
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
) -> Html<maud::Markup> {
    let auth = extract_auth(&headers);
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
        Some(game) => Html(game_detail::game_detail_page(auth, &game)),
        None => Html(pages::not_found_page(
            "The game you're looking for doesn't exist.",
        )),
    }
}

async fn game_tributes_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(game_identifier): axum::extract::Path<uuid::Uuid>,
) -> Html<maud::Markup> {
    let auth = extract_auth(&headers);
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

    Html(game_detail::tributes_page(auth, &identifier, &tributes))
}

async fn game_areas_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(game_identifier): axum::extract::Path<uuid::Uuid>,
) -> Html<maud::Markup> {
    let auth = extract_auth(&headers);
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

    Html(game_detail::areas_page(auth, &identifier, &areas))
}

async fn game_log_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(game_identifier): axum::extract::Path<uuid::Uuid>,
) -> Html<maud::Markup> {
    let auth = extract_auth(&headers);
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

    Html(game_detail::log_page(auth, &identifier, &messages))
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
}

#[derive(serde::Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
    #[serde(default)]
    csrf_token: String,
}

#[derive(serde::Deserialize)]
struct RegisterRequest {
    username: String,
    password: String,
    confirm_password: String,
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
    let auth = extract_auth(&headers);
    if auth.is_authenticated {
        return Redirect::to("/games").into_response();
    }
    let csrf = generate_csrf_token();
    let body = auth::auth_page_with_csrf(&csrf, auth::AuthTab::from_query(params.tab.as_deref()));
    (
        [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
        body,
    )
        .into_response()
}

/// POST /login — authenticate user.
async fn login_post_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Form(form): Form<LoginRequest>,
) -> impl IntoResponse {
    handle_login_post(
        &state,
        &headers,
        form.username,
        form.password,
        form.csrf_token,
        "/auth?tab=login",
    )
    .await
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
        form.username,
        form.password,
        form.confirm_password,
        form.csrf_token,
        "/auth?tab=register",
    )
    .await
}

/// Handle login POST logic.
async fn handle_login_post(
    state: &AppState,
    headers: &axum::http::HeaderMap,
    username: String,
    password: String,
    csrf_token: String,
    redirect_on_error: &str,
) -> Response {
    if !validate_csrf(headers, &csrf_token) {
        return Redirect::to(redirect_on_error).into_response();
    }

    let user_db = (*state.db).clone();
    if user_db
        .use_ns(&state.namespace)
        .use_db(&state.database)
        .await
        .is_err()
    {
        return Redirect::to(redirect_on_error).into_response();
    }

    let result = user_db
        .signin(surrealdb::opt::auth::Record {
            access: "user",
            namespace: &state.namespace,
            database: &state.database,
            params: RegistrationUser {
                username: username.clone(),
                password,
            },
        })
        .await;

    match result {
        Ok(auth_result) => {
            let jwt = auth_result.into_insecure_token();

            use api::auth::{RefreshToken, TokenResponse, store_refresh_token};
            use api::cookies::{set_refresh_cookie, set_session_cookie};
            use surrealdb::sql::Thing;

            let user_id: Thing = match surrealdb::sql::thing(
                &extract_user_id_from_jwt_raw(&jwt).unwrap_or_default(),
            ) {
                Ok(id) => id,
                Err(_) => return Redirect::to(redirect_on_error).into_response(),
            };

            let refresh_token = RefreshToken::new(user_id, username);
            if store_refresh_token(&user_db, &refresh_token).await.is_err() {
                return Redirect::to(redirect_on_error).into_response();
            }

            let pair = TokenResponse {
                access_token: jwt.clone(),
                refresh_token: refresh_token.token,
            };

            let mut response = Redirect::to("/account").into_response();
            set_session_cookie(&mut response, &pair.access_token);
            set_refresh_cookie(&mut response, &pair.refresh_token);
            response
        }
        Err(_) => Redirect::to(redirect_on_error).into_response(),
    }
}

/// Handle register POST logic.
async fn handle_register_post(
    state: &AppState,
    headers: &axum::http::HeaderMap,
    username: String,
    password: String,
    confirm_password: String,
    csrf_token: String,
    redirect_on_error: &str,
) -> Response {
    if !validate_csrf(headers, &csrf_token) {
        return Redirect::to(redirect_on_error).into_response();
    }

    if password != confirm_password {
        return Redirect::to(redirect_on_error).into_response();
    }

    let user_db = (*state.db).clone();
    if user_db
        .use_ns(&state.namespace)
        .use_db(&state.database)
        .await
        .is_err()
    {
        return Redirect::to(redirect_on_error).into_response();
    }

    let result = user_db
        .signup(surrealdb::opt::auth::Record {
            access: "user",
            namespace: &state.namespace,
            database: &state.database,
            params: RegistrationUser {
                username: username.clone(),
                password,
            },
        })
        .await;

    match result {
        Ok(auth_result) => {
            let jwt = auth_result.into_insecure_token();

            use api::auth::{RefreshToken, TokenResponse, store_refresh_token};
            use api::cookies::{set_refresh_cookie, set_session_cookie};
            use surrealdb::sql::Thing;

            let user_id: Thing = match surrealdb::sql::thing(
                &extract_user_id_from_jwt_raw(&jwt).unwrap_or_default(),
            ) {
                Ok(id) => id,
                Err(_) => return Redirect::to(redirect_on_error).into_response(),
            };

            let refresh_token = RefreshToken::new(user_id, username);
            if store_refresh_token(&user_db, &refresh_token).await.is_err() {
                return Redirect::to(redirect_on_error).into_response();
            }

            let pair = TokenResponse {
                access_token: jwt.clone(),
                refresh_token: refresh_token.token,
            };

            let mut response = Redirect::to("/account").into_response();
            set_session_cookie(&mut response, &pair.access_token);
            set_refresh_cookie(&mut response, &pair.refresh_token);
            response
        }
        Err(_) => Redirect::to(redirect_on_error).into_response(),
    }
}

/// POST /logout — revoke refresh token, clear cookies, redirect to /auth.
async fn logout_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Form(form): Form<LogoutRequest>,
) -> impl IntoResponse {
    if !validate_csrf(&headers, &form.csrf_token) {
        return Redirect::to("/auth").into_response();
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

    let mut response = Redirect::to("/auth").into_response();
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
    let session = match require_auth(&state, &headers).await {
        Ok(s) => s,
        Err(_) => return Redirect::to("/auth").into_response(),
    };

    let user_db = (*state.db).clone();
    if user_db
        .use_ns(&state.namespace)
        .use_db(&state.database)
        .await
        .is_err()
    {
        return Redirect::to("/auth").into_response();
    }

    let games: Vec<ListDisplayGame> = user_db
        .query("SELECT * FROM fn::get_list_games(100, 0)")
        .await
        .ok()
        .and_then(|mut r| r.take(0).ok())
        .unwrap_or_default();

    Html(auth::account_page(&session, &games)).into_response()
}

/// GET /games/new — create game form (requires auth).
async fn create_game_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    if require_auth(&state, &headers).await.is_err() {
        return Redirect::to("/auth").into_response();
    }

    let csrf = generate_csrf_token();
    let body = auth::create_game_page_with_csrf(&csrf);
    (
        [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
        body,
    )
        .into_response()
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

/// Extract authentication state from request cookies.
fn extract_auth(headers: &axum::http::HeaderMap) -> AuthState {
    let token = match read_cookie(headers, SESSION_COOKIE) {
        Some(t) => t.to_owned(),
        None => return AuthState::guest(),
    };

    let token_parts: Vec<&str> = token.split('.').collect();
    if token_parts.len() != 3 {
        return AuthState::guest();
    }

    let payload_base64 = token_parts[1].trim_start_matches('=');
    let payload_bytes = match base64_url::decode(payload_base64) {
        Ok(b) => b,
        Err(_) => return AuthState::guest(),
    };

    let payload_str = match String::from_utf8(payload_bytes) {
        Ok(s) => s,
        Err(_) => return AuthState::guest(),
    };

    let payload: Value = match serde_json::from_str(&payload_str) {
        Ok(v) => v,
        Err(_) => return AuthState::guest(),
    };

    let exp = payload.get("exp").and_then(|v| v.as_u64()).unwrap_or(0);
    let now = OffsetDateTime::now_utc().unix_timestamp() as u64;
    if exp < now {
        return AuthState::guest();
    }

    let username = payload
        .get("sub")
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_default();

    if username.is_empty() {
        AuthState::guest()
    } else {
        AuthState::authenticated(username)
    }
}

/// Extract user ID from JWT payload without full validation.
fn extract_user_id_from_jwt_raw(jwt: &str) -> Option<String> {
    let token_parts: Vec<&str> = jwt.split('.').collect();
    if token_parts.len() != 3 {
        return None;
    }
    let payload_base64 = token_parts[1].trim_start_matches('=');
    let payload_bytes = base64_url::decode(payload_base64).ok()?;
    let payload_str = String::from_utf8(payload_bytes).ok()?;
    let payload: Value = serde_json::from_str(&payload_str).ok()?;
    payload.get("id").and_then(|v| v.as_str()).map(String::from)
}

/// Check session cookie exists and is not expired.
async fn require_auth(
    _state: &AppState,
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

    let username = payload
        .get("sub")
        .and_then(|v| v.as_str())
        .map(String::from)
        .ok_or(())?;

    let id = payload
        .get("id")
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_default();

    Ok(UserSession { id, username })
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

use api::games::GAMES_ROUTER;
use api::users::USERS_ROUTER;
use api::AppState;
use axum::error_handling::HandleErrorLayer;
use axum::extract::{Request, State};
use axum::http::header::{ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_ORIGIN, ACCESS_CONTROL_MAX_AGE, AUTHORIZATION, CACHE_CONTROL, CONTENT_TYPE, EXPIRES};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::{middleware, BoxError, Json, Router};
use std::env;
use std::sync::LazyLock;
use std::time::Duration;
use surrealdb::engine::any::Any;
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use surrealdb_migrations::MigrationRunner;
use tower::ServiceBuilder;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub static DATABASE: LazyLock<Surreal<Any>> = LazyLock::new(Surreal::init);

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
    let tracing_level = if production == "true" { "info" } else { "debug" };

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
async fn main() {
    initialize_logging();

    let app_state = AppState { db: Surreal::init() };

    app_state.db.connect(env::var("SURREAL_HOST").unwrap()).await.expect("Failed to connect to database");
    tracing::debug!("connected to SurrealDB");

    app_state.db.signin(Root {
        username: env::var("SURREAL_USER").unwrap().as_str(),
        password: env::var("SURREAL_PASS").unwrap().as_str(),
    }).await.unwrap();
    tracing::debug!("authenticated to SurrealDB");

    app_state.db.use_ns("hangry-games").use_db("games").await.unwrap();
    tracing::debug!("Using 'hangry-games' namespace and 'games' database");

    MigrationRunner::new(&app_state.db)
        .up()
        .await
        .expect("Failed to apply migrations");
    tracing::debug!("Applied migrations");

    let cors_layer = CorsLayer::new()
        .allow_methods(vec![
            "DELETE".parse().unwrap(),
            "GET".parse().unwrap(),
            "HEAD".parse().unwrap(),
            "OPTIONS".parse().unwrap(),
            "POST".parse().unwrap(),
            "PUT".parse().unwrap(),
        ])
        .allow_origin(AllowOrigin::any())
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

    let api_routes = Router::new()
        .nest("/games", GAMES_ROUTER.clone()
            .layer(middleware::from_fn_with_state(app_state.clone(), surreal_jwt)))
        .nest("/users", USERS_ROUTER.clone());

    let router = Router::new()
        .nest("/api", api_routes)
        .route("/", axum::routing::get(move || async { Json(env!("CARGO_PKG_VERSION")) }))
        .with_state(app_state)
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|error: BoxError| async move {
                    if error.is::<tower::timeout::error::Elapsed>() {
                        Ok(StatusCode::REQUEST_TIMEOUT)
                    } else {
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Unhandled error: {error}"),
                        ))
                    }
                }))
                .timeout(Duration::from_secs(10))
                .layer(TraceLayer::new_for_http())
                .layer(cors_layer)
                .into_inner()
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, router).await.unwrap();
}

async fn surreal_jwt(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let token = request.headers().get("authorization");
    match token {
        None => StatusCode::UNAUTHORIZED.into_response(),
        Some(token) => {
            let token = token.to_str().expect("Failed to convert token to str");
            let token = match token.strip_prefix("Bearer ") {
                Some(token) => token,
                None => return StatusCode::UNAUTHORIZED.into_response(),
            };
            let token = surrealdb::opt::auth::Jwt::from(token);
            match state.db.authenticate(token).await {
                Ok(_) => { next.run(request).await },
                Err(_) => { StatusCode::UNAUTHORIZED.into_response() }
            }
        }
    }
}

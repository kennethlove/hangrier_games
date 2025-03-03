#![feature(internal_output_capture)]

mod games;
mod tributes;

use axum::error_handling::HandleErrorLayer;
use axum::http::StatusCode;
use axum::{BoxError, Router};
use games::GAMES_ROUTER;
use std::env;
use std::sync::LazyLock;
use std::time::Duration;
use surrealdb::engine::any::Any;
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use surrealdb_migrations::MigrationRunner;
use tower::ServiceBuilder;
use tower_http::cors::{Any as CorsAny, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub static DATABASE: LazyLock<Surreal<Any>> = LazyLock::new(Surreal::init);

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect("Failed to load .env file");
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug,surrealdb=debug,surrealdb_client=debug", env!("CARGO_CRATE_NAME")).into()
            })
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    DATABASE.connect(env::var("SURREAL_HOST").unwrap()).await.expect("Failed to connect to database");
    tracing::debug!("connected to SurrealDB");

    DATABASE.signin(Root {
        username: env::var("SURREAL_USER").unwrap().as_str(),
        password: env::var("SURREAL_PASS").unwrap().as_str(),
    }).await.unwrap();
    tracing::debug!("authenticated to SurrealDB");

    DATABASE.use_ns("hangry-games").use_db("games").await.unwrap();
    tracing::debug!("Using 'hangry-games' namespace and 'games' database");

    MigrationRunner::new(&DATABASE)
        .up()
        .await
        .expect("Failed to apply migrations");
    tracing::debug!("Applied migrations");

    let cors_layer = CorsLayer::new()
        .allow_origin(CorsAny)
        .allow_headers(CorsAny)
        .allow_methods(vec![
            "GET".parse().unwrap(),
            "POST".parse().unwrap(),
            "PUT".parse().unwrap(),
            "DELETE".parse().unwrap(),
        ]);

    let api_routes = Router::new()
        .nest("/games", GAMES_ROUTER.clone());

    let router = Router::new()
        .nest("/api", api_routes)
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
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, router).await.unwrap();
}


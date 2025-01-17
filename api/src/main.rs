use axum::error_handling::HandleErrorLayer;
use axum::routing::{get, post};
use axum::{BoxError, Json, Router};
use std::env;
use std::sync::LazyLock;
use std::time::Duration;
use axum::http::{Response, StatusCode};
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use surrealdb::engine::remote::ws::{Client, Wss};
use surrealdb::opt::auth::Root;
use surrealdb::sql::{Id, Thing};
use surrealdb::Surreal;
use tower::ServiceBuilder;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use game::games::Game;

pub static DATABASE: LazyLock<Surreal<Client>> = LazyLock::new(Surreal::init);

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

    DATABASE.connect::<Wss>("surrealdb.eyeheartzombies.com").await.unwrap();
    tracing::debug!("connected to SurrealDB");
    DATABASE.signin(Root {
        username: env::var("SURREAL_USER").unwrap().as_str(),
        password: env::var("SURREAL_PASS").unwrap().as_str(),
    }).await.unwrap();
    tracing::debug!("authenticated to SurrealDB");

    let cors_layer = CorsLayer::new()
        .allow_origin(AllowOrigin::any())
        .allow_headers(vec!["content-type".parse().unwrap()])
        .allow_methods(vec![
            "GET".parse().unwrap(),
            "POST".parse().unwrap(),
            "PUT".parse().unwrap(),
            "DELETE".parse().unwrap(),
        ]);

    let router = Router::new()
        .route("/", get(games_list).post(games_create))
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

#[derive(Debug, Deserialize, Serialize)]
struct GameThing {
    tb: String,
    id: String
}

impl From<Thing> for GameThing {
    fn from(thing: Thing) -> Self {
        GameThing {
            tb: thing.tb,
            id: thing.id.to_string(),
        }
    }
}

async fn games_list() -> (StatusCode, Json<Vec<GameThing>>){
    DATABASE.use_ns("hangry-games").use_db("games").await.expect("Failed to use games database");
    let games: Vec<GameThing> = DATABASE.select("game").await.unwrap();
    (StatusCode::OK, Json::<Vec<GameThing>>(games))
}

async fn games_create() -> (StatusCode, Json<Game>){
    DATABASE.use_ns("hangry-games").use_db("games").await.expect("Failed to use game database");
    let new_game = Game::default();
    dbg!(&new_game);
    let game: Option<Game> = DATABASE.create("game").content(new_game).await.expect("Failed to create game");
    (StatusCode::CREATED, game.unwrap().into())
}

use crate::DATABASE;
use axum::http::StatusCode;
use axum::routing::get;
use axum::Json;
use axum::Router;
use game::games::Game;
use std::rc::Rc;
use std::sync::LazyLock;

pub static GAMES_ROUTER: LazyLock<Router> = LazyLock::new(|| {
    Router::new()
        .route("/", get(games_list).post(games_create))
});

pub async fn games_list() -> (StatusCode, Json<Vec<Game>>) {
    DATABASE.use_ns("hangry-games").use_db("games").await.expect("Failed to use games database");
    match DATABASE.select("game").await {
        Ok(games) => {
            (StatusCode::OK, Json::<Vec<Game>>(games))
        }
        Err(e) => {
            tracing::error!("{}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json::<Vec<Game>>(Vec::new()))
        }
    }
}

pub async fn games_create() -> (StatusCode, Json<Game>) {
    DATABASE.use_ns("hangry-games").use_db("games").await.expect("Failed to use game database");
    let new_game = Game::default();
    let game: Option<Game> = DATABASE.create("game").content(new_game).await.expect("Failed to create game");
    (StatusCode::CREATED, game.unwrap().into())
}

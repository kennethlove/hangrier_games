use crate::DATABASE;
use axum::extract::Path;
use axum::http::header::{CACHE_CONTROL, EXPIRES};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Json;
use axum::Router;
use game::areas::Area;
use game::games::Game;
use serde::{Deserialize, Serialize};
use shared::CreateGame;
use std::sync::LazyLock;
use strum::IntoEnumIterator;
use surrealdb::engine::any::Any;
use surrealdb::method::Query;
use surrealdb::RecordId;
use surrealdb::sql::Value;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GameArea {
    // #[serde(rename="in")]
    pub game: RecordId,
    // #[serde(rename="in")]
    pub area: Area,
    pub open: bool,
}

pub static GAMES_ROUTER: LazyLock<Router> = LazyLock::new(|| {
    Router::new()
        .route("/", get(games_list).post(games_create))
        .route("/{name}", get(game_detail).delete(game_delete))
});

pub async fn games_list() -> impl IntoResponse {
    match DATABASE.select("game").await {
        Ok(games) => {
            let mut headers = HeaderMap::new();
            headers.insert(CACHE_CONTROL, "no-store".parse().unwrap());
            headers.insert(EXPIRES, "1".parse().unwrap());
            (StatusCode::OK, headers, Json::<Vec<Game>>(games)).into_response()
        }
        Err(e) => {
            tracing::error!("{}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json::<Vec<Game>>(Vec::new())).into_response()
        }
    }
}

pub async fn games_create(Json(payload): Json<CreateGame>) -> impl IntoResponse {
    let mut new_game = Game::default();
    if payload.name.is_some() {
        new_game.name = payload.name.unwrap();
    }

    let game: Option<Game> = DATABASE
        .create(("game", &new_game.name))
        .content(new_game.clone())
        .await.expect("failed to create game");

    for area in Area::iter() {
        let ga = GameArea {
            game: RecordId::from(("game", new_game.clone().name)),
            // game: game.clone().unwrap(),
            area,
            open: true,
        };
        let _: Vec<GameArea> = DATABASE.insert("game_area").content(ga).await.unwrap();
    }

    (StatusCode::OK, Json::<Game>(game.clone().unwrap())).into_response()
}

// TODO: Move this to somewhere more available, use GameAreas better
#[derive(Serialize, Deserialize)]
struct GameDetail {
    game: Game,
    areas: Vec<GameArea>,
}

pub async fn game_detail(name: Path<String>) -> (StatusCode, Json<Option<GameDetail>>) {
    let mut result = DATABASE
        .query(format!("SELECT * FROM game WHERE name='{}'", &name.0))
        .query(format!("SELECT * FROM game_area WHERE game.name = '{}'", &name.0))
        .await.unwrap();
    let mut game: Option<Game> = result.take(0).expect("no game found");
    let areas: Vec<GameArea> = result.take(1).expect("Expected areas");

    if let Some(game) = game {
        let details = GameDetail { game, areas };
        (StatusCode::OK, Json(Some(details)))
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json::<Option<GameDetail>>(None))
    }
    // match result.take::<Vec<Area>>(0) {
    //     Ok(_) => {
    //         (StatusCode::OK, Json(Some(game.unwrap())))
    //     }
    //     Err(e) => {
    //         tracing::error!("{}", e);
    //         (StatusCode::INTERNAL_SERVER_ERROR, Json::<Option<Game>>(None))
    //     }
    // }
}

pub async fn game_delete(name: Path<String>) -> StatusCode {
    let game: Option<Game> = DATABASE.delete(("game", &name.0)).await.expect("Failed to delete game");
    match game {
        Some(_) => StatusCode::NO_CONTENT,
        None => {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

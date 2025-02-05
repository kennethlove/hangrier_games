use crate::DATABASE;
use axum::extract::Path;
use axum::http::header::{CACHE_CONTROL, EXPIRES};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Json;
use axum::Router;
use game::areas::{Area, AreaDetails};
use game::games::Game;
use serde::{Deserialize, Serialize};
use shared::CreateGame;
use std::sync::LazyLock;
use strum::IntoEnumIterator;
use surrealdb::engine::any::Any;
use surrealdb::method::Query;
use surrealdb::RecordId;
use surrealdb::sql::Value;

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

pub async fn games_create(Json(payload): Json<Game>) -> impl IntoResponse {
    let game: Option<Game> = DATABASE
        .create(("game", &payload.name))
        .content(payload)
        .await.expect("failed to create game");

    (StatusCode::OK, Json::<Game>(game.clone().unwrap()))
}

pub async fn game_detail(name: Path<String>) -> (StatusCode, Json<Option<Game>>) {
    let mut result: Option<Game> = DATABASE
        .select(("game", name.to_string()))
        .await.unwrap();

    if let Some(game) = result {
        (StatusCode::OK, Json(Some(game)))
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json::<Option<Game>>(None))
    }
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

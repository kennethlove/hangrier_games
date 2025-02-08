use crate::tributes::{create_tribute, create_tribute_record, delete_tribute};
use crate::DATABASE;
use axum::extract::Path;
use axum::http::header::{CACHE_CONTROL, EXPIRES};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{delete, get, post};
use axum::Json;
use axum::Router;
use game::areas::{Area, AreaDetails};
use game::games::Game;
use game::tributes::Tribute;
use serde::{Deserialize, Serialize};
use serde_json::json;
use shared::CreateGame;
use std::sync::LazyLock;
use strum::IntoEnumIterator;
use surrealdb::engine::any::Any;
use surrealdb::method::Query;
use surrealdb::opt::PatchOp;
use surrealdb::sql::Value;
use surrealdb::{RecordId, Response};

pub static GAMES_ROUTER: LazyLock<Router> = LazyLock::new(|| {
    Router::new()
        .route("/", get(games_list).post(games_create))
        .route("/{game_name}", get(game_detail).delete(game_delete))
        .route("/{game_name}/tributes", get(game_tributes).post(create_tribute))
        .route("/{game_name}/tributes/{tribute_name}", delete(delete_tribute))
});

pub async fn games_create(Json(payload): Json<Game>) -> impl IntoResponse {
    let game: Option<Game> = DATABASE
        .create(("game", &payload.name))
        .content(payload)
        .await.expect("failed to create game");

    for _ in 0..24 {
        create_tribute_record(None, game.clone().unwrap().name).await;
    }

    (StatusCode::OK, Json::<Game>(game.clone().unwrap()))
}

pub async fn game_delete(game_name: Path<String>) -> StatusCode {
    let game: Option<Game> = DATABASE.delete(("game", &game_name.0)).await.expect("Failed to delete game");
    match game {
        Some(_) => StatusCode::NO_CONTENT,
        None => {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

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

pub async fn game_detail(game_name: Path<String>) -> (StatusCode, Json<Option<Game>>) {
    let mut result = DATABASE
        .query(format!("SELECT * FROM game WHERE name = '{}'", game_name.to_string()))
        .query(format!("RETURN count(SELECT id FROM playing_in WHERE out.name = '{}')", game_name.to_string()))
        .await.unwrap();

    let game: Option<Game> = result.take(0).unwrap();
    let count: Option<u32> = result.take(1).unwrap();
    let mut game = game.unwrap();

    game.tribute_count = count.unwrap();

    // if let Some(game) = result {
        (StatusCode::OK, Json(Some(game)))
    // } else {
    //     (StatusCode::INTERNAL_SERVER_ERROR, Json::<Option<Game>>(None))
    // }
}

pub async fn game_tributes(Path(game_name): Path<String>) -> (StatusCode, Json<Vec<Tribute>>) {
    let record_id = RecordId::from(("game", game_name.to_string()));
    let tributes = DATABASE.query(
        format!("RETURN {}<-playing_in<-tribute.*", record_id)
    ).await.expect("No tributes");
    let mut tributes = tributes.check().expect("Failed to check tributes");
    let tributes = tributes.take(0);
    (StatusCode::OK, Json(tributes.expect("")))
}

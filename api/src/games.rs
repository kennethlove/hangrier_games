use crate::tributes::{tribute_create, tribute_record_create, tribute_delete, tribute_update};
use crate::DATABASE;
use axum::extract::Path;
use axum::http::header::{CACHE_CONTROL, EXPIRES};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{delete, get, post};
use axum::Json;
use axum::Router;
use game::games::Game;
use shared::EditGame;
use std::sync::LazyLock;
use serde::{Deserialize, Serialize};
use game::areas::Area;
use strum::IntoEnumIterator;
use surrealdb::RecordId;
use game::items::Item;
use std::str::FromStr;

pub static GAMES_ROUTER: LazyLock<Router> = LazyLock::new(|| {
    Router::new()
        .route("/", get(game_list).post(game_create))
        .route("/{game_identifier}", get(game_detail).delete(game_delete).put(game_update))
        .route("/{game_identifier}/tributes", post(tribute_create))
        .route("/{game_identifier}/tributes/{tribute_identifier}", delete(tribute_delete).put(tribute_update))
});

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GameItem {
    #[serde(rename="in")]
    game: RecordId,
    #[serde(rename="out")]
    item: RecordId,
    area: Area,
}

pub async fn give_game_items(game_identifier: String) -> Vec<GameItem> {
    let game_id = RecordId::from(("game", game_identifier));
    let mut output: Vec<GameItem> = Vec::new();

    for area in Area::iter() {
        for _ in 0..2 {
            let new_item: Item = Item::new_random(None);
            let new_item_id: RecordId = RecordId::from(("item", &new_item.identifier));
            let _: Option<Item> = DATABASE.insert(new_item_id.clone()).content(new_item.clone()).await.expect("Failed to insert Item");
            let game_record: Vec<GameItem> = DATABASE.insert("items").relation(
                GameItem {
                    game: game_id.clone(),
                    item: new_item_id.clone(),
                    area: area.clone(),
                }
            ).await.expect("Failed to update Items relation");
            output.push(game_record.first().unwrap().clone());
        }
    }

    output
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GameItemDetail {
    area: String,
    item: Item
}

pub async fn game_create(Json(payload): Json<Game>) -> impl IntoResponse {
    let game: Option<Game> = DATABASE
        .create(("game", &payload.identifier))
        .content(payload.clone())
        .await.expect("failed to create game");
    let mut game = game.unwrap();

    for _ in 0..24 {
        tribute_record_create(None, payload.clone().identifier).await;
    }

    give_game_items(payload.clone().identifier).await;

    // for area in Area::iter() {
    //     DATABASE.insert("items")).relation(
    //         GameItem {
    //             game: RecordId::from(("game", payload.identifier.clone())),
    //             item: RecordId::from()
    //         }
    //     )
    // }

    (StatusCode::OK, Json::<Game>(game.clone()))
}

pub async fn game_delete(game_identifier: Path<String>) -> StatusCode {
    let game: Option<Game> = DATABASE.delete(("game", &game_identifier.0)).await.expect("Failed to delete game");
    match game {
        Some(_) => StatusCode::NO_CONTENT,
        None => {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub async fn game_list() -> impl IntoResponse {
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

pub async fn game_detail(game_identifier: Path<String>) -> (StatusCode, Json<Option<Game>>) {
    let identifier = game_identifier.0;
    let mut result = DATABASE.query(format!(r#"
        SELECT *, (
            SELECT *, ->owns->item.* AS items
            FROM tribute
            WHERE identifier INSIDE (
                SELECT <-playing_in<-tribute.identifier
                AS identifiers
                FROM game
                WHERE identifier = "{identifier}"
            )[0]["identifiers"]
            ORDER district
        ) AS tributes, (
            RETURN count(
                SELECT id
                FROM playing_in
                WHERE out.identifier = "{identifier}"
            )
        ) AS tribute_count,
        (
            SELECT *
            FROM item
            WHERE identifier INSIDE (
                SELECT VALUE out.identifier as identifier
            FROM items
            WHERE in.identifier = "{identifier}")
        ) AS items
        FROM game
        WHERE identifier = "{identifier}"
    "#))
    .await.unwrap();
    
    let game: Option<Game> = result.take(0).expect("No game found");
    
    if let Some(game) = game {
        (StatusCode::OK, Json(Some(game)))
    } else {
        (StatusCode::NOT_FOUND, Json(None))
    }
}

pub async fn game_update(Path(_): Path<String>, Json(payload): Json<EditGame>) -> (StatusCode, Json<Option<Game>>) {
    let response = DATABASE.query(
        format!("UPDATE game SET name = '{}' WHERE identifier = '{}'", payload.1, payload.0)
    ).await;

    match response {
        Ok(mut response) => {
            let game: Option<Game> = response.take(0).unwrap();
            (StatusCode::OK, Json::<Option<Game>>(game))
        }
        Err(e) => {
            tracing::error!("{}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json::<Option<Game>>(None))
        }
    }

}

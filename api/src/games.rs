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
use surrealdb::{Error, RecordId, Response};
use game::items::Item;
use std::str::FromStr;
use shared::GameArea;
use uuid::Uuid;

pub static GAMES_ROUTER: LazyLock<Router> = LazyLock::new(|| {
    Router::new()
        .route("/", get(game_list).post(game_create))
        .route("/{game_identifier}", get(game_detail).delete(game_delete).put(game_update))
        .route("/{game_identifier}/tributes", post(tribute_create))
        .route("/{game_identifier}/tributes/{tribute_identifier}", delete(tribute_delete).put(tribute_update))
});

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GameAreaRecord {
    #[serde(rename="in")]
    game: RecordId,
    #[serde(rename="out")]
    area: RecordId,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AreaItem {
    #[serde(rename="in")]
    area: RecordId,
    #[serde(rename="out")]
    item: RecordId
}

async fn game_area_record_create(area: Area, game_identifier: String) -> Option<GameAreaRecord> {
    // Does the `area` exist for the game?
    let mut existing_area = DATABASE.query(
        format!(r#"
        SELECT identifier
        FROM area
        WHERE original_name = '{}'
        AND <-areas<-game.identifier = '{}'"#,
            area,
            game_identifier.clone()
        )
    ).await.expect("Failed to query game area");
    let existing_area: Option<String> = existing_area.take(0).unwrap(); // e.g. "Cornucopia"

    let game_id = RecordId::from(("game", game_identifier));

    if let Some(area_id) = existing_area {
        // if the `area` already exists
        // create the `areas` record
        DATABASE
            .insert::<Option<GameAreaRecord>>(RecordId::from(("areas", area_id.clone())))
            .relation([GameAreaRecord {
                game: game_id.clone(),
                area: RecordId::from_str(&area_id).unwrap(),
            }])
            .await.expect("Failed to link Area and Game")
    } else {
        // if the `area` doesn't exist
        let identifier = Uuid::new_v4().to_string();
        let area_id: RecordId = RecordId::from(("area", identifier.clone()));

        // create the `area` record
        match DATABASE
            .insert::<Option<GameArea>>(area_id.clone())
            .content(GameArea {
                identifier: identifier.clone(),
                name: area.to_string(),
                open: true,
                area: area.to_string()
            })
            .await {
                Ok(Some(_)) => {
                    // create the `areas` record
                    DATABASE
                        .insert::<Option<GameAreaRecord>>(RecordId::from(("areas", area_id.to_string())))
                        .relation([GameAreaRecord {
                            game: game_id.clone(),
                            area: area_id.clone(),
                        }]).await.expect("Failed to link Area and Game")
                }
                Err(e) => {
                    eprintln!("{:?}", e);
                    None
                }
                _ => None
        }
    }
}

pub async fn game_create(Json(payload): Json<Game>) -> impl IntoResponse {
    let game: Option<Game> = DATABASE
        .create(("game", &payload.identifier))
        .content(payload.clone())
        .await.expect("failed to create game");
    let game = game.unwrap();

    for _ in 0..24 {
        tribute_record_create(None, payload.clone().identifier).await;
    }

    for area in Area::iter() {
        let game_area = game_area_record_create(
            area.clone(), // Area to link to,
            payload.clone().identifier // Game to link to,
        ).await.expect("Failed to create game area");

        for _ in 0..2 {
            // Insert an item
            let new_item: Item = Item::new_random(None);
            let new_item_id: RecordId = RecordId::from(("item", &new_item.identifier));
            DATABASE
                .insert::<Option<Item>>(new_item_id.clone())
                .content(new_item.clone())
                .await.expect("failed to insert item");

            // Insert an area-item relationship
            let area_item: AreaItem = AreaItem {
                area: game_area.area.clone(),
                item: new_item_id.clone()
            };
            DATABASE
                .insert::<Option<AreaItem>>(RecordId::from_str("items").unwrap())
                .relation(area_item)
                .await.expect("");
        }
    }

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
    let games = DATABASE.select("game").await;
    
    match games {
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
            FROM area
            WHERE identifier INSIDE (
                SELECT VALUE out.identifier as identifier
                FROM areas
                WHERE in.identifier = "{identifier}"
            )
        ) as areas
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

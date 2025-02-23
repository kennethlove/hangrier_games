use crate::tributes::{tribute_create, tribute_delete, tribute_record_create, tribute_update};
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
use game::items::Item;
use game::tributes::Tribute;
use serde::{Deserialize, Serialize};
use shared::EditGame;
use shared::GameArea;
use std::str::FromStr;
use std::sync::LazyLock;
use strum::IntoEnumIterator;
use surrealdb::{Error, RecordId, Response};
use uuid::Uuid;

pub static GAMES_ROUTER: LazyLock<Router> = LazyLock::new(|| {
    Router::new()
        .route("/", get(game_list).post(game_create))
        .route("/{game_identifier}", get(game_detail).delete(game_delete).put(game_update))
        .route("/{game_identifier}/areas", get(game_areas))
        .route("/{game_identifier}/tributes", get(game_tributes).post(tribute_create))
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

async fn game_area_create(area: Area) -> Option<GameArea> {
    let identifier = Uuid::new_v4().to_string();
    let area_id: RecordId = RecordId::from(("area", identifier.clone()));

    // create the `area` record
    DATABASE
        .insert::<Option<GameArea>>(area_id.clone())
        .content(GameArea {
            identifier: identifier.clone(),
            name: area.to_string(),
            open: true,
            area: area.to_string(),
        })
        .await.expect("Failed to find Area and Game link")
}

async fn game_area_record_create(identifier: String, game_id: RecordId) -> Option<Vec<GameAreaRecord>> {
    DATABASE
        .insert::<Option<Vec<GameAreaRecord>>>(
            RecordId::from(("areas", identifier.clone()))
        ).relation(
        GameAreaRecord {
            game: game_id.clone(),
            area: RecordId::from(("area", &identifier)),
        }
    ).await.expect("Failed to link Area and Game")
}

async fn game_area_record_creator(area: Area, game_identifier: String) -> Option<GameAreaRecord> {
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
    let existing_area: Option<String> = existing_area.take(0).unwrap(); // e.g. a UUID

    let game_id = RecordId::from(("game", game_identifier));
    let gar: Option<Vec<GameAreaRecord>>;

    if let Some(identifier) = existing_area {
        // The `area` exists, create the `areas` connection
        gar = game_area_record_create(identifier, game_id).await;
    } else {
        // The `area` does not exist, create the `area`
        if let Some(area) = game_area_create(area).await {
            // Then create the `areas` connection
            let identifier = area.identifier.clone();
            gar = game_area_record_create(identifier, game_id).await;
        } else { return None; }
    }
    gar.expect("Failed to link area and game.").clone().pop()
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
        let game_area = game_area_record_creator(
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
                item: new_item_id.clone(),
            };
            DATABASE
                .insert::<Vec<AreaItem>>("items")
                .relation([area_item])
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
    // let games = DATABASE.select("game").await;
    let mut games = DATABASE.query(r#"
SELECT *, (
    SELECT *, ->owns->item[*] AS items
    FROM <-playing_in<-tribute[*]
) AS tributes, (
    SELECT *, ->items->item[*] AS items
    FROM ->areas->area
) AS areas, (
    RETURN count(
        SELECT id FROM <-playing_in<-tribute
    )
) AS tribute_count,
count(<-playing_in<-tribute.id) == 24
AND
count(array::distinct(<-playing_in<-tribute.district)) == 12
AS ready
FROM game;"#).await.unwrap();

    match games.take::<Vec<Game>>(0) {
        Ok(games) => {
            (StatusCode::OK, Json::<Vec<Game>>(games)).into_response()
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
    SELECT *, ->owns->item[*]
    AS items
    FROM <-playing_in<-tribute[*]
    ORDER district
)
AS tributes, (
    SELECT *, ->items->item[*] AS items
    FROM ->areas->area
) AS areas, (
    RETURN count(SELECT id FROM <-playing_in<-tribute)
) AS tribute_count,
count(<-playing_in<-tribute.id) == 24
AND
count(array::distinct(<-playing_in<-tribute.district)) == 12
AS ready
FROM game
WHERE identifier = "{identifier}";"#))
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

pub async fn game_areas(Path(identifier): Path<String>) -> (StatusCode, Json<Vec<AreaDetails>>) {
    let response = DATABASE.query(
        format!(r#"
SELECT (
    SELECT *, ->items->item[*] AS items
    FROM ->areas->area
) AS areas FROM game WHERE identifier = "{identifier}";
"#)).await;

    match response {
        Ok(mut response) => {
            let areas: Vec<Vec<AreaDetails>> = response.take("areas").unwrap();
            (StatusCode::OK, Json::<Vec<AreaDetails>>(areas[0].clone()))
        }
        Err(e) => {
            tracing::error!("{}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![]))
        }
    }
}

pub async fn game_tributes(Path(identifier): Path<String>) -> (StatusCode, Json<Vec<Tribute>>) {
    let response = DATABASE.query(
        format!(r#"
SELECT (
    SELECT *, ->owns->item[*] AS items
    FROM <-playing_in<-tribute
    ORDER district
) AS tributes FROM game WHERE identifier = "{identifier}";
"#)).await;

    match response {
        Ok(mut response) => {
            let tributes: Vec<Vec<Tribute>> = response.take("tributes").unwrap();
            (StatusCode::OK, Json::<Vec<Tribute>>(tributes[0].clone()))
        }
        Err(e) => {
            tracing::error!("{}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![]))
        }
    }
}

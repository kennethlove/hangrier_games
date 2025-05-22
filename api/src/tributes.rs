use crate::games::game_tributes;
use crate::{AppError, AppState};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use game::items::Item;
use game::messages::GameMessage;
use game::tributes::Tribute;
use serde::{Deserialize, Serialize};
use shared::EditTribute;
use std::sync::LazyLock;
use surrealdb::engine::any::Any;
use surrealdb::RecordId;
use surrealdb::Surreal;
use uuid::Uuid;

pub static TRIBUTES_ROUTER: LazyLock<Router<AppState>> = LazyLock::new(|| {
    Router::new()
        .route("/", get(game_tributes))
        .route("/{identifier}", get(tribute_detail).delete(tribute_delete).put(tribute_update))
        .route("/{identifier}/log", get(tribute_log))
});

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TributeAreaEdge {
    #[serde(rename = "in")]
    pub tribute: RecordId,
    #[serde(rename = "out")]
    pub item: RecordId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TributeGameEdge {
    #[serde(rename = "in")]
    tribute: RecordId,
    #[serde(rename = "out")]
    game: RecordId,
}

pub async fn create_tribute(tribute: Option<Tribute>, game_identifier: &String, db: &Surreal<Any>, district: u32) -> Result<Tribute, AppError> {
    let game_id = RecordId::from(("game", game_identifier.clone()));
    let tribute_count = db
        .query("RETURN count(SELECT id FROM playing_in WHERE out.identifier=$game)")
        .bind(("game", game_identifier.clone()))
        .await;
    let tribute_count: Option<u32> = tribute_count.unwrap().take(0).unwrap();
    if tribute_count >= Some(24) {
        return Err(AppError::GameFull("Game is full".to_string()));
    }

    let mut tribute = tribute.unwrap_or_else(Tribute::random);
    tribute.district = district + 1;
    tribute.statistics.game = game_identifier.clone();

    let id = RecordId::from(("tribute", &tribute.identifier));

    let new_tribute: Option<Tribute> = db
        .create(&id)
        .content(tribute)
        .await.expect("Failed to create Tribute record");

    let _: Vec<TributeGameEdge> = db.insert("playing_in").relation(
        TributeGameEdge {
            tribute: id.clone(),
            game: game_id.clone(),
        }
    ).await.expect("Failed to connect Tribute to game");

    let new_object: Item = Item::new_random(None);
    let new_object_id: RecordId = RecordId::from(("item", &new_object.identifier));
    let _: Option<Item> = db.insert(new_object_id.clone()).content(new_object.clone()).await.expect("Failed to update Item");
    let _: Vec<TributeAreaEdge> = db.insert("owns").relation(
        TributeAreaEdge {
            tribute: id.clone(),
            item: new_object_id.clone(),
        }
    ).await.expect("Failed to update Owns relation");

    if let Some(tribute) = new_tribute {
        Ok(tribute)
    } else {
        Err(AppError::InternalServerError("Failed to create tribute".to_string()))
    }
}

pub async fn tribute_delete(Path((_, tribute_identifier)): Path<(String, String)>, state: State<AppState>) -> Result<StatusCode, AppError> {
    let tribute: Option<Tribute> = state.db.delete(("tribute", &tribute_identifier)).await.expect("failed to delete tribute");
    match tribute {
        Some(_) => Ok(StatusCode::NO_CONTENT),
        None => {
            Err(AppError::InternalServerError("Could not delete tribute".into()))
        }
    }
}

pub async fn tribute_update(
    Path((_game_identifier, _tribute_identifier)): Path<(Uuid, Uuid)>,
    state: State<AppState>,
    Json(payload): Json<EditTribute>,
) -> Result<StatusCode, AppError> {
    let response = state.db
        .query("UPDATE tribute SET name = $name, district = $district WHERE identifier = $identifier;")
        .bind(("identifier", payload.0))
        .bind(("district", payload.1))
        .bind(("name", payload.2))
        .await;

    match response {
        Ok(mut response) => {
            match response.take::<Option<Tribute>>(0).unwrap() {
                Some(_tribute) => {
                    Ok(StatusCode::OK)
                }
                None => Err(AppError::InternalServerError("Failed to update tribute".into()))
            }
        }
        Err(_) => {
            Err(AppError::InternalServerError("Failed to update tribute".into()))
        }
    }
}

pub async fn tribute_detail(Path((_, tribute_identifier)): Path<(Uuid, Uuid)>, state: State<AppState>) -> Result<Json<Tribute>, AppError> {
    let tribute_identifier = tribute_identifier.to_string();
    let mut result = state.db
        .query(r#"
        SELECT *, ->owns->item[*] AS items,
        (SELECT * FROM fn::get_messages_by_tribute_id($identifier)) AS log,
        (->playing_in->game.status)[0] == "NotStarted" AS editable
        FROM tribute
        WHERE identifier = $identifier
        "#)
        .bind(("identifier", tribute_identifier))
        .await.expect("Failed to find tribute");

    let tribute: Option<Tribute> = result.take(0).expect("");

    if let Some(tribute) = tribute {
        Ok(Json(tribute.clone()))
    } else {
        Err(AppError::NotFound("Tribute not found".to_string()))
    }
}

pub async fn tribute_log(Path((_, identifier)): Path<(Uuid, Uuid)>, state: State<AppState>) -> Result<Json<Vec<GameMessage>>, AppError> {
    let identifier = identifier.to_string();
    let mut result = state.db
        .query("SELECT * FROM fn::get_messages_by_tribute_id($identifier)")
        .bind(("identifier", identifier))
        .await.expect("Failed to find log");

    let logs: Vec<GameMessage> = result.take(0).unwrap();
    Ok(Json(logs))
}

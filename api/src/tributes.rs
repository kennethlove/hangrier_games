use crate::DATABASE;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use game::items::Item;
use game::messages::GameMessage;
use game::tributes::Tribute;
use serde::{Deserialize, Serialize};
use serde_json::json;
use shared::EditTribute;
use std::sync::LazyLock;
use surrealdb::RecordId;


pub static TRIBUTES_ROUTER: LazyLock<Router> = LazyLock::new(|| {
    Router::new()
        .route("/{identifier}/log", get(tribute_log))
});

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TributeOwns {
    #[serde(rename = "in")]
    pub tribute: RecordId,
    #[serde(rename = "out")]
    pub item: RecordId,
}

pub async fn tribute_record_create(tribute: Option<Tribute>, game_identifier: String) -> Option<Tribute> {
    let game_id = RecordId::from(("game", game_identifier.clone()));
    let tribute_count = DATABASE
        .query("RETURN count(SELECT id FROM playing_in WHERE out.identifier=$game)")
        .bind(("game", game_identifier.clone()))
        .await;
    let tribute_count: Option<u32> = tribute_count.unwrap().take(0).unwrap();
    if tribute_count >= Some(24) {
        return None;
    }

    let mut tribute = tribute.unwrap_or_else(Tribute::random);
    tribute.district = (tribute_count.unwrap_or(1) % 12) + 1;
    tribute.statistics.game = game_identifier;

    let id = RecordId::from(("tribute", &tribute.identifier));

    let new_tribute: Option<Tribute> = DATABASE
        .create(&id)
        .content(tribute)
        .await.expect("Failed to create Tribute record");

    let _: Vec<TributePlaysIn> = DATABASE.insert("playing_in").relation(
        TributePlaysIn {
            tribute: id.clone(),
            game: game_id.clone(),
        }
    ).await.expect("Failed to connect Tribute to game");

    let new_object: Item = Item::new_random(None);
    let new_object_id: RecordId = RecordId::from(("item", &new_object.identifier));
    let _: Option<Item> = DATABASE.insert(new_object_id.clone()).content(new_object.clone()).await.expect("Failed to update Item");
    let _: Vec<TributeOwns> = DATABASE.insert("owns").relation(
        TributeOwns {
            tribute: id.clone(),
            item: new_object_id.clone(),
        }
    ).await.expect("Failed to update Owns relation");

    new_tribute
}

pub async fn tribute_create(Path(game_identifier): Path<String>, Json(payload): Json<Tribute>) -> impl IntoResponse {
    let tribute: Option<Tribute> = tribute_record_create(Some(payload), game_identifier).await;
    if tribute.is_none() {
        (StatusCode::BAD_REQUEST, Json(json!({}))).into_response()
    } else {
        (StatusCode::OK, Json::<Tribute>(tribute.clone().unwrap())).into_response()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TributePlaysIn {
    #[serde(rename = "in")]
    tribute: RecordId,
    #[serde(rename = "out")]
    game: RecordId,
}

pub async fn tribute_delete(Path((_, tribute_identifier)): Path<(String, String)>) -> StatusCode {
    let tribute: Option<Tribute> = DATABASE.delete(("tribute", &tribute_identifier)).await.expect("failed to delete tribute");
    match tribute {
        Some(_) => StatusCode::NO_CONTENT,
        None => {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub async fn tribute_update(
    Path((_game_identifier, _tribute_identifier)): Path<(String, String)>,
    Json(payload): Json<EditTribute>
) -> impl IntoResponse {
    tracing::debug!("Payload: {:?}", &payload);
    let response = DATABASE
        .query("UPDATE tribute SET name = $name, district = $district WHERE identifier = $identifier;")
        .bind(("identifier", payload.0))
        .bind(("district", payload.1))
        .bind(("name", payload.2))
        .await;

    match response {
        Ok(mut response) => {
            match response.take::<Option<Tribute>>(0).unwrap() {
                Some(tribute) => {
                    tracing::debug!("Tribute update: {:?}", &tribute);
                    Box::new(StatusCode::OK).into_response()
                }
                None => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update tribute").into_response()
            }
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

pub async fn tribute_detail(Path((_game_identifier, tribute_identifier)): Path<(String, String)>) -> (StatusCode, Json<Option<Tribute>>) {
    let mut result = DATABASE
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
        (StatusCode::OK, Json::<Option<Tribute>>(Some(tribute)))
    } else {
        (StatusCode::NOT_FOUND, Json(None))
    }
}

pub async fn tribute_log(Path(identifier): Path<String>) -> (StatusCode, Json<Vec<GameMessage>>) {
    let mut result = DATABASE
        .query(r#"SELECT * FROM fn::get_messages_by_tribute_id("$identifier")"#)
        .bind(("identifier", identifier))
        .await.expect("Failed to find log");

    let logs: Vec<GameMessage> = result.take(0).unwrap();
    if logs.is_empty() {
        (StatusCode::NOT_FOUND, Json(vec![]))
    } else {
        (StatusCode::OK, Json(logs))
    }
}

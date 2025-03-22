use crate::DATABASE;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use game::items::Item;
use game::tributes::Tribute;
use serde::{Deserialize, Serialize};
use serde_json::json;
use shared::EditTribute;
use surrealdb::RecordId;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TributeOwns {
    #[serde(rename = "in")]
    pub tribute: RecordId,
    #[serde(rename = "out")]
    pub item: RecordId,
}

pub async fn tribute_record_create(tribute: Option<Tribute>, game_identifier: String) -> Option<Tribute> {
    let game_id = RecordId::from(("game", game_identifier.clone()));
    let tribute_count = DATABASE.query(
        format!("RETURN count(SELECT id FROM playing_in WHERE out.identifier='{}')", game_identifier.clone())
    ).await;
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

pub async fn tribute_update(Path((_game_identifier, _tribute_identifier)): Path<(String, String)>, Json(payload): Json<EditTribute>) -> impl IntoResponse {
    let response = DATABASE.query(
        format!("UPDATE tribute SET name = '{}', district = {} WHERE identifier = '{}'", payload.2, payload.1, payload.0)
    ).await;

    match response {
        Ok(mut response) => {
            let tribute: Option<Tribute> = response.take(0).unwrap();
            (StatusCode::OK, Json::<Tribute>(tribute.expect("No tribute updated."))).into_response()
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json::<String>(e.to_string())).into_response()
        }
    }
}

pub async fn tribute_detail(Path((_game_identifier, tribute_identifier)): Path<(String, String)>) -> (StatusCode, Json<Option<Tribute>>) {
    let mut result = DATABASE.query(format!(r#"
SELECT *, ->owns->item[*] AS items,
(SELECT * FROM tribute_log WHERE tribute_identifier = "{tribute_identifier}" ORDER BY day) AS log
FROM tribute
WHERE identifier = "{tribute_identifier}"
"#)).await.expect("Failed to find tribute");

    let tribute: Option<Tribute> = result.take(0).expect("");

    if let Some(tribute) = tribute {
        (StatusCode::OK, Json::<Option<Tribute>>(Some(tribute)))
    } else {
        (StatusCode::NOT_FOUND, Json(None))
    }
}

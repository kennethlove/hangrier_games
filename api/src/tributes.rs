use crate::DATABASE;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use game::tributes::Tribute;
use serde::{Deserialize, Serialize};
use serde_json::json;
use shared::EditTribute;
use surrealdb::RecordId;

pub async fn create_tribute_record(tribute: Option<Tribute>, game_name: String) -> Option<Tribute> {
    let game_id = RecordId::from(("game", game_name.clone()));
    let mut tribute_count = DATABASE.query(
        format!("RETURN count(SELECT id FROM playing_in WHERE out.name='{}')", game_name.clone())
    ).await;
    let tribute_count: Option<u32> = tribute_count.unwrap().take(0).unwrap();

    if tribute_count >= Some(24) {
        return None;
    }

    let mut tribute = tribute.unwrap_or_else(|| Tribute::random());
    tribute.district = (tribute_count.unwrap_or(1) % 12) + 1;

    let id = RecordId::from(("tribute", &tribute.identifier));

    let new_tribute: Option<Tribute> = DATABASE
        .create(&id)
        .content(tribute)
        .await.expect("Failed to create Tribute record");

    let _: Vec<TributePlaysIn> = DATABASE.insert("playing_in").relation(
        TributePlaysIn {
            tribute: id,
            game: game_id.clone(),
        }
    ).await.expect("Failed to connect Tribute to game");

    new_tribute
}

pub async fn create_tribute(Path(game_name): Path<String>, Json(payload): Json<Tribute>) -> impl IntoResponse {
    let tribute: Option<Tribute> = create_tribute_record(Some(payload), game_name).await;
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

pub async fn tribute_detail(Path(name): Path<&str>) -> impl IntoResponse {

}


pub async fn delete_tribute(Path((game_name, tribute_name)): Path<(String, String)>) -> StatusCode {
    let tribute: Option<Tribute> = DATABASE.delete(("tribute", &tribute_name)).await.expect("failed to delete tribute");
    match tribute {
        Some(_) => StatusCode::NO_CONTENT,
        None => {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub async fn update_tribute(Path((game_name, tribute_name)): Path<(String, String)>, Json(payload): Json<EditTribute>) -> impl IntoResponse {
    // dbg!(&payload);
    let game_id = RecordId::from(("game", game_name.clone()));
    let tribute_id = RecordId::from(("tribute", payload.clone().2.clone()));

    let response: Option<Tribute> = DATABASE.update(tribute_id).merge(payload).await.expect("failed to update Tribute");

    if response.is_none() {
        (StatusCode::BAD_REQUEST, Json(json!({}))).into_response()
    } else {
        (StatusCode::OK, Json::<Tribute>(response.clone().unwrap())).into_response()
    }
}

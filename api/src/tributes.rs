use crate::DATABASE;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use game::tributes::Tribute;
use serde::{Deserialize, Serialize};
use surrealdb::RecordId;
use game::games::Game;

pub async fn create_tribute(Path(game_name): Path<String>, Json(payload): Json<Tribute>) -> impl IntoResponse {
    let tribute: Option<Tribute> = DATABASE
        .create(("tribute", &payload.name.to_string()))
        .content(payload.clone())
        .await.expect("failed to create tribute");


    let _: Vec<TributePlaysIn> = DATABASE.insert("playing_in")
        .relation(
            TributePlaysIn {
                tribute: RecordId::from(("tribute", &payload.name.to_string())),
                game: RecordId::from(("game", game_name.to_string())),
            }
        ).await.expect("");

    (StatusCode::OK, Json::<Tribute>(tribute.clone().unwrap()))
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

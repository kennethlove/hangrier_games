use crate::DATABASE;
use axum::extract::Path;
use axum::http::header::{CACHE_CONTROL, EXPIRES};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Json;
use axum::Router;
use game::areas::{Area, AreaDetails};
use game::games::{Game, GAME};
use serde::{Deserialize, Serialize};
use shared::CreateGame;
use std::sync::LazyLock;
use strum::IntoEnumIterator;
use surrealdb::engine::any::Any;
use surrealdb::method::Query;
use surrealdb::RecordId;
use surrealdb::sql::Value;
use game::tributes::Tribute;
use crate::games::{game_delete, game_detail, games_create, games_list};

pub static TRIBUTES_ROUTER: LazyLock<Router> = LazyLock::new(|| {
    Router::new()
        .route("/", post(create_tribute))
});

pub async fn create_tribute(Json(payload): Json<Tribute>) -> impl IntoResponse {
    let tribute: Option<Tribute> = DATABASE
        .create(("tribute", &payload.name.to_string()))
        .content(payload.clone())
        .await.expect("failed to create tribute");

    let game = GAME.with(|g| g.clone());
    dbg!(game);

    let _: Vec<TributePlaysIn> = DATABASE.insert("playing_in")
        .relation(
            TributePlaysIn {
                tribute: RecordId::from(("tribute", &payload.name.to_string())),
                game: RecordId::from(("game", GAME.with_borrow(|g| g.name.clone())))
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

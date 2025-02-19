use crate::tributes::{tribute_create, tribute_record_create, tribute_delete, tribute_update};
use crate::DATABASE;
use axum::extract::Path;
use axum::http::header::{CACHE_CONTROL, EXPIRES};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{delete, get};
use axum::Json;
use axum::Router;
use game::games::Game;
use game::tributes::Tribute;
use shared::EditGame;
use std::sync::LazyLock;

pub static GAMES_ROUTER: LazyLock<Router> = LazyLock::new(|| {
    Router::new()
        .route("/", get(game_list).post(game_create))
        .route("/{game_identifier}", get(game_detail).delete(game_delete).put(game_update))
        .route("/{game_identifier}/tributes", get(game_tributes).post(tribute_create))
        .route("/{game_identifier}/tributes/{tribute_identifier}", delete(tribute_delete).put(tribute_update))
});

pub async fn game_create(Json(payload): Json<Game>) -> impl IntoResponse {
    let game: Option<Game> = DATABASE
        .create(("game", &payload.identifier))
        .content(payload)
        .await.expect("failed to create game");

    for _ in 0..24 {
        tribute_record_create(None, game.clone().unwrap().name).await;
    }

    (StatusCode::OK, Json::<Game>(game.clone().unwrap()))
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
            SELECT * FROM tribute WHERE identifier INSIDE (
                SELECT <-playing_in<-tribute.identifier
                AS identifiers
                FROM game
                WHERE identifier = "{identifier}"
            )[0]["identifiers"]
            ORDER district
        ) AS tributes, (
            RETURN count(
                SELECT id FROM playing_in
                WHERE out.identifier = "{identifier}"
            )
        ) AS tribute_count
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

pub async fn game_tributes(Path(game_identifier): Path<String>) -> (StatusCode, Json<Vec<Tribute>>) {
    let tributes = DATABASE.query(
        format!(r#"SELECT * FROM tribute WHERE identifier IN (
            SELECT <-playing_in<-tribute.identifier AS identifiers FROM game WHERE identifier = "{}"
         )[0]["identifiers"]"#, game_identifier),
    ).await.expect("No tributes");

    match tributes.check() {
        Ok(mut tributes) => {
            let mut tributes: Vec<Tribute> = tributes.take(0).unwrap_or_default();
            tributes.sort_by_key(|t| t.district);
            (StatusCode::OK, Json(tributes.clone()))
        }
        Err(e) => {
            tracing::error!("{}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json::<Vec<Tribute>>(Vec::new()))
        }
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

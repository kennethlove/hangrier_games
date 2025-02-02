use crate::DATABASE;
use axum::extract::Path;
use axum::http::header::{CACHE_CONTROL, EXPIRES};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Json;
use axum::Router;
use game::areas::Area;
use game::games::Game;
use serde::{Deserialize, Serialize};
use shared::CreateGame;
use std::sync::LazyLock;
use surrealdb::method::Select;

pub static GAMES_ROUTER: LazyLock<Router> = LazyLock::new(|| {
    Router::new()
        .route("/", get(games_list).post(games_create))
        .route("/{name}", get(game_detail).delete(game_delete))
});

#[derive(Deserialize, Serialize)]
struct GameArea {
    game: Game,
    area: Area,
    open: bool,
}

pub async fn games_list() -> impl IntoResponse {
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

pub async fn games_create(Json(payload): Json<CreateGame>) -> impl IntoResponse {
    let mut new_game = Game::default();
    if payload.name.is_some() {
        new_game.name = payload.name.unwrap();
    }

    for area in &["the_cornucopia", "northwest", "northeast", "southeast", "southwest"] {
        let db_area: Option<Area> = DATABASE.select(("area", area.clone()))
            .await.expect(&format!("Couldn't find area: {}", &area));

        let _: Option<GameArea> = DATABASE.create("game_area").content(
            GameArea {
                game: new_game.clone(),
                area: db_area.expect(format!("Missing db_area: {}", area).as_str()),
                open: true,
            }
        ).await.expect(&format!("Couldn't create game area: {}", area.clone()));
    }

    match DATABASE.create(("game", &new_game.name)).content(new_game).await {
        Ok(Some(game)) => (StatusCode::OK, Json::<Game>(game)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        Ok(None) => (StatusCode::ACCEPTED, "".to_string()).into_response(),
    }
}

pub async fn game_detail(name: Path<String>) -> (StatusCode, Json<Option<Game>>) {
    match DATABASE.select(("game", &name.0)).await {
        Ok(Some(game)) => {
            (StatusCode::OK, Json::<Option<Game>>(Some(game)))
        }
        Err(e) => {
            tracing::error!("{}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json::<Option<Game>>(None))
        }
        _ => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json::<Option<Game>>(None))
        }
    }
}

pub async fn game_delete(name: Path<String>) -> StatusCode {
    let game: Option<Game> = DATABASE.delete(("game", &name.0)).await.expect("Failed to delete game");
    match game {
        Some(_) => StatusCode::NO_CONTENT,
        None => {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

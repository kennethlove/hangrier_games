use crate::tributes::{tribute_create, tribute_delete, tribute_detail, tribute_record_create, tribute_update, TributeOwns};
use crate::DATABASE;
use announcers::{summarize, summarize_stream};
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Sse};
use axum::routing::{get, put};
use axum::{BoxError, Json};
use axum::Router;
use game::areas::{Area, AreaDetails};
use game::games::{Game, GameStatus};
use game::items::Item;
use game::messages::{get_all_messages, GameMessage};
use game::tributes::Tribute;
use serde::{Deserialize, Serialize};
use shared::EditGame;
use shared::GameArea;
use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt::Result;
use std::str::FromStr;
use std::sync::LazyLock;
use axum::response::sse::Event;
use strum::IntoEnumIterator;
use surrealdb::sql::Thing;
use surrealdb::RecordId;
use uuid::Uuid;
use futures::stream::BoxStream;
use futures::StreamExt;

pub static GAMES_ROUTER: LazyLock<Router> = LazyLock::new(|| {
    Router::new()
        .route("/", get(game_list).post(game_create))
        .route("/{game_identifier}", get(game_detail).delete(game_delete).put(game_update))
        .route("/{game_identifier}/areas", get(game_areas))
        .route("/{game_identifier}/summarize/{day}", get(game_day_summary))
        .route("/{game_identifier}/summarize", get(game_summary))
        .route("/{game_identifier}/log/{day}", get(game_logs))
        .route("/{game_identifier}/log/{day}/{tribute_identifier}", get(tribute_logs))
        .route("/{game_identifier}/next", put(next_step))
        .route("/{game_identifier}/tributes", get(game_tributes).post(tribute_create))
        .route("/{game_identifier}/tributes/{tribute_identifier}", get(tribute_detail).delete(tribute_delete).put(tribute_update))
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
    let game_identifier = game_identifier.to_string().clone();
    let mut result = DATABASE.query(format!(r#"
    SELECT <-playing_in<-tribute as tribute,
           <-playing_in<-tribute->owns->item as item,
           <-playing_in<-tribute->owns as owns
    FROM game WHERE identifier = "{game_identifier}";

    SELECT ->areas->area AS area,
           ->areas->area->items->item AS item,
           ->areas->area->items as items,
           ->areas AS areas
    FROM game WHERE identifier = "{game_identifier}";
    "#)).await.expect("Failed to find game pieces");

    let game_pieces: Option<HashMap<String, Vec<Thing>>> = result.take(0).unwrap();
    let area_pieces: Option<HashMap<String, Vec<Thing>>> = result.take(1).unwrap();
    if game_pieces.is_some() { delete_pieces(game_pieces.unwrap()).await };
    if area_pieces.is_some() { delete_pieces(area_pieces.unwrap()).await };

    let game: Option<Game> = DATABASE.delete(("game", &game_identifier)).await.expect("Failed to delete game");
    match game {
        Some(_) => StatusCode::NO_CONTENT,
        None => {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

async fn delete_pieces(pieces: HashMap<String, Vec<Thing>>) {
    for (table, ids) in pieces {
        let _ = DATABASE.query(
            format!("DELETE {table} WHERE id IN [{}]",
                    ids.iter().map(|i| format!(r#"{table}:{}"#, i.id))
                        .collect::<Vec<String>>().join(","))
        ).await.unwrap_or_else(|_| panic!("Failed to delete {} pieces.", table));
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
    let day = DATABASE.query(format!("SELECT day FROM game WHERE identifier = '{identifier}' LIMIT 1")).await;
    let day: Option<i64> = day.unwrap().take("day").unwrap();
    let day: i64 = day.unwrap_or(0);

    let mut result = DATABASE.query(format!(r#"
SELECT *, (
    SELECT *,
    ->owns->item[*] AS items, (
        SELECT *
        FROM tribute_log
        WHERE tribute_identifier = $parent.identifier
        AND day = {day}
        ORDER BY instant
    ) AS log
    FROM <-playing_in<-tribute[*]
    ORDER district
) AS tributes, (
    SELECT *, ->items->item[*] AS items
    FROM ->areas->area
) AS areas, (
    RETURN count(SELECT id FROM <-playing_in<-tribute)
) AS tribute_count,
count(<-playing_in<-tribute.id) == 24 AND count(array::distinct(<-playing_in<-tribute.district)) == 12 AS ready
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
    let mut game_day = DATABASE.query(format!("SELECT day FROM game WHERE identifier = '{identifier}'")).await.unwrap();
    let game_day: Option<i64> = game_day.take("day").unwrap();
    let _game_day: i64 = game_day.unwrap_or(0);


    let response = DATABASE.query(
        format!(r#"
SELECT (
    SELECT *, ->owns->item[*] as items
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

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct GameResponse {
    game: Option<Game>,
}

pub async fn next_step(Path(identifier): Path<String>) -> impl IntoResponse {
    let record_id = RecordId::from(("game", identifier.clone()));
    let mut result = DATABASE.query(format!(r#"
SELECT status FROM game WHERE identifier = "{identifier}";
RETURN count(
    SELECT in.identifier
    FROM playing_in
    WHERE out.identifier = "{identifier}"
    AND in.status IN ["RecentlyDead", "Dead"]
);"#)).await.expect("No game found");
    let status: Option<String> = result.take("status").expect("No game found");
    let dead_tribute_count: Option<u32> = result.take(1).unwrap_or(Some(0));

    if let Some(status) = status {
        let status = GameStatus::from_str(status.as_str()).expect("Invalid game status");
        match status {
            GameStatus::NotStarted => {
                DATABASE
                    .query(format!(r#"UPDATE {record_id} SET status = "{}""#, GameStatus::InProgress))
                    .await.expect("Failed to start game");
                (StatusCode::CREATED, Json(GameResponse::default())).into_response()
            }
            GameStatus::InProgress => {
                match dead_tribute_count {
                    Some(23) | Some(24) => {
                        DATABASE
                            .query(format!(r#"UPDATE {record_id} SET status = "{}""#, GameStatus::Finished))
                            .await.expect("Failed to end game");
                        (StatusCode::NO_CONTENT, Json(GameResponse::default())).into_response()
                    }
                    _ => {
                        if let Some(mut game) = get_full_game(&identifier).await {
                            // Run day
                            let mut game = game.run_day_night_cycle(true).await;
                            save_game(&game).await;

                            // Run night
                            let game = game.run_day_night_cycle(false).await;
                            save_game(&game).await;

                            (StatusCode::OK, Json(GameResponse { game: Some(game) })).into_response()
                        } else {
                            (StatusCode::NOT_FOUND, Json(GameResponse::default())).into_response()
                        }
                    }
                }
            }
            GameStatus::Finished => {
                (StatusCode::NO_CONTENT, Json(GameResponse::default())).into_response()
            }
        }
    } else { (StatusCode::NOT_FOUND, Json(GameResponse::default())).into_response() }
}

async fn get_full_game(identifier: &str) -> Option<Game> {
    let mut result = DATABASE.query(format!(r#"
SELECT *, (
    SELECT *, ->owns->item[*] AS items
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
    result.take(0).expect("No game found")

}

async fn save_game(game: &Game) -> Option<Game> {
    let game_identifier = RecordId::from(("game", game.identifier.clone()));
    let mut saved_game = game.clone();


    if let Ok(logs) = get_all_messages() {
        for mut log in logs {
            log.game_day = game.day.unwrap_or_default();
            DATABASE
                .upsert::<Option<GameMessage>>(("message", &log.identifier))
                .content(log.clone())
                .await
                .expect(&format!("Failed to save message: {:#?}", log));
        }
    }

    let areas = game.areas.clone();
    saved_game.areas = vec![];

    for mut area in areas {
        let id = RecordId::from(("area", area.identifier.clone()));
        let items = area.items.clone();
        area.items = vec![];

        let _ = save_items(items, id.clone()).await;

        DATABASE
            .update::<Option<AreaDetails>>(id)
            .content(area)
            .await.expect("Failed to update area items");
    }

    let tributes = game.tributes.clone();
    saved_game.tributes = vec![];

    for mut tribute in tributes {
        let id = RecordId::from(("tribute", tribute.identifier.clone()));
        let mut items = tribute.items.clone();
        if !tribute.is_alive() { items.clear(); }
        tribute.items = vec![];

        let _ = save_items(items, id.clone()).await;

        DATABASE
            .update::<Option<Tribute>>(id)
            .content(tribute)
            .await.expect("Failed to update area items");
    }

    DATABASE
        .update::<Option<Game>>(game_identifier.clone())
        .content(saved_game)
        .await.expect("Failed to update game")
}

async fn save_items(items: Vec<Item>, owner: RecordId) {
    let is_area = owner.table() == "area";
    let query: String = format!(
        "DELETE FROM {} WHERE in = {}",
        if is_area { "items" } else { "owns" },
        owner.clone()
    );

    let _ = DATABASE.query(query).await.expect("Failed to delete items");

    for item in items {
        let item_identifier = RecordId::from(("item", item.identifier.clone()));
        if item.quantity == 0 {
            DATABASE
                .delete::<Option<Item>>(item_identifier.clone())
                .await.expect("Failed to delete item");
            return;
        } else {
            DATABASE
                .update::<Option<Item>>(item_identifier.clone())
                .content(item.clone())
                .await.expect("Failed to update item");
        }

        if item.quantity > 0 {
            if is_area {
                let _: Vec<TributeOwns> = DATABASE.insert("items").relation(
                    AreaItem {
                        area: owner.clone(),
                        item: item_identifier.clone(),
                    }
                ).await.expect("Failed to update Items relation");
            } else {
                let _: Vec<TributeOwns> = DATABASE.insert("owns").relation(
                    TributeOwns {
                        tribute: owner.clone(),
                        item: item_identifier.clone(),
                    }
                ).await.expect("Failed to update Owns relation");
            }
        }
    }
}

async fn game_logs(Path((game_identifier, day)): Path<(String, String)>) -> Json<Vec<GameMessage>> {
    match DATABASE.query(format!(r#"SELECT * FROM message WHERE string::starts_with(subject, "{game_identifier}") AND game_day = {day} ORDER BY timestamp;"#)).await {
        Ok(mut logs) => {
            let logs: Vec<GameMessage> = logs.take(0).expect("logs is empty");
            Json(logs)
        }
        Err(err) => {
            eprintln!("{err:?}");
            Json(vec![])
        }
    }
}

async fn tribute_logs(Path((game_identifier, day, tribute_identifier)): Path<(String, String, String)>) -> Json<Vec<GameMessage>> {
    match DATABASE.query(format!(
        r#"SELECT *
        FROM message
        WHERE string::starts_with(subject, "{game_identifier}")
        AND game_day = {day}
        AND source.value = "{tribute_identifier}"
        ORDER BY timestamp;"#
    )).await {
        Ok(mut logs) => {
            let logs: Vec<GameMessage> = logs.take(0).expect("logs is empty");
            Json(logs)
        }
        Err(err) => {
            eprintln!("{err:?}");
            Json(vec![])
        }
    }
}

async fn game_day_summary(Path((game_identifier, day)): Path<(String, String)>) -> Json<String> {
    match DATABASE.query(
        format!(r#"
        SELECT *
        FROM message
        WHERE string::starts_with(subject, "{game_identifier}")
        AND game_day = {day}
        ORDER BY timestamp;
        "#)
    ).await {
        Ok(mut logs) => {
            let logs: Vec<GameMessage> = logs.take(0).expect("logs is empty");
            let logs = logs.into_iter().map(|l| l.content).collect::<Vec<String>>().join("\n");
            if !logs.is_empty() {
                let res = summarize(&logs).await;

                if let Ok(res) = res {
                    Json(res)
                } else {
                    Json(String::new())
                }
            } else {
                Json(String::new())
            }
        }
        Err(err) => {
            Json(String::new())
        }
    }
}

async fn game_summary(Path(game_identifier): Path<String>) -> impl IntoResponse {
    let result = DATABASE.query(
        format!(r#"
        SELECT *
        FROM message
        WHERE string::starts_with(subject, "{game_identifier}")
        ORDER BY timestamp;
        "#)
    ).await;

    let stream = match result {
        Ok(mut result) => {
            let logs: Vec<GameMessage> = match result.take(0) {
                Ok(logs) => logs,
                Err(e) => {
                    eprintln!("Error taking logs: {}", e);
                    return Sse::new(async_stream::stream! {
                        yield Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Error taking logs")) as BoxError);
                    }.boxed());
                }
            };

            let logs = logs.into_iter()
                .map(|l| l.content)
                .collect::<Vec<String>>()
                .join("\n");

            if !logs.is_empty() {
                let text_stream = summarize_stream(&logs).await;

                // Create a new stream that converts String results to Event objects
                let event_stream = async_stream::stream! {
                    let mut stream = text_stream;
                    while let Some(result) = stream.next().await {
                        match result {
                            Ok(text) => yield Ok(Event::default().data(text)),
                            Err(e) => yield Ok(Event::default().data(format!("Error: {}", e)))
                        }
                    }
                };

                event_stream.boxed()
            } else {
                async_stream::stream! {
                    yield Ok(Event::default().data("No logs found"));
                }.boxed()
            }
        }
        Err(e) => {
            eprintln!("Error querying logs: {}", e);
            async_stream::stream! {
                yield Ok(Event::default().data(format!("Error querying logs: {}", e)));
            }.boxed()
        }
    };

    Sse::new(stream)
}

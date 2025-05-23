use crate::tributes::{create_tribute, TributeAreaEdge, TRIBUTES_ROUTER};
use crate::{AppError, AppState};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, put};
use axum::Json;
use axum::Router;
use chrono::{DateTime, Utc};
use game::areas::{Area, AreaDetails};
use game::games::Game;
use game::items::Item;
use game::messages::{get_all_messages, GameMessage, MessageSource};
use game::tributes::Tribute;
use serde::{Deserialize, Serialize};
use shared::{DisplayGame, EditGame, GameStatus};
use shared::{GameArea, ListDisplayGame};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::LazyLock;
use strum::IntoEnumIterator;
use surrealdb::engine::any::Any;
use surrealdb::sql::Thing;
use surrealdb::{RecordId, Surreal};
use uuid::Uuid;

pub static GAMES_ROUTER: LazyLock<Router<AppState>> = LazyLock::new(|| {
    Router::new()
        .route("/", get(game_list).post(create_game))
        .route("/{game_identifier}", get(game_detail).delete(game_delete).put(game_update))
        .route("/{game_identifier}/areas", get(game_areas))
        .route("/{game_identifier}/display", get(game_display))
        .route("/{game_identifier}/log/{day}", get(game_day_logs))
        .route("/{game_identifier}/log/{day}/{tribute_identifier}", get(tribute_logs))
        .route("/{game_identifier}/next", put(next_step))
        .route("/{game_identifier}/publish", put(publish_game))
        .route("/{game_identifier}/unpublish", put(unpublish_game))
        .nest("/{game_identifier}/tributes", TRIBUTES_ROUTER.clone())
});

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GameAreaEdge {
    #[serde(rename="in")]
    game: RecordId,
    #[serde(rename="out")]
    area: RecordId,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AreaItemEdge {
    #[serde(rename="in")]
    area: RecordId,
    #[serde(rename="out")]
    item: RecordId
}

async fn create_game_area(area: Area, db: &Surreal<Any>) -> Result<GameArea, AppError> {
    let identifier = Uuid::new_v4().to_string();
    let area_id: RecordId = RecordId::from(("area", identifier.to_string()));

    // create the `area` record
    let game_area: Option<GameArea> = db
        .insert::<Option<GameArea>>(area_id.clone())
        .content(GameArea {
            identifier: identifier.to_string(),
            name: area.to_string(),
            area: area.to_string(),
        })
        .await.expect("Failed to find Area and Game link");

    if let Some(game_area) = game_area {
        Ok(game_area)
    } else {
        Err(AppError::InternalServerError("Failed to create game area".into()))
    }
}

async fn create_game_area_edge(area: Area, game_identifier: Uuid, db: &Surreal<Any>) -> Result<GameAreaEdge, AppError> {
    let game_identifier_str = game_identifier.to_string();
    let game_id = RecordId::from(("game", &game_identifier_str));

    // Does the `area` exist for the game?
    let existing_area: Option<Area> = db.query(r#"
        SELECT identifier
        FROM area
        WHERE original_name = '$name'
        AND <-areas<-game.identifier = '$game_id'"#,
    )
        .bind(("name", area.clone()))
        .bind(("game_id", game_identifier_str.clone()))
        .await.and_then(|mut resp| resp.take(0)).expect("Failed to find Area");

    let area_uuid = if let Some(identifier) = existing_area {
        Uuid::from_str(&identifier.to_string()).expect("Failed to parse uuid")
    } else {
        match create_game_area(area, &db).await {
            Ok(game_area) => {
                Uuid::from_str(&game_area.identifier.as_str()).expect("Failed to parse uuid")
            }
            Err(_) => {
                return Err(AppError::InternalServerError("Failed to create game area".into()));
            }
        }
    };

    let gar = db
        .insert::<Option<Vec<GameAreaEdge>>>(
            RecordId::from(("areas", area_uuid.to_string()))
        ).relation(
        GameAreaEdge {
            game: game_id.clone(),
            area: RecordId::from(("area", &area_uuid.to_string())),
        }
    ).await.map_err(|_| AppError::InternalServerError("Failed to link game and area".into()))?;

    match gar {
        Some(edges) if !edges.is_empty() => Ok(edges[0].clone()),
        _ => Err(AppError::InternalServerError("Failed to create game area record".into())),
    }
}

pub async fn create_game(state: State<AppState>, Json(payload): Json<Game>) -> Result<Json<Game>, AppError> {
    let game_identifier = payload.clone().identifier;

    let game: Option<Game> = state.db
        .create(("game", &game_identifier))
        .content(payload.clone())
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to create game: {}", e)))?;

    let game = game.ok_or_else(|| AppError::InternalServerError("Game creation returned empty result".into()))?;

    // Create tributes concurrently
    let tribute_futures = (0..24).map(|idx| create_tribute(None, &game_identifier, &state.db, idx % 12));
    let tribute_results = futures::future::join_all(tribute_futures).await;

    if let Some(err) = tribute_results.into_iter().find_map(Result::err) {
        return Err(AppError::InternalServerError(format!("Failed to create tributes: {}", err)));
    }

    // Create areas concurrently
    let area_futures = Area::iter()
        .map(|area| create_area(game_identifier.as_str(), area.clone(), 3, &state.db));
    let area_results = futures::future::join_all(area_futures).await;

    if let Some(err) = area_results.into_iter().find_map(Result::err) {
        return Err(AppError::InternalServerError(format!("Failed to create areas: {}", err)));
    }

    Ok(Json(game))
}

pub async fn create_area(game_identifier: &str, area: Area, num_items: u32, db: &Surreal<Any>) -> Result<(), AppError> {
    let game_uuid = Uuid::from_str(game_identifier).expect("Bad UUID");
    if let Ok(game_area) = create_game_area_edge(area.clone(), game_uuid, &db).await {
        let item_futures = (0..num_items).map(|_| add_item_to_area(&game_area, &db));
        let item_results = futures::future::join_all(item_futures).await;
        if let Some(err) = item_results.into_iter().find_map(Result::err) {
            return Err(AppError::InternalServerError(format!("Failed to create items: {}", err)));
        }
    } else {
        return Err(AppError::InternalServerError("Failed to create game area".into()));
    }

    Ok(())
}

pub async fn add_item_to_area(game_area_edge: &GameAreaEdge, db: &Surreal<Any>) -> Result<(), AppError> {
    // Insert an item
    let new_item: Item = Item::new_random(None);
    let new_item_id: RecordId = RecordId::from(("item", &new_item.identifier));
    if let Err(e) = db.insert::<Option<Item>>(new_item_id.clone())
        .content(new_item.clone())
        .await
    {
        return Err(AppError::InternalServerError(format!("Failed to create item: {}", e)));
    }

    // Insert an area-item relationship
    let area_item: AreaItemEdge = AreaItemEdge {
        area: game_area_edge.area.clone(),
        item: new_item_id.clone(),
    };
    if let Err(e) = db.insert::<Vec<AreaItemEdge>>("items")
        .relation([area_item])
        .await
    {
        Err(AppError::InternalServerError(format!("Failed to create area-item relationship: {}", e)))
    } else {
        Ok(())
    }
}

pub async fn game_delete(game_identifier: Path<Uuid>, state: State<AppState>) -> Result<StatusCode, AppError> {
    let game_identifier = game_identifier.to_string();
    let mut result = state.db.query(r#"
    SELECT <-playing_in<-tribute as tribute,
           <-playing_in<-tribute->owns->item as item,
           <-playing_in<-tribute->owns as owns
    FROM game WHERE identifier = "$game_identifier";

    SELECT ->areas->area AS area,
           ->areas->area->items->item AS item,
           ->areas->area->items as items,
           ->areas AS areas
    FROM game WHERE identifier = "$game_identifier";
    "#.to_string())
        .bind(("game_id", game_identifier.clone()))
        .await.expect("Failed to find game pieces");

    let game_pieces: Option<HashMap<String, Vec<Thing>>> = result.take(0).unwrap();
    let area_pieces: Option<HashMap<String, Vec<Thing>>> = result.take(1).unwrap();
    if game_pieces.is_some() { delete_pieces(game_pieces.unwrap(), &state.db).await? };
    if area_pieces.is_some() { delete_pieces(area_pieces.unwrap(), &state.db).await? };

    let game: Option<Game> = state.db.delete(("game", &game_identifier)).await.expect("Failed to delete game");
    match game {
        Some(_) => Ok(StatusCode::NO_CONTENT),
        None => {
            Err(AppError::InternalServerError("Failed to delete game".into()))
        }
    }
}

async fn delete_pieces(pieces: HashMap<String, Vec<Thing>>, db: &Surreal<Any>) -> Result<(), AppError>{
    for (table, ids) in pieces {
        let db = db
            .query("DELETE $table WHERE id IN [$ids]".to_string())
            .bind(("table", table.clone()))
            .bind(("ids", ids.iter()
                .map(|i| format!(r#"{table}:{}"#, i.id))
                .collect::<Vec<String>>().join(",")
            ))
            .await;
        if db.is_err() {
            return Err(AppError::InternalServerError(format!("Failed to delete {} pieces.", table)));
        }
    }
    Ok(())
}

pub async fn game_list(state: State<AppState>) -> Result<Json<Vec<ListDisplayGame>>, AppError> {
    let mut games = state.db.query(r#"
SELECT name, identifier, status, day, private,
created_by.id == $auth.id AS is_mine,
created_by.username,
(
    RETURN count(
        SELECT id FROM <-playing_in<-tribute
    )
) AS tribute_count,
(
    RETURN count(
        SELECT id FROM <-playing_in<-tribute WHERE attributes.health > 0
    )
) AS living_count,
count(<-playing_in<-tribute.id) == 24
AND
count(array::distinct(<-playing_in<-tribute.district)) == 12
AS ready
FROM game
;"#).await.unwrap();

    match games.take::<Vec<ListDisplayGame>>(0) {
        Ok(games) => {
            Ok(Json::<Vec<ListDisplayGame>>(games))
        }
        Err(e) => {
            dbg!(&games);
            Err(AppError::InternalServerError(format!("Failed to fetch games: {}", e)))
        }
    }
}

pub async fn game_detail(game_identifier: Path<Uuid>, state: State<AppState>) -> Result<Json<DisplayGame>, AppError> {
    // let identifier = game_identifier.0;
    let identifier = game_identifier.to_string();
    let day = state.db
        .query("SELECT day FROM game WHERE identifier = $identifier LIMIT 1")
        .bind(("identifier", identifier.clone()))
        .await;
    let day: Option<i64> = day.unwrap().take("day").unwrap();
    let day: i64 = day.unwrap_or(0);

    let mut result = state.db.query(r#"
SELECT *, (
    SELECT *,
    ->owns->item[*] AS items, (
        SELECT *
        FROM tribute_log
        WHERE tribute_identifier = $parent.identifier
        AND day = $day
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
count(<-playing_in<-tribute.id) == 24 AND count(array::distinct(<-playing_in<-tribute.district)) == 12 AS ready,
created_by.id == $auth.id AS is_mine,
created_by.username
FROM game
WHERE identifier = $identifier;"#)
        .bind(("day", day))
        .bind(("identifier", identifier.clone()))
        .await.unwrap();

    let game: Option<DisplayGame> = result.take(0).expect("No game found");

    if let Some(game) = game {
        Ok(Json(game))
    } else {
        Err(AppError::NotFound("Failed to find game".into()))
    }
}

pub async fn game_update(Path(game_identifier): Path<Uuid>, state: State<AppState>, Json(payload): Json<EditGame>) -> Result<Json<Game>, AppError> {
    let response = state.db.query(r#"
        UPDATE game
        SET name = $name, private = $private
        WHERE identifier = $identifier;
        "#
    )
        .bind(("identifier", game_identifier.to_string()))
        .bind(("name", payload.1.clone()))
        .bind(("private", payload.2))
        .await;

    match response {
        Ok(mut response) => {
            let game: Option<Game> = response.take(0).unwrap();
            if let Some(game) = game {
                Ok(Json::<Game>(game))
            } else if let None = game {
                Err(AppError::NotFound("Failed to find game".into()))
            } else {
                unreachable!()
            }
        }
        Err(_) => {
            Err(AppError::InternalServerError("Failed to update game".into()))
        }
    }
}

pub async fn game_areas(Path(identifier): Path<Uuid>, state: State<AppState>) -> Result<Json<Vec<AreaDetails>>, AppError> {
    let response = state.db.query(r#"
SELECT (
    SELECT *, ->items->item[*] AS items
    FROM ->areas->area
) AS areas FROM game WHERE identifier = $identifier;
"#).bind(("identifier", identifier.to_string())).await;

    match response {
        Ok(mut response) => {
            let areas: Vec<Vec<AreaDetails>> = response.take("areas").unwrap();
            Ok(Json::<Vec<AreaDetails>>(areas[0].clone()))
        }
        Err(e) => {
            Err(AppError::InternalServerError(format!("Failed to fetch areas: {}", e)))
        }
    }
}

pub async fn game_tributes(Path(identifier): Path<Uuid>, state: State<AppState>) -> Result<Json<Vec<Tribute>>, AppError> {
    let mut game_day = state.db
        .query("SELECT day FROM game WHERE identifier = $identifier")
        .bind(("identifier", identifier.to_string()))
        .await.unwrap();
    let game_day: Option<i64> = game_day.take("day").unwrap();
    let _game_day: i64 = game_day.unwrap_or(0);


    let response = state.db.query(r#"
SELECT (
    SELECT *, ->owns->item[*] as items
    FROM <-playing_in<-tribute
    ORDER district
) AS tributes FROM game WHERE identifier = $identifier;"#)
        .bind(("identifier", identifier.to_string())).await;

    match response {
        Ok(mut response) => {
            let tributes: Vec<Vec<Tribute>> = response.take("tributes").unwrap();
            Ok(Json::<Vec<Tribute>>(tributes[0].clone()))
        }
        Err(e) => {
            Err(AppError::InternalServerError(format!("Failed to fetch tributes: {}", e)))
        }
    }
}

async fn get_game_status(db: &Surreal<Any>, identifier: &str) -> Result<GameStatus, AppError> {
    let result = db.query("SELECT status FROM game WHERE identifier = $identifier")
        .bind(("identifier", identifier.to_string()))
        .await;
    match result {
        Ok(mut result) => {
            match result.take::<Option<String>>("status") {
                Ok(Some(game_status)) => {
                    match GameStatus::from_str(game_status.as_str()) {
                        Ok(status) => Ok(status),
                        Err(_) => Err(AppError::InternalServerError("Invalid status".into())),
                    }
                }
                Err(e) => {
                    Err(AppError::NotFound(format!("Failed to find game status: {}", e)))
                }
                _ => {
                    Err(AppError::NotFound("Failed to find game status".into()))
                }
            }
        }
        _ => Err(AppError::NotFound("Failed to find game".into())),
    }
}

async fn update_game_status(db: &Surreal<Any>, record_id: &RecordId, status: GameStatus) -> Result<(), AppError> {
    match db.query("UPDATE $record_id SET status = $status")
        .bind(("record_id", record_id.clone()))
        .bind(("status", status.to_string()))
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => {
            Err(AppError::InternalServerError(format!("Failed to update game status: {}", e)))
        }
    }
}

async fn get_dead_tribute_count(db: &Surreal<Any>, identifier: &str) -> Result<u32, AppError> {
    let result = db.query(r#"
        RETURN count(
            SELECT in.identifier
            FROM playing_in
            WHERE out.identifier = $identifier
            AND in.status IN ["RecentlyDead", "Dead"]
        );"#).bind(("identifier", identifier.to_string()))
        .await;

    match result {
        Ok(mut result) => {
            match result.take::<Option<u32>>(0) {
                Ok(Some(dead_tributes)) => {
                    Ok(dead_tributes)
                }
                _ => {
                    Err(AppError::NotFound("Failed to find game".into()))
                }
            }
        },
        _ => {
            Err(AppError::NotFound("Failed to find game".into()))
        }
    }
}

async fn run_game_cycles(game: &mut Game, db: &Surreal<Any>) -> Result<(), AppError> {
    game.run_day_night_cycle(true);
    game.run_day_night_cycle(false);
    let _ = save_game(game, db).await.expect("Failed to run game cycles");
    Ok(())
}

pub async fn next_step(Path(identifier): Path<Uuid>, state: State<AppState>) -> Result<Json<Option<Game>>, AppError> {
    let id = identifier.to_string();
    let id_str = id.as_str();
    let record_id = RecordId::from(("game", id_str));
    let game_status = get_game_status(&state.db, id_str).await?;

    match game_status {
        GameStatus::NotStarted => {
            update_game_status(&state.db, &record_id, GameStatus::InProgress).await?;
            let mut game = get_full_game(identifier, &state.db).await?.0;
            game.status = GameStatus::InProgress;
            Ok(Json(Some(game)))
        },
        GameStatus::InProgress => {
            let dead_tribute_count = get_dead_tribute_count(&state.db, &id_str).await?;

            if dead_tribute_count >= 24 {
                update_game_status(&state.db, &record_id, GameStatus::Finished).await?;
                Ok(Json(None))
            } else {
                let mut game = get_full_game(identifier, &state.db).await?.0;
                run_game_cycles(&mut game, &state.db).await?;

                Ok(Json(Some(game)))
            }
        },
        GameStatus::Finished => {
            Ok(Json(None))
        }
    }
}


async fn get_full_game(identifier: Uuid, db: &Surreal<Any>) -> Result<Json<Game>, AppError> {
    let identifier = identifier.to_string();
    let mut result = db.query(r#"
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
WHERE identifier = $identifier;"#)
        .bind(("identifier", identifier.clone()))
        .await.unwrap();
    if let Some(game) = result.take(0).expect("No game found") {
        Ok(Json::<Game>(game))
    } else {
        Err(AppError::NotFound("Failed to find game".into()))
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
struct GameLog {
    pub id: RecordId,
    pub identifier: String,
    pub source: MessageSource,
    pub game_day: u32,
    pub subject: String,
    #[serde(with = "chrono::serde::ts_nanoseconds")]
    pub timestamp: DateTime<Utc>,
    pub content: String,
}

async fn save_game(game: &Game, db: &Surreal<Any>) -> Result<Json<Game>, AppError> {
    let game_identifier = RecordId::from(("game", game.identifier.clone()));

    // Start transaction
    db.query("BEGIN TRANSACTION").await.expect("Failed to start transaction");

    if let Ok(logs) = get_all_messages() {
        let game_day = game.day.unwrap_or_default();

        let game_logs: Vec<GameLog> = logs.iter().map(|log| {
            let log = log.clone();
            let game_log = GameLog {
                id: RecordId::from(("message", &log.identifier)),
                identifier: log.identifier,
                source: log.source,
                game_day,
                subject: log.subject,
                timestamp: log.timestamp,
                content: log.content,
            };
            game_log
        }).collect();

        if let Err(e) = db.insert::<Vec<GameMessage>>(()).content(game_logs).await {
            db.query("ROLLBACK").await.expect("Failed to rollback transaction");
            return Err(AppError::InternalServerError(format!("Failed to save game logs: {}", e)));
        }
    }

    let area_results = futures::future::join_all(game.areas.iter().map(|area| async {
        let id = RecordId::from(("area", area.identifier.clone()));
        save_area_items(&area.items, id.clone(), db).await?;

        let mut area_without_items = area.clone();
        area_without_items.items = vec![];
        db.update::<Option<AreaDetails>>(id.clone())
            .content(area_without_items)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Failed to update area: {}", e)))
    })).await;

    if area_results.iter().any(|result| result.is_err()) {
        db.query("ROLLBACK").await.expect("Failed to rollback transaction");
        return Err(AppError::InternalServerError("Failed to save area items".into()));
    }

    let tribute_results = futures::future::join_all(game.tributes.iter().map(|tribute| async {
        let id = RecordId::from(("tribute", tribute.identifier.clone()));
        if tribute.is_alive() {
            save_tribute_items(tribute.items.clone(), id.clone(), db).await?;
        }

        let mut tribute_without_items = tribute.clone();
        tribute_without_items.items = vec![];
        db.update::<Option<Tribute>>(id.clone())
            .content(tribute_without_items)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Failed to update tribute: {}", e)))
    })).await;

    if tribute_results.iter().any(|result| result.is_err()) {
        db.query("ROLLBACK").await.expect("Failed to rollback transaction");
        return Err(AppError::InternalServerError("Failed to save tribute items".into()));
    }

    let mut saved_game = game.clone();
    saved_game.tributes = vec![];
    saved_game.areas = vec![];
    match db.update::<Option<Game>>(game_identifier.clone()).content(saved_game).await {
        Ok(Some(game)) => {
            // Commit transaction
            db.query("COMMIT").await.expect("Failed to commit transaction");
            Ok(Json(game))
        }
        Ok(None) => {
            db.query("ROLLBACK").await.expect("Failed to rollback transaction");
            Err(AppError::NotFound("Failed to find game".into()))
        }
        Err(e) => {
            db.query("ROLLBACK").await.expect("Failed to rollback transaction");
            Err(AppError::InternalServerError(format!("Failed to update game: {}", e)))
        }
    }
}

async fn save_area_items(items: &Vec<Item>, owner: RecordId, db: &Surreal<Any>) -> Result<(), AppError> {
    // Get existing items
    let existing_items: Vec<Item> = db.query("SELECT * FROM items WHERE in = $owner")
        .bind(("owner", owner.clone()))
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to fetch items: {}", e)))?
        .take(0)
        .map_err(|e| AppError::InternalServerError(format!("Failed to take items: {}", e)))?;

    // Create lookups for efficient comparison
    let mut existing_map = HashMap::new();
    for item in existing_items {
        existing_map.insert(item.identifier.clone(), item.clone());
    }

    let mut new_map = HashMap::new();
    for item in items {
        new_map.insert(item.identifier.clone(), item.clone());
    }

    // Find items to delete (in DB but not in new items or quantity is 0)
    let mut items_to_delete = Vec::new();
    for id in existing_map.keys() {
        if !new_map.contains_key(id) || new_map.get(id).unwrap().quantity == 0 {
            items_to_delete.push(id.clone());
        }
    }

    // Find items to update (in DB and in new items with different values)
    let mut items_to_update = Vec::new();
    for (id, item) in &new_map {
        if item.quantity > 0 &&
            (!existing_map.contains_key(id) || existing_map.get(id).unwrap() != item) {
            items_to_update.push(item.clone());
        }
    }

    // Batch delete operations
    let mut delete_failed = false;
    for id in &items_to_delete {
        let item_id = RecordId::from(("item", id.clone()));
        if let Err(_) = db.delete::<Option<Item>>(item_id).await {
            delete_failed = true;
        }
    }

    if delete_failed {
        return Err(AppError::InternalServerError("Failed to delete items".into()));
    }

    // Batch update operations
    let mut update_failed = false;
    for item in &items_to_update {
        let item_id = RecordId::from(("item", item.identifier.clone()));
        if let Err(_) = db.update::<Option<Item>>(item_id)
            .content(item.clone())
            .await
        {
            update_failed = true;
        }
    }

    if update_failed {
        return Err(AppError::InternalServerError("Failed to update items".into()));
    }

    // Update relations - first delete existing relations
    db.query("DELETE FROM items WHERE in = $owner")
        .bind(("owner", owner.clone()))
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to delete items: {}", e)))?;

    for item in &items_to_update {
        let item_id = RecordId::from(("item", item.identifier.clone()));
        match db.insert::<Vec<AreaItemEdge>>("items").relation(
            AreaItemEdge {
                area: owner.clone(),
                item: item_id.clone(),
            }
        ).await {
            Ok(_) => {}
            Err(e) => {
                return Err(AppError::InternalServerError(format!("Failed to create items relation: {}", e)));
            }
        };
    }

    Ok(())
}

async fn save_tribute_items(items: Vec<Item>, owner: RecordId, db: &Surreal<Any>) -> Result<(), AppError> {
    let _ = db.query("DELETE FROM owns WHERE in = $owner")
        .bind(("owner", owner.clone()))
        .await.expect("Failed to delete items");

    for item in items {
        let item_identifier = RecordId::from(("item", item.identifier.clone()));
        if item.quantity == 0 {
            db.delete::<Option<Item>>(item_identifier.clone())
                .await.expect("Failed to delete item");
        } else {
            db.update::<Option<Item>>(item_identifier.clone())
                .content(item.clone())
                .await.expect("Failed to update item");
        }

        if item.quantity > 0 {
            let _: Vec<TributeAreaEdge> = db.insert("owns").relation(
                TributeAreaEdge {
                    tribute: owner.clone(),
                    item: item_identifier.clone(),
                }
            ).await.expect("Failed to update Owns relation");
        }
    }
    Ok(())
}

async fn game_day_logs(Path((game_identifier, day)): Path<(Uuid, u32)>, state: State<AppState>) -> Result<Json<Vec<GameMessage>>, AppError> {
    let game_identifier = game_identifier.to_string();
    match state.db
        .query(r#"SELECT * FROM message
        WHERE string::starts_with(subject, $identifier) AND
        game_day = $day ORDER BY timestamp;"#)
        .bind(("identifier", game_identifier))
        .bind(("day", day))
        .await
    {
        Ok(mut logs) => {
            let logs: Vec<GameMessage> = logs.take(0).unwrap_or_else(|err| {
                eprintln!("Error taking logs: {err:?}");
                vec![]
            });
            Ok(Json(logs))
        }
        Err(err) => {
            Err(AppError::NotFound(format!("Failed to get logs: {err:?}")))
        }
    }
}

async fn tribute_logs(Path((game_identifier, day, tribute_identifier)): Path<(Uuid, u32, Uuid)>, state: State<AppState>) -> Result<Json<Vec<GameMessage>>, AppError> {
    let game_identifier = game_identifier.to_string();
    let tribute_identifier = tribute_identifier.to_string();
    match state.db
        .query(r#"SELECT *
        FROM message
        WHERE string::starts_with(subject, $game_identifier)
        AND game_day = $day
        AND source.value = $tribute_identifier
        ORDER BY timestamp;"#.to_string())
        .bind(("game_identifier", game_identifier))
        .bind(("day", day))
        .bind(("tribute_identifier", tribute_identifier))
        .await
    {
        Ok(mut logs) => {
            let logs: Vec<GameMessage> = logs.take(0).expect("logs is empty");
            Ok(Json(logs))
        }
        Err(err) => {
            Err(AppError::NotFound(format!("Failed to get logs: {err:?}")))
        }
    }
}

async fn publish_game(Path(game_identifier): Path<Uuid>, state: State<AppState>) -> Result<StatusCode, AppError> {
    let game_identifier = game_identifier.to_string();
    let response = state.db
        .query("UPDATE game SET private = false WHERE identifier = '$identifier'")
        .bind(("identifier", game_identifier))
        .await;

    match response {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => {
            Err(AppError::InternalServerError(format!("Failed to publish game: {e:?}")))
        }
    }
}

async fn unpublish_game(Path(game_identifier): Path<Uuid>, state: State<AppState>) -> Result<StatusCode, AppError> {
    let game_identifier = game_identifier.to_string();
    let response = state.db
        .query("UPDATE game SET private = true WHERE identifier = '$identifier'")
        .bind(("identifier", game_identifier))
        .await;

    match response {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => {
            Err(AppError::InternalServerError(format!("Failed to unpublish game: {e:?}")))
        }
    }
}

pub async fn game_display(game_identifier: Path<Uuid>, state: State<AppState>) -> Result<Json<DisplayGame>, AppError> {
    let identifier = game_identifier.to_string();
    let mut result = state.db.query(r#"
LET $tributes = (
    SELECT  in.id,
            in.name,
            in.district,
            in.attributes.health
    FROM playing_in
    WHERE out.identifier = $identifier
);

LET $living_tributes = (
    SELECT * FROM $tributes.in WHERE attributes.health > 0
);

LET $winner = (
    IF count($living_tributes) == 1 THEN
        RETURN $living_tributes[0].name
    ELSE
        RETURN ""
    END
);

SELECT
    identifier,
    name,
    status,
    day,
    private,
    created_by.username,
    created_by.id == $auth.id AS is_mine,
    $tributes.in AS tributes,
    count($tributes.in) as tribute_count,
    count(SELECT * FROM $tributes.in WHERE attributes.health > 0) AS living_count,
    count($tributes.in) == 24 AND
        count(array::distinct(<-playing_in<-tribute.district)) == 12 AS ready,
    $winner AS winner
FROM game
WHERE identifier = $identifier
LIMIT 1
FETCH tribute
;"#)
        .bind(("identifier", identifier.clone()))
        .await.unwrap();

    let game: Option<DisplayGame> = result.take(3).expect("No game found");

    if let Some(game) = game {
        Ok(Json(game))
    } else {
        Err(AppError::NotFound(format!("No game found with identifier: {identifier}")))
    }
}

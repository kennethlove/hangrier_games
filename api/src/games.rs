use crate::tributes::{tribute_record_create, TributeOwns, TRIBUTES_ROUTER};
use crate::{AppError, AppState};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, put};
use axum::Router;
use axum::Json;
use game::areas::{Area, AreaDetails};
use game::games::Game;
use game::items::Item;
use game::messages::{get_all_messages, GameMessage};
use game::tributes::Tribute;
use serde::{Deserialize, Serialize};
use shared::GameArea;
use shared::{DisplayGame, EditGame, GameStatus};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::LazyLock;
use strum::IntoEnumIterator;
use surrealdb::sql::Thing;
use surrealdb::{RecordId, Surreal};
use surrealdb::engine::any::Any;
use uuid::Uuid;

pub static GAMES_ROUTER: LazyLock<Router<AppState>> = LazyLock::new(|| {
    Router::new()
        .route("/", get(game_list).post(game_create))
        .route("/{game_identifier}", get(game_detail).delete(game_delete).put(game_update))
        .route("/{game_identifier}/areas", get(game_areas))
        .route("/{game_identifier}/display", get(game_display))
        .route("/{game_identifier}/log/{day}", get(game_day_logs))
        .route("/{game_identifier}/log/{day}/{tribute_identifier}", get(tribute_logs))
        .route("/{game_identifier}/next", put(next_step))
        .route("/{game_identifier}/publish", put(publish_game))
        // .route("/{game_identifier}/summarize", get(game_summary))
        // .route("/{game_identifier}/summarize/{day}", get(game_day_summary))
        .route("/{game_identifier}/unpublish", put(unpublish_game))
        .nest("/{game_identifier}/tributes", TRIBUTES_ROUTER.clone())
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

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct GameResponse {
    game: Option<Game>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GameSummary {
    pub day: i64,
    pub summary: String,
}

async fn game_area_create(area: Area, db: &Surreal<Any>) -> Result<GameArea, AppError> {
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

async fn game_area_record_create(identifier: Uuid, game_id: RecordId, db: &Surreal<Any>) -> Result<Vec<GameAreaRecord>, AppError> {
    let gar = db
        .insert::<Option<Vec<GameAreaRecord>>>(
            RecordId::from(("areas", identifier.to_string()))
        ).relation(
        GameAreaRecord {
            game: game_id.clone(),
            area: RecordId::from(("area", &identifier.to_string())),
        }
    ).await.expect("Failed to link Area and Game");

    if let Some(gar) = gar {
        Ok(gar)
    } else {
        Err(AppError::InternalServerError("Failed to create game area record".into()))
    }
}

async fn game_area_record_creator(area: Area, game_identifier: Uuid, db: &Surreal<Any>) -> Result<GameAreaRecord, AppError> {
    // Does the `area` exist for the game?
    let existing_area: Option<String>;
    let game_identifier = game_identifier.to_string();

    if let Ok(mut area) = db.query(r#"
        SELECT identifier
        FROM area
        WHERE original_name = '$name'
        AND <-areas<-game.identifier = '$game_id'"#,
    )
        .bind(("name", area.clone()))
        .bind(("game_id", game_identifier.clone()))
        .await
    {
        existing_area = area.take::<Option<String>>(0).unwrap(); // e.g. a UUID
    } else {
        existing_area = None;
    }

    let game_id = RecordId::from(("game", game_identifier));
    let gar: Vec<GameAreaRecord>;

    if let Some(identifier) = existing_area {
        // The `area` exists, create the `areas` connection
        let uuid = Uuid::from_str(identifier.as_str()).expect("Bad UUID");
        gar = game_area_record_create(uuid, game_id, &db).await?;
    } else {
        // The `area` does not exist, create the `area`
        if let Ok(area) = game_area_create(area, &db).await {
            // Then create the `areas` connection
            let identifier = area.identifier.clone();
            let uuid = Uuid::from_str(identifier.as_str()).expect("Bad UUID");
            gar = game_area_record_create(uuid, game_id, &db).await?;
        } else {
            return Err(AppError::InternalServerError("Failed to create game area".into()));
        }
    }

    if !gar.is_empty() {
        if let Some(resp) = gar.clone().pop() {
            Ok(resp)
        } else {
            Err(AppError::InternalServerError("Failed to create game area".into()))
        }
    } else {
        Err(AppError::InternalServerError("Failed to create game area record".into()))
    }
}

pub async fn game_create(state: State<AppState>, Json(payload): Json<Game>) -> Result<Json<Game>, AppError> {
    let game: Option<Game> = state.db
        .create(("game", &payload.identifier))
        .content(payload.clone())
        .await.expect("Failed to create game");
    let game = game.unwrap();

    for _ in 0..24 {
        tribute_record_create(None, payload.clone().identifier, &state.db).await.expect("Failed to create tributes");
    }

    for area in Area::iter() {
        let uuid = Uuid::from_str(payload.clone().identifier.as_str()).expect("Bad UUID");
        let game_area = game_area_record_creator(
            area.clone(), // Area to link to,
            uuid, // Game to link to,
            &state.db // Database connection
        ).await.expect("Failed to create game area");

        for _ in 0..3 {
            // Insert an item
            let new_item: Item = Item::new_random(None);
            let new_item_id: RecordId = RecordId::from(("item", &new_item.identifier));
            state.db
                .insert::<Option<Item>>(new_item_id.clone())
                .content(new_item.clone())
                .await.expect("failed to insert item");

            // Insert an area-item relationship
            let area_item: AreaItem = AreaItem {
                area: game_area.area.clone(),
                item: new_item_id.clone(),
            };
            state.db
                .insert::<Vec<AreaItem>>("items")
                .relation([area_item])
                .await.expect("");
        }
    }

    Ok(Json::<Game>(game.clone()))
}

pub async fn game_delete(game_identifier: Path<String>, state: State<AppState>) -> Result<StatusCode, AppError> {
    let game_identifier = game_identifier.to_string().clone();
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

pub async fn game_list(state: State<AppState>) -> Result<Json<Vec<DisplayGame>>, AppError> {
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

    match games.take::<Vec<DisplayGame>>(0) {
        Ok(games) => {
            Ok(Json::<Vec<DisplayGame>>(games))
        }
        Err(e) => {
            tracing::error!("{}", e);
            Err(AppError::InternalServerError("Failed to fetch games".into()))
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

pub async fn game_update(Path(_): Path<String>, state: State<AppState>, Json(payload): Json<EditGame>) -> Result<Json<Game>, AppError> {
    let response = state.db.query(r#"
        UPDATE game
        SET name = $name, private = $private
        WHERE identifier = $identifier;
        "#
    )
        .bind(("identifier", payload.0.clone()))
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

pub async fn next_step(Path(identifier): Path<Uuid>, state: State<AppState>) -> Result<Json<GameResponse>, AppError> {
    let identifier = identifier.to_string();
    let record_id = RecordId::from(("game", identifier.clone()));
    let mut result = state.db.query(r#"
SELECT status FROM game WHERE identifier = $identifier;
RETURN count(
    SELECT in.identifier
    FROM playing_in
    WHERE out.identifier = $identifier
    AND in.status IN ["RecentlyDead", "Dead"]
);"#).bind(("identifier", identifier.clone())).await.expect("No game found");
    let status: Option<String> = result.take("status").expect("No game found");
    let dead_tribute_count: Option<u32> = result.take(1).unwrap_or(Some(0));

    if let Some(status) = status {
        let status = GameStatus::from_str(status.as_str()).expect("Invalid game status");
        match status {
            GameStatus::NotStarted => {
                state.db
                    .query("UPDATE $record_id SET status = $status")
                    .bind(("record_id", record_id.clone()))
                    .bind(("status", GameStatus::InProgress))
                    .await.expect("Failed to start game");
                Ok(Json(GameResponse::default()))
            }
            GameStatus::InProgress | GameStatus::Finished => {
                let uuid = Uuid::parse_str(identifier.as_str()).expect("Failed to parse UUID");
                if let Ok(Json(mut game)) = get_full_game(uuid, &state.db).await {
                    match dead_tribute_count {
                        Some(24) => {
                            state.db
                                .query("UPDATE $record_id SET status = $status")
                                .bind(("record_id", record_id.clone()))
                                .bind(("status", GameStatus::Finished))
                                .await.expect("Failed to end game");
                            Ok(Json(GameResponse::default()))
                        }
                        _ => {
                            // Run day
                            game.run_day_night_cycle(true).await;
                            let _ = save_game(&game, &state.db).await.expect("Day cycle failed");

                            // Run night
                            game.run_day_night_cycle(false).await;
                            let _ = save_game(&game, &state.db).await.expect("Night cycle failed");

                            Ok(Json(GameResponse { game: Some(game) }))
                        }
                    }
                } else {
                    tracing::info!("Full game could not be found");
                    Err(AppError::NotFound("Failed to find game".into()))
                }
            }
        }
    } else {
        tracing::info!("Game could not be found");
        Err(AppError::NotFound("Failed to find game".into()))
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

async fn save_game(game: &Game, db: &Surreal<Any>) -> Result<Json<Game>, AppError> {
    let game_identifier = RecordId::from(("game", game.identifier.clone()));
    let mut saved_game = game.clone();


    if let Ok(logs) = get_all_messages() {
        for mut log in logs {
            log.game_day = game.day.unwrap_or_default();
            if let Err(_) = db
                .upsert::<Option<GameMessage>>(("message", &log.identifier))
                .content(log.clone())
                .await
            {
                return Err(AppError::InternalServerError("Failed to save game log".into()));
            }
        }
    }

    let areas = game.areas.clone();
    saved_game.areas = vec![];

    for mut area in areas {
        let id = RecordId::from(("area", area.identifier.clone()));
        let items = area.items.clone();
        let _ = save_area_items(items, id.clone(), &db).await;
        area.items = vec![];

        db.update::<Option<AreaDetails>>(id).content(area).await.expect("Failed to update area items");
    }

    let tributes = game.tributes.clone();
    saved_game.tributes = vec![];

    for mut tribute in tributes {
        let id = RecordId::from(("tribute", tribute.identifier.clone()));
        let mut items = tribute.items.clone();
        if !tribute.is_alive() { items.clear(); }
        tribute.items = vec![];

        let _ = save_tribute_items(items, id.clone(), &db).await;

        db.update::<Option<Tribute>>(id).content(tribute).await.expect("Failed to update tributes");
    }

    let game = db.update::<Option<Game>>(game_identifier.clone()).content(saved_game).await.expect("Failed to update game");
    if let Some(game) = game {
        Ok(Json(game))
    } else {
        Err(AppError::NotFound("Failed to find game".into()))
    }
}

async fn save_area_items(items: Vec<Item>, owner: RecordId, db: &Surreal<Any>) -> Result<(), AppError> {
    let _ = db.query("DELETE FROM items WHERE in = $owner")
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
            let _: Vec<TributeOwns> = db.insert("items").relation(
                AreaItem {
                    area: owner.clone(),
                    item: item_identifier.clone(),
                }
            ).await.expect("Failed to update Items relation");
        }
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
            let _: Vec<TributeOwns> = db.insert("owns").relation(
                TributeOwns {
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

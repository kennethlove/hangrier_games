use crate::games::game_tributes;
use crate::storage::{UploadConstraints, validate_upload};
use crate::{AppError, AppState};
use axum::extract::{Multipart, Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use game::items::Item;
use game::messages::GameMessage;
use game::tributes::Tribute;
use serde::{Deserialize, Serialize};
use shared::EditTribute;
use std::sync::LazyLock;
use surrealdb::RecordId;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use uuid::Uuid;

pub static TRIBUTES_ROUTER: LazyLock<Router<AppState>> = LazyLock::new(|| {
    Router::new()
        .route("/", get(game_tributes))
        .route(
            "/{identifier}",
            get(tribute_detail)
                .delete(tribute_delete)
                .put(tribute_update),
        )
        .route("/{identifier}/avatar", post(upload_avatar))
        .route("/{identifier}/log", get(tribute_log))
});

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TributeItemEdge {
    #[serde(rename = "in")]
    pub tribute: RecordId,
    #[serde(rename = "out")]
    pub item: RecordId,
}

pub async fn create_tribute(
    tribute: Option<Tribute>,
    game_identifier: &str,
    db: &Surreal<Any>,
    district: u32,
) -> Result<Tribute, AppError> {
    let game_id = RecordId::from(("game", game_identifier.to_owned()));
    let tribute_count = db
        .query("RETURN count(SELECT id FROM playing_in WHERE out.identifier=$game)")
        .bind(("game", game_identifier.to_owned()))
        .await;
    let tribute_count: Option<u32> = tribute_count.unwrap().take(0).unwrap();
    if tribute_count >= Some(24) {
        return Err(AppError::GameFull("Game is full".to_string()));
    }

    let mut tribute = tribute.unwrap_or_else(Tribute::random);
    tribute.district = district + 1;
    tribute.statistics.game = game_identifier.to_owned();

    let id = RecordId::from(("tribute", &tribute.identifier));

    // Bind via serde_json::Value to bypass the SurrealDB SDK's bespoke type
    // serializer, which collapses externally-tagged enums and Option fields.
    // The generic JSON bind path round-trips cleanly. Mirrors the pattern in
    // save_game in api/src/games.rs.
    let body = serde_json::to_value(&tribute)
        .map_err(|e| AppError::InternalServerError(format!("Failed to encode tribute: {}", e)))?;
    db.query("UPSERT $rid CONTENT $body")
        .bind(("rid", id.clone()))
        .bind(("body", body))
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to create tribute: {}", e)))?;
    crate::verify_record_persisted(db, &id, "create_tribute").await?;
    let new_tribute = Some(tribute);

    db.query("RELATE $tribute->playing_in->$game")
        .bind(("tribute", id.clone()))
        .bind(("game", game_id.clone()))
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to connect tribute to game: {}", e))
        })?;

    let new_object: Item = Item::new_random(None);
    let new_object_id: RecordId = RecordId::from(("item", &new_object.identifier));
    let item_body = serde_json::to_value(&new_object)
        .map_err(|e| AppError::InternalServerError(format!("Failed to encode item: {}", e)))?;
    db.query("UPSERT $rid CONTENT $body")
        .bind(("rid", new_object_id.clone()))
        .bind(("body", item_body))
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to create item: {}", e)))?;
    crate::verify_record_persisted(db, &new_object_id, "create_tribute: item").await?;
    db.query("RELATE $tribute->owns->$item")
        .bind(("tribute", id.clone()))
        .bind(("item", new_object_id.clone()))
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to create owns relation: {}", e))
        })?;

    if let Some(tribute) = new_tribute {
        Ok(tribute)
    } else {
        Err(AppError::InternalServerError(
            "Failed to create tribute".to_string(),
        ))
    }
}

pub async fn tribute_delete(
    Path((_, tribute_identifier)): Path<(String, String)>,
    state: State<AppState>,
) -> Result<StatusCode, AppError> {
    let tribute: Option<Tribute> = state
        .db
        .delete(("tribute", &tribute_identifier))
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to delete tribute: {}", e)))?;
    match tribute {
        Some(_) => Ok(StatusCode::NO_CONTENT),
        None => Err(AppError::InternalServerError(
            "Could not delete tribute".into(),
        )),
    }
}

pub async fn tribute_update(
    Path((_game_identifier, _tribute_identifier)): Path<(Uuid, Uuid)>,
    state: State<AppState>,
    Json(payload): Json<EditTribute>,
) -> Result<StatusCode, AppError> {
    // Validate input
    if let Err(e) = validator::Validate::validate(&payload) {
        return Err(AppError::ValidationError(format!("{}", e)));
    }

    let response = state
        .db
        .query("UPDATE tribute SET name = $name, avatar = $avatar WHERE identifier = $identifier;")
        .bind(("identifier", payload.identifier.clone()))
        .bind(("name", payload.name.clone()))
        .bind(("avatar", Some(payload.avatar.clone())))
        .await;

    match response {
        Ok(mut response) => match response.take::<Option<Tribute>>(0).unwrap() {
            Some(_tribute) => Ok(StatusCode::OK),
            None => Err(AppError::InternalServerError(
                "Failed to update tribute".into(),
            )),
        },
        Err(_) => Err(AppError::InternalServerError(
            "Failed to update tribute".into(),
        )),
    }
}

pub async fn tribute_detail(
    Path((_, tribute_identifier)): Path<(Uuid, Uuid)>,
    state: State<AppState>,
) -> Result<Json<Tribute>, AppError> {
    let tribute_identifier = tribute_identifier.to_string();
    let mut result = state
        .db
        .query("SELECT * FROM fn::get_full_tribute($identifier);")
        .bind(("identifier", tribute_identifier))
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to fetch tribute: {}", e)))?;

    let tribute: Option<Tribute> = result
        .take(0)
        .map_err(|e| AppError::InternalServerError(format!("Failed to take tribute: {}", e)))?;

    if let Some(tribute) = tribute {
        Ok(Json(tribute.clone()))
    } else {
        Err(AppError::NotFound("Tribute not found".to_string()))
    }
}

pub async fn tribute_log(
    Path((_, identifier)): Path<(Uuid, Uuid)>,
    state: State<AppState>,
) -> Result<Json<Vec<GameMessage>>, AppError> {
    let identifier = identifier.to_string();
    let mut result = state
        .db
        .query("SELECT * FROM fn::get_messages_by_tribute_id($identifier)")
        .bind(("identifier", identifier))
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to fetch logs: {}", e)))?;

    let rows: Vec<crate::games::GameLog> = result
        .take(0)
        .map_err(|e| AppError::InternalServerError(format!("Failed to take logs: {}", e)))?;
    let logs: Vec<GameMessage> = rows.into_iter().map(GameMessage::from).collect();
    Ok(Json(logs))
}

/// Upload avatar for a tribute
/// Accepts multipart/form-data with a file field named "avatar"
pub async fn upload_avatar(
    Path((_, tribute_identifier)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, AppError> {
    let tribute_identifier = tribute_identifier.to_string();

    // Find the file field
    let mut file_data: Option<Vec<u8>> = None;
    let mut filename: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to read multipart field: {}", e)))?
    {
        let field_name = field.name().unwrap_or("").to_string();

        if field_name == "avatar" {
            filename = field.file_name().map(|s| s.to_string());
            file_data = Some(
                field
                    .bytes()
                    .await
                    .map_err(|e| AppError::BadRequest(format!("Failed to read file data: {}", e)))?
                    .to_vec(),
            );
            break;
        }
    }

    let file_data = file_data.ok_or_else(|| {
        AppError::BadRequest("No file uploaded. Field 'avatar' required.".to_string())
    })?;

    let filename =
        filename.ok_or_else(|| AppError::BadRequest("No filename provided".to_string()))?;

    // Validate upload
    let constraints = UploadConstraints::default();
    validate_upload(&file_data, &filename, &constraints)?;

    // Generate storage path: avatars/{tribute_id}.{extension}
    let extension = std::path::Path::new(&filename)
        .extension()
        .and_then(|e| e.to_str())
        .ok_or_else(|| AppError::ValidationError("Invalid filename".to_string()))?;

    let storage_path = format!("avatars/{}.{}", tribute_identifier, extension);

    // Save file
    let saved_path = state.storage.save(&storage_path, &file_data).await?;
    let public_url = state.storage.public_url(&saved_path);

    // Update tribute avatar field in database
    let response = state
        .db
        .query("UPDATE tribute SET avatar = $avatar WHERE identifier = $identifier;")
        .bind(("identifier", tribute_identifier.clone()))
        .bind(("avatar", Some(saved_path.clone())))
        .await;

    match response {
        Ok(mut response) => match response.take::<Option<Tribute>>(0).unwrap() {
            Some(_) => Ok(Json(serde_json::json!({
                "url": public_url,
                "path": saved_path
            }))),
            None => Err(AppError::InternalServerError(
                "Failed to update tribute avatar".into(),
            )),
        },
        Err(e) => Err(AppError::InternalServerError(format!(
            "Failed to update tribute: {}",
            e
        ))),
    }
}

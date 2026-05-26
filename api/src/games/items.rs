use crate::AppError;
use game::items::Item;
use std::collections::HashMap;
use surrealdb::engine::any::Any;
use surrealdb::{RecordId, Surreal};

pub(crate) async fn save_area_items(
    items: &Vec<Item>,
    owner: RecordId,
    db: &Surreal<Any>,
) -> Result<(), AppError> {
    // Get existing items
    let existing_items: Vec<Item> = db
        .query("SELECT * FROM items WHERE in = $owner")
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

    // Find items to delete (in DB but not in new items or durability is 0)
    let mut items_to_delete = Vec::new();
    for id in existing_map.keys() {
        if let Some(item) = new_map.get(id) {
            if item.current_durability == 0 {
                items_to_delete.push(id.clone());
            }
        } else {
            items_to_delete.push(id.clone());
        }
    }

    // Find items to update (in DB and in new items with different values)
    let mut items_to_update = Vec::new();
    for (id, item) in &new_map {
        if item.current_durability > 0 {
            if let Some(existing) = existing_map.get(id) {
                if existing != item {
                    items_to_update.push(item.clone());
                }
            } else {
                items_to_update.push(item.clone());
            }
        }
    }

    // Update relations - first delete existing relations
    db.query("DELETE FROM items WHERE in = $owner")
        .bind(("owner", owner.clone()))
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to delete items: {}", e)))?;

    // Batch delete operations
    if !items_to_delete.is_empty() {
        let delete_ids: Vec<String> = items_to_delete
            .iter()
            .map(|id| format!("item:{}", id))
            .collect();

        db.query("DELETE item WHERE id IN $ids")
            .bind(("ids", delete_ids))
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("Failed to batch delete items: {}", e))
            })?;
    }

    // Batch update operations
    if !items_to_update.is_empty() {
        // Use serde_json::to_value + bound CONTENT so ALL Item fields round-trip
        // (the previous hand-rolled CONTENT block silently dropped `rarity`,
        // and string-interpolating fields is fragile around quoting).
        for item in &items_to_update {
            let rid = RecordId::from(("item", item.identifier.to_string().as_str()));
            let body = serde_json::to_value(item).map_err(|e| {
                AppError::InternalServerError(format!("Failed to encode item: {}", e))
            })?;
            db.query("UPSERT $rid CONTENT $body")
                .bind(("rid", rid))
                .bind(("body", body))
                .await
                .map_err(|e| {
                    AppError::InternalServerError(format!("Failed to update item: {}", e))
                })?;
        }

        // Batch insert relations. Hyphenated UUIDs must be wrapped in
        // ⟨angle brackets⟩ or Surreal's SQL parser splits them on `-`.
        let mut relation_parts = Vec::new();
        for item in &items_to_update {
            relation_parts.push(format!(
                "RELATE {}->items->item:⟨{}⟩",
                owner, item.identifier
            ));
        }

        let bulk_relations = relation_parts.join(";\n");
        db.query(&bulk_relations).await.map_err(|e| {
            AppError::InternalServerError(format!("Failed to batch create relations: {}", e))
        })?;
    }

    Ok(())
}

pub(crate) async fn save_tribute_items(
    items: &Vec<Item>,
    owner: RecordId,
    db: &Surreal<Any>,
) -> Result<(), AppError> {
    // Get existing items
    let existing_items: Vec<Item> = db
        .query("SELECT * from owns->items WHERE in = $owner")
        .bind(("owner", owner.clone()))
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to fetch items: {}", e)))?
        .take(0)
        .map_err(|e| AppError::InternalServerError(format!("Failed to take items: {}", e)))?;

    // Create lookups for efficient comparison
    let mut existing_map = HashMap::new();
    for item in existing_items {
        existing_map.insert(item.identifier.clone(), item);
    }

    let mut new_map = HashMap::new();
    for item in items {
        new_map.insert(item.identifier.clone(), item.clone());
    }

    // Find items to delete (in DB but not in new items or durability is 0)
    let mut items_to_delete = Vec::new();
    for id in existing_map.keys() {
        if let Some(item) = new_map.get(id) {
            if item.current_durability == 0 {
                items_to_delete.push(id.clone());
            }
        } else {
            items_to_delete.push(id.clone());
        }
    }

    // Find items to update (in DB and in new items with different values)
    let mut items_to_update = Vec::new();
    for (id, item) in &new_map {
        if item.current_durability > 0 {
            if let Some(existing) = existing_map.get(id) {
                if existing != item {
                    items_to_update.push(item.clone());
                }
            } else {
                items_to_update.push(item.clone());
            }
        }
    }

    // Delete existing relations - do this once for all items
    db.query("DELETE FROM owns WHERE in = $owner")
        .bind(("owner", owner.clone()))
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to delete items: {}", e)))?;

    // Batch delete operations
    if !items_to_delete.is_empty() {
        let delete_ids: Vec<String> = items_to_delete
            .iter()
            .map(|id| format!("item:{}", id))
            .collect();

        db.query("DELETE item WHERE id IN $ids")
            .bind(("ids", delete_ids))
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("Failed to batch delete items: {}", e))
            })?;
    }

    // Batch update operations
    if !items_to_update.is_empty() {
        // Use serde_json::to_value + bound CONTENT so ALL Item fields round-trip
        // (the previous hand-rolled CONTENT block silently dropped `rarity`,
        // and string-interpolating fields is fragile around quoting).
        for item in &items_to_update {
            let rid = RecordId::from(("item", item.identifier.to_string().as_str()));
            let body = serde_json::to_value(item).map_err(|e| {
                AppError::InternalServerError(format!("Failed to encode item: {}", e))
            })?;
            db.query("UPSERT $rid CONTENT $body")
                .bind(("rid", rid))
                .bind(("body", body))
                .await
                .map_err(|e| {
                    AppError::InternalServerError(format!("Failed to update item: {}", e))
                })?;
        }

        // Batch insert relations. Hyphenated UUIDs must be wrapped in
        // ⟨angle brackets⟩ or Surreal's SQL parser splits them on `-`.
        let mut relation_parts = Vec::new();
        for item in &items_to_update {
            relation_parts.push(format!(
                "RELATE {}->owns->item:⟨{}⟩",
                owner, item.identifier
            ));
        }

        let bulk_relations = relation_parts.join(";\n");
        db.query(&bulk_relations).await.map_err(|e| {
            AppError::InternalServerError(format!("Failed to batch create relations: {}", e))
        })?;
    }

    Ok(())
}

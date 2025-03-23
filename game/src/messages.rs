use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::VecDeque;
use std::fmt::Display;
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag="type", content="value")]
pub enum MessageSource {
    #[serde(rename = "Game")]
    Game(String), // Game identifier
    #[serde(rename = "Area")]
    Area(String), // Area name
    #[serde(rename = "Tribute")]
    Tribute(String), // Tribute identifier
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GameMessage {
    pub identifier: String,
    pub source: MessageSource,
    pub game_day: u32,
    pub subject: String,
    #[serde(with = "chrono::serde::ts_nanoseconds")]
    pub timestamp: DateTime<Utc>,
    pub content: String,
}

pub static GLOBAL_MESSAGES: Lazy<Mutex<VecDeque<GameMessage>>> =
    Lazy::new(|| Mutex::new(VecDeque::new()));

pub fn add_message(
    source: MessageSource,
    game_day: u32,
    subject: String,
    content: String,
) -> Result<(), String> {
    let message = GameMessage {
        identifier: Uuid::new_v4().to_string(),
        source,
        game_day,
        subject,
        timestamp: Utc::now(),
        content,
    };

    GLOBAL_MESSAGES
        .lock()
        .map_err(|e| e.to_string())?
        .push_back(message);

    Ok(())
}

pub fn add_game_message(game_id: &str, content: String) -> Result<(), String> {
    add_message(
        MessageSource::Game(game_id.to_string()),
        0,
        game_id.to_string(),
        content,
    )
}

pub fn add_area_message(area_name: &str, game_id: &str, content: String) -> Result<(), String> {
    add_message(
        MessageSource::Area(area_name.to_string()),
        0,
        format!("{game_id}:{area_name}"),
        content,
    )
}

pub fn add_tribute_message(tribute_id: &str, game_id: &str, content: String) -> Result<(), String> {
    add_message(
        MessageSource::Tribute(tribute_id.to_string()),
        0,
        format!("{game_id}:{tribute_id}"),
        content,
    )
}

pub fn get_all_messages() -> Result<Vec<GameMessage>, String> {
    Ok(GLOBAL_MESSAGES
        .lock()
        .map_err(|e| e.to_string())?
        .iter()
        .cloned()
        .collect())
}

pub fn get_messages_by_source(source: &MessageSource) -> Result<Vec<GameMessage>, String> {
    Ok(GLOBAL_MESSAGES
        .lock()
        .map_err(|e| e.to_string())?
        .iter()
        .filter(|msg| msg.source == *source)
        .cloned()
        .collect())
}

pub fn get_messages_by_day(day: u32) -> Result<Vec<GameMessage>, String> {
    Ok(GLOBAL_MESSAGES
        .lock()
        .map_err(|e| e.to_string())?
        .iter()
        .filter(|msg| msg.game_day == day)
        .cloned()
        .collect())
}

pub fn clear_messages() -> Result<(), String> {
    GLOBAL_MESSAGES
        .lock()
        .map_err(|e| e.to_string())?
        .clear();
    Ok(())
}


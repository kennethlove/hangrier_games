use std::cmp::PartialEq;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum LogContext {
    Game,
    Area,
    Tribute
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct LogMessage {
    pub context: LogContext,
    pub instant: u128,
    pub message: String,
    pub subject: String,
    // pub day: u32,
}

pub mod approx_instant {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use serde::de::Error;
    use std::time::{Duration, Instant};

    pub fn serialize<S>(instant: &Instant, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let duration = instant.elapsed();
        duration.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Instant, D::Error>
    where
        D: Deserializer<'de>,
    {
        let duration = Duration::deserialize(deserializer)?;
        let now = Instant::now();
        let instant = now.checked_sub(duration).ok_or_else(|| Error::custom("Error checked_add"))?;
        Ok(instant)
    }
}

lazy_static! {
    // HISTORY is for collected log messages
    static ref HISTORY: Mutex<Vec<LogMessage>> = Mutex::new(Vec::new());
}

async fn add_history(log_message: LogMessage) {
    HISTORY.lock().await.push(log_message);
}

pub async fn add_to_story(story: String, game_identifier: &String) {
    add_history( LogMessage {
            context: LogContext::Game,
            message: story,
            subject: game_identifier.clone(),
            instant: std::time::UNIX_EPOCH.elapsed().unwrap().as_nanos()
        }
    ).await;
}

pub async fn get_story() -> Vec<LogMessage> {
    let mut stories: Vec<LogMessage> = HISTORY.lock().await.clone()
        .iter()
        .filter(|s| s.context == LogContext::Game)
        .cloned()
        .collect();
    stories.sort_by_key(|s| s.instant );
    stories
    // stories.iter()
    //     .map(|story| story.message.clone())
    //     .collect()
}

pub async fn clear_story() -> Result<(), String> {
    // TODO
    Ok(())
}

pub async fn add_to_lore(lore: String, tribute_identifier: &String) {
    add_history( LogMessage {
            context: LogContext::Tribute,
            message: lore,
            subject: tribute_identifier.clone(),
            instant: std::time::UNIX_EPOCH.elapsed().unwrap().as_nanos()
        }
    ).await;
}

pub async fn get_lore() -> Vec<String> {
    let mut lore: Vec<LogMessage> = HISTORY.lock().await.clone()
        .iter()
        .filter(|l| l.context == LogContext::Tribute)
        .cloned()
        .collect();
    lore.sort_by_key(|l| l.instant );
    lore.iter()
        .map( |lore| lore.message.clone() )
        .collect()
}

pub async fn clear_lore() -> Result<(), String> {
    // TODO
    Ok(())
}

pub async fn add_to_guide(message: String, area_identifier: &String) {
    add_history( LogMessage {
        context: LogContext::Area,
        message,
        subject: area_identifier.clone(),
        instant: std::time::UNIX_EPOCH.elapsed().unwrap().as_nanos()
    }
    ).await;
}

pub async fn get_guide() -> Vec<String> {
    let mut guides: Vec<LogMessage> = HISTORY.lock().await.clone()
        .iter()
        .filter(|g| g.context == LogContext::Area)
        .cloned()
        .collect();
    guides.sort_by_key(|g| g.instant );
    guides.iter()
        .map(|guide| guide.message.clone())
        .collect()
}


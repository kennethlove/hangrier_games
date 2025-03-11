use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct LogMessage {
    message: String,
    instant: u128
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
    // STORY is for game log messages
    static ref STORY: Mutex<Vec<LogMessage>> = Mutex::new(Vec::new());
}

lazy_static! {
    // LORE is for tribute log messages
    static ref LORE: Mutex<Vec<LogMessage>> = Mutex::new(Vec::new());
}

pub async fn add_to_story(story: String) {
    STORY.lock().await.push(
        LogMessage {
            message: story,
            instant: std::time::UNIX_EPOCH.elapsed().unwrap().as_nanos()
        }
    );
}

pub async fn get_story() -> Vec<String> {
    let mut stories = STORY.lock().await.clone();
    stories.sort_by_key(|s| s.instant );
    stories.iter()
        .map( |story| story.message.clone() )
        .collect()
}

pub async fn clear_story() -> Result<(), String> {
    STORY.lock().await.clear();
    Ok(())
}

pub async fn add_to_lore(lore: String) {
    LORE.lock().await.push(
        LogMessage {
            message: lore,
            instant: std::time::UNIX_EPOCH.elapsed().unwrap().as_nanos()
        }
    );
}

pub async fn get_lore() -> Vec<String> {
    let mut lore = LORE.lock().await.clone();
    lore.sort_by_key(|s| s.instant );
    lore.iter()
        .map( |lore| lore.message.clone() )
        .collect()
}

pub async fn clear_lore() -> Result<(), String> {
    LORE.lock().await.clear();
    Ok(())
}

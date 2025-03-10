use lazy_static::lazy_static;
use tokio::sync::Mutex;


lazy_static! {
    // STORY is for game log messages
    static ref STORY: Mutex<Vec<String>> = Mutex::new(Vec::new());
}

lazy_static! {
    // LORE is for tribute log messages
    static ref LORE: Mutex<Vec<String>> = Mutex::new(Vec::new());
}

pub async fn add_to_story(story: String) {
    STORY.lock().await.push(story);
}

pub async fn get_story() -> Vec<String> {
    STORY.lock().await.clone()
}

pub async fn clear_story() -> Result<(), String> {
    STORY.lock().await.clear();
    Ok(())
}

pub async fn add_to_lore(lore: String) {
    LORE.lock().await.push(lore);
}

pub async fn get_lore() -> Vec<String> {
    LORE.lock().await.clone()
}

pub async fn clear_lore() -> Result<(), String> {
    LORE.lock().await.clear();
    Ok(())
}

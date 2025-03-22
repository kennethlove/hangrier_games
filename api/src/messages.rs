use game::messages::{GLOBAL_MESSAGES, GameMessage};
use std::collections::VecDeque;
use crate::DATABASE;

// Database operations
pub async fn save_global_messages_to_db() -> Result<(), String> {
    let messages = GLOBAL_MESSAGES.lock().map_err(|e| e.to_string())?;
    for message in messages.iter() {
        todo!();
    }
    Ok(())
}

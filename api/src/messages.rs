use game::messages::GLOBAL_MESSAGES;

// Database operations
pub async fn save_global_messages_to_db() -> Result<(), String> {
    let messages = GLOBAL_MESSAGES.lock().map_err(|e| e.to_string())?;
    for _message in messages.iter() {
        todo!();
    }
    Ok(())
}

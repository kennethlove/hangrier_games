use std::sync::{Arc, LazyLock};
use tokio::sync::Mutex as AsyncMutex;

pub mod areas;
pub mod games;
pub mod items;
pub mod messages;
pub mod threats;
pub mod tributes;

// STORY is for game log messages
pub static STORY: LazyLock<Arc<AsyncMutex<Vec<String>>>> = LazyLock::new(|| Arc::new(AsyncMutex::new(Vec::<String>::new())));

// LORE is for tribute log messages
pub static LORE: LazyLock<Arc<AsyncMutex<Vec<String>>>> = LazyLock::new(|| Arc::new(AsyncMutex::new(Vec::<String>::new())));

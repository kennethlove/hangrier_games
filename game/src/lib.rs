use std::sync::{Arc, LazyLock, Mutex};
use tokio::sync::Mutex as AsyncMutex;

pub mod areas;
pub mod games;
pub mod items;
pub mod messages;
pub mod threats;
pub mod tributes;

// pub static STORY: LazyLock<Arc<T Mutex<Vec<String>> = LazyLock::new(Arc::new(Mutex::new(Vec::new())));

pub static STORY: LazyLock<Arc<AsyncMutex<Vec<String>>>> = LazyLock::new(|| Arc::new(AsyncMutex::new(Vec::<String>::new())));

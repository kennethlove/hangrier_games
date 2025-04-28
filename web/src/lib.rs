use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

mod cache;
pub mod components;
mod routes;
mod storage;

pub static API_HOST: LazyLock<String> = LazyLock::new(|| {
    std::env::var("API_HOST").unwrap_or_else(|_| "http://localhost:3000".to_string())
});

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub enum LoadingState {
    #[default]
    Unloaded,
    Loading,
    Loaded,
}

use serde::{Deserialize, Serialize};

mod cache;
pub mod components;
mod routes;
mod storage;

pub static API_HOST: &str = env!("API_HOST");

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub enum LoadingState {
    #[default]
    Unloaded,
    Loading,
    Loaded,
}

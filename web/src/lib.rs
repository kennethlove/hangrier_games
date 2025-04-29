use serde::{Deserialize, Serialize};

mod cache;
pub mod components;
pub(crate) mod env;
mod routes;
mod storage;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub enum LoadingState {
    #[default]
    Unloaded,
    Loading,
    Loaded,
}

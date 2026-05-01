use serde::{Deserialize, Serialize};

pub mod auth;
mod cache;
pub mod components;
pub(crate) mod env;
pub mod hooks;
mod routes;
mod storage;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub enum LoadingState {
    #[default]
    Unloaded,
    Loading,
    Loaded,
    Error,
}

use serde::{Deserialize, Serialize};

pub mod api_url;
mod cache;
pub mod components;
pub(crate) mod env;
pub mod hooks;
pub mod http;
mod routes;
mod storage;
pub mod icons;
pub mod theme;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub enum LoadingState {
    #[default]
    Unloaded,
    Loading,
    Loaded,
    Error,
}

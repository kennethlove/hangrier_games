pub mod areas;
pub mod config;
pub mod districts;
pub mod games;
pub mod items;
pub mod messages;
pub mod output;
pub mod terrain;
pub mod threats;
pub mod tributes;
mod witty_phrase_generator;

// Re-export key terrain types
pub use terrain::{BaseTerrain, TerrainDescriptor, TerrainType};

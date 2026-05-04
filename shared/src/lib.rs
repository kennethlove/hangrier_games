use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};
use std::str::FromStr;
use validator::{Validate, ValidationError};

pub mod combat_beat;
pub mod messages;

use crate::combat_beat::CombatBeat;
use crate::messages::TributeRef;

/// WebSocket message protocol for real-time game updates
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    /// Client subscribes to game updates
    Subscribe { game_id: String },
    /// Client unsubscribes from game updates
    Unsubscribe { game_id: String },
    /// Server sends game event to subscribed clients
    GameEvent { game_id: String, event: GameEvent },
    /// Server sends error message
    Error { message: String },
}

/// Real-time game events broadcast to WebSocket clients
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "event_type")]
pub enum GameEvent {
    /// Game has started
    GameStarted { day: u32 },
    /// Game has finished
    GameFinished { winner: Option<String> },
    /// New day has started
    DayStarted { day: u32 },
    /// Night phase has started
    NightStarted { day: u32 },
    /// Tribute died
    TributeDied {
        tribute_id: String,
        name: String,
        cause: String,
    },
    /// Area event occurred
    AreaEvent { area: String, event: String },
    /// Combat swing occurred
    Combat { beat: Box<CombatBeat> },
    /// Generic message (tribute action, announcement, etc.)
    Message {
        source: String,
        content: String,
        game_day: u32,
    },
}

/// Item quantity preset for game customization.
/// Controls the base number of items spawned in each area.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub enum ItemQuantity {
    Scarce, // 1-2 items per area
    #[default]
    Normal, // 3 items per area (default)
    Abundant, // 5-6 items per area
}

impl ItemQuantity {
    /// Returns the base item count for an area based on this preset.
    pub fn base_item_count(&self) -> u32 {
        match self {
            ItemQuantity::Scarce => 1,
            ItemQuantity::Normal => 3,
            ItemQuantity::Abundant => 5,
        }
    }
}

/// Event frequency preset for game customization.
/// Controls how often random events occur during the game.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub enum EventFrequency {
    Rare, // 10% event probability per turn
    #[default]
    Normal, // 25% event probability per turn (default)
    Frequent, // 50% event probability per turn
}

impl EventFrequency {
    /// Returns the probability (0.0 - 1.0) of an event occurring per turn.
    pub fn event_probability(&self) -> f32 {
        match self {
            EventFrequency::Rare => 0.1,
            EventFrequency::Normal => 0.25,
            EventFrequency::Frequent => 0.5,
        }
    }
}

/// Custom validator to ensure a string is a valid UUID
fn validate_uuid(value: &str) -> Result<(), ValidationError> {
    uuid::Uuid::parse_str(value)
        .map(|_| ())
        .map_err(|_| ValidationError::new("invalid_uuid"))
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateGame {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Game name must be between 1 and 100 characters"
    ))]
    pub name: Option<String>,

    /// Item spawn quantity preset (Scarce, Normal, Abundant)
    #[serde(default)]
    pub item_quantity: ItemQuantity,

    /// Random event frequency preset (Rare, Normal, Frequent)
    #[serde(default)]
    pub event_frequency: EventFrequency,

    /// Starting health range for tributes (optional, defaults to 80-100)
    pub starting_health_range: Option<(u32, u32)>,
}

pub type DeleteTribute = String;
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct DeleteGame(pub String, pub String); // Identifier, name

/// Used to edit a tribute. Contains the identifier, name, avatar, and game identifier of the tribute.
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize, Validate)]
pub struct EditTribute {
    #[validate(custom(function = "validate_uuid"))]
    pub identifier: String,
    #[validate(length(min = 1, max = 50, message = "Name must be 1-50 characters"))]
    pub name: String,
    #[validate(length(max = 500, message = "Avatar URL must be 500 characters or less"))]
    pub avatar: String,
    #[validate(custom(function = "validate_uuid"))]
    pub game_identifier: String,
}

impl EditTribute {
    /// Create EditTribute from tuple format (for backward compatibility)
    pub fn from_tuple(data: (String, String, String, String)) -> Self {
        Self {
            identifier: data.0,
            name: data.1,
            avatar: data.2,
            game_identifier: data.3,
        }
    }

    /// Convert to tuple format
    pub fn to_tuple(&self) -> (String, String, String, String) {
        (
            self.identifier.clone(),
            self.name.clone(),
            self.avatar.clone(),
            self.game_identifier.clone(),
        )
    }
}

/// This struct is used to edit a game
/// It contains the identifier, name, and a boolean indicating if the game is private
#[derive(Clone, Debug, Default, Serialize, Deserialize, Eq, Hash, PartialEq, Validate)]
pub struct EditGame {
    #[validate(custom(function = "validate_uuid"))]
    pub identifier: String,
    #[validate(length(min = 1, max = 100, message = "Name must be 1-100 characters"))]
    pub name: String,
    pub private: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GameArea {
    pub identifier: String,
    pub name: String,
    pub area: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize, Validate)]
pub struct RegistrationUser {
    #[validate(length(min = 3, max = 50, message = "Username must be 3-50 characters"))]
    pub username: String,
    #[validate(length(min = 8, max = 72, message = "Password must be 8-72 characters"))]
    pub password: String,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct AuthenticatedUser {
    // The API returns this field as `access_token` (see api::auth::TokenResponse).
    // We keep the in-memory field name `jwt` to avoid touching every call site,
    // but accept both names on the wire via serde alias and emit `access_token`
    // when serializing so future producers stay compatible with the API contract.
    #[serde(rename = "access_token", alias = "jwt")]
    pub jwt: String,
    #[serde(default)]
    pub refresh_token: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize)]
pub enum GameStatus {
    #[default]
    NotStarted,
    InProgress,
    Finished,
}

impl Display for GameStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameStatus::NotStarted => write!(f, "NotStarted"),
            GameStatus::InProgress => write!(f, "InProgress"),
            GameStatus::Finished => write!(f, "Finished"),
        }
    }
}

impl FromStr for GameStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "not started" => Ok(GameStatus::NotStarted),
            "notstarted" => Ok(GameStatus::NotStarted),
            "in progress" => Ok(GameStatus::InProgress),
            "inprogress" => Ok(GameStatus::InProgress),
            "finished" => Ok(GameStatus::Finished),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct DisplayGame {
    pub identifier: String,
    pub name: String,
    pub status: GameStatus,
    pub day: Option<u32>,
    #[serde(default)]
    pub tribute_count: u32,
    #[serde(default)]
    pub living_count: u32,
    #[serde(default)]
    pub ready: bool,
    #[serde(default)]
    pub private: bool,
    #[serde(default)]
    pub is_mine: bool,
    pub created_by: CreatedBy,
    #[serde(default)]
    pub winner: Option<TributeRef>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct CreatedBy {
    pub username: String,
}

/// Authenticated user identity returned by `GET /api/users/session`. Reads
/// the per-request `$auth` SurrealDB record so the caller learns who they
/// are without the frontend having to decode the JWT itself.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct UserSession {
    /// SurrealDB record id (e.g. `user:abc123`) as a string. Useful for
    /// cache keys and for matching `created_by` ownership on games.
    pub id: String,
    pub username: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ListDisplayGame {
    pub identifier: String,
    pub name: String,
    pub status: GameStatus,
    pub day: Option<u32>,
    #[serde(default)]
    pub tribute_count: u32,
    #[serde(default)]
    pub living_count: u32,
    #[serde(default)]
    pub ready: bool,
    #[serde(default)]
    pub private: bool,
    #[serde(default)]
    pub is_mine: bool,
    pub created_by: CreatedBy,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct PaginationMetadata {
    pub total: u32,
    pub limit: u32,
    pub offset: u32,
    pub has_more: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct PaginatedGames {
    pub games: Vec<ListDisplayGame>,
    pub pagination: PaginationMetadata,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_username_validation() {
        let user = RegistrationUser {
            username: "ab".to_string(), // Too short (min 3)
            password: "password123".to_string(),
        };
        let result = user.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_uuid_validation() {
        let tribute = EditTribute {
            identifier: "not-a-uuid".to_string(),
            name: "Test Tribute".to_string(),
            avatar: "avatar.png".to_string(),
            game_identifier: "also-not-a-uuid".to_string(),
        };
        let result = tribute.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_password_max_length() {
        let user = RegistrationUser {
            username: "testuser".to_string(),
            password: "a".repeat(73), // Exceeds max of 72
        };
        let result = user.validate();
        assert!(result.is_err());
    }
}

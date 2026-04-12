use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};
use std::str::FromStr;
use validator::{Validate, ValidationError};

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
}

pub type DeleteTribute = String;
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct DeleteGame(pub String, pub String); // Identifier, name

/// Used to edit a tribute. Contains the identifier, name, avatar, and game identifier of the tribute.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, Validate)]
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
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Validate)]
pub struct EditGame {
    #[validate(custom(function = "validate_uuid"))]
    pub identifier: String,
    #[validate(length(min = 1, max = 100, message = "Name must be 1-100 characters"))]
    pub name: String,
    pub private: bool,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct GameArea {
    pub identifier: String,
    pub name: String,
    pub area: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TributeKey {
    pub identifier: String,
    pub district: u32,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, Validate)]
pub struct RegistrationUser {
    #[validate(length(min = 3, max = 50, message = "Username must be 3-50 characters"))]
    pub username: String,
    #[validate(length(min = 8, max = 72, message = "Password must be 8-72 characters"))]
    pub password: String,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct AuthenticatedUser {
    pub jwt: String,
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
    pub winner: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct CreatedBy {
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

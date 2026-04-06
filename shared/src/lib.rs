use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};
use std::str::FromStr;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateGame {
    #[validate(length(min = 1, max = 100, message = "Game name must be 1-100 characters"))]
    pub name: Option<String>,
}

pub type DeleteTribute = String;
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct DeleteGame(pub String, pub String); // Identifier, name

/// Used to edit a tribute. Contains the identifier, name, avatar, and game identifier of the tribute.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, Validate)]
pub struct EditTribute(
    #[validate(length(min = 1, message = "Tribute identifier required"))]
    pub String,
    #[validate(length(min = 1, max = 50, message = "Name must be 1-50 characters"))]
    pub String,
    pub String,
    pub String,
);

/// This struct is used to edit a game
/// It contains the identifier, name, and a boolean indicating if the game is private
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Validate)]
pub struct EditGame(
    #[validate(length(min = 1, message = "Game identifier required"))]
    pub String,
    #[validate(length(min = 1, max = 100, message = "Name must be 1-100 characters"))]
    pub String,
    pub bool,
);

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

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct RegistrationUser {
    pub username: String,
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

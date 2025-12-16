use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};
use std::str::FromStr;

/// Information required to construct a new game.
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateGame {
    pub name: Option<String>,
}

/// Tribute identifier to use for deletion.
pub type DeleteTribute = String;

/// Information required to delete a game.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct DeleteGame(pub String, pub String); // Identifier, name

/// Information used to edit a tribute.
/// Contains the identifier, name, avatar, and game identifier of the tribute.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct EditTribute(pub String, pub String, pub String, pub String);

/// Information used to edit a game.
/// Contains the identifier, name, and a boolean indicating if the game is private
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct EditGame(pub String, pub String, pub bool);

/// Each area in the game (i.e. North, Cornucopia, 7th Circle), has an identifier, a name, and which of the five core areas it represents.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct GameArea {
    pub identifier: String,
    pub name: String,
    pub area: String,
}

/// Tributes have an identifier and also which district they belong to.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TributeKey {
    pub identifier: String,
    pub district: u32,
}

/// The information required to register a user.
/// Currently, users must provide a username and a password.
/// There is currently no validation for the password.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct RegistrationUser {
    pub username: String,
    pub password: String,
}

/// Authenticated users carry around a JWT.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct AuthenticatedUser {
    pub jwt: String,
}

/// The three states a [game](Game) can be in:
///
/// `NotStarted` means the game has been created but has not started day zero.
/// `InProgress` means the game is at or past day zero but doesn't yet have a winner.
/// `Finished` indicates the game has had a winner or all tributes have died.
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

/// The data expected to be available when rendering a [game](Game).
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

/// Which user created the [Game].
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct CreatedBy {
    pub username: String,
}

/// Data available when rendering a [Game] in a list.
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

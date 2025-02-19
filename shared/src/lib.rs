use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateGame {
    pub name: Option<String>,
}

pub type DeleteTribute = String;
pub type DeleteGame = String;

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct EditTribute(pub String, pub u32, pub String); // Identifier, district, name

#[derive(Clone, Debug, Default, Serialize)]
pub struct EditGame(pub String, pub String); // Identifier, name

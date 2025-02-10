use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateGame {
    pub name: Option<String>,
}

pub type DeleteTribute = String;
pub type DeleteGame = String;

// Name, District, Identifier
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct EditTribute(pub String, pub u8, pub String);

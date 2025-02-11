use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateGame {
    pub name: Option<String>,
}

pub type DeleteTribute = String;
pub type DeleteGame = String;

// Name, District, Identifier
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct EditTribute(pub String, pub u32, pub String);

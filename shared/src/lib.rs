use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateGame {
    pub name: Option<String>,
}

pub type DeleteTribute = String;
pub type DeleteGame = String;

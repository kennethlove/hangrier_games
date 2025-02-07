use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateGame {
    pub name: Option<String>,
}

pub type DeleteTribute = String;

#[derive(Debug, Clone)]
pub struct DeleteGame(pub String);

// #[derive(Debug, Clone, Default)]
// pub struct DeleteTribute(pub String);

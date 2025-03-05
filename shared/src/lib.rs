use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::Debug;
use serde::{Deserialize, Serialize};
use tracing::{Event, Id};
use tracing::field::Field;
use tracing::span::{Attributes};
use tracing_subscriber::prelude::*;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateGame {
    pub name: Option<String>,
}

pub type DeleteTribute = String;
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct DeleteGame(pub String, pub String); // Identifier, name

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct EditTribute(pub String, pub u32, pub String); // Identifier, district, name

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct EditGame(pub String, pub String); // Identifier, name

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


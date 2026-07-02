use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use uuid::Uuid;

macro_rules! define_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            pub fn parse_str(s: &str) -> Result<Self, uuid::Error> {
                Ok(Self(Uuid::parse_str(s)?))
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl FromStr for $name {
            type Err = uuid::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self(Uuid::parse_str(s)?))
            }
        }

        impl From<Uuid> for $name {
            fn from(uuid: Uuid) -> Self {
                Self(uuid)
            }
        }

        impl From<$name> for Uuid {
            fn from(id: $name) -> Self {
                id.0
            }
        }

        impl Deref for $name {
            type Target = Uuid;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };
}

define_id!(
    /// Unique identifier for a tribute in the game.
    TributeId
);

define_id!(
    /// Unique identifier for a game.
    GameId
);

define_id!(
    /// Unique identifier for an area in the arena.
    AreaId
);

define_id!(
    /// Unique identifier for an item.
    ItemId
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tribute_id_round_trip() {
        let id = TributeId::new();
        let s = id.to_string();
        let parsed: TributeId = s.parse().unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn game_id_from_uuid() {
        let uuid = Uuid::new_v4();
        let id = GameId::from(uuid);
        assert_eq!(*id, uuid);
    }

    #[test]
    fn area_id_deref() {
        let id = AreaId::new();
        let uuid_val: Uuid = *id;
        assert_eq!(id.0, uuid_val);
    }

    #[test]
    fn item_id_serde() {
        let id = ItemId::new();
        let json = serde_json::to_string(&id).unwrap();
        let parsed: ItemId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, parsed);
    }
}

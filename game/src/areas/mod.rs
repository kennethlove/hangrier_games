pub mod events;

use crate::areas::events::AreaEvent;
use crate::items::{Item, ItemError};
use crate::items::OwnsItems;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;
use strum_macros::EnumIter;
use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq, EnumIter, Hash, Deserialize, Serialize, Ord, PartialOrd)]
pub enum Area {
    Cornucopia,
    North,
    East,
    South,
    West,
}

impl Display for Area {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Area::Cornucopia => f.write_str("Cornucopia"),
            Area::North => f.write_str("North"),
            Area::East => f.write_str("East"),
            Area::South => f.write_str("South"),
            Area::West => f.write_str("West"),
        }
    }
}

impl PartialEq<&Area> for Area {
    fn eq(&self, other: &&Area) -> bool {
        *self == **other
    }
}

impl FromStr for Area {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "north" => Ok(Area::North),
            "east" => Ok(Area::East),
            "south" => Ok(Area::South),
            "west" => Ok(Area::West),
            _ => Ok(Area::Cornucopia)
        }
    }
}

impl Area {
    pub fn neighbors(&self) -> Vec<Area> {
        match self {
            Area::North => vec![Area::Cornucopia, Area::East, Area::West],
            Area::East => vec![Area::Cornucopia, Area::North, Area::South],
            Area::South => vec![Area::Cornucopia, Area::East, Area::West],
            Area::West => vec![Area::Cornucopia, Area::North, Area::South],
            Area::Cornucopia => vec![Area::North, Area::East, Area::South, Area::West]
        }
    }
}

#[derive(Clone, Serialize, Default, Deserialize, Debug, PartialEq)]
pub struct AreaDetails {
    pub identifier: String,
    pub name: String,
    pub area: String,
    #[serde(default)]
    pub items: Vec<Item>,
    #[serde(default)]
    pub events: Vec<AreaEvent>,
}

impl OwnsItems for AreaDetails {
    fn add_item(&mut self, item: Item) {
        self.items.push(item);
    }

    fn has_item(&self, item: &Item) -> bool {
        self.items.iter().any(|i| i == item)
    }

    fn use_item(&mut self, item: Item) -> Result<(), ItemError> {
        let index = self.items.iter().position(|i| *i == item);
        let mut used_item = self.items.swap_remove(index.unwrap());

        if used_item.quantity > 0 {
            let item = used_item.clone();
            used_item.quantity = used_item.quantity.saturating_sub(1);

            if used_item.quantity == 0 {
                self.remove_item(used_item);
            }
            Ok(())
        } else {
            Err(ItemError::ItemNotFound)
        }
    }

    fn remove_item(&mut self, item: Item) -> Result<(), ItemError> {
        self.items.retain(|i| *i.identifier != item.identifier);
        Ok(())
    }
}

impl AreaDetails {
    pub fn new(name: Option<String>, area: Area) -> Self {
        Self {
            identifier: Uuid::new_v4().to_string(),
            name: name.unwrap_or(area.to_string()),
            area: area.to_string(),
            items: vec![],
            events: vec![],
        }
    }

    pub fn is_open(&self) -> bool {
        self.events.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str() {
        let area = Area::from_str("cornucopia");
        assert_eq!(area.unwrap(), Area::Cornucopia);
    }

    #[test]
    fn to_str() {
        assert_eq!(Area::Cornucopia.to_string(), "Cornucopia");
    }

    #[test]
    fn add_item() {
        let mut area_details = AreaDetails::new(None, Area::South);
        let item = Item::new_random_weapon();
        area_details.add_item(item.clone());
        assert!(area_details.items.contains(&item));
    }

    #[test]
    fn remove_item() {
        let mut area_details = AreaDetails::new(None, Area::South);
        let item = Item::new_random_weapon();
        area_details.add_item(item.clone());
        assert!(area_details.items.contains(&item));
        area_details.remove_item(item.clone());
        assert!(!area_details.items.contains(&item));
    }

    #[test]
    fn add_event() {
        let mut area_details = AreaDetails::new(None, Area::North);
        let event = AreaEvent::Wildfire;
        area_details.events.push(event.clone());
        assert!(area_details.events.contains(&event));
    }

    #[test]
    fn process_events_closes_area() {
        let mut area_details = AreaDetails::new(None, Area::North);
        assert!(area_details.is_open());
        let event = AreaEvent::Wildfire;
        area_details.events.push(event.clone());
        assert!(!area_details.is_open());
    }

    #[test]
    fn partial_eq_with_reference() {
        let area = Area::Cornucopia;
        assert_eq!(area, &area);
    }
}

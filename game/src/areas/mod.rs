pub mod events;

use crate::areas::events::AreaEvent;
use crate::items::Item;
use crate::items::OwnsItems;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;
use strum_macros::EnumIter;
use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq, EnumIter, Hash, Deserialize, Serialize, Ord, PartialOrd)]
pub enum Area {
    Cornucopia,
    Northwest,
    Northeast,
    Southeast,
    Southwest,
}

impl Display for Area {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Area::Cornucopia => f.write_str("Cornucopia"),
            Area::Northwest => f.write_str("Northwest"),
            Area::Northeast => f.write_str("Northeast"),
            Area::Southeast => f.write_str("Southeast"),
            Area::Southwest => f.write_str("Southwest"),
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
            "northwest" => Ok(Area::Northwest),
            "northeast" => Ok(Area::Northeast),
            "southeast" => Ok(Area::Southeast),
            "southwest" => Ok(Area::Southwest),
            _ => Ok(Area::Cornucopia)
        }
    }
}

impl Area {
    pub fn neighbors(&self) -> Vec<Area> {
        match self {
            Area::Southeast => vec![Area::Cornucopia, Area::Northeast, Area::Southwest],
            Area::Southwest => vec![Area::Cornucopia, Area::Northwest, Area::Southeast],
            Area::Northeast => vec![Area::Cornucopia, Area::Northwest, Area::Southeast],
            Area::Northwest => vec![Area::Cornucopia, Area::Northeast, Area::Southwest],
            Area::Cornucopia => vec![Area::Northwest, Area::Northeast, Area::Southwest, Area::Southeast],
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

    fn use_item(&mut self, item: Item) -> Option<Item> {
        let index = self.items.iter().position(|i| *i == item);
        let mut used_item = self.items.swap_remove(index.unwrap());

        if used_item.quantity > 0 {
            let item = used_item.clone();
            used_item.quantity = used_item.quantity.saturating_sub(1);

            if used_item.quantity == 0 {
                self.remove_item(used_item);
            }
            return Some(item)
        }
        None
    }

    fn remove_item(&mut self, item: Item) {
        self.items.retain(|i| *i.identifier != item.identifier);
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

    pub fn open(&self) -> bool {
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
        let mut area_details = AreaDetails::new(None, Area::Southeast);
        let item = Item::new_random_weapon();
        area_details.add_item(item.clone());
        assert!(area_details.items.contains(&item));
    }

    #[test]
    fn remove_item() {
        let mut area_details = AreaDetails::new(None, Area::Southeast);
        let item = Item::new_random_weapon();
        area_details.add_item(item.clone());
        assert!(area_details.items.contains(&item));
        area_details.remove_item(item.clone());
        assert!(!area_details.items.contains(&item));
    }

    #[test]
    fn add_event() {
        let mut area_details = AreaDetails::new(None, Area::Northeast);
        let event = AreaEvent::Wildfire;
        area_details.events.push(event.clone());
        assert!(area_details.events.contains(&event));
    }

    #[test]
    fn process_events_closes_area() {
        let mut area_details = AreaDetails::new(None, Area::Northeast);
        assert!(area_details.open());
        let event = AreaEvent::Wildfire;
        area_details.events.push(event.clone());
        assert!(!area_details.open());
    }

    #[test]
    fn partial_eq_with_reference() {
        let area = Area::Cornucopia;
        assert_eq!(area, &area);
    }
}

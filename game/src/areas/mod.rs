pub mod events;
pub mod hex;

use crate::areas::events::AreaEvent;
use crate::areas::hex::{SUB_SLOTS, SubAxial};
use crate::items::OwnsItems;
use crate::items::{Item, ItemError};
use crate::terrain::{BaseTerrain, TerrainType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;
use std::str::FromStr;
use strum_macros::EnumIter;
use uuid::Uuid;

#[derive(
    Copy,
    Clone,
    Debug,
    Eq,
    PartialEq,
    EnumIter,
    Hash,
    Deserialize,
    Serialize,
    Ord,
    PartialOrd,
    Default,
)]
pub enum Area {
    #[default]
    Cornucopia,
    Sector1,
    Sector2,
    Sector3,
    Sector4,
    Sector5,
    Sector6,
}

impl Display for Area {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Area::Cornucopia => f.write_str("Cornucopia"),
            Area::Sector1 => f.write_str("Sector 1"),
            Area::Sector2 => f.write_str("Sector 2"),
            Area::Sector3 => f.write_str("Sector 3"),
            Area::Sector4 => f.write_str("Sector 4"),
            Area::Sector5 => f.write_str("Sector 5"),
            Area::Sector6 => f.write_str("Sector 6"),
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
            "cornucopia" => Ok(Area::Cornucopia),
            "sector 1" | "sector1" => Ok(Area::Sector1),
            "sector 2" | "sector2" => Ok(Area::Sector2),
            "sector 3" | "sector3" => Ok(Area::Sector3),
            "sector 4" | "sector4" => Ok(Area::Sector4),
            "sector 5" | "sector5" => Ok(Area::Sector5),
            "sector 6" | "sector6" => Ok(Area::Sector6),
            _ => Err(format!("Invalid area: {}", s)),
        }
    }
}

impl Area {
    /// Topological neighbors derived from the v1 hex layout: Cornucopia
    /// touches all six sectors, each sector touches Cornucopia plus its
    /// two adjacent sectors (clockwise/counter-clockwise wrap 1..6).
    pub fn neighbors(&self) -> Vec<Area> {
        match self {
            Area::Cornucopia => vec![
                Area::Sector1,
                Area::Sector2,
                Area::Sector3,
                Area::Sector4,
                Area::Sector5,
                Area::Sector6,
            ],
            Area::Sector1 => vec![Area::Cornucopia, Area::Sector6, Area::Sector2],
            Area::Sector2 => vec![Area::Cornucopia, Area::Sector1, Area::Sector3],
            Area::Sector3 => vec![Area::Cornucopia, Area::Sector2, Area::Sector4],
            Area::Sector4 => vec![Area::Cornucopia, Area::Sector3, Area::Sector5],
            Area::Sector5 => vec![Area::Cornucopia, Area::Sector4, Area::Sector6],
            Area::Sector6 => vec![Area::Cornucopia, Area::Sector5, Area::Sector1],
        }
    }
}

/// Information about a destination area that tributes can use to make movement decisions
#[derive(Clone, Debug)]
pub struct DestinationInfo {
    pub area: Area,
    pub terrain: TerrainType,
    pub active_events: Vec<AreaEvent>,
    pub stamina_cost: u32,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct AreaDetails {
    pub identifier: String,
    pub name: String,
    pub area: Option<Area>,
    #[serde(default)]
    pub items: Vec<Item>,
    #[serde(default)]
    pub events: Vec<AreaEvent>,
    #[serde(default = "default_terrain")]
    pub terrain: TerrainType,
    /// Per-tribute sub-tile slot assignments within this area-hex.
    /// Presentation/positioning only — game logic operates at the area
    /// level. Keys are tribute identifiers; values are area-local sub
    /// coordinates from `hex::SUB_SLOTS`.
    #[serde(default)]
    pub tribute_slots: HashMap<String, SubAxial>,
}

fn default_terrain() -> TerrainType {
    TerrainType::new(BaseTerrain::Clearing, vec![]).unwrap()
}

impl Default for AreaDetails {
    fn default() -> Self {
        Self {
            identifier: Uuid::new_v4().to_string(),
            name: String::new(),
            area: None,
            items: vec![],
            events: vec![],
            terrain: TerrainType::new(BaseTerrain::Clearing, vec![]).unwrap(),
            tribute_slots: HashMap::new(),
        }
    }
}

impl OwnsItems for AreaDetails {
    fn add_item(&mut self, item: Item) {
        self.items.push(item);
    }

    fn has_item(&self, item: &Item) -> bool {
        self.items.iter().any(|i| i == item)
    }

    fn use_item(&mut self, item: &Item) -> Result<(), ItemError> {
        let index = self.items.iter().position(|i| i == item);
        let used_item = self.items.swap_remove(index.unwrap());

        if used_item.current_durability > 0 {
            Ok(())
        } else {
            Err(ItemError::ItemNotFound)
        }
    }

    fn remove_item(&mut self, item: &Item) -> Result<(), ItemError> {
        let index = self
            .items
            .iter()
            .position(|i| i.identifier == item.identifier);
        if let Some(index) = index {
            self.items.remove(index);
            Ok(())
        } else {
            Err(ItemError::ItemNotFound)
        }
    }
}

impl AreaDetails {
    pub fn new(name: Option<String>, area: Area) -> Self {
        Self {
            identifier: Uuid::new_v4().to_string(),
            name: name.unwrap_or(area.to_string()),
            area: Some(area),
            items: vec![],
            events: vec![],
            terrain: TerrainType::new(BaseTerrain::Clearing, vec![]).unwrap(),
            tribute_slots: HashMap::new(),
        }
    }

    pub fn new_with_terrain(name: Option<String>, area: Area, terrain: TerrainType) -> Self {
        Self {
            identifier: Uuid::new_v4().to_string(),
            name: name.unwrap_or(area.to_string()),
            area: Some(area),
            items: vec![],
            events: vec![],
            terrain,
            tribute_slots: HashMap::new(),
        }
    }

    pub fn is_open(&self) -> bool {
        self.events.is_empty()
    }

    /// Assign the next available sub-tile slot to `tribute_id`. If the
    /// tribute is already assigned, returns its existing slot. If all 7
    /// slots are taken, falls back to the center slot (overflow — v1
    /// accepts visual stacking past 7 tributes; sub-tiles are
    /// presentation-only).
    pub fn assign_slot(&mut self, tribute_id: &str) -> SubAxial {
        if let Some(slot) = self.tribute_slots.get(tribute_id) {
            return *slot;
        }
        let used: std::collections::HashSet<SubAxial> =
            self.tribute_slots.values().copied().collect();
        let chosen = SUB_SLOTS
            .iter()
            .copied()
            .find(|s| !used.contains(s))
            .unwrap_or(SUB_SLOTS[0]);
        self.tribute_slots.insert(tribute_id.to_string(), chosen);
        chosen
    }

    /// Release the slot currently held by `tribute_id`, if any.
    pub fn release_slot(&mut self, tribute_id: &str) -> Option<SubAxial> {
        self.tribute_slots.remove(tribute_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

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
    fn iter() {
        let areas: Vec<Area> = Area::iter().collect();
        assert_eq!(areas.len(), 7);
        assert_eq!(areas[0], Area::Cornucopia);
        assert_eq!(areas[1], Area::Sector1);
        assert_eq!(areas[2], Area::Sector2);
        assert_eq!(areas[3], Area::Sector3);
        assert_eq!(areas[4], Area::Sector4);
        assert_eq!(areas[5], Area::Sector5);
        assert_eq!(areas[6], Area::Sector6);
    }

    #[test]
    fn add_item() {
        let mut area_details = AreaDetails::new(None, Area::Sector4);
        let item = Item::new_random_weapon();
        area_details.add_item(item.clone());
        assert!(area_details.items.contains(&item));
    }

    #[test]
    fn remove_item() {
        let mut area_details = AreaDetails::new(None, Area::Sector4);
        let item = Item::new_random_weapon();
        area_details.add_item(item.clone());
        assert!(area_details.items.contains(&item));
        area_details.remove_item(&item).unwrap();
        assert!(!area_details.items.contains(&item));
    }

    #[test]
    fn add_event() {
        let mut area_details = AreaDetails::new(None, Area::Sector1);
        let event = AreaEvent::Wildfire;
        area_details.events.push(event.clone());
        assert!(area_details.events.contains(&event));
    }

    #[test]
    fn process_events_closes_area() {
        let mut area_details = AreaDetails::new(None, Area::Sector1);
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

    #[test]
    fn assign_slot_returns_center_first() {
        let mut a = AreaDetails::new(None, Area::Cornucopia);
        let s = a.assign_slot("t1");
        assert_eq!(s, SUB_SLOTS[0]);
    }

    #[test]
    fn assign_slot_is_idempotent_for_same_tribute() {
        let mut a = AreaDetails::new(None, Area::Cornucopia);
        let s1 = a.assign_slot("t1");
        let s2 = a.assign_slot("t1");
        assert_eq!(s1, s2);
        assert_eq!(a.tribute_slots.len(), 1);
    }

    #[test]
    fn assign_slot_gives_unique_slots_until_full() {
        let mut a = AreaDetails::new(None, Area::Cornucopia);
        let mut slots = std::collections::HashSet::new();
        for i in 0..7 {
            let s = a.assign_slot(&format!("t{i}"));
            assert!(slots.insert(s), "duplicate slot {:?} on tribute t{i}", s);
        }
        assert_eq!(slots.len(), 7);
    }

    #[test]
    fn assign_slot_overflows_to_center_when_full() {
        let mut a = AreaDetails::new(None, Area::Cornucopia);
        for i in 0..7 {
            a.assign_slot(&format!("t{i}"));
        }
        let overflow = a.assign_slot("t7");
        assert_eq!(overflow, SUB_SLOTS[0]);
    }

    #[test]
    fn release_slot_frees_slot_for_reassignment() {
        let mut a = AreaDetails::new(None, Area::Cornucopia);
        let original = a.assign_slot("t1");
        let released = a.release_slot("t1");
        assert_eq!(released, Some(original));
        assert!(a.tribute_slots.is_empty());
        let again = a.assign_slot("t2");
        assert_eq!(again, original);
    }
}

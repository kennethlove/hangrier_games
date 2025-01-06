use crate::areas::events::AreaEvent;
use crate::items::Item;
use crate::tributes::statuses::TributeStatus;
use crate::tributes::Tribute;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::fmt::Display;
use std::str::FromStr;

#[serde_as]
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Area {
    pub name: String,
    pub open: bool,
    pub items: Vec<Item>,
    #[serde_as(as = "Vec<DisplayFromStr>")]
    pub neighbors: Vec<Area>,
    pub tributes: Vec<Tribute>,
    pub events: Vec<AreaEvent>,
}

impl Default for Area {
    fn default() -> Self {
        Self {
            name: String::from(""),
            open: true,
            items: vec![],
            neighbors: vec![],
            tributes: vec![],
            events: vec![],
        }
    }
}

impl Display for Area {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
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
        Ok(Area::new(s))
    }
}

impl Area {
    pub fn new(name: &str) -> Self {
        let mut area = Area::default();
        area.name = name.to_string();
        area
    }

    pub fn add_neighbor(&mut self, neighbor: Area) {
        self.neighbors.push(neighbor);
    }

    pub fn add_neighbors(&mut self, neighbors: Vec<&Area>) {
        for neighbor in neighbors {
            self.add_neighbor(neighbor.clone());
        }
    }

    pub fn add_item(&mut self, item: Item) {
        self.items.push(item);
    }

    pub fn remove_item(&mut self, removed_item: &Item) {
        self.items.retain(|item| item != removed_item);
    }

    pub fn add_tribute(&mut self, tribute: Tribute) {
        self.tributes.push(tribute);
    }

    pub fn remove_tribute(&mut self, tribute: &Tribute) {
        self.tributes.retain(|item| item != tribute);
    }

    pub fn add_event(&mut self, event: AreaEvent) {
        self.events.push(event);
    }

    pub fn process_events(&mut self) {
        // If there are events, close the area
        if !self.events.is_empty() {
            self.open = false;
        }

        for event in self.events.iter() {
            for tribute in self.tributes.iter_mut() {
                match event {
                    AreaEvent::Wildfire => tribute.set_status(TributeStatus::Burned),
                    AreaEvent::Flood => tribute.set_status(TributeStatus::Drowned),
                    AreaEvent::Earthquake => tribute.set_status(TributeStatus::Buried),
                    AreaEvent::Avalanche => tribute.set_status(TributeStatus::Buried),
                    AreaEvent::Blizzard => tribute.set_status(TributeStatus::Frozen),
                    AreaEvent::Landslide => tribute.set_status(TributeStatus::Buried),
                    AreaEvent::Heatwave => tribute.set_status(TributeStatus::Overheated),
                }
            }
        }
    }

    pub fn living_tributes(&self) -> Vec<Tribute> {
        self.tributes
            .iter()
            .filter(|t| t.is_alive())
            .cloned()
            .collect()
    }

    pub fn available_items(&self) -> Vec<Item> {
        self.items
            .iter()
            .filter(|i| i.quantity > 0)
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn default_area() {
        let area = Area::default();
        assert_eq!(area.name, "");
        assert_eq!(area.items.len(), 0);
        assert_eq!(area.neighbors.len(), 0);
        assert_eq!(area.tributes.len(), 0);
    }

    #[test]
    fn new_area() {
        let area = Area::new("The Cornucopia");
        assert_eq!(area.name, "The Cornucopia");
        assert_eq!(area.to_string(), "The Cornucopia".to_string());
    }

    #[test]
    fn add_neighbor() {
        let mut area = Area::new("The Cornucopia");
        let neighbor = Area::new("Northwest");
        area.add_neighbor(neighbor.clone());
        assert!(area.neighbors.contains(&neighbor));
    }

    #[test]
    fn add_neighbors() {
        let mut area = Area::new("The Cornucopia");
        let neighbor_a = Area::new("Northwest");
        let neighbor_b = Area::new("Northeast");
        area.add_neighbors(vec![&neighbor_a, &neighbor_b]);
        assert!(area.neighbors.contains(&neighbor_a));
        assert!(area.neighbors.contains(&neighbor_b));
    }

    #[test]
    fn add_item() {
        let mut area = Area::new("The Cornucopia");
        let item = Item::new_random_weapon();
        area.add_item(item.clone());
        assert!(area.items.contains(&item));
        assert!(area.available_items().contains(&item));
    }

    #[test]
    fn remove_item() {
        let mut area = Area::new("The Cornucopia");
        let item = Item::new_random_weapon();
        area.add_item(item.clone());
        assert!(area.items.contains(&item));
        area.remove_item(&item);
        assert!(!area.items.contains(&item));
    }

    #[test]
    fn available_items() {
        let mut area = Area::new("The Cornucopia");
        assert!(area.available_items().is_empty());

        let item = Item::new_random_weapon();
        area.add_item(item.clone());

        assert!(!area.available_items().is_empty());
    }

    #[test]
    fn add_tribute() {
        let mut area = Area::new("The Cornucopia");
        let tribute = Tribute::new("Katniss".to_string(), Some(12), None);
        area.add_tribute(tribute.clone());
        assert!(area.tributes.contains(&tribute));
    }

    #[test]
    fn remove_tribute() {
        let mut area = Area::new("The Cornucopia");
        let tribute = Tribute::new("Katniss".to_string(), Some(12), None);
        area.add_tribute(tribute.clone());
        assert!(area.tributes.contains(&tribute));
        area.remove_tribute(&tribute);
        assert!(!area.tributes.contains(&tribute));
    }

    #[test]
    fn living_tributes() {
        let mut area = Area::new("The Cornucopia");
        let tribute = Tribute::new("Katniss".to_string(), Some(12), None);
        area.add_tribute(tribute.clone());
        assert_eq!(area.living_tributes().len(), 1);
        assert_eq!(area.living_tributes()[0], tribute);
    }

    #[test]
    fn dead_tributes() {
        let mut area = Area::new("The Cornucopia");
        let mut tribute = Tribute::new("Katniss".to_string(), Some(12), None);
        tribute.status = TributeStatus::Dead;
        area.add_tribute(tribute.clone());
        assert_eq!(area.living_tributes().len(), 0);
    }

    #[test]
    fn add_event() {
        let mut area = Area::new("The Cornucopia");
        let event = AreaEvent::Wildfire;
        area.add_event(event.clone());
        assert!(area.events.contains(&event));
    }

    #[test]
    fn process_events_closes_area() {
        let mut area = Area::new("The Cornucopia");
        let event = AreaEvent::Wildfire;
        area.add_event(event.clone());
        area.process_events();
        assert_eq!(area.open, false);
    }

    #[rstest]
    #[case(AreaEvent::Wildfire, TributeStatus::Burned)]
    #[case(AreaEvent::Flood, TributeStatus::Drowned)]
    #[case(AreaEvent::Earthquake, TributeStatus::Buried)]
    #[case(AreaEvent::Avalanche, TributeStatus::Buried)]
    #[case(AreaEvent::Blizzard, TributeStatus::Frozen)]
    #[case(AreaEvent::Landslide, TributeStatus::Buried)]
    #[case(AreaEvent::Heatwave, TributeStatus::Overheated)]
    fn process_events_affects_tributes(#[case] event: AreaEvent, #[case] status: TributeStatus) {
        let mut area = Area::new("The Cornucopia");
        let tribute = Tribute::new("Katniss".to_string(), Some(12), None);
        area.add_tribute(tribute);
        area.add_event(event);
        area.process_events();
        assert_eq!(area.open, false);
        assert_eq!(area.tributes.first().unwrap().status(), status);
    }

    #[test]
    fn partial_eq_with_reference() {
        let area = Area::new("The Cornucopia");
        assert_eq!(area, &area);
    }
}

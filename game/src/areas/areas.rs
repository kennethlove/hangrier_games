use std::fmt::Display;
use serde::{Deserialize, Serialize};
use crate::areas::events::AreaEvent;
use crate::tributes::Tribute;
use crate::items::Item;
use crate::tributes::statuses::TributeStatus;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Area {
    pub name: String,
    pub open: bool,
    pub items: Vec<Item>,
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
                    AreaEvent::Wildfire => { tribute.status = TributeStatus::Burned }
                    AreaEvent::Flood => { tribute.status = TributeStatus::Drowned }
                    AreaEvent::Earthquake => { tribute.status = TributeStatus::Buried }
                    AreaEvent::Avalanche => { tribute.status = TributeStatus::Buried }
                    AreaEvent::Blizzard => { tribute.status = TributeStatus::Frozen }
                    AreaEvent::Landslide => { tribute.status = TributeStatus::Buried }
                    AreaEvent::Heatwave => { tribute.status = TributeStatus::Overheated }
                }
            }
        }
    }

    pub fn living_tributes(&self) -> Vec<Tribute> {
        self.tributes.iter().filter(|t| t.is_alive()).cloned().collect()
    }

    pub fn available_items(&self) -> Vec<Item> {
        self.items.iter().filter(|i| i.quantity > 0).cloned().collect()
    }
}


#[cfg(test)]
mod tests {
    use crate::items::Item;
    use crate::tributes::Tribute;
    use super::Area;

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
        assert_eq!(area.to_string(), "The Cornucopia");
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
}

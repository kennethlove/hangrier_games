use crate::items::Item;
use serde::{Deserialize, Serialize, Serializer};
use std::fmt::Display;
use std::str::FromStr;
use strum_macros::EnumIter;

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
//     pub fn process_events(&mut self, mut tributes: Vec<Tribute>) -> Vec<Tribute> {
//         // If there are events, close the area
//         // if !self.events.is_empty() {
//         //     self.open = false;
//         // }
//
//         // for event in self.events.iter() {
//         //     for tribute in tributes.iter_mut() {
//         //         match event {
//         //             AreaEvent::Wildfire => tribute.set_status(TributeStatus::Burned),
//         //             AreaEvent::Flood => tribute.set_status(TributeStatus::Drowned),
//         //             AreaEvent::Earthquake => tribute.set_status(TributeStatus::Buried),
//         //             AreaEvent::Avalanche => tribute.set_status(TributeStatus::Buried),
//         //             AreaEvent::Blizzard => tribute.set_status(TributeStatus::Frozen),
//         //             AreaEvent::Landslide => tribute.set_status(TributeStatus::Buried),
//         //             AreaEvent::Heatwave => tribute.set_status(TributeStatus::Overheated),
//         //         }
//         //     }
//         // }
//
//         tributes
//     }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct AreaDetails {
    pub identifier: String,
    pub name: String,
    pub area: String,
    pub open: bool,
    #[serde(default)]
    pub items: Vec<Item>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::games::Game;

    thread_local!(pub static GAME: Game = Game::default());

    // TODO: Write tests for from_str and Display impls

    // #[test]
    // #[test]
    // fn new_area() {
    //     let area = Area::new("The Cornucopia");
    //     assert_eq!(area.name, "The Cornucopia");
    //     assert_eq!(area.to_string(), "The Cornucopia".to_string());
    // }
    //
    // #[test]
    // fn add_neighbor() {
    //     let mut area = Area::new("The Cornucopia");
    //     let neighbor = Area::new("Northwest");
    //     area.add_neighbor(neighbor.clone());
    //     assert!(area.neighbors.contains(&neighbor));
    // }
    //
    // #[test]
    // fn add_neighbors() {
    //     let mut area = Area::new("The Cornucopia");
    //     let neighbor_a = Area::new("Northwest");
    //     let neighbor_b = Area::new("Northeast");
    //     area.add_neighbors(vec![&neighbor_a, &neighbor_b]);
    //     assert!(area.neighbors.contains(&neighbor_a));
    //     assert!(area.neighbors.contains(&neighbor_b));
    // }
    //
    // #[test]
    // fn add_item() {
    //     let mut area = Area::new("The Cornucopia");
    //     let item = Item::new_random_weapon();
    //     area.add_item(item.clone());
    //     // assert!(area.items.contains(&item));
    //     assert!(area.available_items().contains(&item));
    // }
    //
    // #[test]
    // fn remove_item() {
    //     let mut area = Area::new("The Cornucopia");
    //     let item = Item::new_random_weapon();
    //     area.add_item(item.clone());
    //     // assert!(area.items.contains(&item));
    //     area.remove_item(&item);
    //     // assert!(!area.items.contains(&item));
    // }
    //
    // #[test]
    // fn available_items() {
    //     let mut area = Area::new("The Cornucopia");
    //     assert!(area.available_items().is_empty());
    //
    //     let item = Item::new_random_weapon();
    //     area.add_item(item.clone());
    //
    //     assert!(!area.available_items().is_empty());
    // }
    //
    // #[ignore]
    // #[test]
    // fn living_tributes() {
    //     let mut game = Game::default();
    //     let area = Area::new("The Cornucopia");
    //     let mut tribute = Tribute::new("Katniss".to_string(), Some(12), None);
    //     tribute.area = area.clone();
    //     game.add_tribute(tribute.clone());
    //     assert_eq!(area.living_tributes().len(), 1);
    //     assert_eq!(area.living_tributes()[0], tribute);
    // }
    //
    // #[ignore]
    // #[test]
    // fn dead_tributes() {
    //     let mut game = Game::default();
    //     let area = Area::new("The Cornucopia");
    //     let mut tribute = Tribute::new("Katniss".to_string(), Some(12), None);
    //     tribute.status = TributeStatus::Dead;
    //     tribute.area = area.clone();
    //     game.add_tribute(tribute.clone());
    //     assert_eq!(area.living_tributes().len(), 0);
    // }
    //
    // #[test]
    // fn add_event() {
    //     let mut area = Area::new("The Cornucopia");
    //     let event = AreaEvent::Wildfire;
    //     area.add_event(event.clone());
    //     // assert!(area.events.contains(&event));
    // }
    //
    // #[test]
    // fn process_events_closes_area() {
    //     let mut area = Area::new("The Cornucopia");
    //     let event = AreaEvent::Wildfire;
    //     area.add_event(event.clone());
    //     area.process_events(area.tributes());
    //     assert_eq!(area.open, false);
    // }
    //
    // #[rstest]
    // #[case(AreaEvent::Wildfire, TributeStatus::Burned)]
    // #[case(AreaEvent::Flood, TributeStatus::Drowned)]
    // #[case(AreaEvent::Earthquake, TributeStatus::Buried)]
    // #[case(AreaEvent::Avalanche, TributeStatus::Buried)]
    // #[case(AreaEvent::Blizzard, TributeStatus::Frozen)]
    // #[case(AreaEvent::Landslide, TributeStatus::Buried)]
    // #[case(AreaEvent::Heatwave, TributeStatus::Overheated)]
    // fn process_events_affects_tributes(#[case] event: AreaEvent, #[case] status: TributeStatus) {
    //     let mut area = Area::new("The Cornucopia");
    //     let tribute = Tribute::new("Katniss".to_string(), Some(12), None);
    //     area.add_event(event);
    //     area.process_events(area.tributes());
    //     assert_eq!(area.open, false);
    // }

    #[test]
    fn partial_eq_with_reference() {
        let area = Area::Cornucopia;
        assert_eq!(area, &area);
    }
}

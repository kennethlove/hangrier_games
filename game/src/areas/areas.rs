use rand::Rng;
use std::fmt::Display;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use super::events::AreaEvent;
use crate::tributes::Tribute;
use crate::tributes::statuses::TributeStatus;
use crate::items::Item;
use crate::messages::GameMessage;

#[derive(Clone, Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct AreaDetails {
    pub open: bool,
    pub items: Vec<Item>,
}

impl Default for AreaDetails {
    fn default() -> Self {
        Self {
            open: true,
            items: vec![],
        }
    }
}

#[derive(Clone, Default, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Area {
    #[default]
    Cornucopia(AreaDetails),
    Northeast(AreaDetails),
    Northwest(AreaDetails),
    Southeast(AreaDetails),
    Southwest(AreaDetails),
}

impl Display for Area {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Area::Cornucopia(_) => write!(f, "the cornucopia"),
            Area::Northeast(_) => write!(f, "northeast"),
            Area::Northwest(_) => write!(f, "northwest"),
            Area::Southeast(_) => write!(f, "southeast"),
            Area::Southwest(_) => write!(f, "southwest"),
        }
    }
}

impl FromStr for Area {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "the cornucopia" => Ok(Area::Cornucopia { 0: Default::default() }),
            "northeast" => Ok(Area::Northeast { 0: Default::default() }),
            "northwest" => Ok(Area::Northwest { 0: Default::default() }),
            "southeast" => Ok(Area::Southeast { 0: Default::default() }),
            "southwest" => Ok(Area::Southwest { 0: Default::default() }),
            _ => Err("invalid area".into()),
        }
    }
}

impl Area {
    pub fn neighbors(&self) -> Vec<&str> {
        match self {
            Area::Cornucopia(_) => vec!["northeast", "northwest", "southeast", "southwest"],
            Area::Northeast(_) => vec!["the cornucopia", "northwest", "southeast"],
            Area::Northwest(_) => vec!["the cornucopia", "northeast", "southwest"],
            Area::Southeast(_) => vec!["the cornucopia", "southwest", "northeast"],
            Area::Southwest(_) => vec!["the cornucopia", "southeast", "northwest"]
        }
    }

    pub fn random() -> Area {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..5) {
            1 => Area::Northeast,
            2 => Area::Northwest,
            3 => Area::Southeast,
            4 => Area::Southwest,
            _ => Cornucopia,
        }
    }

    pub fn id(&self) -> i32 {
        match self {
            Cornucopia => 1,
            Area::Northeast => 2,
            Area::Northwest => 3,
            Area::Southeast => 4,
            Area::Southwest => 5,
        }
    }

    pub fn get_by_id(area_id: i32) -> Option<Area> {
        match area_id {
            1 => Some(Cornucopia),
            2 => Some(Area::Northeast),
            3 => Some(Area::Northwest),
            4 => Some(Area::Southeast),
            5 => Some(Area::Southwest),
            _ => None
        }
    }

    /// Returns a random open area that is not in the list of closed areas.
    /// If it can't find an open area after 5 tries, it defaults to the Cornucopia.
    pub fn random_open_area(closed_areas: Vec<Area>) -> Area {
        let mut count = 0;
        let area = loop {
            let random_area = Area::random();
            if !closed_areas.contains(&random_area.clone()) {
                break random_area;
            }
            if count == 10 {
                break Cornucopia;
            }
            count += 1;
        };
        area
    }

    pub fn tributes(&self, game_id: i32) -> Vec<Tribute> {
        let area = models::Area::from(self.clone());
        area.tributes(game_id).iter()
            .map(|t| Tribute::from(t.clone()))
            .collect()
    }

    pub fn items(&self, game_id: i32) -> Vec<Item> {
        let area = models::Area::from(self.clone());
        area.items(game_id).iter()
            .map(|i| Item::from(i.clone()))
            .collect()
    }

    pub fn available_items(&self, game_id: i32) -> Vec<Item> {
        let items = self.items(game_id);
        items.iter()
            .filter(|i| i.tribute_id.is_none())
            .filter(|i| i.quantity > 0)
            .cloned()
            .collect()
    }

    pub fn do_area_event(game_id: i32) {
        let event = AreaEvent::random();
        let mut game = get_game_by_id(game_id).expect("Game doesn't exist");
        let closed_areas = game.closed_areas();
        let area = Area::random_open_area(closed_areas);

        create_full_log(
            game_id,
            GameMessage::AreaEvent(event.clone(), area.clone()).to_string(),
            Some(area.id()),
            None,
            None,
            None,
        );

        let model_area = models::Area::from(area.clone());
        models::AreaEvent::create(event.to_string(), model_area.id, game.id);
        game.close_area(&model_area);
    }

    pub fn clean_up_area_events(game_id: i32) {
        let mut rng = rand::thread_rng();
        let mut game = get_game_by_id(game_id).expect("Game doesn't exist");
        let closed_areas = game.closed_areas();
        for area in closed_areas {
            let model_area = models::Area::from(area.clone());
            let events = model_area.events(game.id);
            let last_event = events.iter().last().unwrap();
            let mut tributes = model_area.tributes(game.id);
            let tributes = tributes
                .iter_mut()
                .filter(|t| t.day_killed.is_none())
                .map(|t| Tribute::from(t.clone()))
                .collect::<Vec<_>>();

            for mut tribute in tributes {
                create_full_log(
                    game_id,
                    GameMessage::TrappedInArea(tribute.clone(), area.clone()).to_string(),
                    Some(area.id()),
                    Some(tribute.id.unwrap()),
                    None,
                    None,
                );

                if rng.gen_bool(tribute.luck.unwrap_or(0) as f64 / 100.0) {
                    // If the tribute is lucky, they're just harmed by the event
                    let area_event = AreaEvent::from_str(&last_event.name).unwrap();
                    match area_event {
                        AreaEvent::Wildfire => {
                            tribute.status = TributeStatus::Burned
                        }
                        AreaEvent::Flood => {
                            tribute.status = TributeStatus::Drowned
                        }
                        AreaEvent::Earthquake => {
                            tribute.status = TributeStatus::Buried
                        }
                        AreaEvent::Avalanche => {
                            tribute.status = TributeStatus::Buried
                        }
                        AreaEvent::Blizzard => {
                            tribute.status = TributeStatus::Frozen
                        }
                        AreaEvent::Landslide => {
                            tribute.status = TributeStatus::Buried
                        }
                        AreaEvent::Heatwave => {
                            tribute.status = TributeStatus::Overheated
                        }
                    };
                } else {
                    // If the tribute is unlucky, they die
                    tribute.dies();
                    tribute.health = 0;
                    tribute.killed_by = Some(last_event.name.clone());
                    create_full_log(
                        game_id,
                        GameMessage::DiedInArea(tribute.clone(), area.clone()).to_string(),
                        Some(area.id()),
                        Some(tribute.id.unwrap()),
                        None,
                        None,
                    );
                }
                update_tribute(tribute.id.unwrap(), ModelTribute::from(tribute.clone()));
            }

            // Re-open the area?
            if rng.gen_bool(0.5) {
                create_full_log(
                    game_id,
                    GameMessage::AreaOpen(area.clone()).to_string(),
                    Some(area.id()),
                    None,
                    None,
                    None,
                );
                game.open_area(&model_area);
            }
        }
    }
}


impl From<AreaModel> for Area {
    fn from(area: AreaModel) -> Self {
        Self::from_str(area.name.as_str()).unwrap_or(Cornucopia)
    }
}

impl From<String> for Area {
    fn from(s: String) -> Self {
        Self::from_str(s.as_str()).unwrap_or(Cornucopia)
    }
}

#[cfg(test)]
mod tests {
    use super::Area;

    #[test]
    fn area_from_str() {
        assert_eq!(Area::from_str("The Cornucopia"), Some(Cornucopia));
        assert_eq!(Area::from_str("Cornucopia"), Some(Cornucopia));
        assert_eq!(Area::from_str("North East"), Some(Area::Northeast));
        assert_eq!(Area::from_str("Northeast"), Some(Area::Northeast));
        assert_eq!(Area::from_str("NE"), Some(Area::Northeast));
        assert_eq!(Area::from_str("North West"), Some(Area::Northwest));
        assert_eq!(Area::from_str("Northwest"), Some(Area::Northwest));
        assert_eq!(Area::from_str("NW"), Some(Area::Northwest));
        assert_eq!(Area::from_str("South East"), Some(Area::Southeast));
        assert_eq!(Area::from_str("Southeast"), Some(Area::Southeast));
        assert_eq!(Area::from_str("SE"), Some(Area::Southeast));
        assert_eq!(Area::from_str("South West"), Some(Area::Southwest));
        assert_eq!(Area::from_str("Southwest"), Some(Area::Southwest));
        assert_eq!(Area::from_str("SW"), Some(Area::Southwest));
    }

    #[test]
    fn area_as_str() {
        assert_eq!(Cornucopia.as_str(), "The Cornucopia");
        assert_eq!(Area::Northeast.as_str(), "Northeast");
        assert_eq!(Area::Northwest.as_str(), "Northwest");
        assert_eq!(Area::Southeast.as_str(), "Southeast");
        assert_eq!(Area::Southwest.as_str(), "Southwest");
    }

    #[test]
    fn random_area() {
        let area = Area::random();
        assert!(
            area == Cornucopia ||
                area == Area::Northeast ||
                area == Area::Northwest ||
                area == Area::Southeast ||
                area == Area::Southwest
        );
    }

    #[test]
    fn area_neighbors() {
        assert_eq!(Cornucopia.neighbors(), vec![Area::Northeast, Area::Northwest, Area::Southeast, Area::Southwest]);
        assert_eq!(Area::Northeast.neighbors(), vec![Cornucopia, Area::Northwest, Area::Southeast]);
        assert_eq!(Area::Northwest.neighbors(), vec![Cornucopia, Area::Northeast, Area::Southwest]);
        assert_eq!(Area::Southeast.neighbors(), vec![Cornucopia, Area::Southwest, Area::Northeast]);
        assert_eq!(Area::Southwest.neighbors(), vec![Cornucopia, Area::Southeast, Area::Northwest]);
    }
}

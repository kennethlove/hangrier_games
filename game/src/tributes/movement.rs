//! Movement and travel functionality for tributes.
//!
//! This module handles tribute movement between areas including:
//! - Travel mechanics and validation
//! - Movement restrictions based on attributes
//! - Area selection logic

use crate::areas::Area;
use crate::messages::{AreaRef, MessagePayload, TaggedEvent, TributeRef};
use crate::output::GameOutput;
use crate::tributes::Tribute;
use rand::prelude::*;
use rand::rngs::SmallRng;

#[derive(Debug, PartialEq)]
pub enum TravelResult {
    Success(Area),
    Failure,
}

impl Tribute {
    /// Moves a tribute to a new area.
    /// If the tribute has no movement, they cannot move.
    /// If the tribute is already in the suggested area, they stay put.
    /// If the tribute has low movement, they can only move to the suggested area or stay put.
    /// If the tribute has high movement, they can move to any open neighbor or the suggested area.
    pub(crate) fn travels(
        &self,
        closed_areas: &[Area],
        suggested_area: Option<Area>,
        events: &mut Vec<TaggedEvent>,
    ) -> TravelResult {
        let mut rng = SmallRng::from_rng(&mut rand::rng());
        // Where is the tribute?
        let current_area = self.area;

        let tribute_ref = || TributeRef {
            identifier: self.identifier.clone(),
            name: self.name.clone(),
        };
        let area_ref = |a: Area| {
            let s = a.to_string();
            AreaRef {
                identifier: s.clone(),
                name: s,
            }
        };

        // 1. Can the tribute move at all?
        if self.attributes.movement == 0 {
            let current_area_name = self.area.to_string();
            let line =
                GameOutput::TributeTravelTooTired(self.name.as_str(), current_area_name.as_str())
                    .to_string();
            events.push(TaggedEvent::new(
                line,
                MessagePayload::TributeHidden {
                    tribute: tribute_ref(),
                    area: area_ref(current_area),
                },
            ));
            return TravelResult::Failure;
        }

        // 2. Determine the target area based on suggestion and validity.
        let mut target_area: Option<Area> = None;
        if let Some(suggestion) = suggested_area
            && !closed_areas.contains(&suggestion)
        {
            if suggestion == current_area {
                let suggestion_name = suggestion.to_string();
                let line = GameOutput::TributeTravelAlreadyThere(
                    self.name.as_str(),
                    suggestion_name.as_str(),
                )
                .to_string();
                events.push(TaggedEvent::new(
                    line,
                    MessagePayload::TributeHidden {
                        tribute: tribute_ref(),
                        area: area_ref(suggestion),
                    },
                ));
                return TravelResult::Failure;
            }
            target_area = Some(suggestion);
        }

        // 3. Handle movement based on tribute's movement attribute.
        match self.attributes.movement {
            // Low movement: can only move to suggested_area if it's valid and set.
            1..=10 => {
                if let Some(new_area) = target_area {
                    let current_area_name = current_area.to_string();
                    let new_area_name = new_area.to_string();
                    let line = GameOutput::TributeTravel(
                        self.name.as_str(),
                        current_area_name.as_str(),
                        new_area_name.as_str(),
                    )
                    .to_string();
                    events.push(TaggedEvent::new(
                        line,
                        MessagePayload::TributeMoved {
                            tribute: tribute_ref(),
                            from: area_ref(current_area),
                            to: area_ref(new_area),
                        },
                    ));
                    TravelResult::Success(new_area)
                } else {
                    let current_area_name = current_area.to_string();
                    let line = GameOutput::TributeTravelTooTired(
                        self.name.as_str(),
                        current_area_name.as_str(),
                    )
                    .to_string();
                    events.push(TaggedEvent::new(
                        line,
                        MessagePayload::TributeHidden {
                            tribute: tribute_ref(),
                            area: area_ref(current_area),
                        },
                    ));
                    TravelResult::Failure
                }
            }
            // High movement: can move to any open neighbor or the suggested area.
            _ => {
                if let Some(new_area) = target_area {
                    let current_area_name = current_area.to_string();
                    let new_area_name = new_area.to_string();
                    let line = GameOutput::TributeTravel(
                        self.name.as_str(),
                        current_area_name.as_str(),
                        new_area_name.as_str(),
                    )
                    .to_string();
                    events.push(TaggedEvent::new(
                        line,
                        MessagePayload::TributeMoved {
                            tribute: tribute_ref(),
                            from: area_ref(current_area),
                            to: area_ref(new_area),
                        },
                    ));
                    return TravelResult::Success(new_area);
                }

                let neighbors = current_area.neighbors();
                let available_neighbors: Vec<Area> = neighbors
                    .into_iter()
                    .filter(|area| area != &current_area && !closed_areas.contains(area))
                    .collect();

                if available_neighbors.is_empty() {
                    let current_area_name = current_area.to_string();
                    let line = GameOutput::TributeTravelNoOptions(
                        self.name.as_str(),
                        current_area_name.as_str(),
                    )
                    .to_string();
                    events.push(TaggedEvent::new(
                        line,
                        MessagePayload::TributeHidden {
                            tribute: tribute_ref(),
                            area: area_ref(current_area),
                        },
                    ));
                    return TravelResult::Success(current_area);
                }

                // TODO: Loyalty bit goes here

                let chosen_neighbor = available_neighbors.choose(&mut rng).unwrap();
                let current_area_name = current_area.to_string();
                let chosen_area_name = chosen_neighbor.to_string();
                let line = GameOutput::TributeTravel(
                    self.name.as_str(),
                    current_area_name.as_str(),
                    chosen_area_name.as_str(),
                )
                .to_string();
                events.push(TaggedEvent::new(
                    line,
                    MessagePayload::TributeMoved {
                        tribute: tribute_ref(),
                        from: area_ref(current_area),
                        to: area_ref(*chosen_neighbor),
                    },
                ));
                TravelResult::Success(*chosen_neighbor)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::areas::Area::*;
    use crate::areas::AreaDetails;
    use crate::tributes::Tribute;
    use rstest::*;

    #[fixture]
    fn tribute() -> Tribute {
        Tribute::new("Katniss".to_string(), None, None)
    }

    #[rstest]
    #[tokio::test]
    async fn travels_success(tribute: Tribute) {
        let open_area = AreaDetails::new(Some("Forest".to_string()), Cornucopia);
        let result = tribute.travels(&[East, South, North, West], None, &mut Vec::new());
        assert_eq!(result, TravelResult::Success(open_area.area.unwrap()));
    }

    #[rstest]
    #[tokio::test]
    async fn travels_fail_no_movement(mut tribute: Tribute) {
        tribute.attributes.movement = 0;
        let result = tribute.travels(&[], None, &mut Vec::new());
        assert_eq!(result, TravelResult::Failure);
    }

    #[rstest]
    #[tokio::test]
    async fn travels_fail_already_there(mut tribute: Tribute) {
        tribute.area = North;
        let result = tribute.travels(
            &[Cornucopia, East, West, South],
            Some(North),
            &mut Vec::new(),
        );
        assert_eq!(result, TravelResult::Failure);
    }

    #[rstest]
    #[tokio::test]
    async fn travels_fail_low_movement_no_suggestion(mut tribute: Tribute) {
        tribute.attributes.movement = 5;
        let result = tribute.travels(&[Cornucopia, East, West, North], None, &mut Vec::new());
        assert_eq!(result, TravelResult::Failure);
    }

    #[rstest]
    #[tokio::test]
    async fn travels_fail_low_movement_suggestion(mut tribute: Tribute) {
        tribute.attributes.movement = 5;
        let result = tribute.travels(
            &[Cornucopia, East, West, North],
            Some(North),
            &mut Vec::new(),
        );
        assert_eq!(result, TravelResult::Failure);
    }

    #[rstest]
    #[tokio::test]
    async fn travels_success_low_movement_suggestion(mut tribute: Tribute) {
        tribute.area = North;
        tribute.attributes.movement = 5;
        let open_area = AreaDetails::new(Some("Forest".to_string()), Cornucopia);
        let result = tribute.travels(&[East, South], Some(Cornucopia), &mut Vec::new());
        assert_eq!(result, TravelResult::Success(open_area.area.unwrap()));
    }
}

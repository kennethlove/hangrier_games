//! Game-side message helpers.
//!
//! The schema types (`GameMessage`, `MessageKind`, `MessageSource`, the new
//! `MessagePayload` and friends) live in the `shared` crate so the `web`
//! and `api` crates can use them without pulling in `game`. This module
//! re-exports them and adds game-only conveniences:
//!
//! - [`TaggedEvent`]: a per-action accumulator pairing a typed
//!   [`MessagePayload`] with its already-formatted prose line. The
//!   `do_step` drain converts each `TaggedEvent` into a [`GameMessage`]
//!   stamped with `(game_day, phase, tick, emit_index)`.
//! - Terrain-aware narrative helpers used by tribute / area logic.

pub use shared::messages::{
    AreaEventKind, AreaRef, CombatEngagement, CombatOutcome, GameMessage, ItemRef, MessageKind,
    MessagePayload, MessageSource, ParsePhaseError, Phase, TributeRef,
};

use crate::terrain::{BaseTerrain, Harshness, Visibility};

/// Per-action accumulator: a typed payload plus its already-formatted prose line.
///
/// Tribute / area logic pushes `TaggedEvent`s into a local `Vec<TaggedEvent>`.
/// The `do_step` drain in `game/src/games.rs` converts each into a
/// `GameMessage` with `(game_day, phase, tick, emit_index)` causal-ordering
/// fields applied at the boundary.
#[derive(Debug, Clone)]
pub struct TaggedEvent {
    pub content: String,
    pub payload: MessagePayload,
}

impl TaggedEvent {
    pub fn new(content: impl Into<String>, payload: MessagePayload) -> Self {
        Self {
            content: content.into(),
            payload,
        }
    }
}

/// Generate terrain-aware movement narrative.
/// Describes how terrain affects movement with rich descriptive text.
pub fn movement_narrative(terrain: BaseTerrain, tribute_name: &str) -> String {
    match terrain {
        BaseTerrain::Desert => {
            format!(
                "{} struggles through the scorching desert sands, each step draining their energy",
                tribute_name
            )
        }
        BaseTerrain::Tundra => {
            format!(
                "{} trudges through the frozen tundra, breath visible in the frigid air",
                tribute_name
            )
        }
        BaseTerrain::Forest => {
            format!(
                "{} navigates through the dense forest, branches scratching at their clothes",
                tribute_name
            )
        }
        BaseTerrain::Jungle => {
            format!(
                "{} hacks through the thick jungle undergrowth, sweat pouring down their face",
                tribute_name
            )
        }
        BaseTerrain::Mountains => {
            format!(
                "{} climbs the steep mountain path, legs burning with exertion",
                tribute_name
            )
        }
        BaseTerrain::Clearing => {
            format!(
                "{} walks through the open clearing, vulnerable but unimpeded",
                tribute_name
            )
        }
        BaseTerrain::UrbanRuins => {
            format!(
                "{} picks their way through the crumbling urban ruins, alert for danger",
                tribute_name
            )
        }
        BaseTerrain::Grasslands => {
            format!(
                "{} moves swiftly through the swaying grasslands",
                tribute_name
            )
        }
        BaseTerrain::Wetlands => {
            format!(
                "{} wades through the murky wetlands, boots squelching with each step",
                tribute_name
            )
        }
        BaseTerrain::Badlands => {
            format!(
                "{} navigates the treacherous badlands, watching for unstable ground",
                tribute_name
            )
        }
        BaseTerrain::Highlands => {
            format!(
                "{} crosses the windswept highlands, elevation making breathing difficult",
                tribute_name
            )
        }
        BaseTerrain::Geothermal => {
            format!(
                "{} carefully moves through the geothermal area, avoiding steaming vents",
                tribute_name
            )
        }
    }
}

/// Generate terrain-aware hiding spot description.
/// Describes where and how a tribute hides based on terrain visibility.
pub fn hiding_spot_narrative(terrain: BaseTerrain, tribute_name: &str) -> String {
    match terrain.visibility() {
        Visibility::Concealed => match terrain {
            BaseTerrain::Forest => {
                format!(
                    "{} conceals themselves behind dense foliage, nearly invisible",
                    tribute_name
                )
            }
            BaseTerrain::Jungle => {
                format!(
                    "{} disappears into the thick jungle undergrowth, completely hidden from view",
                    tribute_name
                )
            }
            BaseTerrain::UrbanRuins => {
                format!(
                    "{} takes cover in the shadows of a collapsed building, watching through a crack",
                    tribute_name
                )
            }
            _ => {
                format!("{} finds a concealed hiding spot", tribute_name)
            }
        },
        Visibility::Moderate => match terrain {
            BaseTerrain::Clearing => {
                format!(
                    "{} crouches low in the clearing, hoping not to be noticed",
                    tribute_name
                )
            }
            BaseTerrain::Wetlands => {
                format!(
                    "{} submerges themselves in the murky water, only eyes above the surface",
                    tribute_name
                )
            }
            BaseTerrain::Highlands => {
                format!(
                    "{} presses themselves flat against a rocky outcrop",
                    tribute_name
                )
            }
            BaseTerrain::Geothermal => {
                format!(
                    "{} huddles near a steaming vent, using the mist as cover",
                    tribute_name
                )
            }
            _ => {
                format!(
                    "{} attempts to hide, but the terrain offers limited concealment",
                    tribute_name
                )
            }
        },
        Visibility::Exposed => match terrain {
            BaseTerrain::Desert => {
                format!(
                    "{} dives behind a small sand dune, barely concealed in the open desert",
                    tribute_name
                )
            }
            BaseTerrain::Tundra => {
                format!(
                    "{} lies flat in the snow, their dark form standing out against the white expanse",
                    tribute_name
                )
            }
            BaseTerrain::Grasslands => {
                format!(
                    "{} drops into the tall grass, visible to anyone nearby",
                    tribute_name
                )
            }
            BaseTerrain::Badlands => {
                format!(
                    "{} crouches behind a rocky formation, but remains exposed",
                    tribute_name
                )
            }
            _ => {
                format!(
                    "{} tries to hide in the exposed terrain, with little success",
                    tribute_name
                )
            }
        },
    }
}

/// Generate stamina-related narrative based on terrain harshness.
pub fn stamina_narrative(terrain: BaseTerrain, current_stamina: u32) -> String {
    let harshness = terrain.harshness();
    let stamina_level = if current_stamina >= 70 {
        "fresh"
    } else if current_stamina >= 40 {
        "tired"
    } else if current_stamina >= 20 {
        "exhausted"
    } else {
        "on the verge of collapse"
    };

    match (harshness, stamina_level) {
        (Harshness::Harsh, "on the verge of collapse") => {
            format!(
                "The harsh {} terrain is taking a severe toll. Movement has become agonizingly slow.",
                terrain_name(terrain)
            )
        }
        (Harshness::Harsh, "exhausted") => {
            format!(
                "The brutal {} environment is draining energy rapidly.",
                terrain_name(terrain)
            )
        }
        (Harshness::Harsh, "tired") => {
            format!(
                "The demanding {} terrain is wearing them down steadily.",
                terrain_name(terrain)
            )
        }
        (Harshness::Harsh, "fresh") => {
            format!(
                "The challenging {} terrain requires constant vigilance.",
                terrain_name(terrain)
            )
        }
        (Harshness::Moderate, "on the verge of collapse") => {
            format!(
                "Even the moderate {} terrain feels overwhelming in this state.",
                terrain_name(terrain)
            )
        }
        (Harshness::Moderate, "exhausted") => {
            format!(
                "The {} environment is taking its toll.",
                terrain_name(terrain)
            )
        }
        (Harshness::Moderate, "tired") => {
            format!(
                "The {} terrain is beginning to wear them down.",
                terrain_name(terrain)
            )
        }
        (Harshness::Moderate, "fresh") => {
            format!(
                "The {} terrain presents no significant challenges.",
                terrain_name(terrain)
            )
        }
        (Harshness::Mild, "on the verge of collapse") => {
            format!(
                "Despite the relatively easy {} terrain, exhaustion is setting in.",
                terrain_name(terrain)
            )
        }
        (Harshness::Mild, "exhausted") => {
            format!(
                "Even in the gentle {} terrain, fatigue is becoming a problem.",
                terrain_name(terrain)
            )
        }
        (Harshness::Mild, "tired") => {
            format!(
                "The easy {} terrain allows for steady but tiring movement.",
                terrain_name(terrain)
            )
        }
        (Harshness::Mild, "fresh") => {
            format!(
                "The gentle {} terrain allows for effortless movement.",
                terrain_name(terrain)
            )
        }
        _ => {
            format!(
                "They press on through the {} terrain.",
                terrain_name(terrain)
            )
        }
    }
}

/// Helper function to get terrain name as string.
fn terrain_name(terrain: BaseTerrain) -> &'static str {
    match terrain {
        BaseTerrain::Desert => "desert",
        BaseTerrain::Tundra => "tundra",
        BaseTerrain::Forest => "forest",
        BaseTerrain::Jungle => "jungle",
        BaseTerrain::Mountains => "mountain",
        BaseTerrain::Clearing => "clearing",
        BaseTerrain::UrbanRuins => "urban ruin",
        BaseTerrain::Grasslands => "grassland",
        BaseTerrain::Wetlands => "wetland",
        BaseTerrain::Badlands => "badland",
        BaseTerrain::Highlands => "highland",
        BaseTerrain::Geothermal => "geothermal",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tagged_event_constructor_sets_fields() {
        let payload = MessagePayload::SanityBreak {
            tribute: TributeRef {
                identifier: "t1".into(),
                name: "T1".into(),
            },
        };
        let ev = TaggedEvent::new("T1 snaps", payload);
        assert_eq!(ev.content, "T1 snaps");
        assert_eq!(ev.payload.kind(), MessageKind::State);
    }

    #[test]
    fn movement_narrative_returns_terrain_aware_text() {
        let s = movement_narrative(BaseTerrain::Desert, "Alice");
        assert!(s.contains("Alice"));
        assert!(s.contains("desert"));
    }
}

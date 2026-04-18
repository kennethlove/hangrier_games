use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::terrain::{BaseTerrain, Harshness, Visibility};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "value")]
pub enum MessageSource {
    #[serde(rename = "Game")]
    Game(String), // Game identifier
    #[serde(rename = "Area")]
    Area(String), // Area name
    #[serde(rename = "Tribute")]
    Tribute(String), // Tribute identifier
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GameMessage {
    pub identifier: String,
    pub source: MessageSource,
    pub game_day: u32,
    pub subject: String,
    #[serde(with = "chrono::serde::ts_nanoseconds")]
    pub timestamp: DateTime<Utc>,
    pub content: String,
}

impl GameMessage {
    /// Create a new game message
    pub fn new(source: MessageSource, game_day: u32, subject: String, content: String) -> Self {
        GameMessage {
            identifier: Uuid::new_v4().to_string(),
            source,
            game_day,
            subject,
            timestamp: Utc::now(),
            content,
        }
    }
}

/// Generate terrain-aware movement narrative.
/// Describes how terrain affects movement with rich descriptive text.
///
/// # Examples
/// ```
/// use hangrier_games::messages::movement_narrative;
/// use hangrier_games::terrain::BaseTerrain;
///
/// let desc = movement_narrative(BaseTerrain::Desert, "Alice");
/// // Returns: "Alice struggles through the scorching desert sands"
/// ```
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
///
/// # Examples
/// ```
/// use hangrier_games::messages::hiding_spot_narrative;
/// use hangrier_games::terrain::BaseTerrain;
///
/// let desc = hiding_spot_narrative(BaseTerrain::Forest, "Bob");
/// // Returns: "Bob conceals themselves behind dense foliage, nearly invisible"
/// ```
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
/// Describes how terrain affects tribute energy and movement capability.
///
/// # Examples
/// ```
/// use hangrier_games::messages::stamina_narrative;
/// use hangrier_games::terrain::BaseTerrain;
///
/// let desc = stamina_narrative(BaseTerrain::Mountains, 30);
/// // Returns: "The harsh mountain terrain is taking a severe toll..."
/// ```
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

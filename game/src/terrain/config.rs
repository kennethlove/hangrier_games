use serde::{Deserialize, Serialize};

use crate::terrain::BaseTerrain;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Visibility {
    Exposed,   // Tundra, Desert, Grasslands - hard to hide
    Moderate,  // Clearing, Highlands, Wetlands
    Concealed, // Forest, Jungle, UrbanRuins - easy to hide
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Harshness {
    Mild,     // Clearing, Grasslands
    Moderate, // Forest, Jungle, UrbanRuins, Wetlands, Highlands, Geothermal
    Harsh,    // Desert, Tundra, Mountains, Badlands
}

#[derive(Debug, Clone, Copy)]
pub struct ItemWeights {
    pub weapons: f32,
    pub shields: f32,
    pub consumables: f32,
}

impl BaseTerrain {
    pub const fn movement_cost(&self) -> f32 {
        match self {
            BaseTerrain::Clearing => 1.0,
            BaseTerrain::Grasslands => 0.9,
            BaseTerrain::UrbanRuins => 1.2,
            BaseTerrain::Forest => 1.3,
            BaseTerrain::Jungle => 1.4,
            BaseTerrain::Geothermal => 1.4,
            BaseTerrain::Wetlands => 1.5,
            BaseTerrain::Highlands => 1.6,
            BaseTerrain::Badlands => 1.7,
            BaseTerrain::Mountains => 1.8,
            BaseTerrain::Desert => 2.0,
            BaseTerrain::Tundra => 2.0,
        }
    }

    pub const fn visibility(&self) -> Visibility {
        match self {
            BaseTerrain::Forest | BaseTerrain::Jungle | BaseTerrain::UrbanRuins => {
                Visibility::Concealed
            }
            BaseTerrain::Desert
            | BaseTerrain::Tundra
            | BaseTerrain::Grasslands
            | BaseTerrain::Badlands => Visibility::Exposed,
            _ => Visibility::Moderate,
        }
    }

    pub const fn harshness(&self) -> Harshness {
        match self {
            BaseTerrain::Clearing | BaseTerrain::Grasslands => Harshness::Mild,
            BaseTerrain::Desert
            | BaseTerrain::Tundra
            | BaseTerrain::Mountains
            | BaseTerrain::Badlands => Harshness::Harsh,
            _ => Harshness::Moderate,
        }
    }

    pub const fn item_spawn_modifier(&self) -> f32 {
        match self {
            BaseTerrain::Clearing => 1.0,
            BaseTerrain::Jungle => 1.0,
            BaseTerrain::Forest => 1.1,
            BaseTerrain::Grasslands => 1.1,
            BaseTerrain::UrbanRuins => 1.2,
            BaseTerrain::Wetlands => 0.9,
            BaseTerrain::Highlands => 0.8,
            BaseTerrain::Geothermal => 0.8,
            BaseTerrain::Mountains => 0.7,
            BaseTerrain::Badlands => 0.7,
            BaseTerrain::Desert => 0.6,
            BaseTerrain::Tundra => 0.6,
        }
    }

    pub const fn item_weights(&self) -> ItemWeights {
        match self {
            BaseTerrain::Desert => ItemWeights {
                weapons: 0.2,
                shields: 0.2,
                consumables: 0.6,
            },
            BaseTerrain::Tundra => ItemWeights {
                weapons: 0.3,
                shields: 0.4,
                consumables: 0.3,
            },
            BaseTerrain::UrbanRuins => ItemWeights {
                weapons: 0.5,
                shields: 0.3,
                consumables: 0.2,
            },
            BaseTerrain::Forest => ItemWeights {
                weapons: 0.3,
                shields: 0.2,
                consumables: 0.5,
            },
            BaseTerrain::Mountains => ItemWeights {
                weapons: 0.4,
                shields: 0.4,
                consumables: 0.2,
            },
            BaseTerrain::Wetlands => ItemWeights {
                weapons: 0.25,
                shields: 0.25,
                consumables: 0.5,
            },
            BaseTerrain::Jungle => ItemWeights {
                weapons: 0.2,
                shields: 0.3,
                consumables: 0.5,
            },
            BaseTerrain::Clearing => ItemWeights {
                weapons: 0.33,
                shields: 0.33,
                consumables: 0.34,
            },
            BaseTerrain::Grasslands => ItemWeights {
                weapons: 0.30,
                shields: 0.30,
                consumables: 0.40,
            },
            BaseTerrain::Badlands => ItemWeights {
                weapons: 0.35,
                shields: 0.30,
                consumables: 0.35,
            },
            BaseTerrain::Highlands => ItemWeights {
                weapons: 0.30,
                shields: 0.35,
                consumables: 0.35,
            },
            BaseTerrain::Geothermal => ItemWeights {
                weapons: 0.30,
                shields: 0.30,
                consumables: 0.40,
            },
        }
    }
}

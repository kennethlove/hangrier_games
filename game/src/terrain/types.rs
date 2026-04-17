use serde::{Deserialize, Serialize};
use strum::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIter)]
pub enum BaseTerrain {
    Clearing,
    Forest,
    Desert,
    Tundra,
    Wetlands,
    Mountains,
    UrbanRuins,
    Jungle,
    Grasslands,
    Badlands,
    Highlands,
    Geothermal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TerrainDescriptor {
    // Temperature
    Hot,
    Cold,
    Temperate,
    // Density/Structure
    Dense,
    Sparse,
    Open,
    // Moisture
    Wet,
    Dry,
    // Altitude
    HighAltitude,
    Lowland,
    // Condition
    Rocky,
    Sandy,
    Frozen,
    Overgrown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerrainType {
    pub base: BaseTerrain,
    pub descriptors: Vec<TerrainDescriptor>,
}

impl TerrainType {
    pub fn new(base: BaseTerrain, descriptors: Vec<TerrainDescriptor>) -> Result<Self, String> {
        // Validate descriptor compatibility
        for descriptor in &descriptors {
            if !Self::is_compatible(&base, descriptor) {
                return Err(format!(
                    "{:?} cannot have {:?} descriptor",
                    base, descriptor
                ));
            }
        }

        Ok(TerrainType { base, descriptors })
    }

    fn is_compatible(base: &BaseTerrain, descriptor: &TerrainDescriptor) -> bool {
        use BaseTerrain::*;
        use TerrainDescriptor::*;

        match (base, descriptor) {
            // Desert cannot be Wet (except during temporary Flood events)
            (Desert, Wet) => false,
            // Tundra must be Cold or Frozen
            (Tundra, Hot) => false,
            (Tundra, Temperate) => false,
            // Geothermal must be Hot
            (Geothermal, Cold) => false,
            (Geothermal, Frozen) => false,
            // Otherwise compatible
            _ => true,
        }
    }
}

impl BaseTerrain {
    pub fn descriptive_name(&self) -> &'static str {
        match self {
            BaseTerrain::Clearing => "clearing",
            BaseTerrain::Forest => "forest",
            BaseTerrain::Desert => "desert",
            BaseTerrain::Tundra => "tundra",
            BaseTerrain::Wetlands => "wetlands",
            BaseTerrain::Mountains => "mountains",
            BaseTerrain::UrbanRuins => "urban ruins",
            BaseTerrain::Jungle => "jungle",
            BaseTerrain::Grasslands => "grasslands",
            BaseTerrain::Badlands => "badlands",
            BaseTerrain::Highlands => "highlands",
            BaseTerrain::Geothermal => "geothermal area",
        }
    }
}

impl TerrainDescriptor {
    pub fn as_adjective(&self) -> &'static str {
        match self {
            TerrainDescriptor::Hot => "hot",
            TerrainDescriptor::Cold => "cold",
            TerrainDescriptor::Temperate => "temperate",
            TerrainDescriptor::Dense => "dense",
            TerrainDescriptor::Sparse => "sparse",
            TerrainDescriptor::Open => "open",
            TerrainDescriptor::Wet => "wet",
            TerrainDescriptor::Dry => "dry",
            TerrainDescriptor::HighAltitude => "high-altitude",
            TerrainDescriptor::Lowland => "lowland",
            TerrainDescriptor::Rocky => "rocky",
            TerrainDescriptor::Sandy => "sandy",
            TerrainDescriptor::Frozen => "frozen",
            TerrainDescriptor::Overgrown => "overgrown",
        }
    }
}

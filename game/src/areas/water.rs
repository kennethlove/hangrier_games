use crate::areas::weather::Weather;
use crate::terrain::types::BaseTerrain;

/// Pure derivation of an area's water-source strength from terrain + weather.
/// 0 = no water available. 3 = abundant. See spec table.
pub fn water_source(terrain: BaseTerrain, weather: &Weather) -> u8 {
    let base = match terrain {
        BaseTerrain::Wetlands => 3,
        BaseTerrain::Forest
        | BaseTerrain::Jungle
        | BaseTerrain::Mountains
        | BaseTerrain::Geothermal => 2,
        BaseTerrain::Highlands
        | BaseTerrain::Clearing
        | BaseTerrain::UrbanRuins
        | BaseTerrain::Tundra => 1,
        BaseTerrain::Grasslands | BaseTerrain::Badlands | BaseTerrain::Desert => 0,
    };

    match weather {
        Weather::Clear | Weather::Blizzard => base,
        Weather::HeavyRain => match terrain {
            // Spec table: HeavyRain column.
            BaseTerrain::Wetlands => 3,
            BaseTerrain::Forest | BaseTerrain::Jungle => 3,
            BaseTerrain::Geothermal | BaseTerrain::Mountains => 2,
            BaseTerrain::Highlands
            | BaseTerrain::Clearing
            | BaseTerrain::UrbanRuins
            | BaseTerrain::Grasslands => 2,
            BaseTerrain::Badlands | BaseTerrain::Tundra | BaseTerrain::Desert => 1,
        },
        Weather::Heatwave => base / 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(BaseTerrain::Wetlands, Weather::Clear, 3)]
    #[case(BaseTerrain::Forest, Weather::Clear, 2)]
    #[case(BaseTerrain::Jungle, Weather::Clear, 2)]
    #[case(BaseTerrain::Mountains, Weather::Clear, 2)]
    #[case(BaseTerrain::Geothermal, Weather::Clear, 2)]
    #[case(BaseTerrain::Highlands, Weather::Clear, 1)]
    #[case(BaseTerrain::Clearing, Weather::Clear, 1)]
    #[case(BaseTerrain::UrbanRuins, Weather::Clear, 1)]
    #[case(BaseTerrain::Tundra, Weather::Clear, 1)]
    #[case(BaseTerrain::Grasslands, Weather::Clear, 0)]
    #[case(BaseTerrain::Badlands, Weather::Clear, 0)]
    #[case(BaseTerrain::Desert, Weather::Clear, 0)]
    fn water_source_clear_table(
        #[case] terrain: BaseTerrain,
        #[case] weather: Weather,
        #[case] expected: u8,
    ) {
        assert_eq!(water_source(terrain, &weather), expected);
    }

    // Per spec table (2026-05-03-shelter-hunger-thirst-design.md), the
    // HeavyRain column is per-terrain; not derivable from base by a single
    // formula.
    #[rstest]
    #[case(BaseTerrain::Wetlands, 3)]
    #[case(BaseTerrain::Forest, 3)]
    #[case(BaseTerrain::Jungle, 3)]
    #[case(BaseTerrain::Geothermal, 2)]
    #[case(BaseTerrain::Mountains, 2)]
    #[case(BaseTerrain::Highlands, 2)]
    #[case(BaseTerrain::Clearing, 2)]
    #[case(BaseTerrain::UrbanRuins, 2)]
    #[case(BaseTerrain::Grasslands, 2)]
    #[case(BaseTerrain::Badlands, 1)]
    #[case(BaseTerrain::Tundra, 1)]
    #[case(BaseTerrain::Desert, 1)]
    fn water_source_heavy_rain_table(#[case] terrain: BaseTerrain, #[case] expected: u8) {
        assert_eq!(water_source(terrain, &Weather::HeavyRain), expected);
    }

    #[rstest]
    #[case(BaseTerrain::Wetlands, 1)] // 3 / 2 = 1
    #[case(BaseTerrain::Mountains, 1)] // 2 / 2 = 1
    #[case(BaseTerrain::Highlands, 0)] // 1 / 2 = 0
    #[case(BaseTerrain::Desert, 0)]
    fn water_source_heatwave_halves_base(#[case] terrain: BaseTerrain, #[case] expected: u8) {
        assert_eq!(water_source(terrain, &Weather::Heatwave), expected);
    }
}

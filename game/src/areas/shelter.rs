use crate::areas::weather::Weather;
use crate::terrain::types::BaseTerrain;

/// Pure derivation of an area's shelter quality from terrain and current weather.
/// 0 = no shelter possible. 4 = excellent shelter. See spec table.
pub fn shelter_quality(terrain: BaseTerrain, weather: &Weather) -> u8 {
    let base = match terrain {
        BaseTerrain::UrbanRuins => 3,
        BaseTerrain::Forest
        | BaseTerrain::Jungle
        | BaseTerrain::Mountains
        | BaseTerrain::Geothermal => 2,
        BaseTerrain::Wetlands
        | BaseTerrain::Highlands
        | BaseTerrain::Clearing
        | BaseTerrain::Grasslands
        | BaseTerrain::Badlands => 1,
        BaseTerrain::Tundra | BaseTerrain::Desert => 0,
    };

    match weather {
        Weather::Clear => base,
        Weather::HeavyRain | Weather::Blizzard => base.saturating_sub(1),
        Weather::Heatwave => match terrain {
            BaseTerrain::UrbanRuins
            | BaseTerrain::Mountains
            | BaseTerrain::Geothermal
            | BaseTerrain::Forest
            | BaseTerrain::Jungle => base,
            BaseTerrain::Tundra | BaseTerrain::Desert => 0,
            _ => base,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(BaseTerrain::UrbanRuins, Weather::Clear, 3)]
    #[case(BaseTerrain::Forest, Weather::Clear, 2)]
    #[case(BaseTerrain::Jungle, Weather::Clear, 2)]
    #[case(BaseTerrain::Mountains, Weather::Clear, 2)]
    #[case(BaseTerrain::Geothermal, Weather::Clear, 2)]
    #[case(BaseTerrain::Wetlands, Weather::Clear, 1)]
    #[case(BaseTerrain::Highlands, Weather::Clear, 1)]
    #[case(BaseTerrain::Clearing, Weather::Clear, 1)]
    #[case(BaseTerrain::Grasslands, Weather::Clear, 1)]
    #[case(BaseTerrain::Badlands, Weather::Clear, 1)]
    #[case(BaseTerrain::Tundra, Weather::Clear, 0)]
    #[case(BaseTerrain::Desert, Weather::Clear, 0)]
    fn shelter_quality_clear_weather_table(
        #[case] terrain: BaseTerrain,
        #[case] weather: Weather,
        #[case] expected: u8,
    ) {
        assert_eq!(shelter_quality(terrain, &weather), expected);
    }

    #[rstest]
    #[case(BaseTerrain::Forest, Weather::HeavyRain, 1)]
    #[case(BaseTerrain::UrbanRuins, Weather::HeavyRain, 2)]
    #[case(BaseTerrain::Desert, Weather::HeavyRain, 0)]
    #[case(BaseTerrain::Forest, Weather::Blizzard, 1)]
    #[case(BaseTerrain::Tundra, Weather::Blizzard, 0)]
    fn shelter_quality_storm_modifier(
        #[case] terrain: BaseTerrain,
        #[case] weather: Weather,
        #[case] expected: u8,
    ) {
        assert_eq!(shelter_quality(terrain, &weather), expected);
    }

    #[rstest]
    #[case(BaseTerrain::UrbanRuins, 3)]
    #[case(BaseTerrain::Mountains, 2)]
    #[case(BaseTerrain::Geothermal, 2)]
    #[case(BaseTerrain::Forest, 2)]
    #[case(BaseTerrain::Jungle, 2)]
    #[case(BaseTerrain::Tundra, 0)]
    #[case(BaseTerrain::Desert, 0)]
    fn shelter_quality_heatwave_keeps_stone_and_canopy(
        #[case] terrain: BaseTerrain,
        #[case] expected: u8,
    ) {
        assert_eq!(shelter_quality(terrain, &Weather::Heatwave), expected);
    }
}

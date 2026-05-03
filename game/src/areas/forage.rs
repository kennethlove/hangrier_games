use crate::terrain::types::BaseTerrain;

/// Pure derivation of an area's forage richness from terrain.
/// 0 = barren. 4 = abundant. See spec table.
pub fn forage_richness(terrain: BaseTerrain) -> u8 {
    match terrain {
        BaseTerrain::Wetlands | BaseTerrain::Jungle => 3,
        BaseTerrain::Forest | BaseTerrain::UrbanRuins => 2,
        BaseTerrain::Mountains
        | BaseTerrain::Highlands
        | BaseTerrain::Clearing
        | BaseTerrain::Grasslands
        | BaseTerrain::Geothermal => 1,
        BaseTerrain::Badlands | BaseTerrain::Tundra | BaseTerrain::Desert => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(BaseTerrain::Wetlands, 3)]
    #[case(BaseTerrain::Jungle, 3)]
    #[case(BaseTerrain::Forest, 2)]
    #[case(BaseTerrain::UrbanRuins, 2)]
    #[case(BaseTerrain::Mountains, 1)]
    #[case(BaseTerrain::Highlands, 1)]
    #[case(BaseTerrain::Clearing, 1)]
    #[case(BaseTerrain::Grasslands, 1)]
    #[case(BaseTerrain::Geothermal, 1)]
    #[case(BaseTerrain::Badlands, 0)]
    #[case(BaseTerrain::Tundra, 0)]
    #[case(BaseTerrain::Desert, 0)]
    fn forage_richness_table(#[case] terrain: BaseTerrain, #[case] expected: u8) {
        assert_eq!(forage_richness(terrain), expected);
    }
}

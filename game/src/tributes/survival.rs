use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum HungerBand {
    Sated,
    Peckish,
    Hungry,
    Starving,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ThirstBand {
    Sated,
    Thirsty,
    Parched,
    Dehydrated,
}

pub fn hunger_band(value: u8) -> HungerBand {
    match value {
        0 => HungerBand::Sated,
        1..=2 => HungerBand::Peckish,
        3..=4 => HungerBand::Hungry,
        _ => HungerBand::Starving,
    }
}

pub fn thirst_band(value: u8) -> ThirstBand {
    match value {
        0 => ThirstBand::Sated,
        1 => ThirstBand::Thirsty,
        2 => ThirstBand::Parched,
        _ => ThirstBand::Dehydrated,
    }
}

/// True if a band-change event into this band should be surfaced in the
/// public timeline (Action panel). Lower bands are private/Inspect-only.
pub fn hunger_band_is_public(band: HungerBand) -> bool {
    matches!(band, HungerBand::Hungry | HungerBand::Starving)
}

pub fn thirst_band_is_public(band: ThirstBand) -> bool {
    matches!(band, ThirstBand::Parched | ThirstBand::Dehydrated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(0, HungerBand::Sated)]
    #[case(1, HungerBand::Peckish)]
    #[case(2, HungerBand::Peckish)]
    #[case(3, HungerBand::Hungry)]
    #[case(4, HungerBand::Hungry)]
    #[case(5, HungerBand::Starving)]
    #[case(99, HungerBand::Starving)]
    fn hunger_band_thresholds(#[case] value: u8, #[case] expected: HungerBand) {
        assert_eq!(hunger_band(value), expected);
    }

    #[rstest]
    #[case(0, ThirstBand::Sated)]
    #[case(1, ThirstBand::Thirsty)]
    #[case(2, ThirstBand::Parched)]
    #[case(3, ThirstBand::Dehydrated)]
    #[case(99, ThirstBand::Dehydrated)]
    fn thirst_band_thresholds(#[case] value: u8, #[case] expected: ThirstBand) {
        assert_eq!(thirst_band(value), expected);
    }

    #[test]
    fn hunger_starving_is_publicly_visible() {
        assert!(hunger_band_is_public(HungerBand::Starving));
        assert!(hunger_band_is_public(HungerBand::Hungry));
        assert!(!hunger_band_is_public(HungerBand::Peckish));
        assert!(!hunger_band_is_public(HungerBand::Sated));
    }

    #[test]
    fn thirst_dehydrated_is_publicly_visible() {
        assert!(thirst_band_is_public(ThirstBand::Dehydrated));
        assert!(thirst_band_is_public(ThirstBand::Parched));
        assert!(!thirst_band_is_public(ThirstBand::Thirsty));
        assert!(!thirst_band_is_public(ThirstBand::Sated));
    }
}

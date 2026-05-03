//! Pure derivation of `StaminaBand` from a tribute's stamina ratio.
//! Thresholds come from `CombatTuning`. See spec
//! `docs/superpowers/specs/2026-05-03-stamina-combat-resource-design.md`.

use crate::tributes::combat_tuning::CombatTuning;
use shared::messages::StaminaBand;

/// Returns the `StaminaBand` for the given stamina/max_stamina pair.
///
/// - `> band_winded_pct`% => Fresh
/// - `> band_exhausted_pct`% but `<= band_winded_pct`% => Winded
/// - `<= band_exhausted_pct`% => Exhausted
/// - `max_stamina == 0` => Exhausted (defensive; should not occur in practice)
pub fn stamina_band(stamina: u32, max_stamina: u32, tuning: &CombatTuning) -> StaminaBand {
    if max_stamina == 0 {
        return StaminaBand::Exhausted;
    }
    let pct = ((stamina.saturating_mul(100)) / max_stamina) as u8;
    if pct > tuning.band_winded_pct {
        StaminaBand::Fresh
    } else if pct > tuning.band_exhausted_pct {
        StaminaBand::Winded
    } else {
        StaminaBand::Exhausted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn t() -> CombatTuning {
        CombatTuning::default()
    }

    #[rstest]
    #[case(100, 100, StaminaBand::Fresh)]
    #[case(75, 100, StaminaBand::Fresh)]
    #[case(51, 100, StaminaBand::Fresh)]
    #[case(50, 100, StaminaBand::Winded)]
    #[case(30, 100, StaminaBand::Winded)]
    #[case(21, 100, StaminaBand::Winded)]
    #[case(20, 100, StaminaBand::Exhausted)]
    #[case(5, 100, StaminaBand::Exhausted)]
    #[case(0, 100, StaminaBand::Exhausted)]
    fn band_thresholds(#[case] stamina: u32, #[case] max: u32, #[case] expected: StaminaBand) {
        assert_eq!(stamina_band(stamina, max, &t()), expected);
    }

    #[test]
    fn zero_max_returns_exhausted() {
        assert_eq!(stamina_band(0, 0, &t()), StaminaBand::Exhausted);
        assert_eq!(stamina_band(100, 0, &t()), StaminaBand::Exhausted);
    }

    #[test]
    fn custom_thresholds_respected() {
        let tuning = CombatTuning {
            band_winded_pct: 70,
            band_exhausted_pct: 30,
            ..CombatTuning::default()
        };
        assert_eq!(stamina_band(71, 100, &tuning), StaminaBand::Fresh);
        assert_eq!(stamina_band(70, 100, &tuning), StaminaBand::Winded);
        assert_eq!(stamina_band(31, 100, &tuning), StaminaBand::Winded);
        assert_eq!(stamina_band(30, 100, &tuning), StaminaBand::Exhausted);
    }
}

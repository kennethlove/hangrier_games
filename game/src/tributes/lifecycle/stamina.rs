use crate::tributes::Tribute;
use crate::tributes::actions::Action;
use crate::tributes::combat_tuning::CombatTuning;
use crate::tributes::survival::{HungerBand, ThirstBand};

impl Tribute {
    /// Recover stamina based on the tribute's current action and survival state.
    ///
    /// Per-phase rates (from `CombatTuning`):
    /// - idle (any non-Rest action): `recover_idle` (default 5)
    /// - resting in the open: `recover_resting` (default 30)
    /// - resting while sheltered: `recover_resting_sheltered` (default 60)
    ///
    /// Multiplied by `recover_starving_dehydrated_mul` (default 0.5) when the
    /// tribute is Starving OR Dehydrated. Result is capped at `max_stamina`.
    /// See spec `docs/superpowers/specs/2026-05-03-stamina-combat-resource-design.md`.
    pub fn recover_stamina(
        &mut self,
        action: &Action,
        sheltered: bool,
        hunger: HungerBand,
        thirst: ThirstBand,
        tuning: &CombatTuning,
    ) {
        let base = match (action, sheltered) {
            (Action::Rest, true) => tuning.recovery_sheltered_resting,
            (Action::Rest, false) => tuning.recovery_resting,
            _ => tuning.recovery_idle,
        };
        let mul =
            if matches!(hunger, HungerBand::Starving) || matches!(thirst, ThirstBand::Dehydrated) {
                tuning.recovery_starving_dehydrated_mult
            } else {
                1.0
            };
        let amount = ((base as f64) * mul).round() as u32;
        self.stamina = (self.stamina + amount).min(self.max_stamina);
    }
}

#[cfg(test)]
mod tests {
    use crate::tributes::Tribute;
    use crate::tributes::actions::Action;
    use crate::tributes::combat_tuning::CombatTuning;
    use crate::tributes::survival::{HungerBand, ThirstBand};

    fn fresh_state() -> (HungerBand, ThirstBand) {
        (HungerBand::Sated, ThirstBand::Sated)
    }

    #[test]
    fn recover_idle_adds_5() {
        let mut t = Tribute {
            stamina: 50,
            max_stamina: 100,
            ..Tribute::default()
        };
        let tuning = CombatTuning::default();
        let (h, th) = fresh_state();
        t.recover_stamina(&Action::None, false, h, th, &tuning);
        assert_eq!(t.stamina, 55);
    }

    #[test]
    fn recover_resting_adds_30() {
        let mut t = Tribute {
            stamina: 50,
            max_stamina: 100,
            ..Tribute::default()
        };
        let tuning = CombatTuning::default();
        let (h, th) = fresh_state();
        t.recover_stamina(&Action::Rest, false, h, th, &tuning);
        assert_eq!(t.stamina, 80);
    }

    #[test]
    fn recover_sheltered_resting_adds_60() {
        let mut t = Tribute {
            stamina: 30,
            max_stamina: 100,
            ..Tribute::default()
        };
        let tuning = CombatTuning::default();
        let (h, th) = fresh_state();
        t.recover_stamina(&Action::Rest, true, h, th, &tuning);
        assert_eq!(t.stamina, 90);
    }

    #[test]
    fn recover_caps_at_max_stamina() {
        let mut t = Tribute {
            stamina: 80,
            max_stamina: 100,
            ..Tribute::default()
        };
        let tuning = CombatTuning::default();
        let (h, th) = fresh_state();
        t.recover_stamina(&Action::Rest, true, h, th, &tuning);
        assert_eq!(t.stamina, 100);
    }

    #[test]
    fn recover_starving_halves_rate() {
        let mut t = Tribute {
            stamina: 50,
            max_stamina: 100,
            ..Tribute::default()
        };
        let tuning = CombatTuning::default();
        t.recover_stamina(
            &Action::Rest,
            false,
            HungerBand::Starving,
            ThirstBand::Sated,
            &tuning,
        );
        assert_eq!(t.stamina, 65);
    }

    #[test]
    fn recover_dehydrated_halves_rate() {
        let mut t = Tribute {
            stamina: 50,
            max_stamina: 100,
            ..Tribute::default()
        };
        let tuning = CombatTuning::default();
        t.recover_stamina(
            &Action::Rest,
            false,
            HungerBand::Sated,
            ThirstBand::Dehydrated,
            &tuning,
        );
        assert_eq!(t.stamina, 65);
    }
}

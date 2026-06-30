//! Scoring and override functions for the brain pipeline.
//!
//! These free functions help decide what action a tribute should take:
//! - `survival_override`: hunger/thirst-driven overrides
//! - `stamina_override`: exhaust-driven flee-or-rest logic
//! - `target_attack_score`: score a candidate target for Attack
//! - `action_score`: score a candidate action (gate on stamina)

use crate::tributes::Tribute;
use crate::tributes::actions::Action;

/// Survival override branch. Returns `Some(action)` to short-circuit the
/// Brain's normal weighted scoring; returns `None` to fall through.
///
/// Order (per spec §6.4):
/// 1. Dehydrated + at water-source terrain -> `DrinkFromTerrain`.
/// 2. Dehydrated + Water item in inventory -> `DrinkItem`.
/// 3. Starving + Food item in inventory -> `Eat`.
/// 4. Starving + at forageable terrain (and not in combat) -> `Forage`.
///
/// Active combat suppresses all overrides (the existing combat handling
/// preempts decision-making upstream — this is a defensive guard).
pub fn survival_override(
    tribute: &Tribute,
    terrain: crate::terrain::BaseTerrain,
    weather: &crate::areas::weather::Weather,
    in_combat: bool,
) -> Option<Action> {
    use crate::areas::forage::forage_richness;
    use crate::areas::water::water_source;
    use crate::tributes::survival::{HungerBand, ThirstBand, hunger_band, thirst_band};

    if in_combat {
        return None;
    }

    let dehydrated = thirst_band(tribute.thirst) == ThirstBand::Dehydrated;
    let starving = hunger_band(tribute.hunger) == HungerBand::Starving;

    if dehydrated && water_source(terrain, weather) > 0 {
        return Some(Action::DrinkFromTerrain);
    }
    if dehydrated
        && let Some(item) = tribute
            .items
            .iter()
            .find(|i| i.item_type.is_water())
            .cloned()
    {
        return Some(Action::DrinkItem(Some(item)));
    }
    if starving {
        if let Some(item) = tribute
            .items
            .iter()
            .find(|i| i.item_type.is_food())
            .cloned()
        {
            return Some(Action::Eat(Some(item)));
        }
        if forage_richness(terrain) > 0 {
            return Some(Action::Forage);
        }
    }

    None
}

/// Stamina-band override layer. Returns `Some(Action)` to override the
/// standard brain when the actor is Exhausted; returns `None` for Fresh and
/// Winded (Winded is handled at action-scoring time via
/// `winded_attack_score_penalty`).
///
/// Pipeline order (see spec):
/// 1. Combat preempt
/// 2. Gamemaker overrides
/// 3. Hunger/thirst overrides (`survival_override`)
/// 4. Stamina overrides (this fn)
/// 5. Standard brain logic
pub fn stamina_override(
    tribute: &Tribute,
    nearby: &[Tribute],
    sheltered: bool,
    tuning: &crate::tributes::combat_tuning::CombatTuning,
) -> Option<Action> {
    use crate::tributes::stamina_band::stamina_band;
    use shared::messages::StaminaBand;

    let band = stamina_band(tribute.stamina, tribute.max_stamina, tuning);
    if band != StaminaBand::Exhausted {
        return None;
    }

    // Visible-band flee: any nearby living tribute (other than self) with a
    // better band is a threat. If we're not already sheltered, flee. The
    // destination layer fills the destination based on the threat location.
    if !sheltered {
        let any_threat = nearby.iter().any(|other| {
            other.identifier != tribute.identifier && other.effective_health() > 0 && {
                let other_band = stamina_band(other.stamina, other.max_stamina, tuning);
                matches!(other_band, StaminaBand::Fresh | StaminaBand::Winded)
            }
        });
        if any_threat {
            return Some(Action::Move(None));
        }
    }

    // Otherwise hold position and recover.
    Some(Action::Rest)
}

/// Score a candidate target for an `Action::Attack` decision. Higher is better.
///
/// Baseline favors low-HP targets (`-target.health`). When the actor is Fresh
/// and the target is Winded or Exhausted, adds
/// `tuning.fresh_target_visibly_tired_bonus` (predator instinct).
#[allow(dead_code)]
pub fn target_attack_score(
    actor: &Tribute,
    target: &Tribute,
    tuning: &crate::tributes::combat_tuning::CombatTuning,
) -> i32 {
    use crate::tributes::stamina_band::stamina_band;
    use shared::messages::StaminaBand;

    let base: i32 = -(target.effective_health() as i32);

    let actor_band = stamina_band(actor.stamina, actor.max_stamina, tuning);
    let target_band = stamina_band(target.stamina, target.max_stamina, tuning);

    let predator_bonus = if matches!(actor_band, StaminaBand::Fresh)
        && matches!(target_band, StaminaBand::Winded | StaminaBand::Exhausted)
    {
        tuning.fresh_target_visibly_tired_bonus
    } else {
        0
    };

    base + predator_bonus
}

/// Score a candidate action. `i32::MIN` signals "unavailable".
///
/// `Action::Attack` is gated on `actor.stamina >= tuning.stamina_cost_attacker`;
/// Winded actors get `tuning.winded_attack_score_penalty` added (negative).
pub fn action_score(
    actor: &Tribute,
    action: &Action,
    _nearby: &[Tribute],
    tuning: &crate::tributes::combat_tuning::CombatTuning,
) -> i32 {
    use crate::tributes::stamina_band::stamina_band;
    use shared::messages::StaminaBand;

    match action {
        Action::Attack => {
            if actor.stamina < tuning.stamina_cost_attacker {
                return i32::MIN;
            }
            let band = stamina_band(actor.stamina, actor.max_stamina, tuning);
            match band {
                StaminaBand::Fresh => 0,
                StaminaBand::Winded => tuning.winded_attack_score_penalty,
                StaminaBand::Exhausted => tuning.winded_attack_score_penalty,
            }
        }
        _ => 0,
    }
}

//! Deterministic integration tests for the shelter + hunger/thirst
//! survival system (PR1 backend). See
//! docs/superpowers/specs/2026-05-03-shelter-hunger-thirst-design.md.

use game::areas::weather::Weather;
use game::tributes::Tribute;
use game::tributes::survival::{
    ThirstBand, apply_dehydration_drain, apply_starvation_drain, drink_water, thirst_band,
    tick_survival,
};

fn mid_tribute(name: &str) -> Tribute {
    let mut t = Tribute::new(name.to_string(), None, None);
    t.attributes.health = 100;
    // Mid-range strength so the hunger tick lands the +1 base path.
    t.attributes.strength = 50;
    // Stamina at half-max so the thirst tick lands +1 / phase, not the
    // every-other-phase low-stamina path.
    t.stamina = t.max_stamina / 2;
    t
}

#[test]
fn no_food_no_water_dies_of_dehydration_first() {
    let mut t = mid_tribute("Doomed");
    let mut phases_to_dehydrated_band = 0u32;
    for phase in 1..=20 {
        tick_survival(&mut t, &Weather::Clear, false);
        if thirst_band(t.thirst) == ThirstBand::Dehydrated && phases_to_dehydrated_band == 0 {
            phases_to_dehydrated_band = phase;
        }
        let _ = apply_dehydration_drain(&mut t);
        let _ = apply_starvation_drain(&mut t);
        if t.attributes.health == 0 {
            // Confirm thirst drove the death.
            assert_eq!(thirst_band(t.thirst), ThirstBand::Dehydrated);
            assert!(
                phases_to_dehydrated_band > 0 && phases_to_dehydrated_band < phase,
                "must reach Dehydrated before death"
            );
            return;
        }
    }
    panic!("tribute did not die in 20 phases");
}

#[test]
fn carrying_water_extends_survival() {
    // Drink at phase 4 to demonstrate that a water sip resets the
    // dehydration debt and pushes the death window further out.
    fn run_to_death(drink_amount_at_phase_4: u8) -> u32 {
        let mut t = mid_tribute("R");
        for phase in 1..=30 {
            tick_survival(&mut t, &Weather::Clear, false);
            if phase == 4 && drink_amount_at_phase_4 > 0 {
                drink_water(&mut t, drink_amount_at_phase_4);
            }
            let _ = apply_dehydration_drain(&mut t);
            let _ = apply_starvation_drain(&mut t);
            if t.attributes.health == 0 {
                return phase;
            }
        }
        30
    }

    let baseline = run_to_death(0);
    let with_water = run_to_death(4);
    assert!(
        with_water >= baseline + 2,
        "carrying water should extend life by at least 2 phases (baseline={baseline}, with_water={with_water})"
    );
}

#[test]
fn sheltered_in_heatwave_does_not_accrue_weather_thirst() {
    let mut t_sheltered = mid_tribute("S");
    let mut t_exposed = mid_tribute("E");

    for _ in 0..3 {
        tick_survival(&mut t_sheltered, &Weather::Heatwave, true);
        tick_survival(&mut t_exposed, &Weather::Heatwave, false);
    }
    assert!(
        t_exposed.thirst > t_sheltered.thirst,
        "exposed tribute should accrue more thirst (sheltered={}, exposed={})",
        t_sheltered.thirst,
        t_exposed.thirst
    );
}

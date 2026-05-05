//! Per-area environmental conditions for a single phase.
//!
//! Spec: `docs/superpowers/specs/2026-05-03-four-phase-day-design.md` §6.1,
//! §6.2, §6.5.
//!
//! This module defines:
//!
//! - [`LightLevel`] — `Bright`/`Dim`/`Dark`, derived (not stored) from
//!   `(phase, biome, weather)` per area.
//! - [`AfflictionDraft`] — an environmental affliction candidate produced
//!   by the per-area roll, ready to be applied to tributes in that area
//!   during PR2c's pipeline pass.
//! - [`AreaPhaseConditions`] — the cached output stored on each area for
//!   the duration of a phase.
//! - [`derive_light_level`] — pure function deriving light from
//!   `(phase, biome, weather)`.
//! - [`roll_environmental_afflictions`] — deterministic roll producing
//!   `Vec<AfflictionDraft>` for `(phase, biome, weather, sheltered, rng)`.
//!
//! PR2b is substrate-only: callers do not yet exist. PR2c (bd-9sjj) wires
//! these into [`crate::games::Game::execute_cycle`] and into per-tribute
//! affliction application.

use crate::areas::weather::Weather;
use crate::messages::Phase;
use crate::terrain::types::BaseTerrain;
use crate::tributes::statuses::TributeStatus;
use rand::{Rng, RngExt};
use serde::{Deserialize, Serialize};

/// Three discrete light bands derived from `(phase, biome, weather)`. The
/// brain reads this value to bias detection / ambush / movement weights;
/// it is never stored on the area.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum LightLevel {
    /// Full daylight or open visibility.
    #[default]
    Bright,
    /// Dawn / dusk / overcast / forest understory.
    Dim,
    /// Night, blizzard whiteout, deep cave / underground.
    Dark,
}

/// A single environmental affliction candidate produced by the per-area
/// phase roll. PR2c attaches these to tributes via the existing status
/// system (see `Tribute::status` / `TributeStatus`).
///
/// The wire format intentionally only carries the resulting
/// [`TributeStatus`] for now. PR2c may extend with severity / source
/// metadata once the consumer is implemented.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AfflictionDraft {
    pub status: TributeStatus,
}

impl AfflictionDraft {
    pub fn new(status: TributeStatus) -> Self {
        Self { status }
    }
}

/// Per-area, per-phase environmental output. Computed once at the top of
/// each phase by the cycle pipeline (PR2c) and read N times by tributes
/// in that area without recomputation.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AreaPhaseConditions {
    pub light: LightLevel,
    pub weather: Weather,
    /// Affliction drafts unconditionally rolled by this area for this
    /// phase. Per-tribute filtering (sheltered, immune, etc.) is the
    /// caller's responsibility — see [`roll_environmental_afflictions`]
    /// for the canonical roll, which already takes `sheltered`.
    pub afflictions: Vec<AfflictionDraft>,
}

/// Derive [`LightLevel`] from `(phase, biome, weather)`.
///
/// Weather can downgrade a baseline phase light level (storm darkens day,
/// blizzard darkens night further but already maxes at Dark). Biome
/// applies a one-step modifier for dense canopy / underground.
pub fn derive_light_level(phase: Phase, biome: BaseTerrain, weather: Weather) -> LightLevel {
    let baseline = match phase {
        Phase::Day => LightLevel::Bright,
        Phase::Dawn | Phase::Dusk => LightLevel::Dim,
        Phase::Night => LightLevel::Dark,
    };
    let after_weather = match weather {
        Weather::Clear | Weather::Heatwave => baseline,
        Weather::HeavyRain => darken(baseline),
        Weather::Blizzard => darken(darken(baseline)),
    };
    match biome {
        BaseTerrain::Jungle | BaseTerrain::Forest => darken(after_weather),
        _ => after_weather,
    }
}

fn darken(l: LightLevel) -> LightLevel {
    match l {
        LightLevel::Bright => LightLevel::Dim,
        LightLevel::Dim | LightLevel::Dark => LightLevel::Dark,
    }
}

/// Roll environmental afflictions for a single area at the start of a
/// phase. Sheltered tributes are immune — the caller passes `sheltered`
/// to short-circuit the entire roll (the per-area output is the same for
/// every unsheltered tribute in that area).
///
/// Deterministic given a seeded RNG: same `(phase, biome, weather,
/// sheltered, rng-state)` always returns the same `Vec`. PR2c will pass
/// a seeded `SmallRng` derived from `(game_seed, day, phase, area_id)`.
///
/// The probability tables are placeholder values matching the spec's
/// qualitative examples (§6.2). They will be tuned post-observability,
/// using the same pattern as `AfflictionTuning`.
pub fn roll_environmental_afflictions(
    phase: Phase,
    biome: BaseTerrain,
    weather: Weather,
    sheltered: bool,
    rng: &mut impl Rng,
) -> Vec<AfflictionDraft> {
    if sheltered {
        return Vec::new();
    }
    let mut out: Vec<AfflictionDraft> = Vec::new();
    for (status, p) in candidate_probabilities(phase, biome, weather) {
        if p > 0.0 && rng.random_bool(p as f64) {
            out.push(AfflictionDraft::new(status));
        }
    }
    out
}

/// Returns the placeholder probability table for `(phase, biome,
/// weather)`. Each entry is a candidate `(TributeStatus, probability)`
/// rolled independently. Probabilities are intentionally small; PR2c
/// observability will drive tuning.
fn candidate_probabilities(
    phase: Phase,
    biome: BaseTerrain,
    weather: Weather,
) -> Vec<(TributeStatus, f32)> {
    use BaseTerrain::*;
    use Phase::*;
    use Weather::*;

    let mut out: Vec<(TributeStatus, f32)> = Vec::new();

    // ---- Cold / Frozen pressure ------------------------------------
    let cold = match (phase, biome, weather) {
        // Spec §6.2: Night + tundra + any → strong chance Frozen
        (Night, Tundra, _) => 0.40,
        // Spec §6.2: Dawn + tundra + clear → small chance Frozen
        (Dawn, Tundra, Clear) => 0.10,
        (Dawn | Dusk, Tundra, _) => 0.20,
        (Day, Tundra, Blizzard) => 0.30,
        (Day, Tundra, _) => 0.05,
        // Blizzard anywhere
        (_, _, Blizzard) => 0.20,
        // Spec §6.2: Night + temperate + storm → small chance mild
        // Frozen. Treat HeavyRain as the storm proxy for now.
        (Night, Mountains | Highlands, _) => 0.15,
        (Night, _, HeavyRain) => 0.05,
        _ => 0.0,
    };
    if cold > 0.0 {
        out.push((TributeStatus::Frozen, cold));
    }

    // ---- Heat / Overheated pressure --------------------------------
    let heat = match (phase, biome, weather) {
        // Spec §6.2: Day + desert + clear → moderate chance Overheated
        (Day, Desert, Clear) => 0.30,
        (Day, Desert, Heatwave) => 0.50,
        (Day, Badlands | Grasslands, Heatwave) => 0.25,
        (Day, _, Heatwave) => 0.15,
        (Dusk, Desert, _) => 0.10,
        _ => 0.0,
    };
    if heat > 0.0 {
        out.push((TributeStatus::Overheated, heat));
    }

    // ---- Sickness from exposure ------------------------------------
    let sick = match (phase, biome, weather) {
        // Spec §6.2: Day + jungle + storm → moderate chance Sick
        (Day, Jungle, HeavyRain) => 0.25,
        (_, Wetlands, HeavyRain) => 0.15,
        (Night, Jungle | Wetlands, _) => 0.05,
        _ => 0.0,
    };
    if sick > 0.0 {
        out.push((TributeStatus::Sick, sick));
    }

    // Stable order independent of insertion paths above.
    out.sort_by_key(|(s, _)| s.to_string());
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;
    use rstest::rstest;

    #[test]
    fn light_level_default_is_bright() {
        assert_eq!(LightLevel::default(), LightLevel::Bright);
    }

    #[test]
    fn light_level_serde_roundtrip() {
        for l in [LightLevel::Bright, LightLevel::Dim, LightLevel::Dark] {
            let s = serde_json::to_string(&l).unwrap();
            let back: LightLevel = serde_json::from_str(&s).unwrap();
            assert_eq!(back, l);
        }
    }

    #[test]
    fn area_phase_conditions_default_is_clear_bright_empty() {
        let c = AreaPhaseConditions::default();
        assert_eq!(c.light, LightLevel::Bright);
        assert_eq!(c.weather, Weather::Clear);
        assert!(c.afflictions.is_empty());
    }

    #[rstest]
    #[case(Phase::Day, BaseTerrain::Clearing, Weather::Clear, LightLevel::Bright)]
    #[case(Phase::Dawn, BaseTerrain::Clearing, Weather::Clear, LightLevel::Dim)]
    #[case(Phase::Dusk, BaseTerrain::Clearing, Weather::Clear, LightLevel::Dim)]
    #[case(Phase::Night, BaseTerrain::Clearing, Weather::Clear, LightLevel::Dark)]
    fn light_level_baseline_per_phase(
        #[case] p: Phase,
        #[case] b: BaseTerrain,
        #[case] w: Weather,
        #[case] expected: LightLevel,
    ) {
        assert_eq!(derive_light_level(p, b, w), expected);
    }

    #[test]
    fn light_level_jungle_is_one_step_darker_than_clearing() {
        // Day + jungle + clear should be Dim (forest canopy darkens).
        assert_eq!(
            derive_light_level(Phase::Day, BaseTerrain::Jungle, Weather::Clear),
            LightLevel::Dim
        );
        // Day + forest + clear → Dim as well.
        assert_eq!(
            derive_light_level(Phase::Day, BaseTerrain::Forest, Weather::Clear),
            LightLevel::Dim
        );
    }

    #[test]
    fn light_level_blizzard_max_darkens() {
        // Day + clearing + blizzard → Dark (two darken steps from Bright).
        assert_eq!(
            derive_light_level(Phase::Day, BaseTerrain::Clearing, Weather::Blizzard),
            LightLevel::Dark
        );
    }

    #[test]
    fn sheltered_tribute_gets_no_afflictions() {
        let mut rng = SmallRng::seed_from_u64(1);
        let drafts = roll_environmental_afflictions(
            Phase::Night,
            BaseTerrain::Tundra,
            Weather::Blizzard,
            true,
            &mut rng,
        );
        assert!(drafts.is_empty());
    }

    #[test]
    fn roll_is_deterministic_given_seed() {
        let make = || {
            let mut rng = SmallRng::seed_from_u64(42);
            roll_environmental_afflictions(
                Phase::Night,
                BaseTerrain::Tundra,
                Weather::Clear,
                false,
                &mut rng,
            )
        };
        assert_eq!(make(), make());
    }

    #[test]
    fn night_tundra_eventually_inflicts_frozen() {
        // Strong chance per spec — over many rolls we must see Frozen.
        let mut saw_frozen = false;
        for seed in 0..200u64 {
            let mut rng = SmallRng::seed_from_u64(seed);
            let drafts = roll_environmental_afflictions(
                Phase::Night,
                BaseTerrain::Tundra,
                Weather::Clear,
                false,
                &mut rng,
            );
            if drafts.iter().any(|d| d.status == TributeStatus::Frozen) {
                saw_frozen = true;
                break;
            }
        }
        assert!(saw_frozen);
    }

    #[test]
    fn day_desert_eventually_inflicts_overheated() {
        let mut saw = false;
        for seed in 0..200u64 {
            let mut rng = SmallRng::seed_from_u64(seed);
            let drafts = roll_environmental_afflictions(
                Phase::Day,
                BaseTerrain::Desert,
                Weather::Clear,
                false,
                &mut rng,
            );
            if drafts.iter().any(|d| d.status == TributeStatus::Overheated) {
                saw = true;
                break;
            }
        }
        assert!(saw);
    }

    #[test]
    fn day_jungle_storm_eventually_inflicts_sick() {
        let mut saw = false;
        for seed in 0..200u64 {
            let mut rng = SmallRng::seed_from_u64(seed);
            let drafts = roll_environmental_afflictions(
                Phase::Day,
                BaseTerrain::Jungle,
                Weather::HeavyRain,
                false,
                &mut rng,
            );
            if drafts.iter().any(|d| d.status == TributeStatus::Sick) {
                saw = true;
                break;
            }
        }
        assert!(saw);
    }

    #[test]
    fn day_clearing_clear_is_quiet() {
        // Mild conditions → no afflictions across many seeds.
        for seed in 0..50u64 {
            let mut rng = SmallRng::seed_from_u64(seed);
            let drafts = roll_environmental_afflictions(
                Phase::Day,
                BaseTerrain::Clearing,
                Weather::Clear,
                false,
                &mut rng,
            );
            assert!(
                drafts.is_empty(),
                "unexpected draft at seed {seed}: {drafts:?}"
            );
        }
    }

    #[test]
    fn affliction_draft_serde_roundtrip() {
        let d = AfflictionDraft::new(TributeStatus::Frozen);
        let s = serde_json::to_string(&d).unwrap();
        let back: AfflictionDraft = serde_json::from_str(&s).unwrap();
        assert_eq!(back, d);
    }
}

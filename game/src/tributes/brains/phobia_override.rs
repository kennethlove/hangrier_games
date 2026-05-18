//! Phobia override layer for the brain pipeline.
//!
//! Runs after stamina_override, before affliction_override. Provides:
//! 1. Hard override — Severe phobia with freeze roll → `Action::Frozen`
//! 2. Stat penalties — composed from all firing phobias (applied at scoring)
//!
//! Pipeline order: [..., survival, stamina, **phobia**, affliction, preferred, alliance, consumable]
//!
//! See spec §5 (phobia brain layer).

use rand::Rng;
use rand::SeedableRng;

use crate::areas::AreaDetails;
use crate::terrain::BaseTerrain;
use crate::tributes::Tribute;
use crate::tributes::actions::Action;
use crate::tributes::afflictions::phobia::{
    FiringPhobia, Reaction, collect_firing_phobias, strongest_reaction, total_stat_penalty,
};

/// Context available to the phobia override layer. Built from the brain
/// pipeline's inputs — a subset of full `PhobiaContext` since the brain
/// doesn't have cycle messages or the full nearby-tribute list.
#[derive(Clone, Debug)]
pub struct PhobiaBrainContext<'a> {
    /// The tribute's current area (for terrain/event-based triggers).
    pub area: Option<&'a AreaDetails>,
    /// The tribute's current terrain (fallback when area is unavailable).
    pub terrain: Option<BaseTerrain>,
    /// Whether the current phase is night.
    pub is_night: bool,
    /// Count of other living tributes in the same area.
    pub nearby_tributes: u32,
}

/// Phobia override layer entry point for the pre-decision pipeline.
///
/// Returns `Some(action)` to short-circuit the brain pipeline (only for
/// freeze reactions), or `None` to fall through. Stat penalties are
/// computed separately via `phobia_stat_penalty`.
///
/// Gated on `config.phobias_enabled` — caller must check before invoking.
pub fn phobia_override(
    tribute: &Tribute,
    ctx: &PhobiaBrainContext<'_>,
    rng: &mut impl Rng,
) -> Option<Action> {
    if tribute.afflictions.is_empty() {
        return None;
    }

    let firing = collect_firing_phobias(tribute, |trigger| is_trigger_firing(trigger, ctx), rng);
    if firing.is_empty() {
        return None;
    }

    let strongest = strongest_reaction(&firing)?;
    match strongest {
        Reaction::Freeze => Some(Action::Frozen),
        // Penalty and AutoFlee don't hard-override; they affect scoring.
        Reaction::Penalty | Reaction::AutoFlee => None,
    }
}

/// Compute total stat penalty from all firing phobias.
/// Returns the penalty to apply to attack/defense scores (negative value, capped at -10).
///
/// Gated on `config.phobias_enabled` — caller must check before invoking.
pub fn phobia_stat_penalty(tribute: &Tribute, ctx: &PhobiaBrainContext<'_>) -> i32 {
    if tribute.afflictions.is_empty() {
        return 0;
    }

    let mut rng = rand::rngs::SmallRng::from_rng(&mut rand::rng());
    let firing =
        collect_firing_phobias(tribute, |trigger| is_trigger_firing(trigger, ctx), &mut rng);
    if firing.is_empty() {
        return 0;
    }

    total_stat_penalty(&firing)
}

/// Returns the list of firing phobias for external consumers (e.g., message emitters).
pub fn firing_phobias(
    tribute: &Tribute,
    ctx: &PhobiaBrainContext<'_>,
    rng: &mut impl Rng,
) -> Vec<FiringPhobia> {
    if tribute.afflictions.is_empty() {
        return Vec::new();
    }
    collect_firing_phobias(tribute, |trigger| is_trigger_firing(trigger, ctx), rng)
}

/// Check whether a trigger is firing given the limited brain context.
///
/// This is a best-effort approximation — the brain pipeline doesn't have
/// cycle messages (Blood) or the full nearby-tribute list (TraitGroup).
/// Triggers that require unavailable data are conservatively false.
fn is_trigger_firing(
    trigger: &shared::afflictions::PhobiaTrigger,
    ctx: &PhobiaBrainContext<'_>,
) -> bool {
    use shared::afflictions::PhobiaTrigger;

    match trigger {
        PhobiaTrigger::Fire => is_fire_present(ctx),
        PhobiaTrigger::Water => is_water_present(ctx),
        PhobiaTrigger::Dark => ctx.is_night, // light source check unavailable
        PhobiaTrigger::Blood => false,       // cycle messages unavailable
        PhobiaTrigger::Heights => is_heights_present(ctx),
        PhobiaTrigger::Enclosed => is_enclosed_present(ctx),
        PhobiaTrigger::Open => is_open_present(ctx),
        PhobiaTrigger::Animal => false, // not modeled in v1
        PhobiaTrigger::Tribute => ctx.nearby_tributes > 0,
        PhobiaTrigger::TraitGroup => ctx.nearby_tributes > 0, // approximate
    }
}

fn terrain(ctx: &PhobiaBrainContext<'_>) -> Option<BaseTerrain> {
    ctx.area.map(|a| a.terrain.base).or(ctx.terrain)
}

fn is_fire_present(ctx: &PhobiaBrainContext<'_>) -> bool {
    ctx.area
        .map(|a| {
            a.events
                .iter()
                .any(|e| matches!(e, crate::areas::events::AreaEvent::Wildfire))
        })
        .unwrap_or(false)
}

fn is_water_present(ctx: &PhobiaBrainContext<'_>) -> bool {
    let area_has_flood = ctx
        .area
        .map(|a| {
            a.events
                .iter()
                .any(|e| matches!(e, crate::areas::events::AreaEvent::Flood))
        })
        .unwrap_or(false);
    let terrain_is_wetlands = matches!(terrain(ctx), Some(BaseTerrain::Wetlands));
    area_has_flood || terrain_is_wetlands
}

fn is_heights_present(ctx: &PhobiaBrainContext<'_>) -> bool {
    matches!(
        terrain(ctx),
        Some(BaseTerrain::Mountains | BaseTerrain::Highlands)
    )
}

fn is_enclosed_present(ctx: &PhobiaBrainContext<'_>) -> bool {
    matches!(terrain(ctx), Some(BaseTerrain::UrbanRuins))
}

fn is_open_present(ctx: &PhobiaBrainContext<'_>) -> bool {
    matches!(
        terrain(ctx),
        Some(BaseTerrain::Desert | BaseTerrain::Grasslands | BaseTerrain::Clearing)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::areas::Area;
    use crate::areas::events::AreaEvent;
    use crate::terrain::TerrainType;
    use crate::tributes::AfflictionDraft;
    use crate::tributes::traits::Trait;
    use shared::afflictions::{
        AfflictionKind, AfflictionSource, PhobiaMetadata, PhobiaOrigin, PhobiaTrigger, Severity,
    };

    fn make_area(terrain: BaseTerrain, events: Vec<AreaEvent>) -> AreaDetails {
        let mut area = AreaDetails::new(None, Area::Cornucopia);
        area.terrain = TerrainType::new(terrain, vec![]).unwrap();
        area.events = events;
        area
    }

    fn make_ctx(area: Option<&AreaDetails>, is_night: bool, nearby: u32) -> PhobiaBrainContext<'_> {
        PhobiaBrainContext {
            area,
            terrain: area.map(|a| a.terrain.base),
            is_night,
            nearby_tributes: nearby,
        }
    }

    fn make_tribute_with_phobia(trigger: PhobiaTrigger, severity: Severity) -> Tribute {
        let mut tribute = Tribute::new("Test".to_string(), None, None);
        let draft = AfflictionDraft {
            kind: AfflictionKind::Phobia(trigger),
            body_part: None,
            severity,
            source: AfflictionSource::Spawn,
        };
        tribute.try_acquire_affliction(draft);
        if let Some((_, aff)) = tribute.afflictions.iter_mut().next() {
            aff.phobia_metadata = Some(PhobiaMetadata {
                origin: PhobiaOrigin::Innate,
                ..PhobiaMetadata::default()
            });
        }
        tribute
    }

    #[test]
    fn phobia_override_freeze_returns_frozen() {
        let mut tribute = make_tribute_with_phobia(PhobiaTrigger::Heights, Severity::Severe);
        // Remove Reckless so freeze can trigger
        tribute.traits.retain(|t| *t != Trait::Reckless);

        let area = make_area(BaseTerrain::Mountains, vec![]);
        let ctx = make_ctx(Some(&area), false, 0);

        // Run multiple times to catch a freeze roll (25% chance each)
        let mut got_freeze = false;
        for seed in 0..100 {
            let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
            if phobia_override(&tribute, &ctx, &mut rng) == Some(Action::Frozen) {
                got_freeze = true;
                break;
            }
        }
        assert!(
            got_freeze,
            "Should get at least one freeze in 100 attempts (25% chance each)"
        );
    }

    #[test]
    fn phobia_override_no_firing_phobia_returns_none() {
        let tribute = make_tribute_with_phobia(PhobiaTrigger::Fire, Severity::Moderate);
        let area = make_area(BaseTerrain::Forest, vec![]); // no wildfire
        let ctx = make_ctx(Some(&area), false, 0);
        let mut rng = rand::rngs::SmallRng::seed_from_u64(0);
        assert!(phobia_override(&tribute, &ctx, &mut rng).is_none());
    }

    #[test]
    fn phobia_override_mild_no_override() {
        let tribute = make_tribute_with_phobia(PhobiaTrigger::Heights, Severity::Mild);
        let area = make_area(BaseTerrain::Mountains, vec![]);
        let ctx = make_ctx(Some(&area), false, 0);
        let mut rng = rand::rngs::SmallRng::seed_from_u64(0);
        // Mild → Penalty, no override
        assert!(phobia_override(&tribute, &ctx, &mut rng).is_none());
    }

    #[test]
    fn phobia_stat_penalty_single_moderate() {
        let tribute = make_tribute_with_phobia(PhobiaTrigger::Heights, Severity::Moderate);
        let area = make_area(BaseTerrain::Mountains, vec![]);
        let ctx = make_ctx(Some(&area), false, 0);
        assert_eq!(phobia_stat_penalty(&tribute, &ctx), -4);
    }

    #[test]
    fn phobia_stat_penalty_two_mild_stack() {
        let mut tribute = make_tribute_with_phobia(PhobiaTrigger::Heights, Severity::Mild);
        // Add second phobia (Tribute trigger — fires when nearby > 0)
        let draft = AfflictionDraft {
            kind: AfflictionKind::Phobia(PhobiaTrigger::Tribute),
            body_part: None,
            severity: Severity::Mild,
            source: AfflictionSource::Spawn,
        };
        tribute.try_acquire_affliction(draft);
        if let Some((_, aff)) = tribute
            .afflictions
            .iter_mut()
            .find(|(k, _)| matches!(k.0, AfflictionKind::Phobia(PhobiaTrigger::Tribute)))
        {
            aff.phobia_metadata = Some(PhobiaMetadata {
                origin: PhobiaOrigin::Innate,
                ..PhobiaMetadata::default()
            });
        }

        let area = make_area(BaseTerrain::Mountains, vec![]); // triggers Heights
        let ctx = make_ctx(Some(&area), false, 2); // nearby=2 triggers Tribute
        assert_eq!(phobia_stat_penalty(&tribute, &ctx), -4); // -2 + -2
    }

    #[test]
    fn phobia_override_reckless_severe_no_freeze() {
        let mut tribute = make_tribute_with_phobia(PhobiaTrigger::Heights, Severity::Severe);
        // Ensure Reckless is present
        if !tribute.traits.contains(&Trait::Reckless) {
            tribute.traits.push(Trait::Reckless);
        }

        let area = make_area(BaseTerrain::Mountains, vec![]);
        let ctx = make_ctx(Some(&area), false, 0);
        let mut rng = rand::rngs::SmallRng::seed_from_u64(0);
        // Reckless ignores freeze → AutoFlee → no override
        assert!(phobia_override(&tribute, &ctx, &mut rng).is_none());
    }

    #[test]
    fn phobia_override_resilient_severe_becomes_moderate_no_freeze() {
        let mut tribute = make_tribute_with_phobia(PhobiaTrigger::Heights, Severity::Severe);
        tribute.traits.clear();
        tribute.traits.push(Trait::Resilient);

        let area = make_area(BaseTerrain::Mountains, vec![]);
        let ctx = make_ctx(Some(&area), false, 0);
        let mut rng = rand::rngs::SmallRng::seed_from_u64(0);
        // Resilient downgrades Severe → Moderate → AutoFlee → no override
        assert!(phobia_override(&tribute, &ctx, &mut rng).is_none());
    }

    #[test]
    fn phobia_override_fragile_mild_becomes_moderate() {
        let mut tribute = make_tribute_with_phobia(PhobiaTrigger::Heights, Severity::Mild);
        tribute.traits.clear();
        tribute.traits.push(Trait::Fragile);

        let area = make_area(BaseTerrain::Mountains, vec![]);
        let ctx = make_ctx(Some(&area), false, 0);
        let mut rng = rand::rngs::SmallRng::seed_from_u64(0);
        // Fragile upgrades Mild → Moderate → AutoFlee → no override
        assert!(phobia_override(&tribute, &ctx, &mut rng).is_none());
        // But stat penalty should be -4 (Moderate)
        assert_eq!(phobia_stat_penalty(&tribute, &ctx), -4);
    }

    #[test]
    fn phobia_dark_trigger_at_night() {
        let tribute = make_tribute_with_phobia(PhobiaTrigger::Dark, Severity::Moderate);
        let area = make_area(BaseTerrain::Forest, vec![]);
        let ctx = make_ctx(Some(&area), true, 0); // is_night = true
        assert_eq!(phobia_stat_penalty(&tribute, &ctx), -4);
    }

    #[test]
    fn phobia_dark_not_firing_at_day() {
        let tribute = make_tribute_with_phobia(PhobiaTrigger::Dark, Severity::Moderate);
        let area = make_area(BaseTerrain::Forest, vec![]);
        let ctx = make_ctx(Some(&area), false, 0); // is_night = false
        assert_eq!(phobia_stat_penalty(&tribute, &ctx), 0);
    }

    #[test]
    fn phobia_tribute_trigger_with_nearby() {
        let tribute = make_tribute_with_phobia(PhobiaTrigger::Tribute, Severity::Mild);
        let area = make_area(BaseTerrain::Clearing, vec![]);
        let ctx = make_ctx(Some(&area), false, 3); // 3 nearby
        assert_eq!(phobia_stat_penalty(&tribute, &ctx), -2);
    }

    #[test]
    fn phobia_tribute_not_firing_alone() {
        let tribute = make_tribute_with_phobia(PhobiaTrigger::Tribute, Severity::Mild);
        let area = make_area(BaseTerrain::Clearing, vec![]);
        let ctx = make_ctx(Some(&area), false, 0); // alone
        assert_eq!(phobia_stat_penalty(&tribute, &ctx), 0);
    }
}

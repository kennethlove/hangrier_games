#![allow(clippy::collapsible_if)]
//! Fixation override layer for the brain pipeline.
//!
//! Runs after stamina_override, before phobia_override. Provides:
//! 1. Mild: tiebreaker bias toward fixation target
//! 2. Moderate: strong pursuit preference
//! 3. Severe: hard override to pursue target (unless survival overrides)
//!
//! Pipeline order: [..., survival, stamina, **fixation**, phobia, affliction, ...]
//!
//! See spec §6 (fixation brain layer).

use rand::Rng;
use shared::afflictions::{AfflictionKind, FixationOrigin, FixationTarget, Severity};

use crate::areas::Area;
use crate::tributes::Tribute;
use crate::tributes::actions::Action;
use crate::tributes::traits::Trait;

/// Context available to the fixation override layer.
#[derive(Clone, Debug)]
pub struct FixationBrainContext<'a> {
    /// All known areas in the arena.
    pub all_areas: &'a [crate::areas::AreaDetails],
    /// Whether the current phase is night.
    pub is_night: bool,
    /// Count of other living tributes in the same area.
    pub nearby_tributes: u32,
    /// Tribute's current area.
    pub current_area: Option<Area>,
}

/// Fixation override layer entry point for the pre-decision pipeline.
pub fn fixation_override(
    tribute: &Tribute,
    ctx: &FixationBrainContext<'_>,
    _rng: &mut impl Rng,
) -> Option<Action> {
    if tribute.afflictions.is_empty() {
        return None;
    }

    let firing = collect_firing_fixations(tribute, ctx);
    if firing.is_empty() {
        return None;
    }

    let mut sorted: Vec<_> = firing.iter().collect();
    sorted.sort_by_key(|f| std::cmp::Reverse(f.effective_severity));

    for fixation in sorted {
        if let Severity::Severe = fixation.effective_severity {
            if let Some(action) = severe_override(fixation, tribute) {
                return Some(action);
            }
        }
    }

    None
}

/// A firing fixation with its computed effective severity.
#[derive(Debug, Clone)]
pub struct FiringFixation {
    pub target: FixationTarget,
    pub base_severity: Severity,
    pub effective_severity: Severity,
    pub origin: FixationOrigin,
}

/// Collect all firing fixation afflictions for a tribute.
pub fn collect_firing_fixations(
    tribute: &Tribute,
    _ctx: &FixationBrainContext<'_>,
) -> Vec<FiringFixation> {
    let mut firing = Vec::new();

    for (key, aff) in &tribute.afflictions {
        let AfflictionKind::Fixation(target) = &key.0 else {
            continue;
        };
        let Some(meta) = &aff.fixation_metadata else {
            continue;
        };

        let base = aff.severity;
        let effective = effective_fixation_severity(base, &tribute.traits, target);
        firing.push(FiringFixation {
            target: target.clone(),
            base_severity: base,
            effective_severity: effective,
            origin: meta.origin.clone(),
        });
    }

    firing
}

/// Compute the effective severity after applying trait modifiers.
pub fn effective_fixation_severity(
    base: Severity,
    traits: &[Trait],
    target: &FixationTarget,
) -> Severity {
    let tier = base.ordinal() as i32;
    let mut modifier = trait_tier_modifier(traits);

    if matches!(target, FixationTarget::Tribute(_)) && traits.contains(&Trait::Loyal) {
        modifier += 1;
    }

    let adjusted = (tier + modifier).clamp(0, Severity::Severe.ordinal() as i32);
    match adjusted {
        0 => Severity::Mild,
        1 => Severity::Moderate,
        _ => Severity::Severe,
    }
}

fn trait_tier_modifier(traits: &[Trait]) -> i32 {
    let mut modifier = 0i32;
    for t in traits {
        match t {
            Trait::Resilient => modifier -= 1,
            Trait::Fragile => modifier += 1,
            _ => {}
        }
    }
    modifier
}

fn severe_override(fixation: &FiringFixation, tribute: &Tribute) -> Option<Action> {
    match &fixation.target {
        FixationTarget::Area(id) => {
            if let Some(area) = parse_area(id) {
                if area != tribute.area {
                    return Some(Action::Move(Some(area)));
                }
            }
            None
        }
        _ => None,
    }
}

fn parse_area(id: &str) -> Option<Area> {
    use std::str::FromStr;
    Area::from_str(id).ok()
}

/// Compute a destination score bonus for fixation bias.
pub fn fixation_destination_bonus(
    tribute: &Tribute,
    destination: Area,
    ctx: &FixationBrainContext<'_>,
) -> i32 {
    if tribute.afflictions.is_empty() {
        return 0;
    }

    let firing = collect_firing_fixations(tribute, ctx);
    let mut bonus = 0i32;

    for f in &firing {
        match f.effective_severity {
            Severity::Mild => {
                if let FixationTarget::Area(id) = &f.target {
                    if let Some(target_area) = parse_area(id) {
                        if target_area == destination {
                            bonus += 2;
                        }
                    }
                }
            }
            Severity::Moderate => {
                if let FixationTarget::Area(id) = &f.target {
                    if let Some(target_area) = parse_area(id) {
                        if target_area == destination {
                            bonus += 8;
                        }
                    }
                }
                if matches!(f.target, FixationTarget::Tribute(_)) {
                    bonus += 4;
                }
            }
            Severity::Severe => {}
        }
    }

    bonus
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::areas::AreaDetails;
    use crate::tributes::AfflictionDraft;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;
    use shared::afflictions::{AfflictionSource, FixationMetadata};

    #[allow(dead_code)]
    fn make_fixation_tribute(
        target: FixationTarget,
        severity: Severity,
        traits: Vec<Trait>,
    ) -> Tribute {
        let mut tribute = Tribute::new("Test".to_string(), None, None);
        tribute.traits = traits;
        let draft = AfflictionDraft {
            kind: AfflictionKind::Fixation(target),
            body_part: None,
            severity,
            source: AfflictionSource::Spawn,
        };
        tribute.try_acquire_affliction(draft);
        if let Some((_, aff)) = tribute.afflictions.iter_mut().next() {
            aff.fixation_metadata = Some(FixationMetadata::default());
        }
        tribute
    }

    #[allow(dead_code)]
    fn make_ctx(
        areas: Vec<AreaDetails>,
        current_area: Option<Area>,
    ) -> FixationBrainContext<'static> {
        FixationBrainContext {
            all_areas: areas.leak(),
            is_night: false,
            nearby_tributes: 0,
            current_area,
        }
    }

    #[test]
    fn effective_severity_no_traits_unchanged() {
        let target = FixationTarget::Tribute("cato".into());
        assert_eq!(
            effective_fixation_severity(Severity::Mild, &[], &target),
            Severity::Mild
        );
        assert_eq!(
            effective_fixation_severity(Severity::Moderate, &[], &target),
            Severity::Moderate
        );
        assert_eq!(
            effective_fixation_severity(Severity::Severe, &[], &target),
            Severity::Severe
        );
    }

    #[test]
    fn effective_severity_resilient_downgrades() {
        let target = FixationTarget::Tribute("cato".into());
        assert_eq!(
            effective_fixation_severity(Severity::Mild, &[Trait::Resilient], &target),
            Severity::Mild
        );
        assert_eq!(
            effective_fixation_severity(Severity::Moderate, &[Trait::Resilient], &target),
            Severity::Mild
        );
        assert_eq!(
            effective_fixation_severity(Severity::Severe, &[Trait::Resilient], &target),
            Severity::Moderate
        );
    }

    #[test]
    fn effective_severity_fragile_upgrades() {
        let target = FixationTarget::Tribute("cato".into());
        assert_eq!(
            effective_fixation_severity(Severity::Mild, &[Trait::Fragile], &target),
            Severity::Moderate
        );
        assert_eq!(
            effective_fixation_severity(Severity::Moderate, &[Trait::Fragile], &target),
            Severity::Severe
        );
        assert_eq!(
            effective_fixation_severity(Severity::Severe, &[Trait::Fragile], &target),
            Severity::Severe
        );
    }

    #[test]
    fn effective_severity_loyal_tribute_bonus() {
        let target = FixationTarget::Tribute("anyone".into());
        assert_eq!(
            effective_fixation_severity(Severity::Mild, &[Trait::Loyal], &target),
            Severity::Moderate
        );
    }

    #[test]
    fn effective_severity_loyal_non_tribute_no_bonus() {
        let target = FixationTarget::Item("axe".into());
        assert_eq!(
            effective_fixation_severity(Severity::Mild, &[Trait::Loyal], &target),
            Severity::Mild
        );
    }

    #[test]
    fn fixation_override_no_fixations_returns_none() {
        let tribute = Tribute::new("Test".to_string(), None, None);
        let ctx = make_ctx(vec![], Some(Area::Sector1));
        let mut rng = SmallRng::seed_from_u64(0);
        assert!(fixation_override(&tribute, &ctx, &mut rng).is_none());
    }

    #[test]
    fn fixation_override_mild_no_override() {
        let tribute = make_fixation_tribute(
            FixationTarget::Area("Sector2".into()),
            Severity::Mild,
            vec![],
        );
        let ctx = make_ctx(vec![], Some(Area::Sector1));
        let mut rng = SmallRng::seed_from_u64(0);
        assert!(fixation_override(&tribute, &ctx, &mut rng).is_none());
    }

    #[test]
    fn fixation_destination_bonus_mild() {
        let tribute = make_fixation_tribute(
            FixationTarget::Area("Sector2".into()),
            Severity::Mild,
            vec![],
        );
        let ctx = make_ctx(vec![], Some(Area::Sector1));
        assert_eq!(fixation_destination_bonus(&tribute, Area::Sector2, &ctx), 2);
        assert_eq!(fixation_destination_bonus(&tribute, Area::Sector1, &ctx), 0);
    }

    #[test]
    fn fixation_destination_bonus_moderate() {
        let tribute = make_fixation_tribute(
            FixationTarget::Area("Sector2".into()),
            Severity::Moderate,
            vec![],
        );
        let ctx = make_ctx(vec![], Some(Area::Sector1));
        assert_eq!(fixation_destination_bonus(&tribute, Area::Sector2, &ctx), 8);
    }
}

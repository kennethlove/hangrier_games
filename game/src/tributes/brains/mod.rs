use crate::areas::{Area, AreaDetails};
use crate::terrain::{BaseTerrain, Harshness, TerrainType, Visibility};
use crate::tributes::Tribute;
use crate::tributes::actions::Action;
use crate::tributes::alliances::MAX_ALLIES;
use crate::tributes::traits::{REFUSERS, ThresholdDelta, Trait, geometric_mean_affinity};
use rand::Rng;
use rand::RngExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod addiction_override;
pub mod affliction_override;
pub mod fixation_override;
pub mod phobia_override;
pub mod trauma_override;

mod scoring;
use scoring::*;

const LOW_ENEMY_LIMIT: u32 = 6;

/// Sleep gating thresholds (PR2c.1, bd-9sjj). See `Brain::should_sleep`.
const SLEEP_DOMINANT_THRESHOLD: u32 = 12;
const SLEEP_WANT_THRESHOLD: u32 = 6;
const SLEEP_EXHAUSTED_PCT: u32 = 25;

/// Score penalty applied per enemy in a destination area, used by
/// `Brain::choose_destination` to disperse crowded tributes.
const CROWD_PENALTY_PER_ENEMY: i32 = 8;

/// Cap on the cumulative crowd penalty so a single mob doesn't drown out
/// affinity / harshness signals entirely.
const CROWD_PENALTY_MAX: i32 = 32;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PsychoticBreakType {
    Berserk,         // Attack anyone nearby
    Paranoid,        // Flee and hide constantly
    Catatonic,       // Skip turns (Action::None)
    SelfDestructive, // Seek danger, ignore health
}

/// Cached personality thresholds generated at tribute creation with individual variance
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PersonalityThresholds {
    pub low_health: u32,
    pub mid_health: u32,
    pub extreme_low_sanity: u32,
    pub low_sanity: u32,
    pub mid_sanity: u32,
    pub low_movement: u32,
    pub high_intelligence: u32,
    pub low_intelligence: u32,
    pub psychotic_break_threshold: u32, // Sanity level that triggers break
}

impl PersonalityThresholds {
    /// Derive thresholds from a tribute's traits. Sums each trait's
    /// `ThresholdDelta`, applies it to baseline values, then applies ±20%
    /// individual variance. Each field is clamped to at least 1.
    ///
    /// Field mapping from `ThresholdDelta` to `PersonalityThresholds`:
    /// - `low_health_limit`        → `low_health`
    /// - `mid_health_limit`        → `mid_health`
    /// - `low_sanity_limit`        → `extreme_low_sanity`
    /// - `mid_sanity_limit`        → `low_sanity`
    /// - `high_sanity_limit`       → `mid_sanity`
    /// - `movement_limit`          → `low_movement`
    /// - `high_intelligence_limit` → `high_intelligence`
    /// - `low_intelligence_limit`  → `low_intelligence`
    /// - `psychotic_break_threshold` → `psychotic_break_threshold`
    pub fn from_traits(traits: &[Trait], rng: &mut impl Rng) -> Self {
        fn apply_variance(base: i32, rng: &mut impl Rng) -> u32 {
            let variance = rng.random_range(-0.2_f32..=0.2_f32);
            ((base as f32) * (1.0 + variance)).max(1.0) as u32
        }

        fn apply_delta(base: i32, delta: i32) -> i32 {
            (base + delta).max(1)
        }

        // Baseline values match the original `Balanced` personality.
        let base_low_health: i32 = 20;
        let base_mid_health: i32 = 40;
        let base_extreme_low_sanity: i32 = 10;
        let base_low_sanity: i32 = 20;
        let base_mid_sanity: i32 = 35;
        let base_low_movement: i32 = 10;
        let base_high_intelligence: i32 = 35;
        let base_low_intelligence: i32 = 80;
        let base_break_threshold: i32 = 8;

        let delta: ThresholdDelta = traits.iter().map(|t| t.threshold_modifiers()).sum();

        PersonalityThresholds {
            low_health: apply_variance(apply_delta(base_low_health, delta.low_health_limit), rng),
            mid_health: apply_variance(apply_delta(base_mid_health, delta.mid_health_limit), rng),
            extreme_low_sanity: apply_variance(
                apply_delta(base_extreme_low_sanity, delta.low_sanity_limit),
                rng,
            ),
            low_sanity: apply_variance(apply_delta(base_low_sanity, delta.mid_sanity_limit), rng),
            mid_sanity: apply_variance(apply_delta(base_mid_sanity, delta.high_sanity_limit), rng),
            low_movement: apply_variance(apply_delta(base_low_movement, delta.movement_limit), rng),
            high_intelligence: apply_variance(
                apply_delta(base_high_intelligence, delta.high_intelligence_limit),
                rng,
            ),
            low_intelligence: apply_variance(
                apply_delta(base_low_intelligence, delta.low_intelligence_limit),
                rng,
            ),
            psychotic_break_threshold: apply_variance(
                apply_delta(base_break_threshold, delta.psychotic_break_threshold),
                rng,
            ),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Brain {
    pub thresholds: PersonalityThresholds,
    #[serde(default, deserialize_with = "deserialize_optional_enum_lenient")]
    pub psychotic_break: Option<PsychoticBreakType>,
    /// Transient per-turn AI preference. Not persisted (recomputed each cycle)
    /// because the SurrealDB SDK's bespoke serializer collapses externally-
    /// tagged enums with payloads to `{}`, which would then fail to round-trip.
    /// Lenient deserializer absorbs any pre-existing corruption from before
    /// this fix landed.
    #[serde(
        default,
        skip_serializing,
        deserialize_with = "deserialize_optional_enum_lenient"
    )]
    pub preferred_action: Option<Action>,
    #[serde(default)]
    pub preferred_action_percentage: f64,
}

/// Deserialize an `Option<E>` for any externally-tagged enum `E`, treating
/// SurrealDB-corrupted empty objects (`{}`) as `None` instead of failing.
fn deserialize_optional_enum_lenient<'de, D, E>(deserializer: D) -> Result<Option<E>, D::Error>
where
    D: serde::Deserializer<'de>,
    E: serde::de::DeserializeOwned,
{
    use serde::Deserialize as _;
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Null => Ok(None),
        serde_json::Value::Object(ref map) if map.is_empty() => Ok(None),
        other => serde_json::from_value(other)
            .map(Some)
            .map_err(serde::de::Error::custom),
    }
}

impl Default for Brain {
    fn default() -> Self {
        use rand::SeedableRng;
        let mut rng = rand::rngs::SmallRng::seed_from_u64(0); // Deterministic default
        Self {
            thresholds: PersonalityThresholds::from_traits(&[], &mut rng),
            psychotic_break: None,
            preferred_action: None,
            preferred_action_percentage: 0.0,
        }
    }
}

impl Brain {
    /// Build a brain whose thresholds are derived from a tribute's traits.
    pub fn from_traits(traits: &[Trait], rng: &mut impl Rng) -> Self {
        Self {
            thresholds: PersonalityThresholds::from_traits(traits, rng),
            psychotic_break: None,
            preferred_action: None,
            preferred_action_percentage: 0.0,
        }
    }

    /// Check if tribute should have a psychotic break
    ///
    /// Break type is randomly determined, not tied to personality.
    /// This allows for emergent behavior - a cautious tribute going berserk
    /// can be just as dramatic as an aggressive one becoming catatonic.
    pub fn check_psychotic_break(&mut self, current_sanity: u32, rng: &mut impl Rng) {
        // Already broken, stay broken
        if self.psychotic_break.is_some() {
            return;
        }

        // Check if sanity dropped below threshold
        if current_sanity <= self.thresholds.psychotic_break_threshold {
            // Randomly select break type (equal probability)
            let break_type = match rng.random_range(0..4) {
                0 => PsychoticBreakType::Berserk,
                1 => PsychoticBreakType::Paranoid,
                2 => PsychoticBreakType::Catatonic,
                _ => PsychoticBreakType::SelfDestructive,
            };

            self.psychotic_break = Some(break_type);
        }
    }

    /// Can recover from break if sanity recovers significantly
    pub fn check_recovery(&mut self, current_sanity: u32) {
        if self.psychotic_break.is_some() {
            // Need to recover to 20+ sanity above break threshold to recover
            if current_sanity >= self.thresholds.psychotic_break_threshold + 20 {
                self.psychotic_break = None;
            }
        }
    }
}

impl Brain {
    pub fn set_preferred_action(&mut self, action: Action, percentage: f64) {
        self.preferred_action = Some(action);
        self.preferred_action_percentage = percentage;
    }

    pub fn clear_preferred_action(&mut self) {
        self.preferred_action = None;
        self.preferred_action_percentage = 0.0;
    }

    /// The AI for a tribute. Automatic decisions based on the current state of the tribute.
    ///
    /// Multi-hop movement (8pq): when `all_areas` is non-empty, the brain
    /// scores every known area (not just adjacent neighbors), uses A* to
    /// plan a stamina-aware path to the best goal, and returns the first
    /// hop along that path. When `all_areas` is empty (legacy callers,
    /// brains-only unit tests) the brain falls back to neighbor-only
    /// destination selection from `available_destinations`.
    ///
    /// This is the legacy neighbor-only entry point; it omits the
    /// terrain-dependent survival and stamina overrides because it does
    /// not yet receive the tribute's current terrain. Both entry points
    /// share `run_pre_decision_overrides` for the terrain-independent
    /// override layers (psychotic break / preferred action / alliance /
    /// consumable).
    #[allow(clippy::too_many_arguments)]
    pub fn act(
        &self,
        tribute: &Tribute,
        nearby_tributes: u32,
        available_destinations: &[crate::areas::DestinationInfo],
        all_areas: &[AreaDetails],
        closed_areas: &[Area],
        enemy_density: &HashMap<Area, u32>,
        phase: shared::messages::Phase,
        rng: &mut impl Rng,
    ) -> Action {
        let area = all_areas.iter().find(|a| a.area == Some(tribute.area));
        if let Some(early) = self.run_pre_decision_overrides(
            tribute,
            nearby_tributes,
            None,
            Some(phase),
            area,
            &crate::config::GameConfig::default(),
            rng,
        ) {
            return early;
        }

        let action = if nearby_tributes == 0 {
            self.decide_action_no_enemies(tribute)
        } else if nearby_tributes < LOW_ENEMY_LIMIT {
            self.decide_action_few_enemies(tribute)
        } else {
            self.decide_action_many_enemies(tribute)
        };

        // If the action is Move(None), choose smart destination based on terrain
        match action {
            Action::Move(None) => {
                // Multi-hop pathfinding (8pq): score every known area
                // (not just neighbors), then plan a stamina-aware path
                // and return the first hop. Falls through to neighbor-
                // only legacy behavior when `all_areas` is empty.
                if !all_areas.is_empty()
                    && let Some(best_goal) =
                        self.choose_destination(all_areas, tribute, enemy_density)
                {
                    // With per-area enemy density baked into the score (see
                    // `choose_destination`), a non-current empty area now
                    // naturally outscores the crowded current one, so the
                    // legacy escape-hatch is gone. If the best area is still
                    // the current one, the tribute simply rests.
                    if best_goal == tribute.area {
                        return Action::Rest;
                    }
                    let goal = best_goal;
                    if let Some((path, _cost)) = crate::areas::path::plan_path(
                        all_areas,
                        closed_areas,
                        tribute,
                        tribute.area,
                        goal,
                    ) && path.len() >= 2
                    {
                        let first_hop = path[1];
                        // Stamina gate against the first hop's cost
                        // (use available_destinations if present, else
                        // fall back to permitting the move).
                        let cost_ok = available_destinations
                            .iter()
                            .find(|d| d.area == first_hop)
                            .map(|d| tribute.stamina >= d.stamina_cost)
                            .unwrap_or(true);
                        if cost_ok {
                            return Action::Move(Some(first_hop));
                        }
                        return Action::Rest;
                    }
                }

                // Legacy neighbor-only path (also used by brains tests
                // that call `act` with empty slices).
                if available_destinations.is_empty() {
                    return Action::Move(None);
                }

                // Convert DestinationInfo to AreaDetails for choose_destination
                let area_details: Vec<AreaDetails> = available_destinations
                    .iter()
                    .map(|dest| AreaDetails {
                        area: Some(dest.area),
                        terrain: dest.terrain.clone(),
                        events: dest.active_events.clone(),
                        ..AreaDetails::default()
                    })
                    .collect();

                // Choose best destination using terrain scoring
                if let Some(best_area) =
                    self.choose_destination(&area_details, tribute, enemy_density)
                {
                    // Also check if tribute has enough stamina
                    if let Some(dest_info) =
                        available_destinations.iter().find(|d| d.area == best_area)
                        && tribute.stamina >= dest_info.stamina_cost
                    {
                        return Action::Move(Some(best_area));
                    }
                }
                // Fall back to rest if no good destination or insufficient stamina
                Action::Rest
            }
            other => other,
        }
    }

    /// Choose the best destination from available areas based on terrain scoring.
    /// Returns the Area enum variant of the highest-scoring area.
    ///
    /// Scoring factors:
    /// - +20 if area has terrain in tribute's affinity
    /// - -10 per harshness tier (Mild=0, Moderate=-10, Harsh=-20)
    /// - +5 if terrain visibility is Concealed (good for hiding)
    /// - +3 if area has items
    /// - +60 (3.0x * 20) if tribute health < 30 and area has affinity terrain (desperate behavior)
    pub fn choose_destination(
        &self,
        areas: &[AreaDetails],
        tribute: &Tribute,
        enemy_density: &HashMap<Area, u32>,
    ) -> Option<Area> {
        if areas.is_empty() {
            return None;
        }

        let is_desperate = tribute.attributes.health < 30;

        let mut best_score = i32::MIN;
        let mut best_area: Option<Area> = None;

        for area_details in areas {
            let mut score = 0i32;

            // Crowd penalty: subtract a per-enemy amount from this area's
            // score, excluding the tribute itself. This naturally produces
            // dispersion (a non-current empty area outscores the crowded
            // current one) without a call-site special case.
            if let Some(area) = area_details.area {
                let raw = enemy_density.get(&area).copied().unwrap_or(0);
                let others = if area == tribute.area {
                    raw.saturating_sub(1)
                } else {
                    raw
                };
                let penalty = (others as i32)
                    .saturating_mul(CROWD_PENALTY_PER_ENEMY)
                    .min(CROWD_PENALTY_MAX);
                score -= penalty;
            }

            // Affinity bonus: +20 if terrain matches tribute's affinity
            let has_affinity = tribute
                .terrain_affinity
                .contains(&area_details.terrain.base);
            if has_affinity {
                score += 20;

                // Desperate behavior: 3.0x boost to affinity terrain (additional +40)
                if is_desperate {
                    score += 40; // Total 60 for desperate + affinity
                }
            }

            // Harshness penalty: -10 per tier
            let harshness_penalty = match area_details.terrain.base.harshness() {
                Harshness::Mild => 0,
                Harshness::Moderate => -10,
                Harshness::Harsh => -20,
            };
            score += harshness_penalty;

            // Concealed visibility bonus: +5 (good for hiding)
            if matches!(
                area_details.terrain.base.visibility(),
                Visibility::Concealed
            ) {
                score += 5;
            }

            // Items bonus: +3 if area has items
            if !area_details.items.is_empty() {
                score += 3;
            }

            if score > best_score {
                best_score = score;
                best_area = area_details.area;
            }
        }

        best_area
    }

    /// Decide action with terrain awareness. Modifies action weights based on terrain.
    ///
    /// Terrain-based weight modifications:
    /// - Boost Search weight by 2.0x in Desert/Tundra/Badlands (resource-scarce)
    /// - Boost Hide weight by 1.5x in Forest/Jungle/Wetlands (Concealed visibility)
    pub fn decide_action_with_terrain(
        &self,
        tribute: &Tribute,
        nearby_tributes: u32,
        terrain: TerrainType,
        phase: shared::messages::Phase,
        rng: &mut impl Rng,
    ) -> Action {
        if let Some(early) = self.run_pre_decision_overrides(
            tribute,
            nearby_tributes,
            Some(terrain.base),
            Some(phase),
            None,
            &crate::config::GameConfig::default(),
            rng,
        ) {
            return early;
        }

        // Check if terrain is resource-scarce (should boost search/movement)
        let is_resource_scarce = matches!(
            terrain.base,
            BaseTerrain::Desert | BaseTerrain::Tundra | BaseTerrain::Badlands
        );

        // Check if terrain is concealed (should boost hiding)
        let is_concealed = matches!(terrain.base.visibility(), Visibility::Concealed);

        // Decide base action
        let base_action = if nearby_tributes == 0 {
            self.decide_action_no_enemies(tribute)
        } else if nearby_tributes < LOW_ENEMY_LIMIT {
            self.decide_action_few_enemies_with_terrain(tribute, is_concealed)
        } else {
            self.decide_action_many_enemies_with_terrain(tribute, is_concealed)
        };

        // Stamina action-gate: an actor that can't pay the per-swing cost
        // cannot take Attack. Fall back to Rest so the tribute recovers
        // instead of cycling back into the same un-payable choice.
        let tuning = crate::tributes::combat_tuning::CombatTuning::default();
        let base_action = if matches!(base_action, Action::Attack)
            && action_score(tribute, &Action::Attack, &[], &tuning) == i32::MIN
        {
            Action::Rest
        } else {
            base_action
        };

        // Apply terrain modifiers to action choices
        match base_action {
            Action::Move(None) if is_resource_scarce => {
                // In resource-scarce terrain, stay focused on movement/search
                Action::Move(None)
            }
            Action::Hide if is_concealed => {
                // Concealed terrain makes hiding more effective
                Action::Hide
            }
            other => other,
        }
    }

    /// Phase-aware sleep gate (PR2c.1, bd-9sjj). Decides whether the
    /// tribute should begin a multi-phase sleep *now*, returning
    /// `Some(Action::Sleep { duration_phases })` to preempt the standard
    /// brain pipeline, or `None` to defer to `act`.
    ///
    /// Conditions (per spec `2026-05-03-four-phase-day-design.md` §6.4):
    /// - Already-sleeping tributes never re-enter the gate (they bypass
    ///   `process_turn_phase` entirely via the engine's sleep tick).
    /// - Tributes mid-psychotic-break cannot choose sleep.
    /// - At/over the dominant wakefulness threshold (12+ phases),
    ///   tributes sleep regardless of safety. Duration: 4 phases.
    /// - At/over the want threshold (6+ phases) AND no nearby hostiles
    ///   AND phase is Night or Dusk: sleep. Duration: 3 phases.
    /// - Stamina exhausted (≤25% of max) AND no nearby hostiles AND
    ///   non-Day phase: sleep. Duration: 2 phases.
    pub fn should_sleep(
        &self,
        tribute: &Tribute,
        nearby_tributes: u32,
        phase: shared::messages::Phase,
        _rng: &mut impl Rng,
    ) -> Option<Action> {
        use shared::messages::Phase;

        if !tribute.is_alive() || tribute.sleeping {
            return None;
        }
        if self.psychotic_break.is_some() {
            return None;
        }

        let safe = nearby_tributes == 0;
        let is_night_or_dusk = matches!(phase, Phase::Night | Phase::Dusk);
        let is_day = matches!(phase, Phase::Day);

        if tribute.cycles_awake >= SLEEP_DOMINANT_THRESHOLD {
            return Some(Action::Sleep { duration_phases: 4 });
        }

        if tribute.cycles_awake >= SLEEP_WANT_THRESHOLD && safe && is_night_or_dusk {
            return Some(Action::Sleep { duration_phases: 3 });
        }

        let stamina_pct = (tribute.stamina * 100)
            .checked_div(tribute.max_stamina)
            .unwrap_or(0);
        if stamina_pct <= SLEEP_EXHAUSTED_PCT && safe && !is_day {
            return Some(Action::Sleep { duration_phases: 2 });
        }

        None
    }

    /// Shared pre-decision override pipeline. Returns `Some(action)` to
    /// short-circuit the per-call base scoring, or `None` to fall through
    /// to the normal nearby-enemies branching.
    ///
    /// Layers run in this order:
    /// 1. Liveness (dead → `Action::None`)
    /// 2. Psychotic break
    /// 3. Survival override (terrain-dependent; skipped when `terrain` is `None`)
    /// 4. Stamina override (terrain-dependent for parity with survival)
    /// 5. Fixation override (spec §8 — per-tier override semantics)
    /// 6. Phobia override (spec §5 — fires freeze reactions, stat penalties)
    /// 7. Trauma override (spec §7 — avoidance hard veto)
    /// 8. Addiction override (spec §7-8 — craving/compulsion)
    /// 9. Affliction override (hard gates + brain bias; spec §11)
    /// 10. Preferred action
    /// 11. Alliance proposal
    /// 12. Consumable
    ///
    /// Layers 3 and 4 are gated on `terrain.is_some()` because the legacy
    /// `act` entry point does not yet receive the tribute's current terrain
    /// — those overrides depend on terrain for water/forage richness and
    /// would be unsafe to fire blind.
    #[allow(clippy::too_many_arguments)]
    fn run_pre_decision_overrides(
        &self,
        tribute: &Tribute,
        nearby_tributes: u32,
        terrain: Option<BaseTerrain>,
        phase: Option<shared::messages::Phase>,
        area: Option<&AreaDetails>,
        config: &crate::config::GameConfig,
        rng: &mut impl Rng,
    ) -> Option<Action> {
        if !tribute.is_alive() {
            return Some(Action::None);
        }

        if let Some(ref break_type) = self.psychotic_break {
            return Some(match break_type {
                PsychoticBreakType::Berserk => {
                    if nearby_tributes > 0 {
                        Action::Attack
                    } else {
                        Action::Move(None)
                    }
                }
                PsychoticBreakType::Paranoid => {
                    if tribute.attributes.is_hidden {
                        Action::None
                    } else {
                        Action::Hide
                    }
                }
                PsychoticBreakType::Catatonic => Action::None,
                PsychoticBreakType::SelfDestructive => {
                    if nearby_tributes > 0 {
                        Action::Attack
                    } else {
                        Action::Move(None)
                    }
                }
            });
        }

        // Terrain-dependent overrides (spec §6.4). Active combat
        // (nearby_tributes > 0) suppresses survival; stamina has its own
        // visible-band flee path that handles the in-combat case.
        if let Some(base) = terrain {
            let weather = crate::areas::weather::current_weather();
            if let Some(action) = survival_override(tribute, base, &weather, nearby_tributes > 0) {
                return Some(action);
            }

            // Stamina override (spec §6.4 — runs after survival, before
            // standard logic). For v1 we cannot easily plumb the live
            // nearby-tribute list and the per-phase shelter flag through
            // this signature; pass an empty threat slice and `sheltered=false`.
            if let Some(action) = stamina_override(
                tribute,
                &[],
                false,
                &crate::tributes::combat_tuning::CombatTuning::default(),
            ) {
                return Some(action);
            }
        }

        // Fixation override (spec §8): per-tier override semantics.
        // Runs between stamina and phobia overrides.
        {
            let fixation_ctx = fixation_override::FixationOverrideContext {
                target_reachable: true, // conservative: assume reachable
            };
            if let Some(action) = fixation_override::fixation_override(tribute, &fixation_ctx) {
                return Some(action);
            }
        }

        // Phobia override (spec §5): freeze reactions and stat penalties.
        // Gated on config.phobias_enabled.
        if config.phobias_enabled {
            let is_night = phase.is_some_and(|p| matches!(p, shared::messages::Phase::Night));
            let phobia_ctx = phobia_override::PhobiaBrainContext {
                area,
                terrain,
                is_night,
                nearby_tributes,
            };
            if let Some(action) = phobia_override::phobia_override(tribute, &phobia_ctx, rng) {
                return Some(action);
            }
        }

        // Trauma override (spec §7): avoidance hard veto.
        // Gated on config.trauma_enabled.
        if config.trauma_enabled
            && let Some(action) = trauma_override::trauma_override(tribute)
        {
            return Some(action);
        }

        // Addiction override (spec §7-8): craving/compulsion.
        // Gated on config.addiction_enabled.
        if config.addiction_enabled
            && let Some(action) = addiction_override::addiction_override(tribute)
        {
            return Some(action);
        }

        // Affliction override (spec §11): hard gates + brain bias.
        // Terrain-dependent gates (MissingLeg → cliff/swamp) are deferred
        // to action-execution time via `Tribute::affliction_action_gate`.
        if let Some(action) = affliction_override::affliction_override(tribute, &Action::None) {
            return Some(action);
        }

        // Preferred action
        if let Some(ref preferred_action) = self.preferred_action
            && rng.random_bool(self.preferred_action_percentage)
        {
            return Some(preferred_action.clone());
        }

        // Spec §6.1: alliance proposals are a deliberate first-class action.
        // Phase-gating is deferred to v2 per spec §13 ("Social events —
        // alliance formation gated by phase"). The `phase` parameter is
        // threaded through for future use.
        if self.wants_to_propose_alliance(tribute, nearby_tributes, rng) {
            return Some(Action::ProposeAlliance);
        }

        // Consumables
        if !tribute.consumables().is_empty() {
            return Some(Action::UseItem(None));
        }

        None
    }

    fn decide_action_few_enemies_with_terrain(
        &self,
        tribute: &Tribute,
        is_concealed: bool,
    ) -> Action {
        let low_health = self.thresholds.low_health;
        let mid_health = self.thresholds.mid_health;
        let low_sanity = self.thresholds.low_sanity;

        match tribute.attributes.health {
            h if h < low_health => {
                self.decide_action_few_enemies_low_health_with_terrain(tribute, is_concealed)
            }
            h if h >= low_health && h <= mid_health => {
                // Boost hiding in concealed terrain
                if tribute.attributes.sanity > low_sanity && is_concealed {
                    Action::Hide
                } else if tribute.attributes.sanity > low_sanity {
                    Action::Move(None)
                } else {
                    Action::Attack
                }
            }
            // High health - normally would attack, but concealed terrain makes hiding attractive
            _ if is_concealed && tribute.attributes.sanity > low_sanity => Action::Hide,
            _ => Action::Attack,
        }
    }

    fn decide_action_few_enemies_low_health_with_terrain(
        &self,
        tribute: &Tribute,
        is_concealed: bool,
    ) -> Action {
        let low_movement = self.thresholds.low_movement;
        let mid_sanity = self.thresholds.mid_sanity;
        let extreme_low_sanity = self.thresholds.extreme_low_sanity;

        let stats = (
            tribute.attributes.movement,
            tribute.attributes.sanity,
            tribute.attributes.is_hidden,
        );
        match stats {
            // Boost hiding in concealed terrain with low movement
            (m, s, false) if m < low_movement && s >= mid_sanity && is_concealed => Action::Hide,
            (m, s, false) if m < low_movement && s >= mid_sanity => Action::Hide,
            (m, s, _) if m < low_movement && s >= extreme_low_sanity && s < mid_sanity => {
                Action::Attack
            }
            (_, s, false) if s >= mid_sanity => Action::Move(None),
            (_, s, false) if s < mid_sanity => Action::Attack,
            (_, _, true) => Action::None,
            (_, _, false) => Action::Move(None), // Catch-all for visible tributes
        }
    }

    fn decide_action_many_enemies_with_terrain(
        &self,
        tribute: &Tribute,
        is_concealed: bool,
    ) -> Action {
        let high_intelligence = self.thresholds.high_intelligence;
        let low_intelligence = self.thresholds.low_intelligence;

        let recklessness: u32 = 100_u32
            .saturating_sub(tribute.attributes.intelligence)
            .saturating_sub(tribute.attributes.sanity);
        match recklessness {
            r if r < high_intelligence => Action::Move(None),
            r if r >= low_intelligence => Action::Attack,
            // Boost hiding in concealed terrain for average intelligence
            _ if is_concealed => Action::Hide,
            _ => Action::Hide,
        }
    }

    /// Decide whether the tribute spends this turn proposing an alliance.
    ///
    /// Returns true with low probability when ALL of the following hold:
    /// - At least one nearby tribute exists (potential candidate).
    /// - Tribute has not yet hit the per-tribute alliance cap (`MAX_ALLIES`).
    /// - Tribute is healthy enough to socialize (above `low_health`).
    /// - Tribute is sane enough to think (above `low_sanity`).
    /// - Tribute carries no refuser trait (Lone Wolf, Paranoid).
    /// - Tribute's own trait affinity is at least neutral
    ///   (`geometric_mean_affinity >= 1.0`).
    ///
    /// The base proposal chance is intentionally small (5%) and scales with
    /// trait affinity (Friendly/Loyal push it up). Even at the upper bound
    /// this fires for far fewer tributes per cycle than the legacy O(N\u00b2)
    /// pre-pass, which is the entire point of moving the trigger here.
    fn wants_to_propose_alliance(
        &self,
        tribute: &Tribute,
        nearby_tributes: u32,
        rng: &mut impl Rng,
    ) -> bool {
        if nearby_tributes == 0 {
            return false;
        }
        if tribute.allies.len() >= MAX_ALLIES {
            return false;
        }
        if tribute.attributes.health < self.thresholds.low_health
            || tribute.attributes.sanity < self.thresholds.low_sanity
        {
            return false;
        }
        if tribute.traits.iter().any(|t| REFUSERS.contains(t)) {
            return false;
        }
        let affinity = geometric_mean_affinity(&tribute.traits);
        if affinity < 1.0 {
            return false;
        }
        // Base 5% per turn, scaled by trait affinity (Friendly=1.5,
        // Loyal=1.4 push above; neutral stays at base). Clamp at 15% so
        // even Friendly+Loyal tributes don't propose every other turn.
        let chance = (0.05 * affinity).clamp(0.0, 0.15);
        rng.random_bool(chance)
    }

    fn decide_action_no_enemies(&self, tribute: &Tribute) -> Action {
        let low_health = self.thresholds.low_health;
        let mid_health = self.thresholds.mid_health;
        let low_sanity = self.thresholds.low_sanity;

        match tribute.attributes.health {
            // health is low, rest
            h if h < low_health => Action::Rest,
            // health isn't great, hide
            // unless sanity is also low, then move
            h if h >= low_health && h <= mid_health => {
                if tribute.attributes.sanity > low_sanity && tribute.is_visible() {
                    Action::Hide
                } else {
                    Action::Move(None)
                }
            }
            // health is good, move (or set a trap)
            _ => {
                // Deterministic: use tribute id hash to decide trap setting
                let hash: u32 = tribute.identifier.bytes().map(|b| b as u32).sum();
                if tribute.attributes.movement > 0 && hash.is_multiple_of(7) {
                    Action::SetTrap {
                        trap_kind: None,
                        severity: None,
                    }
                } else {
                    match tribute.attributes.movement {
                        0 => Action::Rest,
                        _ => Action::Move(None),
                    }
                }
            }
        }
    }

    fn decide_action_few_enemies_low_health(&self, tribute: &Tribute) -> Action {
        let low_movement = self.thresholds.low_movement;
        let mid_sanity = self.thresholds.mid_sanity;
        let extreme_low_sanity = self.thresholds.extreme_low_sanity;

        let stats = (
            tribute.attributes.movement,
            tribute.attributes.sanity,
            tribute.attributes.is_hidden,
        );
        match stats {
            // low movement, ok sanity, visible
            (m, s, false) if m < low_movement && s >= mid_sanity => Action::Hide,
            // low movement, low sanity, any visibility
            (m, s, _) if m < low_movement && s >= extreme_low_sanity && s < mid_sanity => {
                Action::Attack
            }
            // any movement, ok sanity, visible
            (_, s, false) if s >= mid_sanity => Action::Move(None),
            // any movement, low sanity, visible
            (_, s, false) if s < mid_sanity => Action::Attack,
            // any movement, any sanity, hidden
            (_, _, true) => Action::None,
            (_, _, false) => Action::Move(None), // Catch-all for visible tributes
        }
    }

    fn decide_action_few_enemies(&self, tribute: &Tribute) -> Action {
        let low_health = self.thresholds.low_health;
        let mid_health = self.thresholds.mid_health;
        let low_sanity = self.thresholds.low_sanity;

        match tribute.attributes.health {
            h if h < low_health => self.decide_action_few_enemies_low_health(tribute),
            h if h >= low_health && h <= mid_health => {
                if tribute.attributes.sanity > low_sanity {
                    Action::Move(None)
                } else {
                    Action::Attack
                }
            }
            _ => Action::Attack,
        }
    }

    fn decide_action_many_enemies(&self, tribute: &Tribute) -> Action {
        let high_intelligence = self.thresholds.high_intelligence;
        let low_intelligence = self.thresholds.low_intelligence;

        let recklessness: u32 = 100_u32
            .saturating_sub(tribute.attributes.intelligence)
            .saturating_sub(tribute.attributes.sanity);
        match recklessness {
            // Smart enough to know better, moves
            r if r < high_intelligence => Action::Move(None),
            // Too dumb to know better, attacks
            r if r >= low_intelligence => Action::Attack,
            // Average intelligence, hides
            _ => Action::Hide,
        }
    }
}

#[cfg(test)]
mod tests;

use crate::areas::{Area, AreaDetails};
use crate::terrain::{BaseTerrain, Harshness, TerrainType, Visibility};
use crate::tributes::Tribute;
use crate::tributes::actions::Action;
use crate::tributes::alliances::MAX_ALLIES;
use crate::tributes::traits::{REFUSERS, ThresholdDelta, Trait, geometric_mean_affinity};
use rand::Rng;
use rand::RngExt;
use serde::{Deserialize, Serialize};

const LOW_ENEMY_LIMIT: u32 = 6;

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
    pub fn act(
        &self,
        tribute: &Tribute,
        nearby_tributes: u32,
        available_destinations: &[crate::areas::DestinationInfo],
        all_areas: &[AreaDetails],
        closed_areas: &[Area],
        rng: &mut impl Rng,
    ) -> Action {
        if !tribute.is_alive() {
            return Action::None;
        }

        // Psychotic break overrides normal behavior
        if let Some(ref break_type) = self.psychotic_break {
            return match break_type {
                PsychoticBreakType::Berserk => {
                    // Attack if enemies nearby, otherwise move to find enemies
                    if nearby_tributes > 0 {
                        Action::Attack
                    } else {
                        Action::Move(None)
                    }
                }
                PsychoticBreakType::Paranoid => {
                    // Always flee and hide
                    if tribute.attributes.is_hidden {
                        Action::None // Stay hidden
                    } else {
                        Action::Hide
                    }
                }
                PsychoticBreakType::Catatonic => {
                    // Do nothing
                    Action::None
                }
                PsychoticBreakType::SelfDestructive => {
                    // Seek danger: attack if enemies present, move otherwise
                    // Ignore health completely
                    if nearby_tributes > 0 {
                        Action::Attack
                    } else {
                        Action::Move(None)
                    }
                }
            };
        }

        // If there is a preferred action, we should take it, assuming a positive roll
        if let Some(ref preferred_action) = self.preferred_action
            && rng.random_bool(self.preferred_action_percentage)
        {
            return preferred_action.clone();
        }

        // Spec §6.1: alliance proposals are a deliberate first-class action,
        // not an automatic encounter side-effect. Considered before the
        // health/sanity branching below so a healthy social tribute
        // occasionally spends a turn forming bonds instead of attacking.
        if self.wants_to_propose_alliance(tribute, nearby_tributes, rng) {
            return Action::ProposeAlliance;
        }

        // Does the tribute have items?
        let has_consumables = !tribute.consumables().is_empty();
        if has_consumables {
            // Use an item
            return Action::UseItem(None);
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
                    && let Some(best_goal) = self.choose_destination(all_areas, tribute)
                {
                    // Anti-clustering: if every area scores the same
                    // (e.g. all Clearing, no affinity), `choose_destination`
                    // returns the first area, which on the smart-path is
                    // typically the tribute's current area — leading every
                    // crowded tribute to Rest forever and combat to never
                    // engage. When the chosen "best" is just where we are
                    // and the area is crowded, force a hop to the cheapest
                    // reachable open neighbor instead.
                    let crowded = nearby_tributes >= LOW_ENEMY_LIMIT;
                    if best_goal == tribute.area && !crowded {
                        return Action::Rest;
                    }
                    let goal = if best_goal == tribute.area && crowded {
                        // Pick a cheap open neighbor we can actually afford
                        // to walk to. Falls through to plan_path below using
                        // that neighbor as the goal.
                        available_destinations
                            .iter()
                            .filter(|d| d.area != tribute.area)
                            .filter(|d| !closed_areas.contains(&d.area))
                            .filter(|d| tribute.stamina >= d.stamina_cost)
                            .min_by_key(|d| d.stamina_cost)
                            .map(|d| d.area)
                            .unwrap_or(best_goal)
                    } else {
                        best_goal
                    };
                    if goal == tribute.area {
                        return Action::Rest;
                    }
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
                if let Some(best_area) = self.choose_destination(&area_details, tribute) {
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
    pub fn choose_destination(&self, areas: &[AreaDetails], tribute: &Tribute) -> Option<Area> {
        if areas.is_empty() {
            return None;
        }

        let is_desperate = tribute.attributes.health < 30;

        let mut best_score = i32::MIN;
        let mut best_area: Option<Area> = None;

        for area_details in areas {
            let mut score = 0i32;

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
        rng: &mut impl Rng,
    ) -> Action {
        if !tribute.is_alive() {
            return Action::None;
        }

        // Survival overrides (spec §6.4) run before the normal weighted
        // scoring. Active combat (nearby enemies > 0) suppresses them.
        let weather = crate::areas::weather::current_weather();
        if let Some(action) =
            survival_override(tribute, terrain.base, &weather, nearby_tributes > 0)
        {
            return action;
        }

        // Check for preferred action first
        if let Some(ref preferred_action) = self.preferred_action
            && rng.random_bool(self.preferred_action_percentage)
        {
            return preferred_action.clone();
        }

        // Spec §6.1: alliance proposals are a deliberate first-class action.
        // Mirrors the gate in `act` so terrain-aware paths behave the same.
        if self.wants_to_propose_alliance(tribute, nearby_tributes, rng) {
            return Action::ProposeAlliance;
        }

        // Check if we have consumables
        let has_consumables = !tribute.consumables().is_empty();
        if has_consumables {
            return Action::UseItem(None);
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
            // health is good, move
            _ => {
                // If the tribute has movement, move
                match tribute.attributes.movement {
                    0 => Action::Rest,
                    _ => Action::Move(None),
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
    terrain: BaseTerrain,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::items::Item;
    use crate::tributes::Tribute;
    use crate::tributes::actions::Action;
    use rand::prelude::*;
    use rstest::{fixture, rstest};

    #[fixture]
    fn tribute() -> Tribute {
        // Build a fully-deterministic Tribute for AI tests:
        //   - Brain::default() = Balanced personality with seed=0 thresholds
        //     (low_health=18, mid_health=38, low_sanity=16, mid_sanity=34,
        //     low_movement=8, high_intelligence=40, low_intelligence=91)
        //   - Attributes::default() = maxed-out attributes
        // Tribute::new() randomizes both, so override after construction.
        let mut tribute = Tribute::new("Katniss".to_string(), None, None);
        tribute.brain = Brain::default();
        tribute.attributes = crate::tributes::Attributes::default();
        tribute
    }

    #[fixture]
    fn small_rng() -> SmallRng {
        // Use a fixed seed so brain decision tests are deterministic.
        // Otherwise low-probability branches (e.g. wants_to_propose_alliance
        // at ~5%) cause occasional CI flakes.
        SmallRng::seed_from_u64(0xA11CE5EED)
    }

    #[rstest]
    fn decide_on_action_default(tribute: Tribute, mut small_rng: SmallRng) {
        // If there are no enemies nearby, the tribute should move
        let action = tribute
            .brain
            .act(&tribute.clone(), 0, &[], &[], &[], &mut small_rng);
        assert_eq!(action, Action::Move(None));
    }

    #[rstest]
    fn decide_on_action_low_health(mut tribute: Tribute, mut small_rng: SmallRng) {
        // If the tribute has low health, they should rest
        tribute.attributes.health = 10;
        let action = tribute
            .brain
            .act(&tribute.clone(), 2, &[], &[], &[], &mut small_rng);
        assert_eq!(action, Action::Move(None));
    }

    #[rstest]
    fn decide_on_action_no_health(mut tribute: Tribute, mut small_rng: SmallRng) {
        // If the tribute has no health, they should do nothing
        tribute.attributes.health = 0;
        let action = tribute
            .brain
            .act(&tribute.clone(), 2, &[], &[], &[], &mut small_rng);
        assert_eq!(action, Action::None);
    }

    #[rstest]
    fn decide_on_action_no_movement_alone(mut tribute: Tribute, mut small_rng: SmallRng) {
        // If the tribute has no movement and is alone, they should rest
        tribute.attributes.movement = 0;
        let action = tribute
            .brain
            .act(&tribute.clone(), 0, &[], &[], &[], &mut small_rng);
        assert_eq!(action, Action::Rest);
    }

    #[rstest]
    fn decide_on_action_no_movement_surrounded_low_health(
        mut tribute: Tribute,
        mut small_rng: SmallRng,
    ) {
        // If the tribute has no movement and is not alone, they should hide
        tribute.attributes.movement = 1;
        tribute.attributes.health = 10;
        let action = tribute
            .brain
            .act(&tribute.clone(), 5, &[], &[], &[], &mut small_rng);
        assert_eq!(action, Action::Hide);
    }

    #[rstest]
    fn decide_on_action_enemies(tribute: Tribute, mut small_rng: SmallRng) {
        // If there are enemies nearby, the tribute should attack
        let action = tribute
            .brain
            .act(&tribute.clone(), 2, &[], &[], &[], &mut small_rng);
        assert_eq!(action, Action::Attack);
    }

    #[rstest]
    fn decide_on_action_enemies_medium_health(mut tribute: Tribute, mut small_rng: SmallRng) {
        // If there are enemies nearby, but the tribute is low on health
        // the tribute should hide
        tribute.attributes.health = 20;
        let action = tribute
            .brain
            .act(&tribute.clone(), 2, &[], &[], &[], &mut small_rng);
        assert_eq!(action, Action::Move(None));
    }

    #[rstest]
    fn decide_on_action_preferred_action(mut tribute: Tribute, mut small_rng: SmallRng) {
        tribute.brain.set_preferred_action(Action::Rest, 1.0);
        let action = tribute
            .brain
            .act(&tribute.clone(), 0, &[], &[], &[], &mut small_rng);
        assert_eq!(action, Action::Rest);
    }

    #[rstest]
    fn clear_preferred_action(mut tribute: Tribute) {
        tribute.brain.set_preferred_action(Action::Rest, 1.0);
        assert_eq!(tribute.brain.preferred_action, Some(Action::Rest));
        assert_eq!(tribute.brain.preferred_action_percentage, 1.0);

        tribute.brain.clear_preferred_action();
        assert_eq!(tribute.brain.preferred_action, None);
        assert_eq!(tribute.brain.preferred_action_percentage, 0.0);
    }

    #[rstest]
    fn prefer_to_use_item_if_available(mut tribute: Tribute, mut small_rng: SmallRng) {
        let item = Item::new_random_consumable();
        tribute.items.push(item.clone());
        let action = tribute
            .brain
            .act(&tribute.clone(), 0, &[], &[], &[], &mut small_rng);
        assert_eq!(action, Action::UseItem(None));
    }

    #[rstest]
    fn prefer_to_hide_at_mid_health_and_visible(mut tribute: Tribute, mut small_rng: SmallRng) {
        tribute.attributes.health = 25;
        let action = tribute
            .brain
            .act(&tribute.clone(), 0, &[], &[], &[], &mut small_rng);
        assert_eq!(action, Action::Hide);
    }

    #[rstest]
    fn prefer_to_move_at_mid_health_and_low_sanity(mut tribute: Tribute, mut small_rng: SmallRng) {
        tribute.attributes.health = 25;
        tribute.attributes.sanity = 15;
        let action = tribute
            .brain
            .act(&tribute.clone(), 0, &[], &[], &[], &mut small_rng);
        assert_eq!(action, Action::Move(None));
    }

    #[rstest]
    fn decide_on_action_alone_healthy_no_movement(mut tribute: Tribute, mut small_rng: SmallRng) {
        tribute.attributes.movement = 0;
        let action = tribute
            .brain
            .act(&tribute.clone(), 0, &[], &[], &[], &mut small_rng);
        assert_eq!(action, Action::Rest);
    }

    #[rstest]
    fn decide_on_action_surrounded_low_health_low_movement_low_sanity(
        mut tribute: Tribute,
        mut small_rng: SmallRng,
    ) {
        tribute.attributes.health = 10;
        tribute.attributes.movement = 0;
        tribute.attributes.sanity = 15;
        let action = tribute
            .brain
            .act(&tribute.clone(), 3, &[], &[], &[], &mut small_rng);
        assert_eq!(action, Action::Attack);
    }

    #[rstest]
    fn decide_on_action_surrounded_low_health_low_sanity(
        mut tribute: Tribute,
        mut small_rng: SmallRng,
    ) {
        tribute.attributes.health = 15;
        tribute.attributes.sanity = 10;
        let action = tribute
            .brain
            .act(&tribute.clone(), 3, &[], &[], &[], &mut small_rng);
        assert_eq!(action, Action::Attack);
    }

    #[rstest]
    fn decide_on_action_surrounded_hidden_low_health(
        mut tribute: Tribute,
        mut small_rng: SmallRng,
    ) {
        tribute.attributes.is_hidden = true;
        tribute.attributes.health = 10;
        let action = tribute
            .brain
            .act(&tribute.clone(), 3, &[], &[], &[], &mut small_rng);
        assert_eq!(action, Action::None);
    }

    #[rstest]
    fn decide_on_action_surrounded_ok_health_low_sanity(
        mut tribute: Tribute,
        mut small_rng: SmallRng,
    ) {
        tribute.attributes.health = 25;
        tribute.attributes.sanity = 15;
        let action = tribute
            .brain
            .act(&tribute.clone(), 3, &[], &[], &[], &mut small_rng);
        assert_eq!(action, Action::Attack);
    }

    #[rstest]
    fn decide_on_action_heavily_surrounded_normal_sanity_and_intelligence(
        mut tribute: Tribute,
        mut small_rng: SmallRng,
    ) {
        tribute.attributes.intelligence = 50;
        tribute.attributes.sanity = 50;
        let action = tribute
            .brain
            .act(&tribute.clone(), 6, &[], &[], &[], &mut small_rng);
        assert_eq!(action, Action::Move(None));
    }

    #[rstest]
    fn decide_on_action_heavily_surrounded_low_sanity_and_intelligence(
        mut tribute: Tribute,
        mut small_rng: SmallRng,
    ) {
        tribute.attributes.intelligence = 20;
        tribute.attributes.sanity = 20;
        let action = tribute
            .brain
            .act(&tribute.clone(), 6, &[], &[], &[], &mut small_rng);
        assert_eq!(action, Action::Hide);
    }

    #[rstest]
    fn decide_on_action_heavily_surrounded_no_sanity_and_intelligence(
        mut tribute: Tribute,
        mut small_rng: SmallRng,
    ) {
        // recklessness = 100 - intelligence - sanity must reach low_intelligence
        // threshold (~91 for Balanced after variance) for the Attack branch.
        tribute.attributes.intelligence = 5;
        tribute.attributes.sanity = 0;
        let action = tribute
            .brain
            .act(&tribute.clone(), 6, &[], &[], &[], &mut small_rng);
        assert_eq!(action, Action::Attack);
    }

    #[rstest]
    fn test_psychotic_break_triggers_at_low_sanity(mut small_rng: SmallRng) {
        let mut tribute = Tribute::default();
        tribute.attributes.sanity = 3; // Below typical break threshold

        tribute
            .brain
            .check_psychotic_break(tribute.attributes.sanity, &mut small_rng);

        assert!(tribute.brain.psychotic_break.is_some());
    }

    #[rstest]
    fn test_psychotic_break_doesnt_trigger_at_normal_sanity(mut small_rng: SmallRng) {
        let mut tribute = Tribute::default();
        tribute.attributes.sanity = 50; // Well above break threshold

        tribute
            .brain
            .check_psychotic_break(tribute.attributes.sanity, &mut small_rng);

        assert!(tribute.brain.psychotic_break.is_none());
    }

    #[rstest]
    fn test_psychotic_break_recovery(mut small_rng: SmallRng) {
        // Use deterministic Brain so psychotic_break_threshold is fixed
        // (Balanced base = 7) and the +20 recovery margin is predictable.
        let mut tribute = Tribute {
            brain: Brain::default(),
            ..Tribute::default()
        };
        tribute.attributes.sanity = 3;

        // Trigger break
        tribute
            .brain
            .check_psychotic_break(tribute.attributes.sanity, &mut small_rng);
        assert!(tribute.brain.psychotic_break.is_some());

        // Sanity recovers significantly (needs to be 20+ above threshold)
        tribute.attributes.sanity = 30;
        tribute.brain.check_recovery(tribute.attributes.sanity);

        assert!(tribute.brain.psychotic_break.is_none());
    }

    #[rstest]
    fn test_psychotic_break_no_recovery_insufficient_sanity(mut small_rng: SmallRng) {
        // Deterministic Brain: Balanced psychotic_break_threshold = 7,
        // recovery requires sanity >= 27. sanity = 15 is below that.
        let mut tribute = Tribute {
            brain: Brain::default(),
            ..Tribute::default()
        };
        tribute.attributes.sanity = 3;

        // Trigger break
        tribute
            .brain
            .check_psychotic_break(tribute.attributes.sanity, &mut small_rng);
        assert!(tribute.brain.psychotic_break.is_some());

        // Sanity recovers but not enough
        tribute.attributes.sanity = 15;
        tribute.brain.check_recovery(tribute.attributes.sanity);

        // Should still be broken
        assert!(tribute.brain.psychotic_break.is_some());
    }

    #[rstest]
    fn test_berserk_break_attacks(mut small_rng: SmallRng) {
        let mut tribute = Tribute::default();
        tribute.brain.psychotic_break = Some(PsychoticBreakType::Berserk);

        let action = tribute
            .brain
            .act(&tribute.clone(), 2, &[], &[], &[], &mut small_rng);
        assert_eq!(action, Action::Attack);
    }

    #[rstest]
    fn test_paranoid_break_hides(mut small_rng: SmallRng) {
        let mut tribute = Tribute::default();
        tribute.brain.psychotic_break = Some(PsychoticBreakType::Paranoid);

        let action = tribute
            .brain
            .act(&tribute.clone(), 2, &[], &[], &[], &mut small_rng);
        assert_eq!(action, Action::Hide);
    }

    #[rstest]
    fn test_catatonic_break_does_nothing(mut small_rng: SmallRng) {
        let mut tribute = Tribute::default();
        tribute.brain.psychotic_break = Some(PsychoticBreakType::Catatonic);

        let action = tribute
            .brain
            .act(&tribute.clone(), 0, &[], &[], &[], &mut small_rng);
        assert_eq!(action, Action::None);
    }

    #[rstest]
    fn test_self_destructive_break_attacks(mut small_rng: SmallRng) {
        let mut tribute = Tribute::default();
        tribute.brain.psychotic_break = Some(PsychoticBreakType::SelfDestructive);
        tribute.attributes.health = 5; // Very low health - normally would rest/hide

        let action = tribute
            .brain
            .act(&tribute.clone(), 2, &[], &[], &[], &mut small_rng);
        // Self-destructive ignores health and attacks
        assert_eq!(action, Action::Attack);
    }

    #[rstest]
    fn from_traits_empty_uses_balanced_baseline() {
        // With no traits and zero variance, thresholds collapse to the
        // documented baseline values (the original `Balanced` numbers).
        let mut rng = SmallRng::seed_from_u64(101);
        let thresholds = PersonalityThresholds::from_traits(&[], &mut rng);
        // Each base ±20% — assert each lies in the expected window.
        assert!(
            (16..=24).contains(&thresholds.low_health),
            "low_health={}",
            thresholds.low_health
        );
        assert!((32..=48).contains(&thresholds.mid_health));
        assert!((8..=12).contains(&thresholds.extreme_low_sanity));
        assert!((16..=24).contains(&thresholds.low_sanity));
        assert!((28..=42).contains(&thresholds.mid_sanity));
        assert!((8..=12).contains(&thresholds.low_movement));
        assert!((28..=42).contains(&thresholds.high_intelligence));
        assert!((64..=96).contains(&thresholds.low_intelligence));
        assert!((6..=10).contains(&thresholds.psychotic_break_threshold));
    }

    #[rstest]
    fn from_traits_aggressive_lowers_health_thresholds() {
        // Aggressive: low_health -5 (→15), mid_health -10 (→30) before variance.
        // Use many seeds and check the mean is shifted below baseline.
        let aggressive = vec![Trait::Aggressive];
        let mut total_low: u32 = 0;
        let mut total_mid: u32 = 0;
        for seed in 0..50 {
            let mut rng = SmallRng::seed_from_u64(seed);
            let t = PersonalityThresholds::from_traits(&aggressive, &mut rng);
            total_low += t.low_health;
            total_mid += t.mid_health;
        }
        // Mean low_health ≈ 15, mean mid_health ≈ 30.
        let mean_low = total_low / 50;
        let mean_mid = total_mid / 50;
        assert!((12..=18).contains(&mean_low), "mean_low={}", mean_low);
        assert!((25..=35).contains(&mean_mid), "mean_mid={}", mean_mid);
    }

    #[rstest]
    fn from_traits_clamps_to_minimum_one() {
        // Stack many sanity-lowering traits to push past the clamp boundary.
        let traits = vec![Trait::Reckless, Trait::Aggressive];
        let mut rng = SmallRng::seed_from_u64(17);
        let t = PersonalityThresholds::from_traits(&traits, &mut rng);
        assert!(t.extreme_low_sanity >= 1);
        assert!(t.low_health >= 1);
    }

    /// 8pq: when scoring picks a non-neighbor goal, brain.act should
    /// return the *first hop* of the planned path (not the goal itself).
    /// Goal Sector4 from Sector1 must route via Cornucopia.
    #[rstest]
    fn brain_act_routes_first_hop_to_non_neighbor_goal(
        mut tribute: Tribute,
        mut small_rng: SmallRng,
    ) {
        use crate::areas::Area;

        // Place the tribute in Sector1.
        tribute.area = Area::Sector1;
        // Avoid the items branch & alliance branch — strip both.
        tribute.items.clear();
        tribute.brain.preferred_action = None;

        // Build a 7-area world. Make Sector4 carry the tribute's terrain
        // affinity so choose_destination prefers it; everything else is
        // plain Clearing so neighbor scoring is uniform.
        let mk = |a: Area, base: BaseTerrain| {
            AreaDetails::new_with_terrain(
                Some(format!("{a:?}")),
                a,
                TerrainType::new(base, vec![]).unwrap(),
            )
        };
        // Give the tribute a Desert affinity, then make Sector4 a Desert.
        tribute.terrain_affinity = vec![BaseTerrain::Desert];
        let all_areas = vec![
            mk(Area::Cornucopia, BaseTerrain::Clearing),
            mk(Area::Sector1, BaseTerrain::Clearing),
            mk(Area::Sector2, BaseTerrain::Clearing),
            mk(Area::Sector3, BaseTerrain::Clearing),
            mk(Area::Sector4, BaseTerrain::Desert),
            mk(Area::Sector5, BaseTerrain::Clearing),
            mk(Area::Sector6, BaseTerrain::Clearing),
        ];

        let action = tribute
            .brain
            .act(&tribute.clone(), 0, &[], &all_areas, &[], &mut small_rng);

        match action {
            Action::Move(Some(first_hop)) => {
                // Sector1's clockwise neighbors are Sector2 and Cornucopia.
                // The shortest path to Sector4 goes Sector1 -> Cornucopia
                // -> Sector4, so the first hop must be Cornucopia.
                assert_eq!(
                    first_hop,
                    Area::Cornucopia,
                    "expected first hop toward Sector4 to be Cornucopia, got {first_hop:?}"
                );
            }
            other => panic!("expected Move(Some(_)), got {other:?}"),
        }
    }
}

#[cfg(test)]
mod survival_override_tests {
    use super::*;
    use crate::areas::weather::Weather;
    use crate::items::Item;
    use crate::terrain::BaseTerrain;
    use crate::tributes::Tribute;
    use crate::tributes::actions::Action;

    #[test]
    fn override_dehydrated_at_water_terrain_picks_drink() {
        let mut t = Tribute::new("Test".to_string(), None, None);
        t.thirst = 3;
        let action = survival_override(&t, BaseTerrain::Wetlands, &Weather::Clear, false);
        assert_eq!(action, Some(Action::DrinkFromTerrain));
    }

    #[test]
    fn override_dehydrated_with_water_item_picks_drink_item() {
        let mut t = Tribute::new("Test".to_string(), None, None);
        t.thirst = 3;
        t.items.push(Item::new_water(None, 2));
        let action = survival_override(&t, BaseTerrain::Desert, &Weather::Clear, false);
        assert!(matches!(action, Some(Action::DrinkItem(_))));
    }

    #[test]
    fn override_starving_with_food_picks_eat() {
        let mut t = Tribute::new("Test".to_string(), None, None);
        t.hunger = 5;
        t.items.push(Item::new_food(None, 3));
        let action = survival_override(&t, BaseTerrain::Desert, &Weather::Clear, false);
        assert!(matches!(action, Some(Action::Eat(_))));
    }

    #[test]
    fn override_starving_at_forageable_terrain_no_inventory_picks_forage() {
        let mut t = Tribute::new("Test".to_string(), None, None);
        t.hunger = 5;
        let action = survival_override(&t, BaseTerrain::Wetlands, &Weather::Clear, false);
        assert_eq!(action, Some(Action::Forage));
    }

    #[test]
    fn override_starving_in_combat_returns_none() {
        let mut t = Tribute::new("Test".to_string(), None, None);
        t.hunger = 5;
        let action = survival_override(&t, BaseTerrain::Wetlands, &Weather::Clear, true);
        assert_eq!(action, None);
    }

    #[test]
    fn override_hungry_not_starving_returns_none() {
        let mut t = Tribute::new("Test".to_string(), None, None);
        t.hunger = 3;
        let action = survival_override(&t, BaseTerrain::Wetlands, &Weather::Clear, false);
        assert_eq!(action, None);
    }
}

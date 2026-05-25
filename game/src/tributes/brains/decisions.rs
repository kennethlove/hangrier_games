use crate::terrain::{BaseTerrain, Visibility};
use crate::tributes::Tribute;
use crate::tributes::actions::Action;
use crate::tributes::brains::scoring::action_score;
use crate::tributes::brains::{Brain, LOW_ENEMY_LIMIT};
use crate::tributes::traits::{REFUSERS, geometric_mean_affinity};
use rand::Rng;
use rand::RngExt;

impl Brain {
    pub fn decide_action_with_terrain(
        &self, tribute: &Tribute, nearby_tributes: u32,
        terrain: crate::terrain::TerrainType, phase: shared::messages::Phase, rng: &mut impl Rng,
    ) -> Action {
        if let Some(early) = self.run_pre_decision_overrides(
            tribute, nearby_tributes, Some(terrain.base), Some(phase), None,
            &crate::config::GameConfig::default(), rng,
        ) { return early; }
        let scarce = matches!(terrain.base, BaseTerrain::Desert | BaseTerrain::Tundra | BaseTerrain::Badlands);
        let concealed = matches!(terrain.base.visibility(), Visibility::Concealed);
        let base = if nearby_tributes == 0 { self.decide_action_no_enemies(tribute) }
            else if nearby_tributes < LOW_ENEMY_LIMIT { self.decide_action_few_enemies_with_terrain(tribute, concealed) }
            else { self.decide_action_many_enemies_with_terrain(tribute, concealed) };
        let tuning = crate::tributes::combat_tuning::CombatTuning::default();
        let base = if matches!(base, Action::Attack) && action_score(tribute, &Action::Attack, &[], &tuning) == i32::MIN
        { Action::Rest } else { base };
        match base {
            Action::Move(None) if scarce => Action::Move(None),
            Action::Hide if concealed => Action::Hide,
            other => other,
        }
    }

    fn decide_action_few_enemies_with_terrain(&self, t: &Tribute, concealed: bool) -> Action {
        let lh = self.thresholds.low_health; let mh = self.thresholds.mid_health; let ls = self.thresholds.low_sanity;
        match t.attributes.health {
            h if h < lh => self.decide_action_few_enemies_low_health_with_terrain(t, concealed),
            h if h >= lh && h <= mh => {
                if t.attributes.sanity > ls && concealed { Action::Hide }
                else if t.attributes.sanity > ls { Action::Move(None) } else { Action::Attack }
            }
            _ if concealed && t.attributes.sanity > ls => Action::Hide,
            _ => Action::Attack,
        }
    }

    fn decide_action_few_enemies_low_health_with_terrain(&self, t: &Tribute, concealed: bool) -> Action {
        let lm = self.thresholds.low_movement; let ms = self.thresholds.mid_sanity; let es = self.thresholds.extreme_low_sanity;
        let s = (t.attributes.movement, t.attributes.sanity, t.attributes.is_hidden);
        match s {
            (m, sa, false) if m < lm && sa >= ms && concealed => Action::Hide,
            (m, sa, false) if m < lm && sa >= ms => Action::Hide,
            (m, sa, _) if m < lm && sa >= es && sa < ms => Action::Attack,
            (_, sa, false) if sa >= ms => Action::Move(None),
            (_, sa, false) if sa < ms => Action::Attack,
            (_, _, true) => Action::None,
            _ => Action::Move(None),
        }
    }

    fn decide_action_many_enemies_with_terrain(&self, t: &Tribute, _concealed: bool) -> Action {
        let hi = self.thresholds.high_intelligence; let li = self.thresholds.low_intelligence;
        let r = 100u32.saturating_sub(t.attributes.intelligence).saturating_sub(t.attributes.sanity);
        match r {
            r if r < hi => Action::Move(None), r if r >= li => Action::Attack,
            _ => Action::Hide,
        }
    }

    pub(crate) fn decide_action_no_enemies(&self, t: &Tribute) -> Action {
        let lh = self.thresholds.low_health; let mh = self.thresholds.mid_health; let ls = self.thresholds.low_sanity;
        match t.attributes.health {
            h if h < lh => Action::Rest,
            h if h >= lh && h <= mh => {
                if t.attributes.sanity > ls && t.is_visible() { Action::Hide } else { Action::Move(None) }
            }
            _ => if t.attributes.movement == 0 { Action::Rest } else { Action::Move(None) },
        }
    }

    pub(crate) fn decide_action_few_enemies(&self, t: &Tribute) -> Action {
        let lh = self.thresholds.low_health; let mh = self.thresholds.mid_health; let ls = self.thresholds.low_sanity;
        match t.attributes.health {
            h if h < lh => self.decide_action_few_enemies_low_health(t),
            h if h >= lh && h <= mh => {
                if t.attributes.sanity > ls { Action::Move(None) } else { Action::Attack }
            }
            _ => Action::Attack,
        }
    }

    fn decide_action_few_enemies_low_health(&self, t: &Tribute) -> Action {
        let lm = self.thresholds.low_movement; let ms = self.thresholds.mid_sanity; let es = self.thresholds.extreme_low_sanity;
        let s = (t.attributes.movement, t.attributes.sanity, t.attributes.is_hidden);
        match s {
            (m, sa, false) if m < lm && sa >= ms => Action::Hide,
            (m, sa, _) if m < lm && sa >= es && sa < ms => Action::Attack,
            (_, sa, false) if sa >= ms => Action::Move(None),
            (_, sa, false) if sa < ms => Action::Attack,
            (_, _, true) => Action::None,
            _ => Action::Move(None),
        }
    }

    pub(crate) fn decide_action_many_enemies(&self, t: &Tribute) -> Action {
        let hi = self.thresholds.high_intelligence; let li = self.thresholds.low_intelligence;
        let r = 100u32.saturating_sub(t.attributes.intelligence).saturating_sub(t.attributes.sanity);
        match r { r if r < hi => Action::Move(None), r if r >= li => Action::Attack, _ => Action::Hide }
    }

    pub(crate) fn wants_to_propose_alliance(
        &self, tribute: &Tribute, nearby_tributes: u32, rng: &mut impl Rng,
    ) -> bool {
        use crate::tributes::alliances::MAX_ALLIES;
        if nearby_tributes == 0 { return false; }
        if tribute.allies.len() >= MAX_ALLIES { return false; }
        if tribute.attributes.health < self.thresholds.low_health
            || tribute.attributes.sanity < self.thresholds.low_sanity { return false; }
        if tribute.traits.iter().any(|t| REFUSERS.contains(t)) { return false; }
        let affinity = geometric_mean_affinity(&tribute.traits);
        if affinity < 1.0 { return false; }
        rng.random_bool((0.05 * affinity).clamp(0.0, 0.15))
    }
}

use crate::messages::{MessagePayload, TributeRef};
use crate::tributes::Tribute;
use crate::tributes::wounds;
use rand::RngExt;
use shared::wounds::{BodyPart, Wound, WoundSeverity, WoundType};

/// Attribute maximums
const MAX_HEALTH: u32 = 100;
const MAX_SANITY: u32 = 100;
const MAX_MOVEMENT: u32 = 100;
const MAX_STRENGTH: u32 = 50;
const MAX_BRAVERY: u32 = 100;

/// Default healing amounts
const DEFAULT_HEAL: u32 = 5;
const DEFAULT_MENTAL_HEAL: u32 = 5;

impl Tribute {
    /// Tribute is lonely/homesick/etc., loses some sanity.
    pub(crate) fn misses_home(&mut self) {
        let loneliness = self.attributes.bravery as f64 / 100.0; // how lonely is the tribute?

        if loneliness.round() < 0.25 {
            if self.attributes.sanity < 25 {
                self.attributes.sanity = self
                    .attributes
                    .sanity
                    .saturating_sub(self.attributes.bravery);
            }
            self.attributes.sanity = self
                .attributes
                .sanity
                .saturating_sub(self.attributes.bravery);
        }
    }

    /// Reduces attack strength.
    pub(crate) fn reduce_strength(&mut self, amount: u32) {
        self.attributes.strength = self.attributes.strength.saturating_sub(amount).max(1);
    }

    /// Increases attack strength.
    pub(crate) fn increase_strength(&mut self, amount: u32) {
        self.attributes.strength = self
            .attributes
            .strength
            .saturating_add(amount)
            .min(MAX_STRENGTH);
    }

    /// Reduces movement which limits travel and is used by AI for retreat decisions.
    pub(crate) fn reduce_movement(&mut self, amount: u32) {
        self.attributes.movement = self.attributes.movement.saturating_sub(amount).max(1);
    }

    /// Reduces intelligence which affects decision-making and hiding.
    pub(crate) fn reduce_intelligence(&mut self, amount: u32) {
        self.attributes.intelligence = self.attributes.intelligence.saturating_sub(amount).max(1);
    }

    /// Increases bravery which affects decision-making.
    pub(crate) fn increase_bravery(&mut self, amount: u32) {
        self.attributes.bravery = self
            .attributes
            .bravery
            .saturating_add(amount)
            .min(MAX_BRAVERY);
    }

    /// Increases movement which allows more travel
    pub(crate) fn increase_movement(&mut self, amount: u32) {
        self.attributes.movement = self
            .attributes
            .movement
            .saturating_add(amount)
            .min(MAX_MOVEMENT);
    }

    /// Restores health.
    pub(crate) fn heals(&mut self, health: u32) {
        if self.is_alive() {
            self.attributes.health = self
                .attributes
                .health
                .saturating_add(health)
                .min(MAX_HEALTH);
        }
    }

    /// Restores mental health.
    pub(crate) fn heals_mental_damage(&mut self, sanity: u32) {
        self.attributes.sanity = self
            .attributes
            .sanity
            .saturating_add(sanity)
            .min(MAX_SANITY);
    }

    /// Restores movement.
    pub(crate) fn short_rests(&mut self) {
        self.attributes.movement = MAX_MOVEMENT;
    }

    /// Restores movement, some health, and some sanity
    pub(crate) fn long_rests(&mut self) {
        self.short_rests();
        self.heals(DEFAULT_HEAL);
        self.heals_mental_damage(DEFAULT_MENTAL_HEAL);
    }

    // --- Wound-based blood system ---

    /// Creates a wound on this tribute. Adds the wound and drains blood
    /// immediately based on severity.
    pub(crate) fn create_wound(
        &mut self,
        wound_type: WoundType,
        severity: WoundSeverity,
        body_part: BodyPart,
    ) {
        let mut wound = Wound::new(wound_type, severity, body_part);
        wound.created_day = self.game_day;
        self.wounds.push(wound);
    }

    /// Drains blood from all active bleeding wounds. Returns the total blood
    /// lost this period. Called once per game cycle.
    pub(crate) fn drain_blood_from_wounds(&mut self) -> u32 {
        let mut total_loss = 0u32;
        for wound in &self.wounds {
            total_loss += wound.blood_loss_per_period();
        }
        self.blood = self.blood.saturating_sub(total_loss);
        total_loss
    }

    /// Natural wound healing pass. Each wound attempts to heal; this may stop
    /// bleeding or cause infection for Critical wounds.
    pub(crate) fn heal_wounds(
        &mut self,
        rng: &mut impl rand::Rng,
        events: &mut Vec<crate::messages::TaggedEvent>,
    ) {
        let tribute_name = self.name.clone();
        let tribute_id = self.identifier.clone();

        for wound in &mut self.wounds {
            let was_infected = wound.infected;
            let was_bleeding = wound.bleeding;
            let roll: f64 = rng.random();
            wound.heals_naturally(roll);

            // Emit infection event
            if !was_infected && wound.infected {
                events.push(crate::messages::TaggedEvent::new(
                    format!(
                        "{}'s {} wound became infected",
                        tribute_name, wound.body_part
                    ),
                    MessagePayload::WoundInfected {
                        tribute: TributeRef {
                            identifier: tribute_id.clone().into(),
                            name: tribute_name.clone(),
                        },
                        body_part: wound.body_part.to_string(),
                    },
                ));
            }

            // Emit heal event (stopped bleeding, not infected)
            if was_bleeding && !wound.bleeding && !wound.infected {
                events.push(crate::messages::TaggedEvent::new(
                    format!(
                        "{}'s {} wound stopped bleeding",
                        tribute_name, wound.body_part
                    ),
                    MessagePayload::WoundHealed {
                        tribute: TributeRef {
                            identifier: tribute_id.clone().into(),
                            name: tribute_name.clone(),
                        },
                        body_part: wound.body_part.to_string(),
                    },
                ));
            }
        }
        // Remove fully healed wounds (no longer bleeding, not infected, Minor only)
        self.wounds
            .retain(|w| w.bleeding || w.infected || w.severity != WoundSeverity::Minor);
    }

    /// Restores blood from rest/food.
    #[allow(dead_code)]
    pub(crate) fn restores_blood(&mut self, amount: u32) {
        self.blood = self.blood.saturating_add(amount).min(wounds::MAX_BLOOD);
    }

    // --- Effective stats (base + wound penalties) ---

    /// Effective strength after wound penalties.
    pub fn effective_strength(&self) -> i32 {
        let base = self.attributes.strength as i32;
        let mut penalty = 0i32;
        for wound in &self.wounds {
            let mut p = wounds::strength_penalty(wound.severity);
            p = (p as f64 * wounds::body_part_penalty_multiplier(wound.body_part)) as i32;
            penalty += p;
        }
        (base + penalty).max(0)
    }

    /// Effective movement after wound penalties.
    pub fn effective_movement(&self) -> i32 {
        let base = self.attributes.movement as i32;
        let mut penalty = 0i32;
        for wound in &self.wounds {
            let mut p = wounds::movement_penalty(wound.severity);
            p = (p as f64 * wounds::body_part_penalty_multiplier(wound.body_part)) as i32;
            penalty += p;
        }
        (base + penalty).max(0)
    }

    /// Effective defense after wound penalties.
    pub fn effective_defense(&self) -> i32 {
        let base = self.attributes.defense as i32;
        let mut penalty = 0i32;
        for wound in &self.wounds {
            let mut p = wounds::defense_penalty(wound.severity);
            p = (p as f64 * wounds::body_part_penalty_multiplier(wound.body_part)) as i32;
            penalty += p;
        }
        (base + penalty).max(0)
    }

    /// Effective health after wound penalties.
    pub fn effective_health(&self) -> u32 {
        let base = self.attributes.health as i32;
        let mut penalty = 0i32;
        for wound in &self.wounds {
            let mut p = wounds::health_penalty(wound.severity);
            p = (p as f64 * wounds::body_part_penalty_multiplier(wound.body_part)) as i32;
            penalty += p;
        }
        (base + penalty).max(0) as u32
    }

    /// Effective bravery after wound penalties.
    pub fn effective_bravery(&self) -> i32 {
        let base = self.attributes.bravery as i32;
        let mut penalty = 0i32;
        for wound in &self.wounds {
            let mut p = wounds::bravery_penalty(wound.severity);
            p = (p as f64 * wounds::body_part_penalty_multiplier(wound.body_part)) as i32;
            penalty += p;
        }
        (base + penalty).max(0)
    }

    /// Whether this tribute should trigger heroism (low blood bravery boost).
    pub fn should_heroism(&self) -> bool {
        let blood_ratio = self.blood as f64 / wounds::MAX_BLOOD as f64;
        blood_ratio <= wounds::HEROISM_BLOOD_THRESHOLD
    }

    /// Effective sanity after wound-induced mental distress.
    /// Each wound imposes a sanity penalty based on severity.
    pub fn effective_sanity(&self) -> u32 {
        let base = self.attributes.sanity as i32;
        let mut penalty = 0i32;
        for wound in &self.wounds {
            let p = match wound.severity {
                WoundSeverity::Minor => -1,
                WoundSeverity::Moderate => -3,
                WoundSeverity::Severe => -5,
                WoundSeverity::Critical => -10,
            };
            penalty += p;
        }
        (base + penalty).max(0) as u32
    }

    /// Ticks all mental conditions: applies sanity drain, advances durations,
    /// and removes expired conditions.
    pub(crate) fn tick_mental_conditions(&mut self) {
        let total_drain: u32 = self
            .mental_conditions
            .iter()
            .map(|c| c.severity().sanity_drain())
            .sum();
        if total_drain > 0 {
            self.attributes.sanity = self.attributes.sanity.saturating_sub(total_drain);
        }
        for condition in &mut self.mental_conditions {
            condition.tick();
        }
        self.mental_conditions.retain(|c| !c.is_expired());
    }
}

#[cfg(test)]
mod tests {
    use crate::tributes::Tribute;
    use rstest::*;

    #[fixture]
    fn tribute() -> Tribute {
        Tribute::new("Katniss".to_string(), None, None)
    }

    #[rstest]
    fn saturating_sub_health(mut tribute: Tribute) {
        let health = tribute.attributes.health();
        tribute
            .attributes
            .set_health(tribute.attributes.health().saturating_sub(10));
        assert_eq!(tribute.attributes.health(), health - 10);
    }

    #[rstest]
    fn heals(mut tribute: Tribute) {
        tribute.attributes.set_health(50);
        tribute.heals(10);
        assert_eq!(tribute.attributes.health(), 60);
    }

    #[rstest]
    fn saturating_sub_sanity(mut tribute: Tribute) {
        let sanity = tribute.attributes.sanity();
        tribute
            .attributes
            .set_sanity(tribute.attributes.sanity().saturating_sub(10));
        assert_eq!(tribute.attributes.sanity(), sanity - 10);
    }

    #[rstest]
    fn sanity_saturates_at_zero(mut tribute: Tribute) {
        tribute.attributes.set_sanity(0);
        tribute
            .attributes
            .set_sanity(tribute.attributes.sanity().saturating_sub(10));
        assert_eq!(tribute.attributes.sanity(), 0);
    }

    #[rstest]
    fn heals_mental_damage(mut tribute: Tribute) {
        tribute.attributes.set_sanity(50);
        tribute.heals_mental_damage(10);
        assert_eq!(tribute.attributes.sanity(), 60);
    }

    #[rstest]
    fn short_rests(mut tribute: Tribute) {
        tribute.attributes.movement = 0;
        tribute.short_rests();
        assert_eq!(tribute.attributes.movement, 100);
    }

    #[rstest]
    fn long_rests(mut tribute: Tribute) {
        tribute.attributes.movement = 0;
        tribute.attributes.set_health(50);
        tribute.attributes.set_sanity(50);
        tribute.long_rests();
        assert_eq!(tribute.attributes.movement, 100);
        assert_eq!(tribute.attributes.health(), 55);
        assert_eq!(tribute.attributes.sanity(), 55);
    }

    #[rstest]
    fn misses_home(mut tribute: Tribute) {
        tribute.attributes.bravery = 20;
        tribute.attributes.set_sanity(20);
        let sanity = tribute.attributes.sanity();
        tribute.misses_home();
        assert!(tribute.attributes.sanity() < sanity);
    }
}

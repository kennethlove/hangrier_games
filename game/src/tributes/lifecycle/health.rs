use crate::tributes::Tribute;

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
                self.takes_mental_damage(self.attributes.bravery);
            }
            self.takes_mental_damage(self.attributes.bravery);
        }
    }

    /// Reduces physical health.
    pub(crate) fn takes_physical_damage(&mut self, damage: u32) {
        self.attributes.health = self.attributes.health.saturating_sub(damage);
    }

    /// Reduces mental health.
    pub(crate) fn takes_mental_damage(&mut self, damage: u32) {
        self.attributes.sanity = self.attributes.sanity.saturating_sub(damage);
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
    fn takes_physical_damage(mut tribute: Tribute) {
        let health = tribute.attributes.health;
        tribute.takes_physical_damage(10);
        assert_eq!(tribute.attributes.health, health - 10);
    }

    #[rstest]
    fn heals(mut tribute: Tribute) {
        tribute.attributes.health = 50;
        tribute.heals(10);
        assert_eq!(tribute.attributes.health, 60);
    }

    #[rstest]
    fn takes_mental_damage(mut tribute: Tribute) {
        let sanity = tribute.attributes.sanity;
        tribute.takes_mental_damage(10);
        assert_eq!(tribute.attributes.sanity, sanity - 10);
    }

    #[rstest]
    fn takes_no_mental_damage_when_insane(mut tribute: Tribute) {
        tribute.attributes.sanity = 0;
        tribute.takes_mental_damage(10);
        assert_eq!(tribute.attributes.sanity, 0);
    }

    #[rstest]
    fn heals_mental_damage(mut tribute: Tribute) {
        tribute.attributes.sanity = 50;
        tribute.heals_mental_damage(10);
        assert_eq!(tribute.attributes.sanity, 60);
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
        tribute.attributes.health = 50;
        tribute.attributes.sanity = 50;
        tribute.long_rests();
        assert_eq!(tribute.attributes.movement, 100);
        assert_eq!(tribute.attributes.health, 55);
        assert_eq!(tribute.attributes.sanity, 55);
    }

    #[rstest]
    fn misses_home(mut tribute: Tribute) {
        tribute.attributes.bravery = 20;
        tribute.attributes.sanity = 20;
        let sanity = tribute.attributes.sanity;
        tribute.misses_home();
        assert!(tribute.attributes.sanity < sanity);
    }
}

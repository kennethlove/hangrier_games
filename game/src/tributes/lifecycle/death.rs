use crate::tributes::Tribute;
use crate::tributes::statuses::TributeStatus;
use rand::RngExt;
use rand::prelude::*;
use rand::rngs::SmallRng;

impl Tribute {
    /// Marks the tribute as dead and reveals them.
    pub fn dies(&mut self) {
        self.blood = 0;
        self.set_status(TributeStatus::Dead);
        self.attributes.is_hidden = false;
        self.items.clear();
    }

    /// Does the tribute have health and an OK status?
    pub fn is_alive(&self) -> bool {
        self.blood > 0
            && self.status != TributeStatus::Dead
            && self.status != TributeStatus::RecentlyDead
    }

    /// Hides the tribute from view.
    pub(crate) fn hides(&mut self) -> bool {
        let mut rng = SmallRng::from_rng(&mut rand::rng());
        let hidden = rng.random_bool(self.attributes.intelligence as f64 / 100.0);
        self.attributes.is_hidden = hidden;
        hidden
    }

    /// Helper function to see if the tribute is hidden
    pub fn is_visible(&self) -> bool {
        !self.attributes.is_hidden
    }
}

#[cfg(test)]
mod tests {
    use crate::tributes::Tribute;
    use crate::tributes::statuses::TributeStatus;
    use rstest::*;

    #[fixture]
    fn tribute() -> Tribute {
        Tribute::new("Katniss".to_string(), None, None)
    }

    #[rstest]
    fn takes_no_physical_damage_when_dead(mut tribute: Tribute) {
        tribute.dies();
        // Blood is already 0 after dies(), saturating_sub keeps it at 0
        tribute.blood = tribute.blood.saturating_sub(100);
        assert_eq!(tribute.blood, 0);
    }

    #[rstest]
    fn does_not_heal_when_dead(mut tribute: Tribute) {
        tribute.dies();
        tribute.heals(10);
        assert_eq!(tribute.blood, 0);
    }

    #[rstest]
    fn dies(mut tribute: Tribute) {
        tribute.dies();
        assert_eq!(tribute.blood, 0);
        assert_eq!(tribute.status, TributeStatus::Dead);
        assert!(!tribute.attributes.is_hidden);
        assert_eq!(tribute.items.len(), 0);
    }

    #[rstest]
    fn is_alive(mut tribute: Tribute) {
        assert!(tribute.is_alive());
        tribute.dies();
        assert!(!tribute.is_alive());
    }

    #[rstest]
    fn hides_success(mut tribute: Tribute) {
        tribute.attributes.intelligence = 100;
        let hidden = tribute.hides();
        assert!(hidden);
        assert!(tribute.attributes.is_hidden);
    }

    #[rstest]
    fn hides_fail(mut tribute: Tribute) {
        tribute.attributes.intelligence = 0;
        let hidden = tribute.hides();
        assert!(!hidden);
        assert!(!tribute.attributes.is_hidden);
    }

    #[rstest]
    fn is_visible(mut tribute: Tribute) {
        assert!(tribute.is_visible());
        tribute.attributes.is_hidden = true;
        assert!(!tribute.is_visible());
    }
}

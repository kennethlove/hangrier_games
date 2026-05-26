//! Inventory and item management for tributes.
//!
//! This module handles:
//! - OwnsItems trait implementation
//! - Item filtering (weapons, shields, consumables)
//! - Item usage and effects
//! - Patron gifts
//! - Taking items from areas

use crate::areas::AreaDetails;
use crate::items::{Attribute, Item, ItemError, OwnsItems};
use crate::tributes::Tribute;
use crate::tributes::afflictions::apply_cure;
use rand::prelude::*;
use rand::rngs::SmallRng;

impl OwnsItems for Tribute {
    fn add_item(&mut self, item: Item) {
        self.items.push(item);
    }

    fn has_item(&self, item: &Item) -> bool {
        self.items.iter().any(|i| i == item)
    }

    fn use_item(&mut self, item: &Item) -> Result<(), ItemError> {
        let index = self
            .items
            .iter()
            .position(|i| i.identifier == item.identifier)
            .ok_or(ItemError::ItemNotFound)?;

        let current_durability = self.items[index].current_durability;

        // If the item has 0 durability left, return an error.
        // If the item has 1 durability left, remove it from the inventory.
        // If the item has more than 1 durability left, decrement it.
        match current_durability {
            0 => Err(ItemError::ItemNotFound),
            1 => {
                self.items.remove(index);
                Ok(())
            }
            _ => {
                self.items[index].current_durability =
                    self.items[index].current_durability.saturating_sub(1);
                Ok(())
            }
        }
    }

    fn remove_item(&mut self, item: &Item) -> Result<(), ItemError> {
        let index = self
            .items
            .iter()
            .position(|i| i.identifier == item.identifier);
        if let Some(index) = index {
            self.items.remove(index);
            Ok(())
        } else {
            Err(ItemError::ItemNotFound)
        }
    }
}

impl Tribute {
    /// Take an item from the current area
    pub(crate) fn take_nearby_item(&mut self, area_details: &mut AreaDetails) -> Option<Item> {
        let mut rng = SmallRng::from_rng(&mut rand::rng());
        let items = area_details.items.clone();
        if items.is_empty() {
            None
        } else {
            let item = items.choose(&mut rng).unwrap().clone();
            if let Ok(()) = area_details.use_item(&item) {
                self.add_item(item.clone());

                // ~10% chance to acquire a fixation on the picked-up item.
                crate::tributes::afflictions::fixation::maybe_acquire_item_fixation(self, &item);

                return Some(item.clone());
            }
            None
        }
    }

    /// Use consumable item from inventory
    pub(crate) fn try_use_consumable(&mut self, chosen_item: &Item) -> Result<(), ItemError> {
        let items = self.consumables();

        // If the tribute has the item...
        let item = match items
            .iter()
            .find(|i| i.identifier == chosen_item.identifier)
        {
            Some(selected_item) => {
                // select it
                selected_item.clone()
            }
            None => {
                // otherwise, quit because you can't use an item you don't have
                return Err(ItemError::ItemNotFound);
            }
        };

        if self.use_item(&item).is_err() {
            return Err(ItemError::ItemNotUsable);
        }

        // Apply item effect
        match item.attribute {
            Attribute::Health => self.heals(item.effect as u32),
            Attribute::Sanity => self.heals_mental_damage(item.effect as u32),
            Attribute::Movement | Attribute::Speed => self.increase_movement(item.effect as u32),
            Attribute::Bravery => self.increase_bravery(item.effect as u32),
            Attribute::Strength => self.increase_strength(item.effect as u32),
            _ => return Err(ItemError::InvalidAttribute),
        }

        // Apply cure effect if item is a cure item (bandage, splint, antibiotic).
        let afflictions: Vec<_> = self.afflictions.values().cloned().collect();
        let mut affliction_vec = afflictions;
        let cure_result = apply_cure(&mut affliction_vec, &item.name);
        // Update the tribute's affliction map with the cured state.
        self.afflictions = affliction_vec.into_iter().map(|a| (a.key(), a)).collect();

        match cure_result {
            crate::tributes::afflictions::CureOutcome::Cured { .. } => {
                // Cure applied successfully; afflictions map already updated.
            }
            crate::tributes::afflictions::CureOutcome::NoEffect { .. } => {
                // No matching affliction; not an error, just no cure effect.
            }
        }

        Ok(())
    }

    /// What items does the tribute have?
    pub(crate) fn available_items(&self) -> Vec<Item> {
        self.items
            .iter()
            .filter(|i| i.current_durability > 0)
            .cloned()
            .collect()
    }

    /// Which items are marked as consumable?
    pub fn consumables(&self) -> Vec<Item> {
        self.available_items()
            .iter()
            .filter(|i| i.is_consumable())
            .cloned()
            .collect()
    }

    /// Get a mutable reference to the tribute's currently equipped weapon.
    ///
    /// Returns the last weapon (matching `weapons()` selection semantics) with
    /// `current_durability > 0`. Used by combat to apply wear directly to the
    /// equipped item rather than to a clone.
    pub(crate) fn equipped_weapon_mut(&mut self) -> Option<&mut Item> {
        self.items
            .iter_mut()
            .filter(|i| i.is_weapon() && i.current_durability > 0)
            .last()
    }

    /// Get a mutable reference to the tribute's currently equipped shield.
    ///
    /// Returns the last shield (matching `shields()` selection semantics) with
    /// `current_durability > 0`. Used by combat to apply wear directly to the
    /// equipped item rather than to a clone.
    pub(crate) fn equipped_shield_mut(&mut self) -> Option<&mut Item> {
        self.items
            .iter_mut()
            .filter(|i| i.is_defensive() && i.current_durability > 0)
            .last()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tributes::Tribute;
    use rstest::*;

    #[fixture]
    fn tribute() -> Tribute {
        Tribute::new("Katniss".to_string(), None, None)
    }

    #[rstest]
    fn add_item(mut tribute: Tribute) {
        let item = Item::new_random_weapon();
        tribute.add_item(item.clone());
        assert!(tribute.has_item(&item));
    }

    #[rstest]
    fn has_item(mut tribute: Tribute) {
        let item = Item::new_random_weapon();
        tribute.add_item(item.clone());
        assert!(tribute.has_item(&item));
    }

    #[rstest]
    fn use_item(mut tribute: Tribute) {
        let mut item = Item::new_random_consumable();
        item.current_durability = 1;
        item.max_durability = 1;
        tribute.add_item(item.clone());
        assert!(tribute.use_item(&item).is_ok());
        assert!(!tribute.has_item(&item));
    }

    #[rstest]
    fn use_item_reusable(mut tribute: Tribute) {
        let mut item = Item::new_random_weapon();
        item.current_durability = 2;
        item.max_durability = 2;
        tribute.add_item(item.clone());
        assert!(tribute.use_item(&item).is_ok());
        // After use, stored item has current_durability=1, so equality with the
        // original `item` (current_durability=2) no longer holds. Verify directly.
        assert_eq!(tribute.items.len(), 1);
        assert_eq!(tribute.items[0].current_durability, 1);
    }

    #[rstest]
    fn equipped_weapon_mut_returns_actual_item(mut tribute: Tribute) {
        let mut weapon = Item::new_random_weapon();
        weapon.current_durability = 5;
        weapon.max_durability = 5;
        tribute.add_item(weapon.clone());

        // Mutate via equipped_weapon_mut and verify it persists in inventory.
        {
            let equipped = tribute.equipped_weapon_mut().expect("weapon equipped");
            equipped.current_durability = 3;
        }
        assert_eq!(tribute.items[0].current_durability, 3);
    }

    #[rstest]
    fn equipped_shield_mut_returns_actual_item(mut tribute: Tribute) {
        let mut shield = Item::new_random_shield();
        shield.current_durability = 5;
        shield.max_durability = 5;
        tribute.add_item(shield.clone());

        {
            let equipped = tribute.equipped_shield_mut().expect("shield equipped");
            equipped.current_durability = 2;
        }
        assert_eq!(tribute.items[0].current_durability, 2);
    }

    #[rstest]
    fn equipped_weapon_mut_skips_broken(mut tribute: Tribute) {
        let mut broken = Item::new_random_weapon();
        broken.current_durability = 0;
        broken.max_durability = 5;
        tribute.add_item(broken);
        assert!(tribute.equipped_weapon_mut().is_none());
    }
}

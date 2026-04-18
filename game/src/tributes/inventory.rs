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

        let current_quantity = self.items[index].quantity;

        // If the item has 0 uses left, return an error.
        // If the item has 1 use left, remove it from the inventory.
        // If the item has more than 1 use left, decrement the quantity.
        match current_quantity {
            0 => Err(ItemError::ItemNotFound),
            1 => {
                self.items.remove(index);
                Ok(())
            }
            _ => {
                self.items[index].quantity = self.items[index].quantity.saturating_sub(1);
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
    /// Receive a gift from patrons based on district
    pub(crate) fn receive_patron_gift(&mut self, mut rng: impl Rng) -> Option<Item> {
        // Gift from patrons?
        let chance = match self.district {
            1 | 2 => 1.0 / 10.0,
            3 | 4 => 1.0 / 15.0,
            5 | 6 => 1.0 / 20.0,
            7 | 8 => 1.0 / 25.0,
            9 | 10 => 1.0 / 30.0,
            11 | 12 => 1.0 / 50.0,
            _ => 1.0, // Mainly for testing/debugging purposes
        };

        if rng.random_bool(chance) {
            Some(Item::new_random_consumable())
        } else {
            None
        }
    }

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

                return Some(item.clone());
            }
            None
        }
    }

    /// Use consumable item from inventory
    pub(crate) fn try_use_consumable(&mut self, chosen_item: &Item) -> Result<(), ItemError> {
        let items = self.consumables();
        let item: Item;

        // If the tribute has the item...
        match items
            .iter()
            .find(|i| i.identifier == chosen_item.identifier)
        {
            Some(selected_item) => {
                // select it
                item = selected_item.clone();
            }
            None => {
                // otherwise, quit because you can't use an item you don't have
                return Err(ItemError::ItemNotFound);
            }
        }

        if self.use_item(&item).is_err() {
            return Err(ItemError::ItemNotUsable);
        }

        // Apply item effect
        match item.attribute {
            Attribute::Health => self.heals(item.effect as u32),
            Attribute::Sanity => self.heals_mental_damage(item.effect as u32),
            Attribute::Movement => self.increase_movement(item.effect as u32),
            Attribute::Bravery => self.increase_bravery(item.effect as u32),
            Attribute::Speed => self.increase_speed(item.effect as u32),
            Attribute::Strength => self.increase_strength(item.effect as u32),
            _ => return Err(ItemError::InvalidAttribute),
        }
        Ok(())
    }

    /// What items does the tribute have?
    pub(crate) fn available_items(&self) -> Vec<Item> {
        self.items
            .iter()
            .filter(|i| i.quantity > 0)
            .cloned()
            .collect()
    }

    /// Which items are marked as weapons?
    pub(crate) fn weapons(&self) -> Vec<Item> {
        self.available_items()
            .iter()
            .filter(|i| i.is_weapon())
            .cloned()
            .collect()
    }

    /// Which items are marked as shields?
    pub(crate) fn shields(&self) -> Vec<Item> {
        self.available_items()
            .iter()
            .filter(|i| i.is_defensive())
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
        item.quantity = 1;
        tribute.add_item(item.clone());
        assert!(tribute.use_item(&item).is_ok());
        assert!(!tribute.has_item(&item));
    }

    #[rstest]
    fn use_item_reusable(mut tribute: Tribute) {
        let mut item = Item::new_random_weapon();
        item.quantity = 2;
        tribute.add_item(item.clone());
        assert!(tribute.use_item(&item).is_ok());
        assert!(tribute.has_item(&item));
        assert_eq!(tribute.items[0].quantity, 1);
    }
}

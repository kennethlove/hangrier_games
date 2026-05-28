mod generation;
pub mod name_generator;

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};
use shared::afflictions::Substance;
use std::fmt::Display;
use std::str::FromStr;
use strum::EnumIter;
use thiserror::Error;
use uuid::Uuid;

/// Item rarity determines effect strength and spawn probability.
/// Distribution: Common 60%, Uncommon 25%, Rare 12%, Legendary 3%
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum ItemRarity {
    Common,
    Uncommon,
    Rare,
    Legendary,
}

impl ItemRarity {
    /// Get effect range for this rarity tier.
    pub fn effect_range(&self) -> (i32, i32) {
        match self {
            ItemRarity::Common => (1, 3),
            ItemRarity::Uncommon => (3, 5),
            ItemRarity::Rare => (5, 8),
            ItemRarity::Legendary => (8, 12),
        }
    }

    /// Get durability range for weapons of this rarity tier.
    pub fn weapon_durability_range(&self) -> (u32, u32) {
        match self {
            ItemRarity::Common => (2, 4),
            ItemRarity::Uncommon => (3, 6),
            ItemRarity::Rare => (5, 8),
            ItemRarity::Legendary => (7, 12),
        }
    }

    /// Get durability range for shields of this rarity tier.
    pub fn shield_durability_range(&self) -> (u32, u32) {
        match self {
            ItemRarity::Common => (3, 5),
            ItemRarity::Uncommon => (4, 7),
            ItemRarity::Rare => (6, 10),
            ItemRarity::Legendary => (9, 15),
        }
    }
}

/// Outcome of applying wear to an item.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WearOutcome {
    /// Item still has more than 50% durability remaining.
    Pristine,
    /// Item has crossed below 50% durability after this wear.
    Worn,
    /// Item durability has reached 0.
    Broken,
}

impl Display for ItemRarity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ItemRarity::Common => write!(f, "Common"),
            ItemRarity::Uncommon => write!(f, "Uncommon"),
            ItemRarity::Rare => write!(f, "Rare"),
            ItemRarity::Legendary => write!(f, "Legendary"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum ItemError {
    #[error("Item not found")]
    ItemNotFound,
    #[error("Item not usable")]
    ItemNotUsable,
    #[error("Item affects an invalid attribute")]
    InvalidAttribute,
}

pub trait OwnsItems {
    fn add_item(&mut self, item: Item);
    fn has_item(&self, item: &Item) -> bool;
    fn use_item(&mut self, item: &Item) -> Result<(), ItemError>;
    fn remove_item(&mut self, item: &Item) -> Result<(), ItemError>;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub identifier: String,
    pub name: String,
    pub item_type: ItemType,
    // Legacy `item` rows written before games.rs:1099's CONTENT-bind fix
    // omitted `rarity` entirely. Without a default, `Vec<Item>`
    // deserialization fails, which cascades up to make `/games/:id/areas`
    // (and other endpoints that fetch items) return an error and renders
    // as "Something went wrong" in the UI.
    #[serde(default = "default_rarity")]
    pub rarity: ItemRarity,
    pub current_durability: u32,
    pub max_durability: u32,
    pub attribute: Attribute,
    pub effect: i32,
}

fn default_rarity() -> ItemRarity {
    ItemRarity::Common
}

impl Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.rarity)
    }
}

impl Default for Item {
    fn default() -> Self {
        let identifier = Uuid::new_v4().to_string();
        Self {
            identifier,
            name: String::from("Useless health potion"),
            item_type: ItemType::Consumable,
            rarity: ItemRarity::Common,
            current_durability: 1,
            max_durability: 1,
            attribute: Attribute::Health,
            effect: 0,
        }
    }
}

impl Item {
    pub fn new(
        name: &str,
        item_type: ItemType,
        rarity: ItemRarity,
        max_durability: u32,
        attribute: Attribute,
        effect: i32,
    ) -> Item {
        let identifier = Uuid::new_v4().to_string();
        Item {
            identifier,
            name: name.to_string(),
            item_type,
            rarity,
            current_durability: max_durability,
            max_durability,
            attribute,
            effect,
        }
    }

    /// Apply wear to this item, reducing durability.
    ///
    /// Returns a [`WearOutcome`] describing the new state:
    /// - `Broken` if durability reached 0
    /// - `Worn` if this wear caused the item to cross below 50% durability
    /// - `Pristine` otherwise
    ///
    /// Uses saturating subtraction so durability cannot underflow.
    pub fn wear(&mut self, wear_amount: u32) -> WearOutcome {
        let was_above_half = self.current_durability * 2 > self.max_durability;
        self.current_durability = self.current_durability.saturating_sub(wear_amount);

        if self.current_durability == 0 {
            WearOutcome::Broken
        } else if was_above_half && self.current_durability * 2 <= self.max_durability {
            WearOutcome::Worn
        } else {
            WearOutcome::Pristine
        }
    }

    pub fn is_weapon(&self) -> bool {
        self.item_type == ItemType::Weapon && self.attribute == Attribute::Strength
    }

    pub fn is_defensive(&self) -> bool {
        self.item_type == ItemType::Weapon && self.attribute == Attribute::Defense
    }

    pub fn is_consumable(&self) -> bool {
        self.item_type == ItemType::Consumable
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ItemType {
    Consumable,
    Weapon,
    Food(u8),
    Water(u8),
}

impl ItemType {
    pub fn is_food(&self) -> bool {
        matches!(self, ItemType::Food(_))
    }

    pub fn is_water(&self) -> bool {
        matches!(self, ItemType::Water(_))
    }

    pub fn food_value(&self) -> Option<u8> {
        if let ItemType::Food(n) = self {
            Some(*n)
        } else {
            None
        }
    }

    pub fn water_value(&self) -> Option<u8> {
        if let ItemType::Water(n) = self {
            Some(*n)
        } else {
            None
        }
    }
}

impl Display for ItemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ItemType::Consumable => write!(f, "consumable"),
            ItemType::Weapon => write!(f, "weapon"),
            ItemType::Food(n) => write!(f, "food({})", n),
            ItemType::Water(n) => write!(f, "water({})", n),
        }
    }
}

impl FromStr for ItemType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower = s.to_lowercase();
        if let Some(inner) = lower
            .strip_prefix("food(")
            .and_then(|x| x.strip_suffix(')'))
        {
            return inner
                .parse::<u8>()
                .map(ItemType::Food)
                .map_err(|e| e.to_string());
        }
        if let Some(inner) = lower
            .strip_prefix("water(")
            .and_then(|x| x.strip_suffix(')'))
        {
            return inner
                .parse::<u8>()
                .map(ItemType::Water)
                .map_err(|e| e.to_string());
        }
        match lower.as_str() {
            "consumable" => Ok(ItemType::Consumable),
            "weapon" => Ok(ItemType::Weapon),
            _ => Err("Invalid item type".to_string()),
        }
    }
}

#[derive(Debug, Clone, Eq, EnumIter, PartialEq, Serialize, Deserialize)]
pub enum Attribute {
    Health,   // Heals health
    Sanity,   // Heals sanity
    Movement, // Increase movement
    Bravery,  // Increase bravery
    Speed,    // Increase speed
    Strength, // Increases damage done, i.e., weapon
    Defense,  // Reduces damage taken
}

pub trait ConsumableAttribute {
    fn consumable_name(&self) -> String;
}

impl ConsumableAttribute for Attribute {
    fn consumable_name(&self) -> String {
        match &self {
            // restore health
            Attribute::Health => "health kit".to_string(),
            // restore sanity
            Attribute::Sanity => "memento".to_string(),
            // move further
            Attribute::Movement => "trail mix".to_string(),
            // sure, you can win that fight
            Attribute::Bravery => "yayo".to_string(),
            // move faster
            Attribute::Speed => "go-juice".to_string(),
            // hit harder
            Attribute::Strength => "adrenaline".to_string(),
            // take hits better
            Attribute::Defense => "bear spray".to_string(),
        }
    }
}

impl Attribute {
    /// Map this attribute to a [`Substance`] if the item is addictive.
    pub fn substance(&self) -> Option<Substance> {
        match self {
            Attribute::Bravery | Attribute::Speed | Attribute::Strength => {
                Some(Substance::Stimulant)
            }
            _ => None,
        }
    }
}

impl Display for Attribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Attribute::Health => write!(f, "health"),
            Attribute::Sanity => write!(f, "sanity"),
            Attribute::Movement => write!(f, "movement"),
            Attribute::Bravery => write!(f, "bravery"),
            Attribute::Speed => write!(f, "speed"),
            Attribute::Strength => write!(f, "strength"),
            Attribute::Defense => write!(f, "defense"),
        }
    }
}

impl FromStr for Attribute {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "health" => Ok(Attribute::Health),
            "sanity" => Ok(Attribute::Sanity),
            "movement" => Ok(Attribute::Movement),
            "bravery" => Ok(Attribute::Bravery),
            "speed" => Ok(Attribute::Speed),
            "strength" => Ok(Attribute::Strength),
            "defense" => Ok(Attribute::Defense),
            _ => Err("Invalid attribute"),
        }
    }
}

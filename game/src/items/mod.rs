mod name_generator;

use crate::items::name_generator::{generate_shield_name, generate_weapon_name};
use crate::terrain::BaseTerrain;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;
use strum::{EnumIter, IntoEnumIterator};
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
    /// Roll for item rarity using weighted distribution.
    pub fn random() -> ItemRarity {
        let mut rng = SmallRng::from_rng(&mut rand::rng());
        let roll: f32 = rng.random();

        if roll < 0.60 {
            ItemRarity::Common
        } else if roll < 0.85 {
            ItemRarity::Uncommon
        } else if roll < 0.97 {
            ItemRarity::Rare
        } else {
            ItemRarity::Legendary
        }
    }

    /// Get effect range for this rarity tier.
    pub fn effect_range(&self) -> (i32, i32) {
        match self {
            ItemRarity::Common => (1, 3),
            ItemRarity::Uncommon => (3, 5),
            ItemRarity::Rare => (5, 8),
            ItemRarity::Legendary => (8, 12),
        }
    }
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
    pub rarity: ItemRarity,
    pub quantity: u32,
    pub attribute: Attribute,
    pub effect: i32,
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
            quantity: 1,
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
        quantity: u32,
        attribute: Attribute,
        effect: i32,
    ) -> Item {
        let identifier = Uuid::new_v4().to_string();
        Item {
            identifier,
            name: name.to_string(),
            item_type,
            rarity,
            quantity,
            attribute,
            effect,
        }
    }

    pub fn new_random(name: Option<&str>) -> Item {
        let mut rng = SmallRng::from_rng(&mut rand::rng());

        let item_type = ItemType::random();
        let is_shield = rng.random_bool(0.5);

        match (item_type, name) {
            (ItemType::Consumable, Some(name)) => Self::new_consumable(name),
            (ItemType::Consumable, None) => Self::new_random_consumable(),
            (ItemType::Weapon, Some(name)) => match is_shield {
                false => Self::new_weapon(name),
                true => Self::new_shield(name),
            },
            (ItemType::Weapon, None) => match is_shield {
                false => Self::new_random_weapon(),
                true => Self::new_random_shield(),
            },
        }
    }

    /// Create a random item using terrain-based weights for item type distribution.
    ///
    /// Uses the terrain's `item_weights()` to determine the probability of creating
    /// weapons, shields, or consumables, making item spawning terrain-appropriate.
    ///
    /// # Arguments
    /// * `terrain` - The terrain type that influences item distribution
    /// * `name` - Optional item name; generates one if None
    ///
    /// # Example
    /// ```
    /// use hangrier_games::items::Item;
    /// use hangrier_games::terrain::BaseTerrain;
    ///
    /// // Desert terrain favors consumables (0.6 weight)
    /// let item = Item::new_random_with_terrain(BaseTerrain::Desert, None);
    ///
    /// // Urban ruins favor weapons (0.5 weight)
    /// let item = Item::new_random_with_terrain(BaseTerrain::UrbanRuins, None);
    /// ```
    pub fn new_random_with_terrain(terrain: BaseTerrain, name: Option<&str>) -> Item {
        let mut rng = SmallRng::from_rng(&mut rand::rng());
        let weights = terrain.item_weights();

        // Use weighted random selection based on terrain
        let roll: f32 = rng.random();

        if roll < weights.weapons {
            // Generate weapon
            match name {
                Some(n) => Self::new_weapon(n),
                None => Self::new_random_weapon(),
            }
        } else if roll < weights.weapons + weights.shields {
            // Generate shield
            match name {
                Some(n) => Self::new_shield(n),
                None => Self::new_random_shield(),
            }
        } else {
            // Generate consumable
            match name {
                Some(n) => Self::new_consumable(n),
                None => Self::new_random_consumable(),
            }
        }
    }

    pub fn new_weapon(name: &str) -> Item {
        let mut rng = SmallRng::from_rng(&mut rand::rng());

        let rarity = ItemRarity::random();
        let quantity = 1;
        let attribute = Attribute::Strength;
        let (min, max) = rarity.effect_range();
        let effect = rng.random_range(min..=max);

        Item::new(name, ItemType::Weapon, rarity, quantity, attribute, effect)
    }

    pub fn new_random_weapon() -> Item {
        let name = generate_weapon_name();
        Item::new_weapon(name.as_str())
    }

    pub fn new_consumable(name: &str) -> Item {
        let mut rng = SmallRng::from_rng(&mut rand::rng());

        let rarity = ItemRarity::random();
        let quantity = 1;
        let attribute = Attribute::random();
        let (min, max) = rarity.effect_range();
        let effect = rng.random_range(min..=max);

        Item::new(name, ItemType::Consumable, rarity, quantity, attribute, effect)
    }

    pub fn new_random_consumable() -> Item {
        let mut rng = SmallRng::from_rng(&mut rand::rng());
        let rarity = ItemRarity::random();
        let attribute = Attribute::random();
        let name = attribute.consumable_name();
        let quantity = 1;
        let (min, max) = rarity.effect_range();
        let effect = rng.random_range(min..=max);

        Item::new(&name, ItemType::Consumable, rarity, quantity, attribute, effect)
    }

    pub fn new_shield(name: &str) -> Item {
        let mut rng = SmallRng::from_rng(&mut rand::rng());

        let rarity = ItemRarity::random();
        let item_type = ItemType::Weapon;
        let quantity = 1;
        let attribute = Attribute::Defense;
        let (min, max) = rarity.effect_range();
        let effect = rng.random_range(min..=max);

        Item::new(name, item_type, rarity, quantity, attribute, effect)
    }

    pub fn new_random_shield() -> Item {
        let name = generate_shield_name();
        Item::new_shield(name.as_str())
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
}

impl ItemType {
    pub fn random() -> ItemType {
        let mut rng = SmallRng::from_rng(&mut rand::rng());
        match rng.random_bool(0.5) {
            true => ItemType::Consumable,
            false => ItemType::Weapon,
        }
    }
}

impl Display for ItemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ItemType::Consumable => write!(f, "consumable"),
            ItemType::Weapon => write!(f, "weapon"),
        }
    }
}

impl FromStr for ItemType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
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

impl Attribute {
    pub fn random() -> Attribute {
        let mut rng = SmallRng::from_rng(&mut rand::rng());
        Attribute::iter().choose(&mut rng).unwrap()
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[test]
    fn default_item() {
        let item = Item::default();
        assert_eq!(item.name, "Useless health potion".to_string());
    }

    #[test]
    fn item_to_string() {
        let item = Item::default();
        assert_eq!(item.to_string(), "Useless health potion (Common)".to_string());
    }

    #[test]
    fn new_item() {
        let item = Item::new("Test item", ItemType::Weapon, ItemRarity::Rare, 1, Attribute::Defense, 10);
        assert_eq!(item.name, "Test item");
        assert_eq!(item.item_type, ItemType::Weapon);
        assert_eq!(item.rarity, ItemRarity::Rare);
        assert_eq!(item.quantity, 1);
        assert_eq!(item.attribute, Attribute::Defense);
        assert_eq!(item.effect, 10);
    }

    #[test]
    fn new_random_item() {
        let item = Item::new_random(Some("Test item"));
        assert_eq!(item.name, "Test item");
        assert!((1..=3).contains(&item.quantity));
    }

    #[test]
    fn new_random_item_no_name() {
        let item = Item::new_random(None);
        assert!(!item.name.is_empty());
        assert!((1..=3).contains(&item.quantity));
    }

    #[test]
    fn new_weapon() {
        let weapon = Item::new_weapon("Test weapon");
        assert_eq!(weapon.item_type, ItemType::Weapon);
        assert_eq!(weapon.attribute, Attribute::Strength);
        assert!(weapon.is_weapon());
    }

    #[test]
    fn new_random_weapon() {
        let weapon = Item::new_random_weapon();
        assert_eq!(weapon.item_type, ItemType::Weapon);
        assert!(!weapon.name.is_empty());
        assert!(weapon.is_weapon());
    }

    #[test]
    fn new_consumable() {
        let consumable = Item::new_consumable("Test item");
        assert_eq!(consumable.item_type, ItemType::Consumable);
        assert!(consumable.is_consumable());
    }

    #[test]
    fn new_random_consumable() {
        let consumable = Item::new_random_consumable();
        assert_eq!(consumable.item_type, ItemType::Consumable);
        assert!(!consumable.name.is_empty());
        assert!(consumable.is_consumable());
    }

    #[test]
    fn new_shield() {
        let shield = Item::new_shield("Test shield");
        assert_eq!(shield.item_type, ItemType::Weapon);
        assert_eq!(shield.attribute, Attribute::Defense);
        assert!(shield.is_defensive());
    }

    #[test]
    fn new_random_shield() {
        let shield = Item::new_random_shield();
        assert_eq!(shield.item_type, ItemType::Weapon);
        assert!(!shield.name.is_empty());
        assert!(shield.is_defensive());
    }

    #[rstest]
    #[case(ItemType::Consumable, "consumable")]
    #[case(ItemType::Weapon, "weapon")]
    fn item_type_to_string(#[case] item_type: ItemType, #[case] expected: &str) {
        assert_eq!(item_type.to_string(), expected);
    }

    #[rstest]
    #[case("consumable", ItemType::Consumable)]
    #[case("weapon", ItemType::Weapon)]
    fn item_type_from_str(#[case] input: &str, #[case] item_type: ItemType) {
        assert_eq!(ItemType::from_str(input).unwrap(), item_type);
    }

    #[test]
    fn item_type_from_str_invalid() {
        assert!(ItemType::from_str("nuclear").is_err());
    }

    #[test]
    fn random_item_type() {
        let item_type = ItemType::random();
        assert!([ItemType::Weapon, ItemType::Consumable].contains(&item_type));
    }

    #[test]
    fn random_attribute() {
        let attribute = Attribute::random();
        assert!(
            Attribute::iter()
                .find(|a| *a == attribute.clone())
                .is_some()
        );
    }

    #[rstest]
    #[case(Attribute::Health, "health")]
    #[case(Attribute::Sanity, "sanity")]
    #[case(Attribute::Movement, "movement")]
    #[case(Attribute::Bravery, "bravery")]
    #[case(Attribute::Speed, "speed")]
    #[case(Attribute::Strength, "strength")]
    #[case(Attribute::Defense, "defense")]
    fn attribute_to_string(#[case] attribute: Attribute, #[case] expected: String) {
        assert_eq!(attribute.to_string(), expected);
    }

    #[rstest]
    #[case("health", Attribute::Health)]
    #[case("sanity", Attribute::Sanity)]
    #[case("movement", Attribute::Movement)]
    #[case("bravery", Attribute::Bravery)]
    #[case("speed", Attribute::Speed)]
    #[case("strength", Attribute::Strength)]
    #[case("defense", Attribute::Defense)]
    fn attribute_from_str(#[case] input: &str, #[case] attribute: Attribute) {
        assert_eq!(Attribute::from_str(input).unwrap(), attribute);
    }

    #[test]
    fn attribute_from_str_invalid() {
        assert!(Attribute::from_str("mana").is_err());
    }

    #[rstest]
    #[case(Attribute::Health, "health kit")]
    #[case(Attribute::Sanity, "memento")]
    #[case(Attribute::Movement, "trail mix")]
    #[case(Attribute::Bravery, "yayo")]
    #[case(Attribute::Speed, "go-juice")]
    #[case(Attribute::Strength, "adrenaline")]
    #[case(Attribute::Defense, "bear spray")]
    fn attribute_to_consumable_name(#[case] attribute: Attribute, #[case] expected: String) {
        assert_eq!(attribute.consumable_name(), expected);
    }

    #[test]
    fn rarity_random_distribution() {
        // Test that random() returns valid rarities
        for _ in 0..100 {
            let rarity = ItemRarity::random();
            assert!([
                ItemRarity::Common,
                ItemRarity::Uncommon,
                ItemRarity::Rare,
                ItemRarity::Legendary
            ]
            .contains(&rarity));
        }
    }

    #[rstest]
    #[case(ItemRarity::Common, (1, 3))]
    #[case(ItemRarity::Uncommon, (3, 5))]
    #[case(ItemRarity::Rare, (5, 8))]
    #[case(ItemRarity::Legendary, (8, 12))]
    fn rarity_effect_ranges(#[case] rarity: ItemRarity, #[case] expected: (i32, i32)) {
        assert_eq!(rarity.effect_range(), expected);
    }

    #[rstest]
    #[case(ItemRarity::Common, "Common")]
    #[case(ItemRarity::Uncommon, "Uncommon")]
    #[case(ItemRarity::Rare, "Rare")]
    #[case(ItemRarity::Legendary, "Legendary")]
    fn rarity_to_string(#[case] rarity: ItemRarity, #[case] expected: &str) {
        assert_eq!(rarity.to_string(), expected);
    }

    #[test]
    fn weapon_has_rarity() {
        let weapon = Item::new_weapon("Test weapon");
        // Verify rarity is set
        assert!([
            ItemRarity::Common,
            ItemRarity::Uncommon,
            ItemRarity::Rare,
            ItemRarity::Legendary
        ]
        .contains(&weapon.rarity));
    }

    #[test]
    fn weapon_effect_matches_rarity() {
        for _ in 0..100 {
            let weapon = Item::new_random_weapon();
            let (min, max) = weapon.rarity.effect_range();
            assert!(
                weapon.effect >= min && weapon.effect <= max,
                "Effect {} not in range {}..={}",
                weapon.effect,
                min,
                max
            );
        }
    }

    #[test]
    fn consumable_effect_matches_rarity() {
        for _ in 0..100 {
            let consumable = Item::new_random_consumable();
            let (min, max) = consumable.rarity.effect_range();
            assert!(
                consumable.effect >= min && consumable.effect <= max,
                "Effect {} not in range {}..={}",
                consumable.effect,
                min,
                max
            );
        }
    }

    #[test]
    fn shield_effect_matches_rarity() {
        for _ in 0..100 {
            let shield = Item::new_random_shield();
            let (min, max) = shield.rarity.effect_range();
            assert!(
                shield.effect >= min && shield.effect <= max,
                "Effect {} not in range {}..={}",
                shield.effect,
                min,
                max
            );
        }
    }
}

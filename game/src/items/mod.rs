mod name_generator;

use crate::items::name_generator::{generate_shield_name, generate_weapon_name};
use crate::terrain::BaseTerrain;
use rand::RngExt;
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
            (ItemType::Food(n), name) => Self::new_food(name, n),
            (ItemType::Water(n), name) => Self::new_water(name, n),
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
    /// use game::items::Item;
    /// use game::terrain::BaseTerrain;
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
        let attribute = Attribute::Strength;
        let (min, max) = rarity.effect_range();
        let effect = rng.random_range(min..=max);
        let (dur_min, dur_max) = rarity.weapon_durability_range();
        let durability = rng.random_range(dur_min..=dur_max);

        Item::new(
            name,
            ItemType::Weapon,
            rarity,
            durability,
            attribute,
            effect,
        )
    }

    pub fn new_random_weapon() -> Item {
        let name = generate_weapon_name();
        Item::new_weapon(name.as_str())
    }

    pub fn new_consumable(name: &str) -> Item {
        let mut rng = SmallRng::from_rng(&mut rand::rng());

        let rarity = ItemRarity::random();
        let attribute = Attribute::random();
        let (min, max) = rarity.effect_range();
        let effect = rng.random_range(min..=max);

        Item::new(name, ItemType::Consumable, rarity, 1, attribute, effect)
    }

    pub fn new_random_consumable() -> Item {
        let mut rng = SmallRng::from_rng(&mut rand::rng());
        let rarity = ItemRarity::random();
        let attribute = Attribute::random();
        let name = attribute.consumable_name();
        let (min, max) = rarity.effect_range();
        let effect = rng.random_range(min..=max);

        Item::new(&name, ItemType::Consumable, rarity, 1, attribute, effect)
    }

    pub fn new_shield(name: &str) -> Item {
        let mut rng = SmallRng::from_rng(&mut rand::rng());

        let rarity = ItemRarity::random();
        let item_type = ItemType::Weapon;
        let attribute = Attribute::Defense;
        let (min, max) = rarity.effect_range();
        let effect = rng.random_range(min..=max);
        let (dur_min, dur_max) = rarity.shield_durability_range();
        let durability = rng.random_range(dur_min..=dur_max);

        Item::new(name, item_type, rarity, durability, attribute, effect)
    }

    pub fn new_random_shield() -> Item {
        let name = generate_shield_name();
        Item::new_shield(name.as_str())
    }

    /// Construct a Food item carrying `value` hunger-debt relief. `name` is
    /// optional; if absent, a generic "ration" name is generated.
    pub fn new_food(name: Option<&str>, value: u8) -> Item {
        let display = name.map(|s| s.to_string()).unwrap_or_else(|| {
            // Tiny deterministic label pool keeps generation cheap and the
            // payload self-describing without dragging in the full name
            // generator just yet.
            format!("ration ({})", value)
        });
        Item::new(
            &display,
            ItemType::Food(value),
            ItemRarity::Common,
            1,
            Attribute::Health,
            value as i32,
        )
    }

    /// Construct a Water item carrying `value` thirst-debt relief.
    pub fn new_water(name: Option<&str>, value: u8) -> Item {
        let display = name
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("waterskin ({})", value));
        Item::new(
            &display,
            ItemType::Water(value),
            ItemRarity::Common,
            1,
            Attribute::Health,
            value as i32,
        )
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
    pub fn random() -> ItemType {
        let mut rng = SmallRng::from_rng(&mut rand::rng());
        // Weighted distribution: weapons and consumables remain dominant;
        // food and water enter the spawn pool but are rarer (spec).
        match rng.random_range(0..10) {
            0..=3 => ItemType::Consumable,
            4..=6 => ItemType::Weapon,
            7..=8 => ItemType::Food(rng.random_range(1..=5)),
            _ => ItemType::Water(rng.random_range(1..=3)),
        }
    }

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
    fn item_type_food_serializes_round_trip() {
        let it = ItemType::Food(3);
        let json = serde_json::to_string(&it).unwrap();
        let back: ItemType = serde_json::from_str(&json).unwrap();
        assert_eq!(it, back);
    }

    #[test]
    fn item_type_water_serializes_round_trip() {
        let it = ItemType::Water(2);
        let json = serde_json::to_string(&it).unwrap();
        let back: ItemType = serde_json::from_str(&json).unwrap();
        assert_eq!(it, back);
    }

    #[test]
    fn item_type_legacy_consumable_string_still_loads() {
        let back: ItemType = serde_json::from_str("\"Consumable\"").unwrap();
        assert_eq!(back, ItemType::Consumable);
    }

    #[test]
    fn item_type_food_helpers() {
        assert!(ItemType::Food(3).is_food());
        assert_eq!(ItemType::Food(3).food_value(), Some(3));
        assert_eq!(ItemType::Water(2).water_value(), Some(2));
        assert!(!ItemType::Weapon.is_food());
        assert_eq!(ItemType::Consumable.food_value(), None);
    }

    #[test]
    fn item_type_display_and_fromstr_round_trip() {
        for v in [ItemType::Food(5), ItemType::Water(2)] {
            let s = v.to_string();
            let back: ItemType = s.parse().unwrap();
            assert_eq!(v, back);
        }
    }

    #[test]
    fn default_item() {
        let item = Item::default();
        assert_eq!(item.name, "Useless health potion".to_string());
    }

    #[test]
    fn item_to_string() {
        let item = Item::default();
        assert_eq!(
            item.to_string(),
            "Useless health potion (Common)".to_string()
        );
    }

    #[test]
    fn new_item() {
        let item = Item::new(
            "Test item",
            ItemType::Weapon,
            ItemRarity::Rare,
            1,
            Attribute::Defense,
            10,
        );
        assert_eq!(item.name, "Test item");
        assert_eq!(item.item_type, ItemType::Weapon);
        assert_eq!(item.rarity, ItemRarity::Rare);
        assert_eq!(item.current_durability, 1);
        assert_eq!(item.max_durability, 1);
        assert_eq!(item.attribute, Attribute::Defense);
        assert_eq!(item.effect, 10);
    }

    #[test]
    fn new_random_item() {
        let item = Item::new_random(Some("Test item"));
        assert_eq!(item.name, "Test item");
        assert!(item.current_durability >= 1);
        assert_eq!(item.current_durability, item.max_durability);
    }

    #[test]
    fn new_random_item_no_name() {
        let item = Item::new_random(None);
        assert!(!item.name.is_empty());
        assert!(item.current_durability >= 1);
        assert_eq!(item.current_durability, item.max_durability);
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
        assert!(matches!(
            item_type,
            ItemType::Weapon | ItemType::Consumable | ItemType::Food(_) | ItemType::Water(_)
        ));
    }

    #[test]
    fn random_attribute() {
        let attribute = Attribute::random();
        assert!(Attribute::iter().any(|a| a == attribute.clone()));
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
            assert!(
                [
                    ItemRarity::Common,
                    ItemRarity::Uncommon,
                    ItemRarity::Rare,
                    ItemRarity::Legendary
                ]
                .contains(&rarity)
            );
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
        assert!(
            [
                ItemRarity::Common,
                ItemRarity::Uncommon,
                ItemRarity::Rare,
                ItemRarity::Legendary
            ]
            .contains(&weapon.rarity)
        );
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

    #[rstest]
    #[case(ItemRarity::Common, (2, 4))]
    #[case(ItemRarity::Uncommon, (3, 6))]
    #[case(ItemRarity::Rare, (5, 8))]
    #[case(ItemRarity::Legendary, (7, 12))]
    fn weapon_durability_ranges(#[case] rarity: ItemRarity, #[case] expected: (u32, u32)) {
        assert_eq!(rarity.weapon_durability_range(), expected);
    }

    #[rstest]
    #[case(ItemRarity::Common, (3, 5))]
    #[case(ItemRarity::Uncommon, (4, 7))]
    #[case(ItemRarity::Rare, (6, 10))]
    #[case(ItemRarity::Legendary, (9, 15))]
    fn shield_durability_ranges(#[case] rarity: ItemRarity, #[case] expected: (u32, u32)) {
        assert_eq!(rarity.shield_durability_range(), expected);
    }

    #[test]
    fn weapon_durability_within_rarity_range() {
        for _ in 0..100 {
            let weapon = Item::new_random_weapon();
            let (min, max) = weapon.rarity.weapon_durability_range();
            assert!(
                weapon.max_durability >= min && weapon.max_durability <= max,
                "Durability {} not in range {}..={}",
                weapon.max_durability,
                min,
                max
            );
            assert_eq!(weapon.current_durability, weapon.max_durability);
        }
    }

    #[test]
    fn shield_durability_within_rarity_range() {
        for _ in 0..100 {
            let shield = Item::new_random_shield();
            let (min, max) = shield.rarity.shield_durability_range();
            assert!(
                shield.max_durability >= min && shield.max_durability <= max,
                "Durability {} not in range {}..={}",
                shield.max_durability,
                min,
                max
            );
            assert_eq!(shield.current_durability, shield.max_durability);
        }
    }

    #[test]
    fn consumable_has_one_use() {
        let consumable = Item::new_random_consumable();
        assert_eq!(consumable.max_durability, 1);
        assert_eq!(consumable.current_durability, 1);
    }

    #[test]
    fn wear_pristine_when_above_half() {
        let mut item = Item::new(
            "Test",
            ItemType::Weapon,
            ItemRarity::Rare,
            10,
            Attribute::Strength,
            5,
        );
        let outcome = item.wear(1);
        assert_eq!(outcome, WearOutcome::Pristine);
        assert_eq!(item.current_durability, 9);
    }

    #[test]
    fn wear_worn_when_crossing_half() {
        let mut item = Item::new(
            "Test",
            ItemType::Weapon,
            ItemRarity::Rare,
            10,
            Attribute::Strength,
            5,
        );
        // 10 -> 6 (still > 5, pristine)
        assert_eq!(item.wear(4), WearOutcome::Pristine);
        // 6 -> 5 (now <= half, worn)
        assert_eq!(item.wear(1), WearOutcome::Worn);
    }

    #[test]
    fn wear_worn_only_on_first_crossing() {
        let mut item = Item::new(
            "Test",
            ItemType::Weapon,
            ItemRarity::Rare,
            10,
            Attribute::Strength,
            5,
        );
        assert_eq!(item.wear(5), WearOutcome::Worn); // 10 -> 5
        assert_eq!(item.wear(1), WearOutcome::Pristine); // 5 -> 4, already worn
    }

    #[test]
    fn wear_broken_at_zero() {
        let mut item = Item::new(
            "Test",
            ItemType::Weapon,
            ItemRarity::Common,
            2,
            Attribute::Strength,
            5,
        );
        assert_eq!(item.wear(1), WearOutcome::Worn); // 2 -> 1
        assert_eq!(item.wear(1), WearOutcome::Broken); // 1 -> 0
        assert_eq!(item.current_durability, 0);
    }

    #[test]
    fn wear_saturating_subtraction() {
        let mut item = Item::new(
            "Test",
            ItemType::Weapon,
            ItemRarity::Common,
            3,
            Attribute::Strength,
            5,
        );
        let outcome = item.wear(100);
        assert_eq!(outcome, WearOutcome::Broken);
        assert_eq!(item.current_durability, 0);
    }

    /// Legacy `item` rows persisted before the games.rs:1099 CONTENT-bind
    /// fix omitted `rarity` entirely. Without `#[serde(default)]`, those
    /// rows fail to deserialize and break every endpoint that fetches
    /// items (e.g. /games/:id/areas → "Something went wrong").
    #[test]
    fn item_deserializes_when_rarity_field_missing() {
        let json = r#"{
            "identifier": "x",
            "name": "Old item",
            "item_type": "Weapon",
            "current_durability": 1,
            "max_durability": 1,
            "attribute": "Strength",
            "effect": 1
        }"#;
        let item: Item = serde_json::from_str(json).expect("legacy item must deserialize");
        assert_eq!(item.rarity, ItemRarity::Common);
    }
}

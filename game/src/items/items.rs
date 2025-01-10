use crate::items::name_generator::{generate_shield_name, generate_weapon_name};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;
use strum::{EnumIter, IntoEnumIterator};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub name: String,
    pub item_type: ItemType,
    pub quantity: u32,
    pub attribute: Attribute,
    pub effect: i32,
}

impl Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Default for Item {
    fn default() -> Self {
        Self {
            name: String::from("Useless health potion"),
            item_type: ItemType::Consumable,
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
        quantity: u32,
        attribute: Attribute,
        effect: i32,
    ) -> Item {
        Item {
            name: name.to_string(),
            item_type,
            quantity,
            attribute,
            effect,
        }
    }

    pub fn new_random(name: &str) -> Item {
        let mut rng = rand::thread_rng();

        let item_type = ItemType::random();
        let quantity = rng.gen_range(1..=3);
        let attribute = Attribute::random();
        let effect = rng.gen_range(1..=10);

        Item::new(name, item_type, quantity, attribute.unwrap(), effect)
    }

    pub fn new_weapon(name: &str) -> Item {
        let mut rng = rand::thread_rng();

        let quantity = rng.gen_range(1..=2);
        let attribute = Attribute::Strength;
        let effect = rng.gen_range(1..=5);

        Item::new(name, ItemType::Weapon, quantity, attribute, effect)
    }

    pub fn new_random_weapon() -> Item {
        let name = generate_weapon_name();
        Item::new_weapon(name.as_str())
    }

    pub fn new_consumable(name: &str) -> Item {
        let mut rng = rand::thread_rng();

        let quantity = 1;
        let attribute = Attribute::random();
        let effect = rng.gen_range(1..=10);

        Item::new(
            name,
            ItemType::Consumable,
            quantity,
            attribute.unwrap(),
            effect,
        )
    }

    pub fn new_random_consumable() -> Item {
        let mut item = Item::new_consumable("NONE");
        item.name = item.attribute.consumable_name();
        item
    }

    pub fn new_shield(name: &str) -> Item {
        let mut rng = rand::thread_rng();

        let item_type = ItemType::Weapon;
        let quantity = rng.gen_range(1..=3);
        let attribute = Attribute::Defense;
        let effect = rng.gen_range(1..=7);

        Item::new(name, item_type, quantity, attribute, effect)
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
        let mut rng = rand::thread_rng();
        match rng.gen_bool(0.5) {
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
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "consumable" => Ok(ItemType::Consumable),
            "weapon" => Ok(ItemType::Weapon),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Eq, EnumIter, PartialEq, Serialize, Deserialize)]
pub enum Attribute {
    Health,   // Heals health
    Sanity,   // Heals sanity
    Movement, // Increases movement
    Bravery,  // Increases bravery
    Speed,    // Increases speed
    Strength, // Increases damage done, i.e. weapon
    Defense,  // Reduces damage taken
}

impl Attribute {
    pub fn random() -> Option<Attribute> {
        let mut rng = rand::thread_rng();
        Attribute::iter().choose(&mut rng)
    }
}

pub trait ConsumableAttribute {
    fn consumable_name(&self) -> String;
}

impl ConsumableAttribute for Attribute {
    fn consumable_name(&self) -> String {
        match &self {
            // restores health
            Attribute::Health => { "health kit".to_string() }
            // restores sanity
            Attribute::Sanity => { "memento".to_string() }
            // move further
            Attribute::Movement => { "trail mix".to_string() }
            // sure, you can win that fight
            Attribute::Bravery => { "yayo".to_string() }
            // move faster
            Attribute::Speed => { "go-juice".to_string() }
            // hit harder
            Attribute::Strength => { "adrenaline".to_string() }
            // take hits better
            Attribute::Defense => { "bear spray".to_string() }
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
        assert_eq!(item.to_string(), "Useless health potion".to_string());
    }

    #[test]
    fn new_item() {
        let item = Item::new(
            "Test item",
            ItemType::Weapon,
            1,
            Attribute::Defense,
            10,
        );
        assert_eq!(item.name, "Test item");
        assert_eq!(item.item_type, ItemType::Weapon);
        assert_eq!(item.quantity, 1);
        assert_eq!(item.attribute, Attribute::Defense);
        assert_eq!(item.effect, 10);
    }

    #[test]
    fn new_random_item() {
        let item = Item::new_random("Test item");
        assert_eq!(item.name, "Test item");
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
        assert!(attribute.is_some());
        assert!(Attribute::iter().find(|a| *a == attribute.clone().unwrap()).is_some());
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
}

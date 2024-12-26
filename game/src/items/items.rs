use crate::items::name_generator::{generate_shield_name, generate_weapon_name};
use rand::Rng;
use rand::prelude::IteratorRandom;
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

    pub fn new_generic_consumable() -> Item {
        let mut item = Item::new_consumable("NONE");
        match item.attribute {
            Attribute::Health => {
                // restores health
                item.name = "health kit".to_string();
            }
            Attribute::Sanity => {
                // restores sanity
                item.name = "memento".to_string();
            }
            Attribute::Movement => {
                // move further
                item.name = "trail mix".to_string();
            }
            Attribute::Bravery => {
                // sure, you can win that fight
                item.name = "yayo".to_string();
            }
            Attribute::Speed => {
                // move faster
                item.name = "go-juice".to_string();
            }
            Attribute::Strength => {
                // hit harder
                item.name = "adrenaline".to_string();
            }
            Attribute::Defense => {
                // take hits better
                item.name = "bear spray".to_string();
            }
        }
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
            && self.attribute != Attribute::Strength
            && self.attribute != Attribute::Defense
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

    pub fn from_int(i: i32) -> ItemType {
        match i {
            0 => ItemType::Consumable,
            1 => ItemType::Weapon,
            _ => panic!("Invalid item type"),
        }
    }

    pub fn to_int(&self) -> i32 {
        match self {
            ItemType::Consumable => 0,
            ItemType::Weapon => 1,
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
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "consumable" => Ok(ItemType::Consumable),
            "weapon" => Ok(ItemType::Weapon),
            _ => Err("Invalid item type"),
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

impl Display for Attribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Attribute::Health => write!(f, "Health"),
            Attribute::Sanity => write!(f, "Sanity"),
            Attribute::Movement => write!(f, "Movement"),
            Attribute::Bravery => write!(f, "Bravery"),
            Attribute::Speed => write!(f, "Speed"),
            Attribute::Strength => write!(f, "Strength"),
            Attribute::Defense => write!(f, "Defense"),
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

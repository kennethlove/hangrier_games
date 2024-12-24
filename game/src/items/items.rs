use rand::Rng;
use std::fmt::Display;
use std::str::FromStr;
use crate::areas::Area;
use crate::games::Game;
use crate::items::name_generator::{generate_shield_name, generate_weapon_name};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub id: Option<i32>,
    pub name: String,
    pub item_type: ItemType,
    pub game_id: Option<i32>,
    pub area_id: Option<i32>,
    pub tribute_id: Option<i32>,
    pub quantity: i32,
    pub attribute: Attribute,
    pub effect: i32,
}

impl Item {
    pub fn area(&self) -> Area {
        Area::from(get_area_by_id(self.area_id).unwrap())
    }

    pub fn game(&self) -> Game {
        Game::from(get_game_by_id(self.game_id.unwrap()).unwrap())
    }

    pub fn get_item_by_name(name: &str) -> Option<Item> {
        Some(Item::from(ItemModel::get_by_name(name.to_string())))
    }

    pub fn create(
        name: String,
        item_type: String,
        quantity: i32,
        attribute: String,
        effect: i32,
        game_id: Option<i32>,
        area_id: Option<i32>,
        tribute_id: Option<i32>
    ) -> Item {
        let new_item = NewItem {
            name,
            item_type,
            game_id,
            area_id,
            tribute_id,
            quantity,
            attribute,
            effect,
        };
        let item = create_item(new_item);
        Item::from(item)
    }

    pub fn save(&self) {
        let instance = UpdateItem::from(self.clone());
        update_item(instance);
    }

    pub fn delete(&self) {
        ItemModel::delete(self.id.unwrap());
    }

    pub fn new_random(name: String, game_id: Option<i32>, area_id: Option<i32>, tribute_id: Option<i32>) -> Item {
        let mut rng = rand::thread_rng();

        let item_type = ItemType::random();
        let quantity = rng.gen_range(1..=3);
        let attribute = Attribute::random();
        let effect = rng.gen_range(1..=10);

        Item::create(name, item_type.to_string(), quantity, attribute.to_string(), effect, game_id, area_id, tribute_id)
    }

    pub fn new_weapon(name: String, game_id: Option<i32>, area_id: Option<i32>, tribute_id: Option<i32>) -> Item {
        let mut rng = rand::thread_rng();

        let item_type = ItemType::Weapon;
        let quantity = rng.gen_range(1..=2);
        let attribute = Attribute::Strength;
        let effect = rng.gen_range(1..=5);

        Item::create(name, item_type.to_string(), quantity, attribute.to_string(), effect, game_id, area_id, tribute_id)
    }

    pub fn new_random_weapon(game_id: Option<i32>, area_id: Option<i32>, tribute_id: Option<i32>) -> Item {
        let name = generate_weapon_name();
        Item::new_weapon(name, game_id, area_id, tribute_id)
    }

    pub fn new_consumable(name: String, game_id: Option<i32>, area_id: Option<i32>, tribute_id: Option<i32>) -> Item {
        let mut rng = rand::thread_rng();

        let item_type = ItemType::Consumable;
        let quantity = 1;
        let attribute = Attribute::random();
        let effect = rng.gen_range(1..=10);

        Item::create(name, item_type.to_string(), quantity, attribute.to_string(), effect, game_id, area_id, tribute_id)
    }

    pub fn new_generic_consumable(game_id: Option<i32>, area_id: Option<i32>, tribute_id: Option<i32>) -> Item {
        let mut item = Item::new_consumable("NONE".to_string(), game_id, area_id, tribute_id);
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
        item.save();
        item
    }

    pub fn new_shield(name: String, game_id: Option<i32>, area_id: Option<i32>, tribute_id: Option<i32>) -> Item {
        let mut rng = rand::thread_rng();

        let item_type = ItemType::Weapon;
        let quantity = rng.gen_range(1..=3);
        let attribute = Attribute::Defense;
        let effect = rng.gen_range(1..=7);

        Item::create(name, item_type.to_string(), quantity, attribute.to_string(), effect, game_id, area_id, tribute_id)
    }

    pub fn new_random_shield(game_id: Option<i32>, area_id: Option<i32>, tribute_id: Option<i32>) -> Item {
        let name = generate_shield_name();
        Item::new_shield(name, game_id, area_id, tribute_id)
    }

    pub fn is_weapon(&self) -> bool {
        self.item_type == ItemType::Weapon && self.attribute == Attribute::Strength
    }

    pub fn is_defensive(&self) -> bool {
        self.item_type == ItemType::Weapon && self.attribute == Attribute::Defense
    }

    pub fn is_consumable(&self) -> bool {
        self.item_type == ItemType::Consumable &&
            self.attribute != Attribute::Strength &&
            self.attribute != Attribute::Defense
    }
}

impl From<ItemModel> for Item {
    fn from(item: ItemModel) -> Self {
        Item {
            id: Some(item.id),
            name: item.name,
            item_type: ItemType::from_str(item.item_type.as_str()).unwrap(),
            game_id: item.game_id,
            area_id: item.area_id,
            tribute_id: item.tribute_id,
            quantity: item.quantity,
            attribute: Attribute::from_str(item.attribute.as_str()).unwrap(),
            effect: item.effect,
        }
    }
}

impl Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl FromStr for Item {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let item = ItemModel::get_by_name(s.to_string());
        Ok(Item::from(item))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
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
            ItemType::Consumable => write!(f, "Consumable"),
            ItemType::Weapon => write!(f, "Weapon"),
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Attribute {
    Health, // Heals health
    Sanity, // Heals sanity
    Movement, // Increases movement
    Bravery, // Increases bravery
    Speed, // Increases speed
    Strength, // Increases damage done, i.e. weapon
    Defense, // Reduces damage taken
}

impl Attribute {
    pub fn random() -> Attribute {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..7) {
            0 => Attribute::Health,
            1 => Attribute::Sanity,
            2 => Attribute::Movement,
            3 => Attribute::Bravery,
            4 => Attribute::Speed,
            5 => Attribute::Strength,
            6 => Attribute::Defense,
            _ => panic!("Invalid attribute"),
        }
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

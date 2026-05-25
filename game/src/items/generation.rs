use super::*;
use crate::items::name_generator::{generate_shield_name, generate_weapon_name};
use crate::terrain::BaseTerrain;
use rand::RngExt;
use rand::prelude::*;
use strum::IntoEnumIterator;

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
}

impl Attribute {
    pub fn random() -> Attribute {
        let mut rng = SmallRng::from_rng(&mut rand::rng());
        Attribute::iter().choose(&mut rng).unwrap()
    }
}

impl Item {
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
}

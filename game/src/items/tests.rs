use super::*;
use rstest::*;
use strum::IntoEnumIterator;

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
/// items (e.g. /games/:id/areas -> "Something went wrong").
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

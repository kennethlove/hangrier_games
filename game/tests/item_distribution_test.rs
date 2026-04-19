use game::items::Item;
use game::terrain::BaseTerrain;

/// Test that Desert terrain creates consumables more frequently than other types.
/// Desert has 0.6 consumable weight vs 0.2 weapons + 0.2 shields.
#[test]
fn test_desert_favors_consumables() {
    let terrain = BaseTerrain::Desert;
    let mut consumable_count = 0;
    let mut weapon_count = 0;
    let mut shield_count = 0;

    // Generate 100 items and count distribution
    for _ in 0..100 {
        let item = Item::new_random_with_terrain(terrain, None);
        if item.is_weapon() {
            weapon_count += 1;
        } else if item.is_defensive() {
            shield_count += 1;
        } else {
            consumable_count += 1;
        }
    }

    // Consumables should be the majority (expect ~60% = 60 items)
    // Allow variance but should be clearly dominant
    assert!(
        consumable_count > 40,
        "Desert should favor consumables: got {} consumables, {} weapons, {} shields",
        consumable_count,
        weapon_count,
        shield_count
    );
    assert!(
        consumable_count > weapon_count + shield_count,
        "Desert should have more consumables than weapons+shields combined"
    );
}

/// Test that UrbanRuins terrain creates weapons more frequently.
/// UrbanRuins has 0.5 weapon weight vs 0.3 shields + 0.2 consumables.
#[test]
fn test_urban_ruins_favors_weapons() {
    let terrain = BaseTerrain::UrbanRuins;
    let mut weapon_count = 0;
    let mut shield_count = 0;
    let mut consumable_count = 0;

    for _ in 0..100 {
        let item = Item::new_random_with_terrain(terrain, None);
        if item.is_weapon() {
            weapon_count += 1;
        } else if item.is_defensive() {
            shield_count += 1;
        } else {
            consumable_count += 1;
        }
    }

    // Weapons should be the plurality (expect ~50% = 50 items)
    assert!(
        weapon_count > 30,
        "UrbanRuins should favor weapons: got {} weapons, {} shields, {} consumables",
        weapon_count,
        shield_count,
        consumable_count
    );
    assert!(
        weapon_count > shield_count && weapon_count > consumable_count,
        "UrbanRuins should have more weapons than any other type"
    );
}

/// Test that Clearing terrain has balanced distribution.
/// Clearing has ~0.33 for each type (0.33, 0.33, 0.34).
#[test]
fn test_clearing_balanced_distribution() {
    let terrain = BaseTerrain::Clearing;
    let mut weapon_count = 0;
    let mut shield_count = 0;
    let mut consumable_count = 0;

    for _ in 0..150 {
        let item = Item::new_random_with_terrain(terrain, None);
        if item.is_weapon() {
            weapon_count += 1;
        } else if item.is_defensive() {
            shield_count += 1;
        } else {
            consumable_count += 1;
        }
    }

    // All three should be roughly equal (expect ~50 each out of 150)
    // None should be dominant
    assert!(
        weapon_count > 30 && weapon_count < 70,
        "Clearing should have balanced weapons: got {}",
        weapon_count
    );
    assert!(
        shield_count > 30 && shield_count < 70,
        "Clearing should have balanced shields: got {}",
        shield_count
    );
    assert!(
        consumable_count > 30 && consumable_count < 70,
        "Clearing should have balanced consumables: got {}",
        consumable_count
    );
}

/// Test that Mountains terrain favors weapons and shields equally.
/// Mountains has 0.4 weapons + 0.4 shields + 0.2 consumables.
#[test]
fn test_mountains_favors_combat_gear() {
    let terrain = BaseTerrain::Mountains;
    let mut weapon_count = 0;
    let mut shield_count = 0;
    let mut consumable_count = 0;

    for _ in 0..100 {
        let item = Item::new_random_with_terrain(terrain, None);
        if item.is_weapon() {
            weapon_count += 1;
        } else if item.is_defensive() {
            shield_count += 1;
        } else {
            consumable_count += 1;
        }
    }

    // Combat gear (weapons + shields) should dominate
    let combat_gear = weapon_count + shield_count;
    assert!(
        combat_gear > 60,
        "Mountains should favor combat gear: got {} weapons + {} shields = {} total",
        weapon_count,
        shield_count,
        combat_gear
    );
    assert!(
        combat_gear > 3 * consumable_count,
        "Mountains should have much more combat gear than consumables"
    );
}

/// Test that Tundra terrain has specific distribution.
/// Tundra has 0.3 weapons + 0.4 shields + 0.3 consumables.
#[test]
fn test_tundra_distribution() {
    let terrain = BaseTerrain::Tundra;
    let mut weapon_count = 0;
    let mut shield_count = 0;
    let mut consumable_count = 0;

    for _ in 0..100 {
        let item = Item::new_random_with_terrain(terrain, None);
        if item.is_weapon() {
            weapon_count += 1;
        } else if item.is_defensive() {
            shield_count += 1;
        } else {
            consumable_count += 1;
        }
    }

    // Shields should be the plurality
    assert!(
        shield_count > weapon_count && shield_count > consumable_count,
        "Tundra should favor shields: got {} shields, {} weapons, {} consumables",
        shield_count,
        weapon_count,
        consumable_count
    );
}

/// Test that all terrain types produce valid items.
#[test]
fn test_all_terrains_produce_valid_items() {
    let terrains = vec![
        BaseTerrain::Desert,
        BaseTerrain::Tundra,
        BaseTerrain::Forest,
        BaseTerrain::Jungle,
        BaseTerrain::Mountains,
        BaseTerrain::Clearing,
        BaseTerrain::UrbanRuins,
        BaseTerrain::Grasslands,
        BaseTerrain::Wetlands,
        BaseTerrain::Badlands,
        BaseTerrain::Highlands,
        BaseTerrain::Geothermal,
    ];

    for terrain in terrains {
        for _ in 0..10 {
            let item = Item::new_random_with_terrain(terrain, None);

            // Item should have valid properties
            assert!(!item.identifier.is_empty(), "Item should have identifier");
            assert!(!item.name.is_empty(), "Item should have name");
            assert!(item.quantity > 0, "Item should have positive quantity");

            // Effect should be within the rarity tier ranges (max Legendary = 12).
            assert!(
                item.effect > 0 && item.effect <= 12,
                "Item effect {} out of range 1-12 (rarity {:?})",
                item.effect,
                item.rarity
            );
        }
    }
}

/// Test that item weights sum approximately to 1.0 for all terrains.
#[test]
fn test_item_weights_sum_to_one() {
    let terrains = vec![
        BaseTerrain::Desert,
        BaseTerrain::Tundra,
        BaseTerrain::Forest,
        BaseTerrain::Jungle,
        BaseTerrain::Mountains,
        BaseTerrain::Clearing,
        BaseTerrain::UrbanRuins,
        BaseTerrain::Grasslands,
        BaseTerrain::Wetlands,
        BaseTerrain::Badlands,
        BaseTerrain::Highlands,
        BaseTerrain::Geothermal,
    ];

    for terrain in terrains {
        let weights = terrain.item_weights();
        let sum = weights.weapons + weights.shields + weights.consumables;

        // Allow small floating point error
        assert!(
            (sum - 1.0).abs() < 0.01,
            "Terrain {:?} weights should sum to ~1.0, got {}",
            terrain,
            sum
        );
    }
}

/// Test that named items respect terrain weights.
#[test]
fn test_named_items_respect_terrain_weights() {
    let terrain = BaseTerrain::Desert;

    // Desert should still favor consumables even with custom names
    let item = Item::new_random_with_terrain(terrain, Some("Special Item"));
    assert!(!item.name.is_empty(), "Named item should have a name");

    // Generate multiple and verify distribution still works
    let mut consumable_count = 0;
    for _ in 0..50 {
        let item = Item::new_random_with_terrain(terrain, Some("Test Item"));
        if !item.is_weapon() && !item.is_defensive() {
            consumable_count += 1;
        }
    }

    // Should still favor consumables
    assert!(
        consumable_count > 20,
        "Named items in Desert should still favor consumables: got {}",
        consumable_count
    );
}

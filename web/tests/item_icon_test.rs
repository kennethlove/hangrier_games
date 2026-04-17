#![allow(non_snake_case)]

use dioxus::prelude::*;
use game::items::{Item, ItemType};
use web::components::ItemIcon;

/// Helper function to create a test weapon
fn create_test_weapon() -> Item {
    Item {
        identifier: "test-weapon".to_string(),
        name: "Test Sword".to_string(),
        item_type: ItemType::Weapon,
        attack: Some(10),
        defense: None,
        health: None,
        sanity: None,
        movement: None,
    }
}

/// Helper function to create a test shield
fn create_test_shield() -> Item {
    Item {
        identifier: "test-shield".to_string(),
        name: "Test Shield".to_string(),
        item_type: ItemType::Shield,
        attack: None,
        defense: Some(5),
        health: None,
        sanity: None,
        movement: None,
    }
}

/// Helper function to create a test consumable
fn create_test_consumable() -> Item {
    Item {
        identifier: "test-consumable".to_string(),
        name: "Health Potion".to_string(),
        item_type: ItemType::Consumable,
        attack: None,
        defense: None,
        health: Some(25),
        sanity: None,
        movement: None,
    }
}

/// Test ItemIcon renders for weapon
#[test]
fn test_item_icon_weapon() {
    let mut dom = VirtualDom::new(|| {
        let item = create_test_weapon();

        rsx! {
            ItemIcon { item }
        }
    });

    let _edits = dom.rebuild_to_vec();
    // Component should render without panicking
}

/// Test ItemIcon renders for shield
#[test]
fn test_item_icon_shield() {
    let mut dom = VirtualDom::new(|| {
        let item = create_test_shield();

        rsx! {
            ItemIcon { item }
        }
    });

    let _edits = dom.rebuild_to_vec();
    // Component should render without panicking
}

/// Test ItemIcon renders for consumable
#[test]
fn test_item_icon_consumable() {
    let mut dom = VirtualDom::new(|| {
        let item = create_test_consumable();

        rsx! {
            ItemIcon { item }
        }
    });

    let _edits = dom.rebuild_to_vec();
    // Component should render without panicking
}

/// Test multiple ItemIcons render together
#[test]
fn test_multiple_item_icons() {
    let mut dom = VirtualDom::new(|| {
        let weapon = create_test_weapon();
        let shield = create_test_shield();
        let consumable = create_test_consumable();

        rsx! {
            div {
                ItemIcon { item: weapon }
                ItemIcon { item: shield }
                ItemIcon { item: consumable }
            }
        }
    });

    let _edits = dom.rebuild_to_vec();
    // Multiple components should render without panicking
}

/// Test ItemIcon in a list
#[test]
fn test_item_icons_in_list() {
    let mut dom = VirtualDom::new(|| {
        let items = vec![
            create_test_weapon(),
            create_test_shield(),
            create_test_consumable(),
        ];

        rsx! {
            ul {
                for item in items {
                    li {
                        key: "{item.identifier}",
                        ItemIcon { item }
                    }
                }
            }
        }
    });

    let _edits = dom.rebuild_to_vec();
    // List of ItemIcons should render without panicking
}

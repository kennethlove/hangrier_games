#![allow(non_snake_case)]

use dioxus::prelude::*;
use game::tributes::{Attributes, Tribute};
use web::components::TributeStatusIcon;

/// Helper function to create a test tribute with specified health
fn create_test_tribute(health: i32, sanity: i32) -> Tribute {
    Tribute {
        identifier: "test-tribute".to_string(),
        game_identifier: "test-game".to_string(),
        name: "Test Tribute".to_string(),
        attributes: Attributes {
            health,
            sanity,
            movement: 100,
            intelligence: 50,
            bravery: 50,
        },
        ..Default::default()
    }
}

/// Test TributeStatusIcon renders for healthy tribute
#[test]
fn test_tribute_status_healthy() {
    let mut dom = VirtualDom::new(|| {
        let tribute = create_test_tribute(100, 100);

        rsx! {
            TributeStatusIcon { tribute }
        }
    });

    let _edits = dom.rebuild_to_vec();
    // Component should render without panicking
}

/// Test TributeStatusIcon renders for injured tribute
#[test]
fn test_tribute_status_injured() {
    let mut dom = VirtualDom::new(|| {
        let tribute = create_test_tribute(50, 75);

        rsx! {
            TributeStatusIcon { tribute }
        }
    });

    let _edits = dom.rebuild_to_vec();
    // Component should render without panicking
}

/// Test TributeStatusIcon renders for critically injured tribute
#[test]
fn test_tribute_status_critical() {
    let mut dom = VirtualDom::new(|| {
        let tribute = create_test_tribute(10, 20);

        rsx! {
            TributeStatusIcon { tribute }
        }
    });

    let _edits = dom.rebuild_to_vec();
    // Component should render without panicking
}

/// Test TributeStatusIcon renders for dead tribute
#[test]
fn test_tribute_status_dead() {
    let mut dom = VirtualDom::new(|| {
        let tribute = create_test_tribute(0, 0);

        rsx! {
            TributeStatusIcon { tribute }
        }
    });

    let _edits = dom.rebuild_to_vec();
    // Component should render without panicking
}

/// Test TributeStatusIcon with low sanity
#[test]
fn test_tribute_status_low_sanity() {
    let mut dom = VirtualDom::new(|| {
        let tribute = create_test_tribute(100, 10);

        rsx! {
            TributeStatusIcon { tribute }
        }
    });

    let _edits = dom.rebuild_to_vec();
    // Component should render without panicking
}

/// Test multiple TributeStatusIcons render together
#[test]
fn test_multiple_tribute_status_icons() {
    let mut dom = VirtualDom::new(|| {
        let tribute1 = create_test_tribute(100, 100);
        let tribute2 = create_test_tribute(50, 50);
        let tribute3 = create_test_tribute(0, 0);

        rsx! {
            div {
                TributeStatusIcon { tribute: tribute1 }
                TributeStatusIcon { tribute: tribute2 }
                TributeStatusIcon { tribute: tribute3 }
            }
        }
    });

    let _edits = dom.rebuild_to_vec();
    // Multiple components should render without panicking
}

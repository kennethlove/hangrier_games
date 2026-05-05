#![cfg(not(target_arch = "wasm32"))]

use web::icons::{IconName, SPRITE};

#[test]
fn sprite_is_non_empty() {
    assert!(SPRITE.starts_with("<svg"));
    assert!(SPRITE.contains("symbol id="));
    assert!(SPRITE.ends_with("</svg>"));
}

#[test]
fn known_ui_icon_resolves() {
    assert_eq!(IconName::Edit.sprite_id(), "ui-edit");
    assert_eq!(IconName::Edit.view_box(), "0 0 24 24");
}

#[test]
fn known_narrative_icon_resolves() {
    assert_eq!(IconName::Fist.sprite_id(), "narrative-fist");
    assert_eq!(IconName::Fist.view_box(), "0 0 512 512");
}

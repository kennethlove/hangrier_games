#![allow(non_snake_case)]

use dioxus::prelude::*;
use game::tributes::statuses::TributeStatus;
use web::components::TributeStatusIcon;

#[derive(Props, Clone, PartialEq)]
struct Harness {
    status: TributeStatus,
    css_class: String,
}

fn IconHarness(p: Harness) -> Element {
    rsx! {
        TributeStatusIcon {
            status: p.status.clone(),
            css_class: p.css_class.clone(),
        }
    }
}

#[test]
fn test_status_icon_renders_for_healthy() {
    let mut dom = VirtualDom::new_with_props(
        IconHarness,
        Harness {
            status: TributeStatus::Healthy,
            css_class: "w-4 h-4".to_string(),
        },
    );
    let _edits = dom.rebuild_to_vec();
}

#[test]
fn test_status_icon_renders_for_dead() {
    let mut dom = VirtualDom::new_with_props(
        IconHarness,
        Harness {
            status: TributeStatus::Dead,
            css_class: "w-6 h-6".to_string(),
        },
    );
    let _edits = dom.rebuild_to_vec();
}

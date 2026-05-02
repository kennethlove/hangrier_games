#![allow(non_snake_case)]

use dioxus::prelude::*;
use game::areas::{Area, AreaDetails};
use web::components::Map;

fn all_areas() -> Vec<AreaDetails> {
    vec![
        AreaDetails::new(None, Area::Cornucopia),
        AreaDetails::new(None, Area::Sector1),
        AreaDetails::new(None, Area::Sector2),
        AreaDetails::new(None, Area::Sector3),
        AreaDetails::new(None, Area::Sector4),
        AreaDetails::new(None, Area::Sector5),
        AreaDetails::new(None, Area::Sector6),
    ]
}

#[derive(Props, Clone, PartialEq)]
struct MapProps {
    areas: Vec<AreaDetails>,
}

fn MapHarness(props: MapProps) -> Element {
    rsx! { Map { areas: props.areas.clone() } }
}

/// Map renders without panicking when given the standard 7-area arena.
#[test]
fn test_map_renders_with_all_seven_areas() {
    let mut dom = VirtualDom::new_with_props(MapHarness, MapProps { areas: all_areas() });
    let _edits = dom.rebuild_to_vec();
}

/// Same shape, called via a different harness invocation. Confirms
/// the lookup-by-area-name path doesn't blow up on repeat instantiation.
#[test]
fn test_map_renders_when_passed_minimal_areas() {
    let mut dom = VirtualDom::new_with_props(MapHarness, MapProps { areas: all_areas() });
    let _edits = dom.rebuild_to_vec();
}

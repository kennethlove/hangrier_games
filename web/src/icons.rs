//! Public icon API. The codegen module is private; consumers go through here.

mod generated {
    include!(concat!(env!("OUT_DIR"), "/icons_generated.rs"));
}

pub use generated::*;

use crate::components::ui::{Icon, IconSize, IconTier};
use dioxus::prelude::*;

/// Loading spinner: a styled wrapper around `MockingjayArrowIcon`.
#[component]
pub fn LoadingIcon() -> Element {
    rsx! {
        MockingjayArrowIcon {
            size: IconSize::Xl,
            class: "motion-safe:animate-pulse motion-reduce:animate-none text-primary"
                .to_string(),
        }
    }
}

/// Render a narrative icon by sprite name (without the `narrative-` prefix).
/// Used by callers that look up an icon name dynamically from a domain enum
/// (e.g. `ItemIcon`, `TributeStatusIcon`).
#[component]
pub fn NarrativeIcon(name: String, class: String) -> Element {
    let sprite_id = format!("narrative-{}", name.replace('_', "-"));
    rsx! {
        Icon {
            sprite_id,
            view_box: "0 0 512 512".to_string(),
            tier: IconTier::Narrative,
            class,
        }
    }
}

/// Legacy lookup for items → sprite name (pre-prefix). Kept verbatim from the
/// previous `svg_icon::icon_name_for_item` helper so callers in `item_icon.rs`
/// keep working without per-callsite changes.
pub fn icon_name_for_item_legacy(item: &game::items::Item) -> String {
    use game::items::{Attribute, ItemType};

    match item.item_type {
        ItemType::Consumable => match item.attribute {
            Attribute::Health => "health_potion",
            Attribute::Sanity => "spinning_top",
            Attribute::Movement => "trail_mix",
            Attribute::Bravery => "powder",
            Attribute::Speed => "fizzing_flask",
            Attribute::Strength => "hypodermic_test",
            Attribute::Defense => "spray",
        },
        ItemType::Food(_) => "trail_mix",
        ItemType::Water(_) => "fizzing_flask",
        ItemType::Weapon => match item.attribute {
            Attribute::Strength => {
                let name = item.to_string().to_lowercase();
                let weapon_name = name.rsplit_once(' ').map(|(_, w)| w).unwrap_or(&name);

                match weapon_name {
                    "sword" => "pointy_sword",
                    "spear" => "spear_hook",
                    "dagger" => "plain_dagger",
                    "knife" => "switchblade",
                    "net" => "fishing_net",
                    "trident" => "harpoon_trident",
                    "bow" => "high_shot",
                    "mace" => "spiked_mace",
                    "axe" => "wood_axe",
                    _ => "fist",
                }
            }
            Attribute::Defense => "shield",
            _ => "fist",
        },
    }
    .to_string()
}

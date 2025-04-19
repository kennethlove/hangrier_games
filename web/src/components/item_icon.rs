use crate::components::icons::game_icons_net::*;
use dioxus::prelude::*;
use game::items::{Attribute, Item, ItemType};

#[component]
fn Icon(item: Item, css_class: String) -> Element {
    match item.item_type {
        ItemType::Consumable => {
            match item.attribute {
                Attribute::Health => rsx!(HealthPotionIcon { class: css_class }),
                Attribute::Sanity => rsx!(SpinningTopIcon { class: css_class }),
                Attribute::Movement => rsx!(TrailMixIcon { class: css_class }),
                Attribute::Bravery => rsx!(PowderIcon { class: css_class }),
                Attribute::Speed => rsx!(FizzingFlaskIcon { class: css_class }),
                Attribute::Strength => rsx!(HypodermicTestIcon { class: css_class }),
                Attribute::Defense => rsx!(SprayIcon { class: css_class }),
            }

        },
        ItemType::Weapon => {
            match item.attribute {
                Attribute::Strength => {
                    let name = item.to_string().to_lowercase();
                    let weapon_name = name.rsplit_once(' ').unwrap().1;

                    match weapon_name {
                        "sword" => rsx!(PointySwordIcon { class: css_class }),
                        "spear" => rsx!(SpearHookIcon { class: css_class }),
                        "dagger" => rsx!(PlainDaggerIcon { class: css_class }),
                        "knife" => rsx!(SwitchbladeIcon { class: css_class }),
                        "net" => rsx!(FishingNetIcon { class: css_class }),
                        "trident" => rsx!(HarpoonTridentIcon { class: css_class }),
                        "bow" => rsx!(HighShotIcon { class: css_class }),
                        "mace" => rsx!(SpikedMaceIcon { class: css_class }),
                        "axe" => rsx!(WoodAxeIcon { class: css_class }),
                        _ => rsx!(FistIcon { class: css_class }),
                    }

                },
                Attribute::Defense => rsx!(ShieldIcon { class: css_class }),
                _ => { rsx! { p { "Unknown" }}}
            }

        }
    }
}

#[component]
pub fn ItemIcon(item: Item, css_class: String) -> Element {
    rsx! {
        span {
            title: item.to_string(),
            Icon { item: item.clone(), css_class }
            span {
                class: "sr-only",
                title: item.to_string(),
                "{item.to_string()}"
            }
        }
    }
}

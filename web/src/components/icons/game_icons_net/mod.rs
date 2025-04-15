mod broken_bone;
mod burned;
mod dead;
mod dehydrated;
mod drowning;
mod electrocuted;
mod falling_rocks;
mod fishing_net;
mod fist;
mod fizzing_flask;
mod frozen_body;
mod harpoon_trident;
mod health_potion;
mod hearts;
mod heat_haze;
mod high_shot;
mod hypodermic_test;
mod infection;
mod mauled;
mod plain_dagger;
mod pointy_sword;
mod poison_bottle;
mod powder;
mod recently_dead;
mod shield;
mod spear_hook;
mod spiked_mace;
mod spinning_top;
mod spray;
mod starving;
mod switchblade;
mod trail_mix;
mod vomiting;
mod wounded;

use dioxus::prelude::*;
pub use broken_bone::BrokenBoneIcon;
pub use burned::BurnedIcon;
pub use dead::*;
pub use dehydrated::*;
pub use drowning::*;
pub use electrocuted::*;
pub use falling_rocks::*;
pub use fishing_net::*;
pub use fist::*;
pub use fizzing_flask::*;
pub use frozen_body::*;
pub use harpoon_trident::*;
pub use health_potion::*;
pub use hearts::*;
pub use heat_haze::*;
pub use high_shot::*;
pub use hypodermic_test::*;
pub use infection::*;
pub use mauled::*;
pub use plain_dagger::*;
pub use pointy_sword::*;
pub use poison_bottle::*;
pub use powder::*;
pub use recently_dead::*;
pub use shield::*;
pub use spear_hook::*;
pub use spiked_mace::*;
pub use spinning_top::*;
pub use spray::*;
pub use starving::*;
pub use switchblade::*;
pub use trail_mix::*;
pub use vomiting::*;
pub use wounded::*;

pub fn get_icon(name: &str, css_class: String) -> Option<Element> {
    match name {
        "broken_bone" => Some(rsx!(BrokenBoneIcon { class: css_class })),
        "burned" => Some(rsx!(BurnedIcon { class: css_class })),
        "dead" => Some(rsx!(DeadIcon { class: css_class })),
        "dehydrated" => Some(rsx!(DehydratedIcon { class: css_class })),
        "drowning" => Some(rsx!(DrowningIcon { class: css_class })),
        "electrocuted" => Some(rsx!(ElectrocutedIcon { class: css_class })),
        "falling_rocks" => Some(rsx!(FallingRocksIcon { class: css_class })),
        "fishing_net" => Some(rsx!(FishingNetIcon { class: css_class })),
        "fist" => Some(rsx!(FistIcon { class: css_class })),
        "fizzing_flask" => Some(rsx!(FizzingFlaskIcon { class: css_class })),
        "frozen_body" => Some(rsx!(FrozenBodyIcon { class: css_class })),
        "harpoon_trident" => Some(rsx!(HarpoonTridentIcon { class: css_class })),
        "health_potion" => Some(rsx!(HealthPotionIcon { class: css_class })),
        "hearts" => Some(rsx!(HeartsIcon { class: css_class })),
        "heat_haze" => Some(rsx!(HeatHazeIcon { class: css_class })),
        "high_shot" => Some(rsx!(HighShotIcon { class: css_class })),
        "hypodermic_test" => Some(rsx!(HypodermicTestIcon { class: css_class })),
        "infection" => Some(rsx!(InfectionIcon { class: css_class })),
        "mauled" => Some(rsx!(MauledIcon { class: css_class })),
        "plain_dagger" => Some(rsx!(PlainDaggerIcon { class: css_class })),
        "pointy_sword" => Some(rsx!(PointySwordIcon { class: css_class })),
        "poison_bottle" => Some(rsx!(PoisonBottleIcon { class: css_class })),
        "powder" => Some(rsx!(PowderIcon { class: css_class })),
        "recently_dead" => Some(rsx!(RecentlyDeadIcon { class: css_class })),
        "shield" => Some(rsx!(ShieldIcon { class: css_class })),
        "spear_hook" => Some(rsx!(SpearHookIcon { class: css_class })),
        "spiked_mace" => Some(rsx!(SpikedMaceIcon { class: css_class })),
        "spinning_top" => Some(rsx!(SpinningTopIcon { class: css_class })),
        "spray" => Some(rsx!(SprayIcon { class: css_class })),
        "starving" => Some(rsx!(StarvingIcon { class: css_class })),
        "switchblade" => Some(rsx!(SwitchbladeIcon { class: css_class })),
        "trail_mix" => Some(rsx!(TrailMixIcon { class: css_class })),
        "vomiting" => Some(rsx!(VomitingIcon { class: css_class })),
        "wounded" => Some(rsx!(WoundedIcon { class: css_class })),
        _ => None,
    }
}

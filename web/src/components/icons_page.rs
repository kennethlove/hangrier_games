use dioxus::html::a::class;
use dioxus::prelude::*;
use crate::components::icons::game_icons_net::*;

#[component]
pub fn IconsPage() -> Element {
    rsx! {
        div {
            h1 { "Icons" }
            div {
                class: "grid grid-cols-6 gap-4",
                BrokenBoneIcon {
                    class: "fill-black",
                }
                BurnedIcon {
                    class: "fill-black",
                }
                DeadIcon {
                    class: "fill-black",
                }
                DehydratedIcon {
                    class: "fill-black",
                }
                DrowningIcon {
                    class: "fill-black",
                }
                ElectrocutedIcon {
                    class: "fill-black",
                }
                FallingRocksIcon {
                    class: "fill-black",
                }
                FishingNetIcon {
                    class: "fill-black",
                }
                FistIcon {
                    class: "fill-black",
                }
                FizzingFlaskIcon {
                    class: "fill-black",
                }
                FrozenBodyIcon {
                    class: "fill-black",
                }
                HarpoonTridentIcon {
                    class: "fill-black",
                }
                HealthPotionIcon {
                    class: "fill-black",
                }
                HeartsIcon {
                    class: "fill-black",
                }
                HeatHazeIcon {
                    class: "fill-black",
                }
                HighShotIcon {
                    class: "fill-black",
                }
                HypodermicTestIcon {
                    class: "fill-black",
                }
                InfectionIcon {
                    class: "fill-black",
                }
                MauledIcon {
                    class: "fill-black",
                }
                PlainDaggerIcon {
                    class: "fill-black",
                }
                PointySwordIcon {
                    class: "fill-black",
                }
                PoisonBottleIcon {
                    class: "fill-black",
                }
                PowderIcon {
                    class: "fill-black",
                }
                RecentlyDeadIcon {
                    class: "fill-black",
                }
                ShieldIcon {
                    class: "fill-black",
                }
                SpearHookIcon {
                    class: "fill-black",
                }
                SpikedMaceIcon {
                    class: "fill-black",
                }
                SpinningTopIcon {
                    class: "fill-black",
                }
                SprayIcon {
                    class: "fill-black",
                }
                StarvingIcon {
                    class: "fill-black",
                }
                SwitchbladeIcon {
                    class: "fill-black",
                }
                TrailMixIcon {
                    class: "fill-black",
                }
                VomitingIcon {
                    class: "fill-black",
                }
                WoundedIcon {
                    class: "fill-black",
                }
            }
        }
    }
}

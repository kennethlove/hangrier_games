use crate::components::icons::game_icons_net::*;
use dioxus::prelude::*;

#[component]
pub fn IconsPage() -> Element {
    rsx! {
        div {
            h1 {
                class: "text-3xl font-bold",
                "Icons"
            }
            div {
                class: "grid grid-cols-6 gap-4",
                BrokenBoneIcon {
                    class: "fill-black size-24",
                }
                BurnedIcon {
                    class: "fill-black size-24",
                }
                DeadIcon {
                    class: "fill-black size-24",
                }
                DehydratedIcon {
                    class: "fill-black size-24",
                }
                DrowningIcon {
                    class: "fill-black size-24",
                }
                ElectrocutedIcon {
                    class: "fill-black size-24",
                }
                FallingRocksIcon {
                    class: "fill-black size-24",
                }
                FishingNetIcon {
                    class: "fill-black size-24",
                }
                FistIcon {
                    class: "fill-black size-24",
                }
                FizzingFlaskIcon {
                    class: "fill-black size-24",
                }
                FrozenBodyIcon {
                    class: "fill-black size-24",
                }
                HarpoonTridentIcon {
                    class: "fill-black size-24",
                }
                HealthPotionIcon {
                    class: "fill-black size-24",
                }
                HeartsIcon {
                    class: "fill-black size-24",
                }
                HeatHazeIcon {
                    class: "fill-black size-24",
                }
                HighShotIcon {
                    class: "fill-black size-24",
                }
                HypodermicTestIcon {
                    class: "fill-black size-24",
                }
                InfectionIcon {
                    class: "fill-black size-24",
                }
                MauledIcon {
                    class: "fill-black size-24",
                }
                PlainDaggerIcon {
                    class: "fill-black size-24",
                }
                PointySwordIcon {
                    class: "fill-black size-24",
                }
                PoisonBottleIcon {
                    class: "fill-black size-24",
                }
                PowderIcon {
                    class: "fill-black size-24",
                }
                RecentlyDeadIcon {
                    class: "fill-black size-24",
                }
                ShieldIcon {
                    class: "fill-black size-24",
                }
                SpearHookIcon {
                    class: "fill-black size-24",
                }
                SpikedMaceIcon {
                    class: "fill-black size-24",
                }
                SpinningTopIcon {
                    class: "fill-black size-24",
                }
                SprayIcon {
                    class: "fill-black size-24",
                }
                StarvingIcon {
                    class: "fill-black size-24",
                }
                SwitchbladeIcon {
                    class: "fill-black size-24",
                }
                TrailMixIcon {
                    class: "fill-black size-24",
                }
                VomitingIcon {
                    class: "fill-black size-24",
                }
                WoodAxeIcon {
                    class: "fill-black size-24",
                }
                WoundedIcon {
                    class: "fill-black size-24",
                }
            }
        }
    }
}

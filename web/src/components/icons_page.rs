use crate::components::ui::IconSize;
use crate::icons::*;
use dioxus::prelude::*;

#[component]
pub fn IconsPage() -> Element {
    let sz = IconSize::Xxl;
    let cls = "text-text".to_string();

    rsx! {
        div {
            h1 {
                class: "text-3xl font-bold",
                "Icons"
            }
            div {
                class: "grid grid-cols-6 gap-4",
                BrokenBoneIcon { size: sz, class: cls.clone() }
                BurnedIcon { size: sz, class: cls.clone() }
                DeadIcon { size: sz, class: cls.clone() }
                DehydratedIcon { size: sz, class: cls.clone() }
                DrowningIcon { size: sz, class: cls.clone() }
                ElectrocutedIcon { size: sz, class: cls.clone() }
                FallingRocksIcon { size: sz, class: cls.clone() }
                FishingNetIcon { size: sz, class: cls.clone() }
                FistIcon { size: sz, class: cls.clone() }
                FizzingFlaskIcon { size: sz, class: cls.clone() }
                FrozenBodyIcon { size: sz, class: cls.clone() }
                HarpoonTridentIcon { size: sz, class: cls.clone() }
                HealthPotionIcon { size: sz, class: cls.clone() }
                HeartsIcon { size: sz, class: cls.clone() }
                HeatHazeIcon { size: sz, class: cls.clone() }
                HighShotIcon { size: sz, class: cls.clone() }
                HypodermicTestIcon { size: sz, class: cls.clone() }
                InfectionIcon { size: sz, class: cls.clone() }
                MauledIcon { size: sz, class: cls.clone() }
                PlainDaggerIcon { size: sz, class: cls.clone() }
                PointySwordIcon { size: sz, class: cls.clone() }
                PoisonBottleIcon { size: sz, class: cls.clone() }
                PowderIcon { size: sz, class: cls.clone() }
                RecentlyDeadIcon { size: sz, class: cls.clone() }
                ShieldIcon { size: sz, class: cls.clone() }
                SpearHookIcon { size: sz, class: cls.clone() }
                SpikedMaceIcon { size: sz, class: cls.clone() }
                SpinningTopIcon { size: sz, class: cls.clone() }
                SprayIcon { size: sz, class: cls.clone() }
                StarvingIcon { size: sz, class: cls.clone() }
                SwitchbladeIcon { size: sz, class: cls.clone() }
                TrailMixIcon { size: sz, class: cls.clone() }
                VomitingIcon { size: sz, class: cls.clone() }
                WoodAxeIcon { size: sz, class: cls.clone() }
                WoundedIcon { size: sz, class: cls }
            }
        }
    }
}

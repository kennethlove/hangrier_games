use crate::components::icons::game_icons_net::*;
use dioxus::prelude::*;
use game::tributes::statuses::TributeStatus;

#[component]
fn StatusIcon(status: TributeStatus, css_class: String) -> Element {
    match status {
        TributeStatus::Broken => rsx!(BrokenBoneIcon { class: css_class }),
        TributeStatus::Buried => rsx!(FallingRocksIcon { class: css_class }),
        TributeStatus::Burned => rsx!(BurnedIcon { class: css_class }),
        TributeStatus::Dead => rsx!(DeadIcon { class: css_class }),
        TributeStatus::Dehydrated => rsx!(DehydratedIcon { class: css_class }),
        TributeStatus::Drowned => rsx!(DrowningIcon { class: css_class }),
        TributeStatus::Electrocuted => rsx!(ElectrocutedIcon { class: css_class }),
        TributeStatus::Frozen => rsx!(FrozenBodyIcon { class: css_class }),
        TributeStatus::Healthy => rsx!(HeartsIcon { class: css_class }),
        TributeStatus::Infected => rsx!(InfectionIcon { class: css_class }),
        TributeStatus::Overheated => rsx!(HeatHazeIcon { class: css_class }),
        TributeStatus::Mauled(_) => rsx!(MauledIcon { class: css_class }),
        TributeStatus::Poisoned => rsx!(PoisonBottleIcon { class: css_class }),
        TributeStatus::RecentlyDead => rsx!(RecentlyDeadIcon { class: css_class }),
        TributeStatus::Starving => rsx!(StarvingIcon { class: css_class }),
        TributeStatus::Sick => rsx!(VomitingIcon { class: css_class }),
        TributeStatus::Wounded => rsx!(WoundedIcon { class: css_class }),
    }
}

#[component]
pub fn TributeStatusIcon(status: TributeStatus, css_class: String) -> Element {
    rsx! {
        span {
            title: status.to_string(),
            StatusIcon { status: status.clone(), css_class },
            span { class: "sr-only", "{status}"},
        }
    }
}

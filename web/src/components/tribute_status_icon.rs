use crate::components::icons::svg_icon::SvgIcon;
use dioxus::prelude::*;
use game::tributes::statuses::TributeStatus;

/// Maps tribute status to icon names
fn icon_name_for_status(status: &TributeStatus) -> String {
    match status {
        TributeStatus::Broken => "broken_bone",
        TributeStatus::Buried => "falling_rocks",
        TributeStatus::Burned => "burned",
        TributeStatus::Dead => "dead",
        TributeStatus::Dehydrated => "dehydrated",
        TributeStatus::Drowned => "drowning",
        TributeStatus::Electrocuted => "electrocuted",
        TributeStatus::Frozen => "frozen_body",
        TributeStatus::Healthy => "hearts",
        TributeStatus::Infected => "infection",
        TributeStatus::Overheated => "heat_haze",
        TributeStatus::Mauled(_) => "mauled",
        TributeStatus::Poisoned => "poison_bottle",
        TributeStatus::RecentlyDead => "recently_dead",
        TributeStatus::Starving => "starving",
        TributeStatus::Sick => "vomiting",
        TributeStatus::Wounded => "wounded",
    }
    .to_string()
}

#[component]
pub fn TributeStatusIcon(status: TributeStatus, css_class: String) -> Element {
    let icon_name = icon_name_for_status(&status);

    rsx! {
        span {
            title: status.to_string(),
            SvgIcon { name: icon_name, class: css_class },
            span { class: "sr-only", "{status}"},
        }
    }
}

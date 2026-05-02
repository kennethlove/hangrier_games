use crate::components::icons::svg_icon::SvgIcon;
use dioxus::prelude::*;
use game::tributes::statuses::TributeStatus;

/// Maps tribute status to icon names
pub(crate) fn icon_name_for_status(status: &TributeStatus) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_statuses_map_to_icons() {
        assert_eq!(icon_name_for_status(&TributeStatus::Healthy), "hearts");
        assert_eq!(icon_name_for_status(&TributeStatus::Dead), "dead");
        assert_eq!(icon_name_for_status(&TributeStatus::Drowned), "drowning");
        assert_eq!(icon_name_for_status(&TributeStatus::Sick), "vomiting");
        assert_eq!(
            icon_name_for_status(&TributeStatus::Buried),
            "falling_rocks"
        );
        assert_eq!(icon_name_for_status(&TributeStatus::Wounded), "wounded");
        assert_eq!(
            icon_name_for_status(&TributeStatus::RecentlyDead),
            "recently_dead"
        );
        assert_eq!(icon_name_for_status(&TributeStatus::Burned), "burned");
        assert_eq!(icon_name_for_status(&TributeStatus::Frozen), "frozen_body");
        assert_eq!(
            icon_name_for_status(&TributeStatus::Overheated),
            "heat_haze"
        );
        assert_eq!(icon_name_for_status(&TributeStatus::Broken), "broken_bone");
        assert_eq!(icon_name_for_status(&TributeStatus::Infected), "infection");
        assert_eq!(
            icon_name_for_status(&TributeStatus::Poisoned),
            "poison_bottle"
        );
        assert_eq!(icon_name_for_status(&TributeStatus::Starving), "starving");
        assert_eq!(
            icon_name_for_status(&TributeStatus::Electrocuted),
            "electrocuted"
        );
        assert_eq!(
            icon_name_for_status(&TributeStatus::Dehydrated),
            "dehydrated"
        );
    }
}

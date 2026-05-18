use crate::icons::NarrativeIcon;
use dioxus::prelude::*;
use game::tributes::statuses::TributeStatus;

/// Maps tribute status to icon names.
/// Note: Most status effects are now represented as afflictions.
/// This component only handles the core life-state statuses.
pub(crate) fn icon_name_for_status(status: &TributeStatus) -> String {
    match status {
        TributeStatus::Healthy => "hearts",
        TributeStatus::RecentlyDead => "recently_dead",
        TributeStatus::Dead => "dead",
        TributeStatus::Mauled(_) => "mauled",
    }
    .to_string()
}

#[component]
pub fn TributeStatusIcon(status: TributeStatus, css_class: String) -> Element {
    let icon_name = icon_name_for_status(&status);

    rsx! {
        span {
            title: status.to_string(),
            NarrativeIcon { name: icon_name, class: css_class },
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
        assert_eq!(
            icon_name_for_status(&TributeStatus::RecentlyDead),
            "recently_dead"
        );
    }
}

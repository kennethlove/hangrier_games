use crate::icons::{NarrativeIcon, icon_name_for_item_legacy};
use dioxus::prelude::*;
use game::items::Item;

#[component]
pub fn ItemIcon(item: Item, css_class: String) -> Element {
    let icon_name = icon_name_for_item_legacy(&item);

    rsx! {
        span {
            title: item.to_string(),
            NarrativeIcon { name: icon_name, class: css_class }
        }
    }
}

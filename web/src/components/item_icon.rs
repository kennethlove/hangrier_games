use crate::components::icons::svg_icon::{SvgIcon, icon_name_for_item};
use dioxus::prelude::*;
use game::items::Item;

#[component]
pub fn ItemIcon(item: Item, css_class: String) -> Element {
    let icon_name = icon_name_for_item(&item);

    rsx! {
        span {
            title: item.to_string(),
            SvgIcon { name: icon_name, class: css_class }
        }
    }
}

use dioxus::prelude::*;

#[component]
pub fn FishingNetIcon(class: String) -> Element {
    rsx! {
        svg {
            view_box: "0 0 512 512",
            class,
            path {
                d: "M256 32l-64 128 64 64-128 128 64 64 128-128 64 64 64-128-64-64 128-128-64-64-128 128z",
            }
        }
    }
}

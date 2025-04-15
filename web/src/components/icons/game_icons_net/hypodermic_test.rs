use dioxus::prelude::*;

#[component]
pub fn HypodermicTestIcon(class: String) -> Element {
    rsx! {
        svg {
            view_box: "0 0 512 512",
            class,
            path {
                d: "M256 32l-64 96h128l-64 96 96 128H160l96-128-64-96h128z",
            }
        }
    }
}

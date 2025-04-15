use dioxus::prelude::*;

#[component]
pub fn FallingRocksIcon(class: String) -> Element {
    rsx! {
        svg {
            view_box: "0 0 512 512",
            class,
            path {
                d: "M432 32l-64 96 64 32-96 128 128 64-64 96H64l96-128-64-32 128-192-96-64h304z",
            }
        }
    }
}

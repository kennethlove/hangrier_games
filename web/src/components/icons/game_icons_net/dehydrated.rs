use dioxus::prelude::*;

#[component]
pub fn DehydratedIcon(class: String) -> Element {
    rsx! {
        svg {
            view_box: "0 0 512 512",
            class,
            path {
                d: "M256 0C167.64 0 96 71.64 96 160c0 88.36 160 352 160 352s160-263.64 160-352c0-88.36-71.64-160-160-160zm0 240c-44.18 0-80-35.82-80-80s35.82-80 80-80 80 35.82 80 80-35.82 80-80 80z",
            }
        }
    }
}

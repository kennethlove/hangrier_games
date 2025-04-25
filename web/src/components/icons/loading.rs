use dioxus::prelude::*;

#[component]
pub fn LoadingIcon(class: String) -> Element {
    rsx! {
        svg {
            view_box: "0 0 50 50",
            class: "{class}",
            circle {
                // Use a unique class name linked to the style block
                class: "spinner",
                cx: "25",
                cy: "25",
                r: "20",
            }
        }
    }
}

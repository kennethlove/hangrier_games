use dioxus::prelude::*;

#[component]
pub fn ShieldIcon(class: String) -> Element {
    rsx! {
        svg {
            view_box: "0 0 512 512",
            class,
            path {
                d: "M256 16c25 24 100 72 150 72v96c0 96-75 240-150 312-75-72-150-216-150-312V88c50 0 125-48 150-72z",
                fill: "#fff",
                fill_opacity: "1",
            }
        }
    }
}

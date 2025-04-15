use dioxus::prelude::*;

#[component]
pub fn PlainDaggerIcon(class: String) -> Element {
    rsx! {
        svg {
            view_box: "0 0 512 512",
            class,
            path {
                d: "M0 0h512v512H0z",
                fill: "#000",
                fill_opacity: "1",
            }
            path {
                d: "M43.53 15.75c-15.73 0-28.31 12.583-28.31 28.313 0 14.086 10.092 25.644 ...existing path data...",
                fill: "#fff",
                fill_opacity: "1",
            }
        }
    }
}

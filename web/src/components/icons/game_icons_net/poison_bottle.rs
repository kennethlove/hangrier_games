use dioxus::prelude::*;

#[component]
pub fn PoisonBottleIcon(class: String) -> Element {
    rsx! {
        svg {
            view_box: "0 0 512 512",
            class,
            path {
                d: "M181.78 33v51.53h149.407V33H181.78zm14.19 70.22c-.66 2.9-1.554 5.263-2.75 6.936 ...existing path data...",
            }
        }
    }
}

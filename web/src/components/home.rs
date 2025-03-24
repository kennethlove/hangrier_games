use dioxus::prelude::*;

#[component]
pub fn Home() -> Element {
    rsx! {
        p { "May the odds be ever in your favor!" }
    }
}


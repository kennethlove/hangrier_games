use crate::components::icons::mockingjay_arrow::MockingjayArrow;
use dioxus::prelude::*;

#[component]
pub fn LoadingIcon() -> Element {
    rsx! {
        MockingjayArrow {
            class: r#"
            size-16
            motion-safe:animate-pulse
            motion-reduce:animate-none

            "#
        }
    }
}

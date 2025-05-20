use dioxus::prelude::*;
use crate::components::icons::mockingjay_arrow::MockingjayArrow;

#[component]
pub fn LoadingIcon() -> Element {
    rsx! {
        MockingjayArrow {
            class: r#"
            size-16
            motion-safe:animate-pulse
            motion-reduce:animate-none
            theme1:fill-red-900
            theme2:fill-green-800
            theme3:fill-amber-600
            theme3:size-24
            "#
        }
    }
}
